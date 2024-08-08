use clap::Parser;
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// The command to execute and have reported
    command: String,

    /// Where to write memory usage numbers
    #[arg(short, long, value_name = "FILE")]
    out_file: PathBuf,

    /// How frequently, in seconds, should memory usage be recorded
    #[arg(short = 'n', long, default_value_t = 60)]
    interval: u64,
}

#[cfg(target_os = "linux")]
fn main() -> anyhow::Result<()> {
    use anyhow::Context;
    use dumbmem::memory_stats;
    use std::fs::OpenOptions;
    use std::io::Write;
    use std::process::Child;
    use std::sync::Arc;
    use std::sync::Mutex;
    use std::time::Duration;
    use std::time::Instant;

    let cli = Cli::parse();

    let mut parsed_command = shellwords::split(cli.command.as_str())?;

    if parsed_command.is_empty() {
        anyhow::bail!("invalid: empty command");
    }

    let program = parsed_command.remove(0);
    let args = parsed_command; // Warning - not doing *any* escaping or sanitization!

    let child: Arc<Mutex<Option<Child>>> = Arc::new(Mutex::new(None));
    let child_ref = Arc::clone(&child);

    let id: Arc<Mutex<Option<u32>>> = Arc::new(Mutex::new(None));
    let id_ref = Arc::clone(&id);

    let (done_tx, done_rx) = std::sync::mpsc::channel::<bool>();
    let done_tx_ref = done_tx.clone();

    let mut out_file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(cli.out_file)?;

    // interrupt handler
    ctrlc::set_handler(move || {
        println!("interrupt received, killing the child process...");

        done_tx_ref.send(true).expect("sending done signal failed");

        let mut child = child_ref.lock().expect("child process lock poisoned");

        if let Some(mut child_process) = child.take() {
            let _ = child_process.kill();
        }
    })
    .expect("error setting ctrl+c handler");

    // spawn child in new thread
    let handle = std::thread::spawn(move || -> anyhow::Result<()> {
        let cmd = Command::new(program)
            .args(args)
            .spawn()
            .expect("failed to start child process");

        let id = cmd.id();

        *child.lock().expect("child process lock poisoned") = Some(cmd);
        *id_ref.lock().expect("child id lock poisoned") = Some(id);

        let mut child = child.lock().expect("child process lock poisoned");
        if let Some(mut child) = child.take() {
            println!("waiting for child process {}", id);
            let _ = child.wait();
        }

        done_tx.send(true).expect("sending done signal failed");

        println!("child process finished.");

        Ok(())
    });

    let mut next_stats = Instant::now();
    while done_rx.try_recv().is_err() {
        let now = Instant::now();

        if now > next_stats {
            next_stats = now
                .checked_add(Duration::from_secs(cli.interval))
                .context("unable to calculate next stat collection instant")?;

            let maybe_id = id.lock().expect("child id lock poisoned").to_owned();

            if let Some(id) = maybe_id {
                if let Some(stats) = memory_stats(id) {
                    let mem_kb = stats.physical_mem / 1024;
                    let mem_mb = mem_kb / 1024;
                    let when = chrono::Local::now().format("%H:%M:%S");
                    let stat_line = format!("{when}\t{mem_mb}\n");

                    out_file.write_all(stat_line.as_bytes())?;
                }
            }
        }

        std::thread::sleep(Duration::from_millis(100));
    }

    handle.join().expect("child process had a panic")?;

    Ok(())
}

#[cfg(not(target_os = "linux"))]
fn main() -> anyhow::Result<()> {
    anyhow::bail!("dumbmem is linux only")
}

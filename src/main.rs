use clap::Parser;
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// The command to execute and have reported
    command: String,

    /// How frequently, in seconds, should memory usage be recorded
    #[arg(short = 'n', long, default_value_t = 60)]
    interval: usize,

    /// Where to write memory usage numbers
    #[arg(short, long, value_name = "FILE")]
    out_file: Option<PathBuf>,
}

#[cfg(target_family = "unix")]
fn main() -> anyhow::Result<()> {
    use std::process::Child;
    use std::sync::Arc;
    use std::sync::Mutex;

    let cli = Cli::parse();

    let mut parsed_command = shellwords::split(cli.command.as_str())?;

    if parsed_command.is_empty() {
        anyhow::bail!("invalid: empty command");
    }

    let program = parsed_command.remove(0);
    let args = parsed_command; // Warning - not doing *any* escaping or sanitization!

    let child: Arc<Mutex<Option<Child>>> = Arc::new(Mutex::new(None));
    let child_ref = Arc::clone(&child);

    // interrupt handler
    ctrlc::set_handler(move || {
        println!("interrupt received, killing the child process...");

        let mut child = child_ref.lock().expect("child process lock poisoned");

        if let Some(mut child_process) = child.take() {
            let _ = child_process.kill();
        }

        std::process::exit(1);
    })
    .expect("error setting ctrl+c handler");

    // spawn child in new thread
    let handle = std::thread::spawn(move || -> anyhow::Result<()> {
        let cmd = Command::new(program)
            .args(args)
            .spawn()
            .expect("failed to start child process");

        *child.lock().expect("child process lock poisoned") = Some(cmd);

        let mut child = child.lock().expect("child process lock poisoned");
        if let Some(mut child) = child.take() {
            println!("waiting for child process {}", child.id());
            let _ = child.wait();
        }

        println!("child process finished.");

        Ok(())
    });

    handle.join().expect("child process had a panic")?;

    Ok(())
}

#[cfg(not(target_family = "unix"))]
fn main() -> anyhow::Result<()> {
    anyhow::bail!("dumbmem is unix only")
}

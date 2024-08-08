use clap::Parser;

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// The command to execute and have reported
    command: String,

    /// How frequently, in seconds, should memory usage be recorded
    #[arg(short = 'n', long)]
    interval: u8,
}

fn main() {
    let cli = Cli::parse();

    dbg!(cli);
}

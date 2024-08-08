use clap::Parser;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// The command to execute and have reported
    command: String,

    /// How frequently, in seconds, should memory usage be recorded
    #[arg(short = 'n', long, default_value_t = 5)]
    interval: u8,

    /// Where to write memory usage numbers
    #[arg(short, long, value_name = "FILE")]
    out_file: Option<PathBuf>,
}

fn main() {
    let cli = Cli::parse();

    dbg!(cli);
}

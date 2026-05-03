mod cli;
mod commands;
pub mod diagnostics;

use clap::Parser;

fn main() {
    let cli = cli::Cli::parse();
    commands::run(cli.command);
}

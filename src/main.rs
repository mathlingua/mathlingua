mod cli;
mod commands;
mod constants;
pub mod diagnostics;
pub mod proto;

use clap::Parser;

fn main() {
    let cli = cli::Cli::parse();
    commands::run(cli.command);
}

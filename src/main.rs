mod cli;
mod commands;

use clap::Parser;

fn main() {
    let cli = cli::Cli::parse();
    commands::run(cli.command);
}

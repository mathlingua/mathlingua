use clap::Parser;
use mlg::cli::{self, Cli};

fn main() {
    let cli = Cli::parse();
    cli::run(cli.command);
}

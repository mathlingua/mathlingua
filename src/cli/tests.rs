use super::Cli;
use clap::CommandFactory;

#[test]
fn verify_cli() {
    Cli::command().debug_assert();
}

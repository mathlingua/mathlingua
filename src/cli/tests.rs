use super::{CheckArgs, Cli, Command};
use clap::{CommandFactory, Parser};
use std::path::PathBuf;

#[test]
fn verify_cli() {
    Cli::command().debug_assert();
}

#[test]
fn parses_check_without_paths() {
    let cli = Cli::parse_from(["mlg", "check"]);

    assert!(matches!(
        cli.command,
        Command::Check(CheckArgs { paths }) if paths.is_empty()
    ));
}

#[test]
fn parses_check_with_multiple_paths() {
    let cli = Cli::parse_from(["mlg", "check", "content", "notes/example.mlg"]);

    assert!(matches!(
        cli.command,
        Command::Check(CheckArgs { paths })
            if paths == vec![PathBuf::from("content"), PathBuf::from("notes/example.mlg")]
    ));
}

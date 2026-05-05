mod commands;

use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(name = "mlg")]
#[command(version, about, long_about = None)]
#[command(arg_required_else_help = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Clone, Debug, Subcommand)]
pub enum Command {
    /// Check Mathlingua files for errors
    Check(CheckArgs),
    /// Initialize a Mathlingua collection
    Init,
    /// Print version information and quit
    Version,
    /// View rendered Mathlingua files
    View,
}

#[derive(Clone, Debug, Args, PartialEq, Eq)]
pub struct CheckArgs {
    /// Directories or .mlg files to check. Defaults to the collection's content directory.
    #[arg(value_name = "PATH")]
    pub paths: Vec<PathBuf>,
}

pub fn run(command: Command) {
    commands::run(command);
}

// =============================================================================

#[cfg(test)]
mod tests {
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
}

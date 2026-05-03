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

#[cfg(test)]
mod tests;

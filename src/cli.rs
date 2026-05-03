use clap::{Parser, Subcommand};

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
    Check,
    /// Initialize a Mathlingua collection
    Init,
    /// Print version information and quit
    Version,
    /// View rendered Mathlingua files
    View,
}

#[cfg(test)]
mod tests;

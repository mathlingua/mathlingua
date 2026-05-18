use crate::events::{Audience, EventFilter, Level};
use clap::{Args, Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

/// Parsed command-line interface for the `mlg` executable.
///
/// Global event flags are collected here and converted into an `EventFilter`
/// before the selected subcommand is executed.
#[derive(Debug, Parser)]
#[command(name = "mlg")]
#[command(version, about, long_about = None)]
#[command(arg_required_else_help = true)]
pub struct Cli {
    /// Which event audiences should be printed to the console.
    #[arg(
        long = "event-audience",
        visible_alias = "event-scope",
        value_enum,
        value_delimiter = ',',
        global = true
    )]
    pub event_audiences: Vec<CliEventAudience>,

    /// Which event levels should be printed to the console.
    #[arg(long = "event-level", value_enum, value_delimiter = ',', global = true)]
    pub event_levels: Vec<CliEventLevel>,

    /// Whether marker events should be printed to the console.
    #[arg(long = "event-markers", global = true, default_value_t = false)]
    pub event_markers: bool,

    /// Subcommand requested by the user.
    #[command(subcommand)]
    pub command: Command,
}

impl Cli {
    /// Converts CLI event flags into the filter used by console event output.
    pub fn event_filter(&self) -> EventFilter {
        let audiences = if self.event_audiences.is_empty() {
            vec![Audience::User]
        } else {
            self.event_audiences
                .iter()
                .copied()
                .map(Into::into)
                .collect()
        };
        let levels = if self.event_levels.is_empty() {
            Level::all().to_vec()
        } else {
            self.event_levels.iter().copied().map(Into::into).collect()
        };

        EventFilter::new()
            .with_audiences(audiences)
            .with_levels(levels)
            .include_markers(self.event_markers)
    }
}

/// CLI representation of event audiences accepted by `--event-audience`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum CliEventAudience {
    /// User-facing messages.
    User,
    /// Internal/system messages.
    System,
}

impl From<CliEventAudience> for Audience {
    /// Converts the CLI audience enum into the shared event audience enum.
    fn from(value: CliEventAudience) -> Self {
        match value {
            CliEventAudience::User => Audience::User,
            CliEventAudience::System => Audience::System,
        }
    }
}

/// CLI representation of event levels accepted by `--event-level`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum CliEventLevel {
    /// Informational messages.
    Log,
    /// Warning diagnostics.
    Warning,
    /// Error diagnostics.
    Error,
    /// Debug diagnostics.
    Debug,
}

impl From<CliEventLevel> for Level {
    /// Converts the CLI level enum into the shared event level enum.
    fn from(value: CliEventLevel) -> Self {
        match value {
            CliEventLevel::Log => Level::Log,
            CliEventLevel::Warning => Level::Warning,
            CliEventLevel::Error => Level::Error,
            CliEventLevel::Debug => Level::Debug,
        }
    }
}

/// Top-level subcommands supported by `mlg`.
#[derive(Clone, Debug, Subcommand)]
pub enum Command {
    /// Check Mathlingua files for errors
    Check(CheckArgs),
    /// Initialize a Mathlingua collection
    Init,
    /// Print version information and quit
    Version,
    /// View rendered Mathlingua files
    View(ViewArgs),
}

/// Arguments for `mlg check`.
#[derive(Clone, Debug, Args, PartialEq, Eq)]
pub struct CheckArgs {
    /// Directories or .mlg files to check. Defaults to the collection's content directory.
    #[arg(value_name = "PATH")]
    pub paths: Vec<PathBuf>,
}

/// Arguments for `mlg view`.
#[derive(Clone, Debug, Args, PartialEq, Eq)]
pub struct ViewArgs {
    /// The local port used for the rendered viewer.
    #[arg(long, default_value_t = 3000)]
    pub port: u16,
}

// =============================================================================

#[cfg(test)]
mod tests {
    use super::{CheckArgs, Cli, CliEventAudience, CliEventLevel, Command, ViewArgs};
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

    #[test]
    fn parses_global_event_filter_flags() {
        let cli = Cli::parse_from([
            "mlg",
            "--event-audience",
            "user,system",
            "--event-level",
            "warning,error",
            "--event-markers",
            "check",
        ]);

        assert_eq!(
            cli.event_audiences,
            vec![CliEventAudience::User, CliEventAudience::System]
        );
        assert_eq!(
            cli.event_levels,
            vec![CliEventLevel::Warning, CliEventLevel::Error]
        );
        assert!(cli.event_markers);
    }

    #[test]
    fn parses_view_with_a_custom_port() {
        let cli = Cli::parse_from(["mlg", "view", "--port", "4000"]);

        assert!(matches!(
            cli.command,
            Command::View(ViewArgs { port: 4000 })
        ));
    }
}

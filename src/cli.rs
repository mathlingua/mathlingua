use crate::events::{EventAudience, EventFilter, EventLevel};
use clap::{Args, Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

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

    #[command(subcommand)]
    pub command: Command,
}

impl Cli {
    pub fn event_filter(&self) -> EventFilter {
        let audiences = if self.event_audiences.is_empty() {
            vec![EventAudience::User]
        } else {
            self.event_audiences
                .iter()
                .copied()
                .map(Into::into)
                .collect()
        };
        let levels = if self.event_levels.is_empty() {
            EventLevel::all().to_vec()
        } else {
            self.event_levels.iter().copied().map(Into::into).collect()
        };

        EventFilter::new()
            .with_audiences(audiences)
            .with_levels(levels)
            .include_markers(self.event_markers)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum CliEventAudience {
    User,
    System,
}

impl From<CliEventAudience> for EventAudience {
    fn from(value: CliEventAudience) -> Self {
        match value {
            CliEventAudience::User => EventAudience::User,
            CliEventAudience::System => EventAudience::System,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum CliEventLevel {
    Log,
    Warning,
    Error,
    Debug,
}

impl From<CliEventLevel> for EventLevel {
    fn from(value: CliEventLevel) -> Self {
        match value {
            CliEventLevel::Log => EventLevel::Log,
            CliEventLevel::Warning => EventLevel::Warning,
            CliEventLevel::Error => EventLevel::Error,
            CliEventLevel::Debug => EventLevel::Debug,
        }
    }
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

// =============================================================================

#[cfg(test)]
mod tests {
    use super::{CheckArgs, Cli, CliEventAudience, CliEventLevel, Command};
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
}

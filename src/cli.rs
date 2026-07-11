use crate::events::{Audience, EventFilter, Level};
use clap::{Args, Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(name = "mlg")]
#[command(version, about, long_about = None)]
#[command(arg_required_else_help = true)]
pub struct Cli {
    #[arg(
        long = "event-audience",
        visible_alias = "event-scope",
        value_enum,
        value_delimiter = ',',
        global = true
    )]
    pub event_audiences: Vec<CliEventAudience>,

    #[arg(long = "event-level", value_enum, value_delimiter = ',', global = true)]
    pub event_levels: Vec<CliEventLevel>,

    #[arg(long = "event-markers", global = true, default_value_t = false)]
    pub event_markers: bool,

    #[command(subcommand)]
    pub command: Command,
}

impl Cli {
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum CliEventAudience {
    User,
    System,
}

impl From<CliEventAudience> for Audience {
    fn from(value: CliEventAudience) -> Self {
        match value {
            CliEventAudience::User => Audience::User,
            CliEventAudience::System => Audience::System,
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

impl From<CliEventLevel> for Level {
    fn from(value: CliEventLevel) -> Self {
        match value {
            CliEventLevel::Log => Level::Log,
            CliEventLevel::Warning => Level::Warning,
            CliEventLevel::Error => Level::Error,
            CliEventLevel::Debug => Level::Debug,
        }
    }
}

#[derive(Clone, Debug, Subcommand)]
pub enum Command {
    Check(CheckArgs),
    #[command(hide = true)]
    Debug,
    Export(ExportArgs),
    Init,
    Lsp,
    Release(ReleaseArgs),
    Version,
    View(ViewArgs),
    #[command(name = "whte_rbt.obj", hide = true)]
    WhteRbtObj,
}

#[derive(Clone, Debug, Args, PartialEq, Eq)]
pub struct CheckArgs {
    #[arg(long, default_value_t = false)]
    pub json: bool,

    #[arg(long = "diagnostic-schema", default_value_t = false)]
    pub diagnostic_schema: bool,

    #[arg(value_name = "PATH")]
    pub paths: Vec<PathBuf>,
}

#[derive(Clone, Debug, Args, PartialEq, Eq)]
pub struct ExportArgs {
    #[arg(long, short = 'o', value_name = "DIR", default_value = "dist")]
    pub output: PathBuf,

    #[arg(long, value_name = "PATH")]
    pub base_path: Option<String>,

    #[arg(long, value_name = "DOMAIN")]
    pub cname: Option<String>,

    #[arg(long, default_value_t = false)]
    pub force: bool,
}

#[derive(Clone, Debug, Args, PartialEq, Eq)]
pub struct ReleaseArgs {
    #[arg(long, value_name = "TEXT")]
    pub summary: String,

    /// Show what the release would record without writing any files.
    #[arg(long, default_value_t = false)]
    pub dry_run: bool,

    /// Also show a diff of each item whose contents changed.
    #[arg(long, default_value_t = false)]
    pub diff: bool,
}

#[derive(Clone, Debug, Args, PartialEq, Eq)]
pub struct ViewArgs {
    #[arg(long, default_value_t = 3000)]
    pub port: u16,
}

// ===============================[ tests ]=====================================

#[cfg(test)]
mod tests {
    use super::{
        CheckArgs, Cli, CliEventAudience, CliEventLevel, Command, ExportArgs, ReleaseArgs, ViewArgs,
    };
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
            Command::Check(CheckArgs {
                json: false,
                diagnostic_schema: false,
                paths,
            }) if paths.is_empty()
        ));
    }

    #[test]
    fn parses_check_with_multiple_paths() {
        let cli = Cli::parse_from(["mlg", "check", "content", "notes/example.mlg"]);

        assert!(matches!(
            cli.command,
            Command::Check(CheckArgs {
                json: false,
                diagnostic_schema: false,
                paths,
            })
                if paths == vec![PathBuf::from("content"), PathBuf::from("notes/example.mlg")]
        ));
    }

    #[test]
    fn parses_check_json_flags() {
        let cli = Cli::parse_from(["mlg", "check", "--json", "--diagnostic-schema"]);

        assert!(matches!(
            cli.command,
            Command::Check(CheckArgs {
                json: true,
                diagnostic_schema: true,
                paths,
            }) if paths.is_empty()
        ));
    }

    #[test]
    fn parses_hidden_debug_command() {
        let cli = Cli::parse_from(["mlg", "debug"]);

        assert!(matches!(cli.command, Command::Debug));
    }

    #[test]
    fn parses_export_defaults() {
        let cli = Cli::parse_from(["mlg", "export"]);

        assert!(matches!(
            cli.command,
            Command::Export(ExportArgs { output, base_path: None, cname: None, force: false }) if output == PathBuf::from("dist")
        ));
    }

    #[test]
    fn parses_export_output_and_force() {
        let cli = Cli::parse_from(["mlg", "export", "--output", "site", "--force"]);

        assert!(matches!(
            cli.command,
            Command::Export(ExportArgs { output, base_path: None, cname: None, force: true }) if output == PathBuf::from("site")
        ));
    }

    #[test]
    fn parses_export_github_pages_options() {
        let cli = Cli::parse_from([
            "mlg",
            "export",
            "--base-path",
            "/mathlore",
            "--cname",
            "math.example.org",
        ]);

        assert!(matches!(
            cli.command,
            Command::Export(ExportArgs { output, base_path: Some(base_path), cname: Some(cname), force: false })
                if output == PathBuf::from("dist")
                    && base_path == "/mathlore"
                    && cname == "math.example.org"
        ));
    }

    #[test]
    fn parses_hidden_whte_rbt_obj_command() {
        let cli = Cli::parse_from(["mlg", "whte_rbt.obj"]);

        assert!(matches!(cli.command, Command::WhteRbtObj));
    }

    #[test]
    fn help_does_not_show_hidden_commands() {
        let mut command = Cli::command();
        let mut help = Vec::new();
        command.write_long_help(&mut help).unwrap();
        let help = String::from_utf8(help).unwrap();

        assert!(
            !help
                .lines()
                .any(|line| line.trim_start().starts_with("debug"))
        );
        assert!(!help.contains("whte_rbt.obj"));
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
    fn parses_release_with_a_summary() {
        let cli = Cli::parse_from(["mlg", "release", "--summary", "first cut"]);

        assert!(matches!(
            cli.command,
            Command::Release(ReleaseArgs { summary, dry_run: false, diff: false })
                if summary == "first cut"
        ));
    }

    #[test]
    fn parses_release_dry_run_and_diff_flags() {
        let cli = Cli::parse_from(["mlg", "release", "--summary", "x", "--dry-run", "--diff"]);

        assert!(matches!(
            cli.command,
            Command::Release(ReleaseArgs {
                dry_run: true,
                diff: true,
                ..
            })
        ));
    }

    #[test]
    fn release_requires_a_summary() {
        let result = Cli::try_parse_from(["mlg", "release"]);

        assert!(result.is_err(), "release should require --summary");
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

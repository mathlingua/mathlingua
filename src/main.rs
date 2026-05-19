use clap::Parser;
use mlg::cli::{Cli, Command, ViewArgs};
use mlg::environment::current_working_directory;
use mlg::events::{ColorMode, EventConsoleWriter, EventFilter, EventLog};
use std::path::{Path, PathBuf};
use std::process;

/// Binary entrypoint for the `mlg` executable.
///
/// The entrypoint parses CLI arguments, attaches a console event listener, runs
/// the selected subcommand, and exits with a non-zero code only after all events
/// have had a chance to render.
fn main() {
    let cli = Cli::parse();
    let filter = cli.event_filter();
    let mut event_log = EventLog::new();
    let base_path = std::env::current_dir().ok();

    attach_console_writer(&mut event_log, &filter, base_path.as_deref());

    let exit_code = match cli.command {
        Command::Check(args) => run_check(&args.paths, &mut event_log),
        Command::Init => run_init(&mut event_log),
        Command::Version => {
            mlg::version(&mut event_log);
            0
        }
        Command::View(args) => run_view(&args, &mut event_log),
    };

    if exit_code != 0 {
        process::exit(exit_code);
    }
}

/// Runs `mlg check` and converts the resulting event state into a process code.
fn run_check(paths: &[PathBuf], event_log: &mut EventLog) -> i32 {
    let cwd = current_working_directory(event_log);

    let Some(cwd) = cwd else {
        return 1;
    };

    let _ = mlg::check_in(&cwd, paths, event_log);

    if event_log.has_errors() { 1 } else { 0 }
}

/// Runs `mlg init` in the current working directory.
fn run_init(event_log: &mut EventLog) -> i32 {
    let cwd = current_working_directory(event_log);

    let Some(cwd) = cwd else {
        return 1;
    };

    if mlg::init(&cwd, event_log).is_err() || event_log.has_errors() {
        1
    } else {
        0
    }
}

/// Runs `mlg view` with the configured viewer port.
fn run_view(args: &ViewArgs, event_log: &mut EventLog) -> i32 {
    let cwd = current_working_directory(event_log);

    let Some(cwd) = cwd else {
        return 1;
    };

    if mlg::view_in(&cwd, args.port, event_log).is_err() || event_log.has_errors() {
        1
    } else {
        0
    }
}

/// Attaches console output to the event log using the selected filter.
///
/// A base path is supplied when available so diagnostics can print relative file
/// paths instead of long absolute paths.
fn attach_console_writer(event_log: &mut EventLog, filter: &EventFilter, base_path: Option<&Path>) {
    let writer = match base_path {
        Some(base_path) => EventConsoleWriter::new()
            .with_filter(filter.clone())
            .with_color_mode(ColorMode::Auto)
            .with_base_path(base_path),
        None => EventConsoleWriter::new()
            .with_filter(filter.clone())
            .with_color_mode(ColorMode::Auto),
    };

    event_log.add_listener(writer);
}

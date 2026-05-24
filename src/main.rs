use clap::Parser;
use mlg::cli::{Cli, Command};
use mlg::events::{ColorMode, EventConsoleWriter, EventFilter, EventLogListener};
use mlg::{check, init, version, view};
use std::path::{Path, PathBuf};
use std::process;

/// Binary entrypoint for the `mlg` executable.
///
/// The entrypoint parses CLI arguments, resolves the current working directory,
/// dispatches to the selected subcommand with a console event listener attached,
/// and exits with a non-zero code when the command reports failure.
fn main() {
    let cli = Cli::parse();
    let filter = cli.event_filter();
    let cwd = std::env::current_dir().ok();
    let listener_cwd = cwd.as_deref();

    let successful = match cli.command {
        Command::Check(args) => {
            let listener = console_listener(&filter, listener_cwd);
            let resolved = cwd.clone().unwrap_or_else(|| PathBuf::from("."));
            check(&resolved, &args.paths, Some(listener)).successful
        }
        Command::Init => {
            let listener = console_listener(&filter, listener_cwd);
            let resolved = cwd.clone().unwrap_or_else(|| PathBuf::from("."));
            init(&resolved, Some(listener)).successful
        }
        Command::Version => {
            let listener = console_listener(&filter, listener_cwd);
            version(Some(listener)).successful
        }
        Command::View(args) => {
            let listener = console_listener(&filter, listener_cwd);
            let resolved = cwd.clone().unwrap_or_else(|| PathBuf::from("."));
            view(&resolved, args.port, Some(listener)).successful
        }
    };

    if !successful {
        process::exit(1);
    }
}

/// Builds a boxed console event listener using the configured filter.
///
/// A base path is supplied when available so diagnostics can print relative file
/// paths instead of long absolute paths.
fn console_listener(filter: &EventFilter, base_path: Option<&Path>) -> Box<dyn EventLogListener> {
    let writer = match base_path {
        Some(base_path) => EventConsoleWriter::new()
            .with_filter(filter.clone())
            .with_color_mode(ColorMode::Auto)
            .with_base_path(base_path),
        None => EventConsoleWriter::new()
            .with_filter(filter.clone())
            .with_color_mode(ColorMode::Auto),
    };

    Box::new(writer)
}

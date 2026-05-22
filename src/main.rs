use clap::Parser;
use mlg::Mlg;
use mlg::cli::Cli;
use mlg::events::{ColorMode, EventConsoleWriter, EventFilter, EventLog};
use std::path::Path;
use std::process;

/// Binary entrypoint for the `mlg` executable.
///
/// The entrypoint parses CLI arguments, attaches a console event listener, runs
/// the selected subcommand, and exits with a non-zero code only after all events
/// have had a chance to render.
fn main() {
    let cli = Cli::parse();
    let filter = cli.event_filter();
    let mut mlg = Mlg::new();
    let base_path = mlg.working_directory().map(Path::to_path_buf);

    attach_console_writer(mlg.event_log_mut(), &filter, base_path.as_deref());

    let exit_code = mlg.run(cli.command);
    if exit_code != 0 {
        process::exit(exit_code);
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

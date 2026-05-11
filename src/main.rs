use clap::Parser;
use mlg::cli::{Cli, Command, ViewArgs};
use mlg::environment::current_working_directory;
use mlg::events::{ColorMode, EventConsoleWriter, EventFilter, EventLog};
use std::path::{Path, PathBuf};
use std::process;

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
            let _ = mlg::version(&mut event_log);
            0
        }
        Command::View(args) => run_view(&args, &mut event_log),
    };

    if exit_code != 0 {
        process::exit(exit_code);
    }
}

fn run_check(paths: &[PathBuf], event_log: &mut EventLog) -> i32 {
    let cwd = current_working_directory(event_log);

    let Some(cwd) = cwd else {
        return 1;
    };

    let _ = mlg::check_in(&cwd, paths, event_log);

    if event_log.has_errors() { 1 } else { 0 }
}

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

use clap::Parser;
use mlg::cli::{Cli, Command};
use mlg::diagnostics::{ColorMode, DiagnosticTracker};
use mlg::environment::current_working_directory;
use std::process;

fn main() {
    let cli = Cli::parse();
    let mut tracker = DiagnosticTracker::new()
        .with_live(true)
        .with_color_mode(ColorMode::Auto);

    let exit_code = match cli.command {
        Command::Check(args) => run_check(&args.paths, &mut tracker),
        Command::Init => run_init(&mut tracker),
        Command::Version => {
            mlg::version(&mut tracker);
            0
        }
        Command::View => {
            mlg::view(&mut tracker);
            0
        }
    };

    if exit_code != 0 {
        process::exit(exit_code);
    }
}

fn run_check(paths: &[std::path::PathBuf], tracker: &mut DiagnosticTracker) -> i32 {
    let Some(cwd) = current_working_directory(tracker) else {
        return 1;
    };

    tracker.set_base_path(&cwd);
    let _ = mlg::check_in(&cwd, paths, tracker);

    if tracker.has_errors() { 1 } else { 0 }
}

fn run_init(tracker: &mut DiagnosticTracker) -> i32 {
    let Some(cwd) = current_working_directory(tracker) else {
        return 1;
    };

    if mlg::init(&cwd, tracker).is_err() || tracker.has_errors() {
        1
    } else {
        0
    }
}

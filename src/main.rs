use clap::Parser;
use mlg::cli::{Cli, Command};
use mlg::diagnostics::{ColorMode, DiagnosticTracker};
use std::env;
use std::process;

fn main() {
    let cli = Cli::parse();
    let mut diagnostics = DiagnosticTracker::new()
        .with_live(true)
        .with_color_mode(ColorMode::Auto);

    let exit_code = match cli.command {
        Command::Check(args) => run_check(&args.paths, &mut diagnostics),
        Command::Init => run_init(&mut diagnostics),
        Command::Version => {
            mlg::version(&mut diagnostics);
            0
        }
        Command::View => {
            mlg::view(&mut diagnostics);
            0
        }
    };

    if exit_code != 0 {
        process::exit(exit_code);
    }
}

fn run_check(paths: &[std::path::PathBuf], diagnostics: &mut DiagnosticTracker) -> i32 {
    let cwd = match env::current_dir() {
        Ok(cwd) => cwd,
        Err(error) => {
            diagnostics.global_error(format!(
                "Failed to determine the current working directory: {error}"
            ));
            return 1;
        }
    };

    diagnostics.set_base_path(&cwd);
    let _ = mlg::check_in(&cwd, paths, diagnostics);

    if diagnostics.has_errors() { 1 } else { 0 }
}

fn run_init(diagnostics: &mut DiagnosticTracker) -> i32 {
    let cwd = match env::current_dir() {
        Ok(cwd) => cwd,
        Err(error) => {
            diagnostics.global_error(format!(
                "Failed to determine the current working directory: {error}"
            ));
            return 1;
        }
    };

    if mlg::init(&cwd, diagnostics).is_err() || diagnostics.has_errors() {
        1
    } else {
        0
    }
}

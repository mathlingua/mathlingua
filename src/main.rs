use clap::Parser;
use mlg::cli::{Cli, Command};
use mlg::diagnostics::{ColorMode, DiagnosticFormatter};
use std::env;
use std::path::PathBuf;
use std::process;

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Command::Check(args) => run_check(&args.paths),
        Command::Init => run_init(),
        Command::Version => println!("{}", mlg::version()),
        Command::View => println!("{}", mlg::view()),
    }
}

fn run_check(paths: &[PathBuf]) {
    let cwd = match env::current_dir() {
        Ok(cwd) => cwd,
        Err(error) => {
            eprintln!("Failed to determine the current working directory: {error}");
            process::exit(1);
        }
    };

    let result = mlg::check_in(&cwd, paths);

    let rendered = DiagnosticFormatter::new()
        .with_base_path(&cwd)
        .with_color_mode(ColorMode::Auto)
        .format_all(result.diagnostics.diagnostics());

    if !rendered.is_empty() {
        eprintln!("{rendered}");
    }

    if result.diagnostics.has_errors() {
        eprintln!(
            "Found {}.",
            format_diagnostic_count(result.diagnostics.diagnostics().len())
        );
        process::exit(1);
    }

    println!("{}", render_check_success(result.files_checked));
}

fn run_init() {
    let cwd = match env::current_dir() {
        Ok(cwd) => cwd,
        Err(error) => {
            eprintln!("Failed to determine the current working directory: {error}");
            process::exit(1);
        }
    };

    match mlg::init(&cwd) {
        Ok(messages) => {
            for message in messages {
                println!("{message}");
            }
        }
        Err(error) => {
            eprintln!("Failed to initialize Mathlingua collection: {error}");
            process::exit(1);
        }
    }
}

fn render_check_success(files_checked: usize) -> String {
    if files_checked == 1 {
        "Checked 1 file".to_string()
    } else {
        format!("Checked {files_checked} files")
    }
}

fn format_diagnostic_count(diagnostic_count: usize) -> String {
    if diagnostic_count == 1 {
        "1 diagnostic".to_string()
    } else {
        format!("{diagnostic_count} diagnostics")
    }
}

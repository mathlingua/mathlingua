use clap::Parser;
use mlg::cli::{Cli, Command};
use mlg::events::{ColorMode, EventConsoleWriter, EventLogListener};
use mlg::{check, init, version, view};
use std::process;

fn main() {
    let cli = Cli::parse();
    let filter = cli.event_filter();
    let cwd = std::env::current_dir().unwrap_or_else(|_| ".".into());
    let listener: Box<dyn EventLogListener> = Box::new(
        EventConsoleWriter::new()
            .with_filter(filter)
            .with_color_mode(ColorMode::Auto)
            .with_base_path(&cwd),
    );

    let successful = match cli.command {
        Command::Check(args) => check(&cwd, &args.paths, Some(listener)).successful,
        Command::Init => init(&cwd, Some(listener)).successful,
        Command::Version => version(Some(listener)).successful,
        Command::View(args) => view(&cwd, args.port, Some(listener)).successful,
    };

    if !successful {
        process::exit(1);
    }
}

use clap::Parser;
use mlg::cli::{Cli, Command};
use mlg::events::{ColorMode, EventConsoleWriter, EventLogListener};
use mlg::mlg::init;
use mlg::mlg::version;
use std::{env, process};

fn main() {
    let cli = Cli::parse();
    let filter = cli.event_filter();
    let cwd = env::current_dir().unwrap_or_else(|_| ".".into());
    let listener: Option<Box<dyn EventLogListener>> = Some(Box::new(
        EventConsoleWriter::new()
            .with_filter(filter)
            .with_color_mode(ColorMode::Auto)
            .with_base_path(&cwd),
    ));

    let successful = match cli.command {
        Command::Check(_) => true,
        Command::Init => init(&cwd, listener).successful,
        Command::Version => version(listener).successful,
        Command::View(_) => true,
    };

    if !successful {
        process::exit(1);
    }
}

use clap::Parser;
use mlg::cli::{Cli, Command};
use mlg::events::{ColorMode, EventConsoleWriter, EventLog};
use std::{env, process};

fn main() {
    let cli = Cli::parse();
    let filter = cli.event_filter();
    let cwd = env::current_dir().unwrap_or_else(|_| ".".into());
    let listener = EventConsoleWriter::new()
        .with_filter(filter)
        .with_color_mode(ColorMode::Auto)
        .with_base_path(cwd);

    let mut event_log = EventLog::new();
    event_log.add_listener(listener);

    let successful = match cli.command {
        Command::Check(_) => {
            event_log.user_log(None, "check");
            true
        }
        Command::Init => {
            event_log.user_log(None, "init");
            true
        }
        Command::Version => {
            event_log.user_log(None, "version");
            true
        }
        Command::View(_) => {
            event_log.user_log(None, "view");
            true
        }
    };

    if !successful {
        process::exit(1);
    }
}

mod check;
mod version;
mod view;

use crate::cli::Command;

pub fn run(command: Command) {
    match command {
        Command::Check => check::run(),
        Command::Version => version::run(),
        Command::View => view::run(),
    }
}

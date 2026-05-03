mod check;
mod init;
mod version;
mod view;

use crate::cli::Command;

pub fn run(command: Command) {
    match command {
        Command::Check(args) => check::run(&args.paths),
        Command::Init => init::run(),
        Command::Version => version::run(),
        Command::View => view::run(),
    }
}

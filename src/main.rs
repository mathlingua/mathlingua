use clap::Parser;
use mlg::cli::{Cli, Command};
use mlg::events::{ColorMode, EventConsoleWriter, EventFilter, EventLogListener};
use mlg::{
    check, check_diagnostics_report, check_diagnostics_schema, clean, debug, export, extract,
    format, init, lsp, release, report, version, view, whte_rbt_obj,
};
use serde::Serialize;
use std::io::{self, Write};
use std::path::Path;
use std::process;

fn main() {
    let cli = Cli::parse();
    let filter = cli.event_filter();
    let cwd = std::env::current_dir().unwrap_or_else(|_| ".".into());

    let successful = match cli.command {
        Command::Check(args) if args.diagnostic_schema => {
            write_json_stdout(&check_diagnostics_schema())
        }
        Command::Check(args) if args.json => {
            let result = check(&cwd, &args.paths, None);
            let report = check_diagnostics_report(&result, &cwd);
            write_json_stdout(&report) && result.successful
        }
        Command::Check(args) => {
            check(&cwd, &args.paths, Some(console_listener(filter, &cwd))).successful
        }
        Command::Clean => clean(&cwd, Some(console_listener(filter, &cwd))).successful,
        Command::Debug => debug(Some(console_listener(filter, &cwd))).successful,
        Command::Export(args) => {
            export(
                &cwd,
                args.base_path.as_deref(),
                args.cname.as_deref(),
                args.force,
                Some(console_listener(filter, &cwd)),
            )
            .successful
        }
        Command::Extract(args) => {
            extract(&cwd, &[args.id], Some(console_listener(filter, &cwd))).successful
        }
        Command::Format => format(&cwd, Some(console_listener(filter, &cwd))).successful,
        Command::Init => init(&cwd, Some(console_listener(filter, &cwd))).successful,
        Command::Lsp => lsp().successful,
        Command::Release(args) => {
            let released = release(
                &cwd,
                &args.summary,
                args.dry_run,
                args.diff,
                Some(console_listener(filter.clone(), &cwd)),
            )
            .successful;
            // A real release regenerates the site: remove `docs/` and re-export it
            // so the published docs reflect the release just recorded.
            released && (args.dry_run || regenerate_docs(&filter, &cwd))
        }
        Command::Report(args) => {
            report(&cwd, &args.ids, Some(console_listener(filter, &cwd))).successful
        }
        Command::Version => version(Some(console_listener(filter, &cwd))).successful,
        Command::View(args) => {
            view(&cwd, args.port, Some(console_listener(filter, &cwd))).successful
        }
        Command::WhteRbtObj => whte_rbt_obj(Some(console_listener(filter, &cwd))).successful,
    };

    if !successful {
        process::exit(1);
    }
}

/// Remove the generated `docs/` directory and rebuild it with `mlg export`, so a
/// release publishes a site matching the version just recorded.
fn regenerate_docs(filter: &EventFilter, cwd: &Path) -> bool {
    clean(cwd, Some(console_listener(filter.clone(), cwd))).successful
        && export(
            cwd,
            None,
            None,
            true,
            Some(console_listener(filter.clone(), cwd)),
        )
        .successful
}

fn console_listener(filter: EventFilter, cwd: &Path) -> Box<dyn EventLogListener> {
    Box::new(
        EventConsoleWriter::new()
            .with_filter(filter)
            .with_color_mode(ColorMode::Auto)
            .with_base_path(cwd),
    )
}

fn write_json_stdout(value: &impl Serialize) -> bool {
    let mut stdout = io::stdout().lock();
    if let Err(error) = serde_json::to_writer_pretty(&mut stdout, value) {
        eprintln!("error: Could not write JSON output: {error}");
        return false;
    }
    if let Err(error) = writeln!(stdout) {
        eprintln!("error: Could not finish JSON output: {error}");
        return false;
    }
    true
}

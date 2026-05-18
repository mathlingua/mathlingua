use crate::backend::semantic::{ParsedSourceFile, check_documents};
use crate::constants::CONFIG_FILE;
use crate::environment::current_working_directory;
use crate::events::{Audience, Event, EventLog, Level, MarkerRange};
use crate::frontend::structural::parse_document;
use crate::mlg::collection::{find_collection_root, resolve_source_files};
use crate::mlg::config::validate_config_file;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Event origin used by the check command.
const ORIGIN: &str = "mlg_check";

/// Summary returned by a completed check run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckResult {
    /// Number of source files selected for checking.
    pub files_checked: usize,
    /// Marker range bounding the events emitted by the check run.
    pub marker_range: MarkerRange,
}

/// Runs `mlg check` from the process current working directory.
pub fn check(paths: &[PathBuf], event_log: &mut EventLog) -> io::Result<CheckResult> {
    let Some(cwd) = current_working_directory(event_log) else {
        return Err(io::Error::other(
            "Failed to determine the current working directory",
        ));
    };

    Ok(check_in(&cwd, paths, event_log))
}

/// Runs `mlg check` from an explicit working directory.
///
/// The command validates collection configuration, resolves source files,
/// structurally parses each file, runs backend semantic checks, and emits a
/// success or issue-count summary.
pub fn check_in(cwd: &Path, paths: &[PathBuf], event_log: &mut EventLog) -> CheckResult {
    let begin = event_log.begin_marker("check_in", Some(ORIGIN));
    let starting_event_count = event_log.events().len();

    event_log.system_debug(
        Some(ORIGIN),
        format!("Checking {} explicit path(s)", paths.len()),
    );

    if let Some(root) = find_collection_root(cwd) {
        validate_config_file(&root.join(CONFIG_FILE), event_log, ORIGIN);
    }

    let files = resolve_source_files(cwd, paths, event_log, ORIGIN);
    let files_checked = files.len();

    let mut parsed_files = Vec::new();
    for file in files {
        if let Some(parsed_file) = parse_source_file(&file, event_log) {
            parsed_files.push(parsed_file);
        }
    }
    check_documents(&parsed_files, event_log);

    let new_events = &event_log.events()[starting_event_count..];
    let has_new_blocking_user_issues =
        new_events
            .iter()
            .filter_map(Event::as_message)
            .any(|event| {
                event.audience == Audience::User
                    && matches!(event.level, Level::Error | Level::Debug)
            });
    let new_user_issue_count = new_events
        .iter()
        .filter_map(Event::as_message)
        .filter(|event| event.audience == Audience::User && event.level != Level::Log)
        .count();

    if has_new_blocking_user_issues {
        event_log.user_log(
            Some(ORIGIN),
            format!("Found {}.", format_issue_count(new_user_issue_count)),
        );
    } else {
        event_log.user_log(Some(ORIGIN), render_check_success(files_checked));
    }

    let end = event_log.end_marker(&begin, Some(ORIGIN));

    CheckResult {
        files_checked,
        marker_range: MarkerRange::new(begin, end),
    }
}

/// Reads and structurally parses one source file for checking.
///
/// Parser diagnostics are rewritten with the file path so downstream console
/// output can report precise file locations.
fn parse_source_file(path: &Path, event_log: &mut EventLog) -> Option<ParsedSourceFile> {
    event_log.system_debug(Some(ORIGIN), format!("Parsing {}", path.display()));

    let source = match fs::read_to_string(path) {
        Ok(source) => source,
        Err(error) => {
            event_log.user_error_at_path(
                Some(ORIGIN),
                path.to_path_buf(),
                format!("Failed to read file: {error}"),
            );
            return None;
        }
    };

    let mut file_event_log = EventLog::new();
    let document = parse_document(&source, &mut file_event_log);

    for event in file_event_log.events() {
        event_log.push(event.clone().with_file_path(path.to_path_buf()));
    }

    Some(ParsedSourceFile {
        path: path.to_path_buf(),
        source,
        document,
    })
}

/// Formats the success summary for a check run.
fn render_check_success(files_checked: usize) -> String {
    if files_checked == 1 {
        "Checked 1 file".to_string()
    } else {
        format!("Checked {files_checked} files")
    }
}

/// Formats a singular or plural issue count.
fn format_issue_count(issue_count: usize) -> String {
    if issue_count == 1 {
        "1 issue".to_string()
    } else {
        format!("{issue_count} issues")
    }
}

// =============================================================================

#[cfg(test)]
mod tests;

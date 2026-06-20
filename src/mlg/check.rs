use crate::events::{EventLog, EventLogListener, MarkerRange};
use crate::mlg::util::{has_blocking_user_issues_since, no_errors_since, user_issue_count_since};
use std::path::{Path, PathBuf};

const ORIGIN: &str = "mlg_check";

pub struct CheckResult {
    pub event_log: EventLog,
    pub successful: bool,
    pub files_checked: usize,
    pub marker_range: MarkerRange,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct CheckSummary {
    pub files_checked: usize,
    pub marker_range: MarkerRange,
}

pub fn check(
    cwd: &Path,
    paths: &[PathBuf],
    listener: Option<Box<dyn EventLogListener>>,
) -> CheckResult {
    let mut event_log = EventLog::new();
    if let Some(listener) = listener {
        event_log.add_listener(listener);
    }

    let starting_event_count = event_log.events().len();
    let summary = check_in(cwd, paths, &mut event_log);
    let successful = no_errors_since(&event_log, starting_event_count);

    CheckResult {
        event_log,
        successful,
        files_checked: summary.files_checked,
        marker_range: summary.marker_range,
    }
}

pub(super) fn check_in(_cwd: &Path, paths: &[PathBuf], event_log: &mut EventLog) -> CheckSummary {
    let begin = event_log.begin_marker("check_in", Some(ORIGIN));
    let starting_event_count = event_log.events().len();

    event_log.system_debug(
        Some(ORIGIN),
        format!("Checking {} explicit path(s)", paths.len()),
    );

    let files_checked = 0;
    // TODO: Perform the actual checks here
    let has_new_blocking_user_issues =
        has_blocking_user_issues_since(event_log, starting_event_count);
    let new_user_issue_count = user_issue_count_since(event_log, starting_event_count);

    if has_new_blocking_user_issues {
        event_log.user_log(
            Some(ORIGIN),
            format!("Found {}.", format_issue_count(new_user_issue_count)),
        );
    } else {
        event_log.user_log(Some(ORIGIN), render_check_success(files_checked));
    }

    let end = event_log.end_marker(&begin, Some(ORIGIN));

    CheckSummary {
        files_checked,
        marker_range: MarkerRange::new(begin, end),
    }
}

fn render_check_success(files_checked: usize) -> String {
    if files_checked == 1 {
        "Checked 1 file".to_string()
    } else {
        format!("Checked {files_checked} files")
    }
}

fn format_issue_count(issue_count: usize) -> String {
    if issue_count == 1 {
        "1 issue".to_string()
    } else {
        format!("{issue_count} issues")
    }
}

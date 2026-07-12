use crate::backend::collection::SourceCollection;
use crate::events::{Audience, EventLocation, EventLog, EventLogListener, Level, MarkerRange};
use crate::mlg::util::{has_blocking_user_issues_since, no_errors_since, user_issue_count_since};
use serde::Serialize;
use serde_json::{Value, json};
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
        event_log.add_boxed_listener(listener);
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

pub(super) fn check_in(cwd: &Path, paths: &[PathBuf], event_log: &mut EventLog) -> CheckSummary {
    let begin = event_log.begin_marker("check_in", Some(ORIGIN));
    let starting_event_count = event_log.events().len();

    event_log.system_debug(
        Some(ORIGIN),
        format!("Checking {} explicit path(s)", paths.len()),
    );

    let mut collection = SourceCollection::load(cwd, event_log, ORIGIN);
    let diagnostic_filter = collection.diagnostic_filter(cwd, paths, event_log, ORIGIN);
    let files_checked = diagnostic_filter.selected_file_count(&collection);

    collection.run_check_passes_filtered(event_log, ORIGIN, &diagnostic_filter);

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

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckDiagnosticsReport {
    pub schema_version: u32,
    pub command: String,
    pub successful: bool,
    pub files_checked: usize,
    pub issue_count: usize,
    pub diagnostics: Vec<CheckDiagnostic>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckDiagnostic {
    pub level: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<CheckDiagnosticLocation>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckDiagnosticLocation {
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub absolute_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub span: Option<CheckDiagnosticSpan>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckDiagnosticSpan {
    pub start: CheckDiagnosticPosition,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end: Option<CheckDiagnosticPosition>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckDiagnosticPosition {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub column: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<usize>,
}

pub fn check_diagnostics_report(result: &CheckResult, cwd: &Path) -> CheckDiagnosticsReport {
    let diagnostics = result
        .event_log
        .events()
        .iter()
        .filter_map(|event| event.as_message())
        .filter(|event| event.audience == Audience::User && event.level != Level::Log)
        .map(|event| CheckDiagnostic {
            level: diagnostic_level(event.level).to_owned(),
            message: event.message.clone(),
            origin: event.origin.clone(),
            location: event
                .location
                .as_ref()
                .map(|location| diagnostic_location(location, cwd)),
        })
        .collect::<Vec<_>>();

    CheckDiagnosticsReport {
        schema_version: 1,
        command: "check".to_string(),
        successful: result.successful,
        files_checked: result.files_checked,
        issue_count: diagnostics.len(),
        diagnostics,
    }
}

pub fn check_diagnostics_schema() -> Value {
    json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "title": "MathLingua check diagnostics",
        "type": "object",
        "additionalProperties": false,
        "required": [
            "schemaVersion",
            "command",
            "successful",
            "filesChecked",
            "issueCount",
            "diagnostics"
        ],
        "properties": {
            "schemaVersion": {
                "type": "integer",
                "const": 1
            },
            "command": {
                "type": "string",
                "const": "check"
            },
            "successful": {
                "type": "boolean",
                "description": "True when mlg check completed without error-level diagnostics."
            },
            "filesChecked": {
                "type": "integer",
                "minimum": 0
            },
            "issueCount": {
                "type": "integer",
                "minimum": 0,
                "description": "The number of user-facing non-log diagnostics in diagnostics."
            },
            "diagnostics": {
                "type": "array",
                "items": {
                    "type": "object",
                    "additionalProperties": false,
                    "required": ["level", "message"],
                    "properties": {
                        "level": {
                            "type": "string",
                            "enum": ["warning", "error", "debug"]
                        },
                        "message": {
                            "type": "string"
                        },
                        "origin": {
                            "type": "string",
                            "description": "Internal checker component that produced the diagnostic."
                        },
                        "location": {
                            "type": "object",
                            "additionalProperties": false,
                            "required": ["kind"],
                            "properties": {
                                "kind": {
                                    "type": "string",
                                    "enum": ["file", "memory"]
                                },
                                "path": {
                                    "type": "string",
                                    "description": "File path relative to the invocation cwd when possible."
                                },
                                "absolutePath": {
                                    "type": "string"
                                },
                                "name": {
                                    "type": "string",
                                    "description": "In-memory source name, when available."
                                },
                                "span": {
                                    "type": "object",
                                    "additionalProperties": false,
                                    "required": ["start"],
                                    "properties": {
                                        "start": { "$ref": "#/$defs/position" },
                                        "end": { "$ref": "#/$defs/position" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        },
        "$defs": {
            "position": {
                "type": "object",
                "additionalProperties": false,
                "properties": {
                    "line": {
                        "type": "integer",
                        "minimum": 1,
                        "description": "One-based line number."
                    },
                    "column": {
                        "type": "integer",
                        "minimum": 1,
                        "description": "One-based column number."
                    },
                    "offset": {
                        "type": "integer",
                        "minimum": 0,
                        "description": "Zero-based byte offset when available."
                    }
                }
            }
        }
    })
}

fn diagnostic_level(level: Level) -> &'static str {
    match level {
        Level::Log => "log",
        Level::Warning => "warning",
        Level::Error => "error",
        Level::Debug => "debug",
    }
}

fn diagnostic_location(location: &EventLocation, cwd: &Path) -> CheckDiagnosticLocation {
    match location {
        EventLocation::File { path, span } => CheckDiagnosticLocation {
            kind: "file".to_string(),
            path: Some(display_path(path, cwd)),
            absolute_path: Some(absolute_path(path, cwd)),
            name: None,
            span: span.as_ref().map(diagnostic_span),
        },
        EventLocation::InMemory { name, span } => CheckDiagnosticLocation {
            kind: "memory".to_string(),
            path: None,
            absolute_path: None,
            name: name.clone(),
            span: span.as_ref().map(diagnostic_span),
        },
    }
}

fn diagnostic_span(span: &crate::events::EventSpan) -> CheckDiagnosticSpan {
    CheckDiagnosticSpan {
        start: diagnostic_position(&span.start),
        end: span.end.as_ref().map(diagnostic_position),
    }
}

fn diagnostic_position(position: &crate::events::EventPosition) -> CheckDiagnosticPosition {
    CheckDiagnosticPosition {
        line: position.row.map(|row| row + 1),
        column: position.column.map(|column| column + 1),
        offset: position.offset,
    }
}

fn display_path(path: &Path, cwd: &Path) -> String {
    if let Some(relative) = relative_path(path, cwd) {
        return relative;
    }
    if let Ok(canonical_cwd) = cwd.canonicalize() {
        if let Some(relative) = relative_path(path, &canonical_cwd) {
            return relative;
        }
    }

    path.display().to_string()
}

fn relative_path(path: &Path, base: &Path) -> Option<String> {
    path.strip_prefix(base)
        .ok()
        .filter(|relative| !relative.as_os_str().is_empty())
        .map(display_relative_path)
}

fn absolute_path(path: &Path, cwd: &Path) -> String {
    let path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        cwd.join(path)
    };

    path.canonicalize().unwrap_or(path).display().to_string()
}

fn display_relative_path(path: &Path) -> String {
    path.strip_prefix("content")
        .unwrap_or(path)
        .display()
        .to_string()
}

fn format_issue_count(issue_count: usize) -> String {
    if issue_count == 1 {
        "1 issue".to_string()
    } else {
        format!("{issue_count} issues")
    }
}

// ===============================[ tests ]=====================================

#[cfg(test)]
mod tests {
    use super::{check, check_diagnostics_report, check_diagnostics_schema, check_in};
    use crate::backend::config::default_config_contents;
    use crate::events::{Audience, Event, EventLog, Level};
    use std::fs;
    use std::io;
    use std::path::{Path, PathBuf};

    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    pub(super) static NEXT_TEST_DIR_ID: AtomicUsize = AtomicUsize::new(0);

    pub(super) fn user_events(event_log: &EventLog) -> Vec<Event> {
        event_log
            .events()
            .iter()
            .filter_map(|event| {
                event
                    .as_message()
                    .and_then(|message| (message.audience == Audience::User).then(|| event.clone()))
            })
            .collect()
    }

    pub(super) fn has_user_error_at(
        event_log: &EventLog,
        path: &Path,
        row: usize,
        column: usize,
        message: &str,
    ) -> bool {
        event_log
            .events()
            .iter()
            .filter_map(Event::as_message)
            .any(|event| {
                event.message == message
                    && event.location.as_ref().is_some_and(|location| {
                        matches!(
                            location,
                            crate::events::EventLocation::File {
                                path: event_path,
                                span: Some(span)
                            } if event_path == path
                                && span.start.row == Some(row)
                                && span.start.column == Some(column)
                        )
                    })
            })
    }

    pub(super) struct TestDir {
        pub(super) path: PathBuf,
    }

    impl TestDir {
        pub(super) fn new() -> Self {
            let unique = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let id = NEXT_TEST_DIR_ID.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                "mlg-check-test-{}-{}-{}",
                std::process::id(),
                unique,
                id
            ));
            fs::create_dir(&path).unwrap();
            Self { path }
        }

        pub(super) fn path(&self) -> &Path {
            &self.path
        }
    }

    impl Drop for TestDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    fn write_mlg_fixture(path: &Path, source: &str) -> io::Result<()> {
        fs::write(path, unindent_mlg_fixture(source))
    }

    fn unindent_mlg_fixture(source: &str) -> String {
        let mut lines = source.lines().collect::<Vec<_>>();
        while lines.last().is_some_and(|line| line.trim().is_empty()) {
            lines.pop();
        }

        let indentation = lines
            .iter()
            .skip(1)
            .filter(|line| !line.trim().is_empty())
            .map(|line| {
                line.chars()
                    .take_while(|ch| matches!(ch, ' ' | '\t'))
                    .map(char::len_utf8)
                    .sum::<usize>()
            })
            .min()
            .unwrap_or(0);

        let mut output = String::new();
        for (index, line) in lines.iter().enumerate() {
            if index > 0 {
                output.push('\n');
            }

            if index == 0 {
                output.push_str(line);
            } else if line.trim().is_empty() {
                continue;
            } else {
                output.push_str(&line[indentation..]);
            }
        }
        output.push('\n');
        output
    }

    #[test]
    fn check_without_arguments_uses_collection_root_from_a_nested_directory() {
        let temp_dir = TestDir::new();
        let root = temp_dir.path().join("repo");
        let nested_cwd = root.join("content/algebra");

        fs::create_dir_all(&nested_cwd).unwrap();
        fs::write(root.join("mlg.json"), default_config_contents()).unwrap();
        fs::write(root.join("content/sets.mlg"), "Title: \"Sets\"\n").unwrap();
        fs::write(nested_cwd.join("groups.mlg"), "Title: \"Groups\"\n").unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(&nested_cwd, &[], &mut event_log);

        assert_eq!(result.files_checked, 2);
        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 2 files").with_origin("mlg_check")]
        );
        assert!(
            event_log
                .events_between(&result.marker_range.begin, &result.marker_range.end)
                .is_some()
        );
    }

    #[test]
    fn check_without_arguments_uses_command_root_when_not_in_a_collection() {
        let temp_dir = TestDir::new();

        let mut event_log = EventLog::new();
        let result = check_in(temp_dir.path(), &[], &mut event_log);

        assert_eq!(result.files_checked, 0);
        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 0 files").with_origin("mlg_check")]
        );
    }

    #[test]
    fn check_with_directory_argument_processes_mlg_files_recursively() {
        let temp_dir = TestDir::new();
        let docs = temp_dir.path().join("docs/logic");

        fs::create_dir_all(&docs).unwrap();
        fs::write(docs.join("intro.mlg"), "Title: \"Intro\"\n").unwrap();
        fs::write(docs.join("notes.txt"), "ignore me").unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(temp_dir.path(), &[PathBuf::from("docs")], &mut event_log);

        assert_eq!(result.files_checked, 1);
        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 1 file").with_origin("mlg_check")]
        );
    }

    #[test]
    fn check_generates_missing_top_level_ids_before_checking() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("intro.mlg");

        fs::write(&file, "Title: \"Intro\"\n\nText: \"Body\"\n").unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("intro.mlg")],
            &mut event_log,
        );
        let updated = fs::read_to_string(&file).expect("expected updated source");
        let ids = updated
            .lines()
            .filter_map(|line| {
                line.strip_prefix("Id: \"")
                    .and_then(|value| value.strip_suffix('"'))
            })
            .collect::<Vec<_>>();

        assert_eq!(result.files_checked, 1);
        assert_eq!(ids.len(), 2);
        assert_ne!(ids[0], ids[1]);
        assert!(ids.iter().all(|id| id.len() == 36));
        assert!(ids.iter().all(|id| &id[14..15] == "4"));
        assert!(
            ids.iter()
                .all(|id| matches!(id.as_bytes()[19] as char, '8' | '9' | 'a' | 'b'))
        );
        assert!(!updated.contains("------------------------------------------"));
        assert!(updated.contains("Title: \"Intro\"\nId: \""));
        assert!(updated.contains("Text: \"Body\"\nId: \""));
        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 1 file").with_origin("mlg_check")]
        );
    }

    #[test]
    fn check_reports_duplicate_top_level_ids() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("intro.mlg");
        let id = "18582990-701a-40d3-8ce3-ae12bd08a561";

        fs::write(
            &file,
            format!(
                "Title: \"One\"\n------------------------------------------\nId: \"{id}\"\n\nTitle: \"Two\"\n------------------------------------------\nId: \"{id}\"\n"
            ),
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("intro.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert!(
            event_log
                .events()
                .iter()
                .filter_map(Event::as_message)
                .any(|event| event
                    .message
                    .starts_with("Duplicate Id `18582990-701a-40d3-8ce3-ae12bd08a561`"))
        );
        assert_eq!(
            user_events(&event_log)
                .last()
                .cloned()
                .expect("expected summary event"),
            Event::user_log("Found 1 issue.").with_origin("mlg_check")
        );
    }

    #[test]
    fn check_reports_malformed_top_level_ids() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("intro.mlg");

        fs::write(
            &file,
            "Title: \"Intro\"\n------------------------------------------\nId: \"not-a-uuid\"\n",
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("intro.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert!(
            event_log
                .events()
                .iter()
                .filter_map(Event::as_message)
                .any(|event| event.message == "`Id:` value `not-a-uuid` must be a UUID")
        );
        assert_eq!(
            user_events(&event_log)
                .last()
                .cloned()
                .expect("expected summary event"),
            Event::user_log("Found 1 issue.").with_origin("mlg_check")
        );
    }

    #[test]
    fn check_ignores_mathlingua_examples_inside_text_code_fences() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("intro.mlg");

        write_mlg_fixture(
            &file,
            r#"
            Text: "
            Example:

            ```mlg
            [\function:on{A}:to{B}]
            Describes: f(x__) ::= y_
            Documented:
            . called: \"function\"
            Id: \"123\"
            ```
            "
            Id: "fce2c58a-edeb-4af2-b2a3-c1f67b8d31d0"
            "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("intro.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 1 file").with_origin("mlg_check")]
        );
    }

    #[test]
    fn check_with_empty_content_directory_succeeds() {
        let temp_dir = TestDir::new();
        let root = temp_dir.path().join("repo");

        fs::create_dir_all(root.join("content")).unwrap();
        fs::write(root.join("mlg.json"), default_config_contents()).unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(&root, &[], &mut event_log);

        assert_eq!(result.files_checked, 0);
        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 0 files").with_origin("mlg_check")]
        );
    }

    #[test]
    fn check_reports_config_validation_errors() {
        let temp_dir = TestDir::new();
        let root = temp_dir.path().join("repo");
        fs::create_dir_all(root.join("content")).unwrap();
        fs::write(root.join("mlg.json"), r#"{"name": 5}"#).unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(&root, &[], &mut event_log);

        assert_eq!(result.files_checked, 0);
        let messages: Vec<&str> = event_log
            .events()
            .iter()
            .filter_map(Event::as_message)
            .filter(|message| message.audience == Audience::User && message.level == Level::Error)
            .map(|message| message.message.as_str())
            .collect();
        assert_eq!(
            messages,
            vec![
                "mlg.json field \"name\" must be a string",
                "mlg.json is missing required field \"version\"",
            ]
        );
        assert!(event_log.has_errors());
    }

    #[test]
    fn check_reports_proto_events_for_invalid_files() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("broken.mlg");

        fs::write(&file, "Defines: 'f(x_)'\n").unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("broken.mlg")],
            &mut event_log,
        );
        let events = event_log.events();

        assert_eq!(result.files_checked, 1);
        assert!(events.iter().filter_map(Event::as_message).any(|event| {
            event.location.as_ref().is_some_and(|location| {
                matches!(
                    location,
                    crate::events::EventLocation::File { path, .. }
                        if *path == file.canonicalize().unwrap()
                )
            }) && event.message == "Single-quoted formulations are not allowed"
        }));
        assert_eq!(
            user_events(&event_log)
                .last()
                .cloned()
                .expect("expected summary event"),
            Event::user_log("Found 2 issues.").with_origin("mlg_check")
        );
    }

    #[test]
    fn check_diagnostics_report_contains_structured_user_issues() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("invalid.mlg");

        fs::write(&file, "Defines: 'f(x_)'\n").unwrap();

        let result = check(temp_dir.path(), &[PathBuf::from("invalid.mlg")], None);
        let report = check_diagnostics_report(&result, temp_dir.path());
        let value = serde_json::to_value(&report).expect("expected report to serialize");

        assert!(!report.successful);
        assert_eq!(report.files_checked, 1);
        assert_eq!(report.issue_count, report.diagnostics.len());
        assert!(report.issue_count > 0);
        assert_eq!(value["schemaVersion"], 1);
        assert_eq!(value["command"], "check");
        assert_eq!(value["diagnostics"][0]["level"], "error");
        assert_eq!(value["diagnostics"][0]["location"]["kind"], "file");
        assert_eq!(value["diagnostics"][0]["location"]["path"], "invalid.mlg");
        assert!(
            value["diagnostics"][0]["location"]["absolutePath"]
                .as_str()
                .is_some_and(|path| path.ends_with("invalid.mlg"))
        );
    }

    #[test]
    fn check_diagnostics_report_omits_content_prefix_from_paths() {
        let temp_dir = TestDir::new();
        let root = temp_dir.path().join("repo");
        let content = root.join("content/sets");
        let file = content.join("invalid.mlg");

        fs::create_dir_all(&content).unwrap();
        fs::write(root.join("mlg.json"), default_config_contents()).unwrap();
        fs::write(&file, "Defines: 'f(x_)'\n").unwrap();

        let result = check(&root, &[], None);
        let report = check_diagnostics_report(&result, &root);
        let value = serde_json::to_value(&report).expect("expected report to serialize");

        assert!(!report.successful);
        assert_eq!(
            value["diagnostics"][0]["location"]["path"],
            "sets/invalid.mlg"
        );
        assert!(
            value["diagnostics"][0]["location"]["absolutePath"]
                .as_str()
                .is_some_and(|path| path.ends_with("content/sets/invalid.mlg"))
        );
    }

    #[test]
    fn check_diagnostics_schema_describes_report_shape() {
        let schema = check_diagnostics_schema();

        assert_eq!(
            schema["$schema"],
            "https://json-schema.org/draft/2020-12/schema"
        );
        assert_eq!(schema["properties"]["schemaVersion"]["const"], 1);
        assert_eq!(schema["properties"]["command"]["const"], "check");
        assert_eq!(
            schema["properties"]["diagnostics"]["items"]["properties"]["location"]["properties"]["span"]
                ["properties"]["start"]["$ref"],
            "#/$defs/position"
        );
    }

    #[test]
    fn check_with_explicit_file_processes_collection_but_filters_diagnostics() {
        let temp_dir = TestDir::new();
        let root = temp_dir.path().join("repo");
        let selected = root.join("selected.mlg");
        let hidden = root.join("hidden.mlg");

        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("mlg.json"), default_config_contents()).unwrap();
        write_mlg_fixture(
            &selected,
            r#"Theorem:
    given: x is \thing
    then:
    . x = x
    "#,
        )
        .unwrap();
        write_mlg_fixture(
            &hidden,
            r#"[\thing]
    Describes: value
    Documented:
    . written: "\operatorname{thing}"

    Theorem:
    then:
    . y is \missing
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(&root, &[PathBuf::from("selected.mlg")], &mut event_log);

        assert_eq!(result.files_checked, 1);
        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 1 file").with_origin("mlg_check")]
        );
    }

    #[test]
    fn check_result_markers_bound_the_check_events() {
        let temp_dir = TestDir::new();
        let root = temp_dir.path().join("repo");

        fs::create_dir_all(root.join("content")).unwrap();
        fs::write(root.join("mlg.json"), default_config_contents()).unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(&root, &[], &mut event_log);
        let range_events = event_log
            .events_between(&result.marker_range.begin, &result.marker_range.end)
            .expect("expected event range");

        assert!(
            range_events
                .iter()
                .filter_map(|event| event.as_message())
                .any(|event| event.level == Level::Log && event.message == "Checked 0 files")
        );
    }

    #[test]
    fn check_rejects_non_mlg_files_when_given_explicitly() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("notes.txt");

        fs::write(&file, "not mathlingua").unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("notes.txt")],
            &mut event_log,
        );
        let user_events = user_events(&event_log);

        assert_eq!(result.files_checked, 0);
        assert_eq!(user_events.len(), 2);
        assert_eq!(
            user_events[0],
            Event::user_path_error(file.canonicalize().unwrap(), "Not a .mlg file")
                .with_origin("mlg_check")
        );
        assert_eq!(
            user_events[1],
            Event::user_log("Found 1 issue.").with_origin("mlg_check")
        );
    }

    #[test]
    fn check_reports_structural_and_formulation_events_for_invalid_files() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("broken-structural.mlg");

        fs::write(&file, "[\\function]\nDefines: x |plus|\n").unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("broken-structural.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert!(
            event_log
                .events()
                .iter()
                .filter_map(Event::as_message)
                .any(|event| {
                    event.location.as_ref().is_some_and(|location| {
                        matches!(
                            location,
                            crate::events::EventLocation::File { path, .. }
                                if *path == file.canonicalize().unwrap()
                        )
                    }) && event.message.starts_with("Invalid Defines formulation:")
                })
        );
        assert!(
            event_log
                .events()
                .iter()
                .filter_map(Event::as_message)
                .all(|event| !event.message.contains("UnrecognizedToken")
                    && !event.message.contains("token:"))
        );
        assert!(user_events(&event_log).last().is_some_and(
            |event| event == &Event::user_log("Found 1 issue.").with_origin("mlg_check")
        ));
    }

    #[test]
    fn check_reports_duplicate_command_signatures_across_definition_kinds() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("duplicates.mlg");

        write_mlg_fixture(
            &file,
            r#"[\function{A, B}]
    Defines: A ::= B "defines" B
    Documented:
    . [docs.called]
      written:
      . "\operatorname{function}"

    [\function{A}]
    Theorem:
    then:
    . A = A
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("duplicates.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        let canonical_file = file.canonicalize().unwrap();
        assert!(has_user_error_at(
            &event_log,
            &canonical_file,
            8,
            1,
            &format!(
                "Duplicate command signature `\\function` in Theorem; previously defined as Defines in {}:1:2",
                canonical_file.display()
            )
        ));
        assert!(event_log.has_errors());
    }

    #[test]
    fn check_accepts_relation_item_with_declared_subjects() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("relation.mlg");

        write_mlg_fixture(
            &file,
            r#"Relation:
between: a is \\expression
and: b is \\expression
when:
. a = b
means: a = b
"#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("relation.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 1 file").with_origin("mlg_check")]
        );
    }

    #[test]
    fn check_reports_undeclared_symbol_in_relation_means() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("relation-scope.mlg");

        write_mlg_fixture(
            &file,
            r#"Relation:
between: a is \\expression
and: b is \\expression
means: c = c
"#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("relation-scope.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert!(
            user_events(&event_log).iter().any(|event| event
                .as_message()
                .is_some_and(|message| message.message.contains("Unrecognized symbol")
                    && message.message.contains('c'))),
            "expected an unrecognized-symbol error for `c`: {:#?}",
            user_events(&event_log)
        );
    }

    #[test]
    fn check_reports_references_to_undefined_command_signatures() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("undefined.mlg");

        write_mlg_fixture(
            &file,
            r#"[\function:on{A}:to{B}]
    Defines: A ::= B "defines" B
    when: A, B is \\opaque
    Documented:
    . [docs.called]
      written:
      . "\operatorname{function}"

    Theorem:
    then:
    . x is \function{A, B}
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("undefined.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert!(has_user_error_at(
            &event_log,
            &file.canonicalize().unwrap(),
            11,
            7,
            "Undefined command signature `\\function`"
        ));
        assert!(event_log.has_errors());
    }

    #[test]
    fn check_reports_command_argument_shape_mismatches() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("arity.mlg");

        write_mlg_fixture(
            &file,
            r#"[\foo{A, B}(x)]
    Defines: A "defines" B
    Documented:
    . [docs.called]
      written:
      . "\operatorname{foo}"

    Theorem:
    then:
    . y is \foo{A}(x, z)
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("arity.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert!(has_user_error_at(
            &event_log,
            &file.canonicalize().unwrap(),
            10,
            7,
            "Command signature `\\foo` expects argument shape `{2}(1)` but found `{1}(2)`"
        ));
        assert!(event_log.has_errors());
    }

    #[test]
    fn check_accepts_defined_command_references_with_matching_argument_shape() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("valid-reference.mlg");

        write_mlg_fixture(
            &file,
            r#"[\thing]
    Describes: value
    Documented:
    . written: "\operatorname{thing}"

    [\foo{A, B}(x)]
    Defines: A "defines" B
    when: A, B is \thing
    Documented:
    . [docs.called]
      written:
      . "\operatorname{foo}"

    Theorem:
    given:
    . y, C, D, z is \thing
    then:
    . y is \foo{C, D}(z)
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("valid-reference.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 1 file").with_origin("mlg_check")]
        );
    }

    #[test]
    fn check_accepts_command_references_that_omit_defined_paren_arguments() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("optional-parens.mlg");

        write_mlg_fixture(
            &file,
            r#"[\thing]
    Describes: value
    Documented:
    . written: "\operatorname{thing}"

    [\some.function{A}(x, y)]
    Defines: A is \thing
    when: A is \thing
    Documented:
    . [docs.called]
      written:
      . "\operatorname{someFunction}"

    Theorem:
    given:
    . f, X, g, a, b is \thing
    then:
    . f is \some.function{X}
    . g is \some.function{X}(a, b)
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("optional-parens.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 1 file").with_origin("mlg_check")]
        );
    }

    #[test]
    fn check_accepts_composed_refined_command_references_in_given_sections() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("refined-list.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Describes: X
    Enables:
    . capability: x_ "in" X :-> \\abstract
    Documented:
    . written: "\operatorname{set}"

    [\function:on{A}:to{B}]
    Describes: f(x__)
    when: A, B is \set
    Documented:
    . written: "A? \to B?"

    [\(bounded)::function:on{A}:to{B}]
    Refines: f(x__)
    when: A, B is \set
    Documented:
    . adjective: "bounded"
    . written: "\operatorname{bounded}"

    [\(continuous)::function:on{A}:to{B}]
    Refines: f(x__)
    when: A, B is \set
    Documented:
    . adjective: "continuous"
    . written: "\operatorname{continuous}"

    Theorem:
    given:
    . A, B is \set
    . f is \(continuous, bounded)::function:on{A}:to{B}
    then: f is? \function:on{A}:to{B}
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("refined-list.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 1 file").with_origin("mlg_check")]
        );
    }

    #[test]
    fn check_accepts_refines_adjectives_and_optional_expression_tails() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("refined-adjective.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Describes: X
    Enables:
    . capability: x_ "in" X :-> \\abstract
    Documented:
    . called: "set"
    . written: "\operatorname{set}"

    [\function:?on{A}:?to{B}]
    Describes: f(x__)
    when: A, B is \set
    Documented:
    . called: "function"
    . written: "f?"

    [\(injective)::function:?on{A}:?to{B}]
    Refines: f(x__)
    when: A, B is \set
    satisfies:
    . forAll: x1, x2 "in" A
      then:
      . if: f(x1) = f(x2)
        then: x1 = x2
    Documented:
    . adjective: "injective"
    . written: "\operatorname{injective}"

    [\(surjective)::function:?on{A}:?to{B}]
    Refines: f(x__)
    when: A, B is \set
    Documented:
    . adjective: "surjective"
    . written: "\operatorname{surjective}"

    Theorem:
    given: f is \(injective, surjective)::function
    then: f is? \(injective)::function
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("refined-adjective.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 1 file").with_origin("mlg_check")]
        );
    }

    #[test]
    fn check_uses_describes_function_signature_specifies() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("function-signature-specifies.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Describes: X
    Enables:
    . capability: x_ "in" X :-> \\abstract
    Documented:
    . written: "\operatorname{set}"

    [\function:?on{A}:?to{B}]
    Describes: f(x__) ::= y_
    when: A, B is \set
    specifies:
    . x__ "in" A
    . y_ "in" B
    Documented:
    . called: "function"
    . written: "f?"

    [\ternary.function:?on{A}:?to{B}]
    Describes: g(x_, y_, z_) ::= w_
    when: A, B is \set
    specifies:
    . x_ "in" A
    . y_ "in" A
    . z_ "in" A
    . w_ "in" B
    Documented:
    . called: "ternary function"
    . written: "g?"

    [f \.function.compose./ g]
    Defines: h(x__) := f(g(x__)) is \function:on{A}:to{C}
    using: A, B, C is \set
    when:
    . g is \function:on{A}:to{B}
    . f is \function:on{B}:to{C}
    Documented:
    . written: "f? \circ g?"
    . called: "function composition"

    Theorem:
    given:
    . A, B is \set
    . x "in" A
    . f is \function:on{A}:to{B}
    then: f(x) "in" B

    Theorem:
    given:
    . A, B is \set
    . (a, b) "in" A
    . f is \function:on{A}:to{B}
    then: f(a, b) "in" B

    Theorem:
    given:
    . A, B is \set
    . a, b, c "in" A
    . g is \ternary.function:on{A}:to{B}
    then: g(a, b, c) "in" B
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("function-signature-specifies.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 1 file").with_origin("mlg_check")]
        );
    }

    #[test]
    fn check_requires_used_optional_describes_parameters_in_when() {
        let temp_dir = TestDir::new();
        let file = temp_dir
            .path()
            .join("used-optional-describes-parameter.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Describes: X
    Enables:
    . capability: x_ "in" X :-> \\abstract
    Documented:
    . written: "\operatorname{set}"

    [\function:?on{A}:?to{B}]
    Describes: f(x__) ::= y_
    when: A is \set
    specifies:
    . x__ "in" A
    . y_ "in" B
    satisfies:
    . forAll: x "in" A
      then:
      . existsUnique: y "in" B
        suchThat: f(x) = y
    Documented:
    . called: "function on $A?$ to $B?$"
    . written: "f? \: : \: A? \rightarrow B?"
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("used-optional-describes-parameter.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        let messages = user_events(&event_log)
            .iter()
            .filter_map(|event| event.as_message().map(|message| message.message.clone()))
            .collect::<Vec<_>>();
        assert!(
            messages
                .iter()
                .any(|message| message.contains("Missing `when:` requirement for parameter `B`")),
            "{messages:#?}"
        );
        assert!(event_log.has_errors());
    }

    #[test]
    fn check_applies_refines_extends_to_dynamic_refined_base() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("dynamic-refined-base.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Describes: X
    Documented:
    . written: "\operatorname{set}"

    [\function]
    Describes: f(x__)
    Documented:
    . written: "\operatorname{function}"

    [\bounded.function]
    Describes: f(x__)
    extends: f is \function
    Documented:
    . written: "\operatorname{boundedFunction}"

    [\(injective)::function]
    Refines: f(x__)
    Documented:
    . adjective: "injective"

    [\(surjective)::function]
    Refines: f(x__)
    Documented:
    . adjective: "surjective"

    [\(bijective)::function]
    Refines: f(x__)
    extends: f is \(injective, surjective)::[[f]]
    Documented:
    . adjective: "bijective"

    Theorem:
    given: f is \bounded.function
    where: f is? \(bijective)::bounded.function
    then: f is? \(injective)::bounded.function
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("dynamic-refined-base.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 1 file").with_origin("mlg_check")]
        );
    }

    #[test]
    fn check_uses_requires_capabilities_for_type_provided_specs() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("requires-capability.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Describes: X
    Requires:
    . capability: x_ "in" X :-> \\abstract
      written: "x_? \in X?"
    Documented:
    . written: "\operatorname{set}"

    Theorem:
    given:
    . X is \set
    . x is \\opaque
    then: x "in" X
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("requires-capability.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 1 file").with_origin("mlg_check")]
        );
    }

    #[test]
    fn check_validates_requires_definition_against_defines_outputs() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("requires-definition.mlg");

        write_mlg_fixture(
            &file,
            r#"[\natural]
    Describes: n
    Requires:
    . definition: \natural.0 is \natural
    Documented:
    . written: "\mathbb{N}"

    [\natural.0]
    Defines: n is \natural
    Documented:
    . written: "0"
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("requires-definition.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 1 file").with_origin("mlg_check")]
        );
    }

    #[test]
    fn check_builtin_type_predicate_recognizes_describes_only() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("builtin-type.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Describes: X
    Documented:
    . written: "\operatorname{set}"

    [\sqrt]
    Defines: y is \set
    Documented:
    . written: "\sqrt{}"

    Theorem:
    then: \set is? \\type

    Theorem:
    then: \sqrt is_not? \\type
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("builtin-type.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 1 file").with_origin("mlg_check")]
        );
    }

    #[test]
    fn check_reports_undeclared_optional_expression_tail_arguments() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("optional-expression-tail.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Describes: X
    Documented:
    . written: "\operatorname{set}"

    [\tag:?on{A}]
    Describes: x
    when: A is \set
    Documented:
    . called: "tag"
    . written: "\operatorname{tag}"

    Theorem:
    given: x is \tag:?on{A}
    then: x is? \tag
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("optional-expression-tail.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert!(user_events(&event_log).iter().any(|event| {
            event
                .as_message()
                .is_some_and(|message| message.message == "Unrecognized symbol `A`")
        }));
        assert!(event_log.has_errors());
    }

    #[test]
    fn check_counts_comma_separated_refines_when_requirements() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("refined-comma-when.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Describes: X
    Enables:
    . capability: x_ "in" X :-> \\abstract
    Documented:
    . written: "\operatorname{set}"

    [\function:?on{A}:?to{B}]
    Describes: f(x__)
    when: A, B is \set
    Documented:
    . called: "function"

    [\(injective)::function:?on{A}:?to{B}]
    Refines: f(x__)
    when: A, B is \set
    satisfies:
    . forAll: x1, x2 "in" A
      then:
      . if: f(x1) = f(x2)
        then: x1 = x2
    Documented:
    . adjective: "injective"
    Id: "8ae265b6-2112-4576-9976-6ba3beb95829"
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("refined-comma-when.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 1 file").with_origin("mlg_check")]
        );
    }

    #[test]
    fn check_counts_comma_separated_states_when_requirements() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("states-comma-when.mlg");

        write_mlg_fixture(
            &file,
            r#"[P \.and./ Q]
    States:
    when: P, Q is \\statement
    that:
    . allOf:
      . P
      . Q
    Documented:
    . written: "P? \text{ and } Q?"
    Id: "da152255-eeb1-498e-9ef4-f0ee017406d2"
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("states-comma-when.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 1 file").with_origin("mlg_check")]
        );
    }

    #[test]
    fn check_counts_comma_separated_when_requirements_in_collections() {
        let temp_dir = TestDir::new();
        let root = temp_dir.path();
        let content = root.join("content");
        fs::create_dir(&content).unwrap();
        fs::write(root.join("mlg.json"), default_config_contents()).unwrap();
        write_mlg_fixture(
            &content.join("logic.mlg"),
            r#"Title: "Logical Background"
    Id: "66a3817c-cca1-4afd-9e0c-f842963cc5e1"

    Text: "
    Second-order logic will serve as the logical foundation for
    the mathematics in this work.
    "
    Id: "b213d859-14fe-4612-8c8f-a6e38cc23c0e"

    [P \.and./ Q]
    States:
    when: P, Q is \\statement
    that:
    . allOf:
      . P
      . Q
    Documented:
    . written: "P? \text{ and } Q?"
    Id: "da152255-eeb1-498e-9ef4-f0ee017406d2"

    [P \.or./ Q]
    States:
    when: P, Q is \\statement
    that:
    . anyOf:
      . P
      . Q
    Documented:
    . written: "P? \text{ or } Q?"
    Id: "93149456-ff84-40af-8c41-b06906405ffa"
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(root, &[], &mut event_log);

        assert_eq!(result.files_checked, 1);
        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 1 file").with_origin("mlg_check")]
        );
    }

    #[test]
    fn check_treats_equality_as_tighter_than_infix_commands() {
        let temp_dir = TestDir::new();
        let root = temp_dir.path();
        let content = root.join("content");
        fs::create_dir(&content).unwrap();
        fs::write(root.join("mlg.json"), default_config_contents()).unwrap();
        write_mlg_fixture(
            &content.join("operations.mlg"),
            r#"[\set]
    Describes: X
    Documented:
    . called: "set"
    Id: "059126b9-dc83-41a2-aa1c-84f8e942f8d6"

    [P \.or./ Q]
    States:
    when: P, Q is \\statement
    that:
    . anyOf:
      . P
      . Q
    Documented:
    . written: "P? \text{ or } Q?"
    Id: "93149456-ff84-40af-8c41-b06906405ffa"

    [\pair:of{a}:and{b}]
    Defines: P := {x_ : x_ is \set | x = a \.or./ x = b} is \set
    when: a, b is \set
    Documented:
    . called: "pair of $a?$ and $b?$"
    . written: "\{a?, b?\}"
    Id: "10faf153-d005-4feb-b620-c31589aefea1"
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(root, &[], &mut event_log);

        assert_eq!(result.files_checked, 1);
        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 1 file").with_origin("mlg_check")]
        );
    }

    #[test]
    fn check_accepts_builtin_clause_commands_with_scoped_arguments() {
        let temp_dir = TestDir::new();
        let root = temp_dir.path();
        let content = root.join("content");
        fs::create_dir(&content).unwrap();
        fs::write(root.join("mlg.json"), default_config_contents()).unwrap();
        write_mlg_fixture(
            &content.join("builtin-clauses.mlg"),
            r#"[\real]
    Describes: x
    Documented:
    . called: "real"
    Id: "f1a2b3c4-1111-4a22-8333-111111111111"

    [\natural]
    Describes: n
    Documented:
    . called: "natural"
    Id: "f1a2b3c4-2222-4a22-8333-222222222222"

    Theorem:
    given: x is \real
    then:
    . \\and{x = x; \\forAll{y is \real}:then{\\exists{a, b is \real; n is \natural}:suchThat{x = y}}}
    Id: "f1a2b3c4-3333-4a22-8333-333333333333"
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(root, &[], &mut event_log);

        assert_eq!(result.files_checked, 1);
        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 1 file").with_origin("mlg_check")]
        );
    }

    #[test]
    fn check_accepts_builtin_clause_commands_inside_set_predicates() {
        let temp_dir = TestDir::new();
        let root = temp_dir.path();
        let content = root.join("content");
        fs::create_dir(&content).unwrap();
        fs::write(root.join("mlg.json"), default_config_contents()).unwrap();
        write_mlg_fixture(
            &content.join("builtin-set-predicate.mlg"),
            r#"[\set]
    Describes: S
    Documented:
    . called: "set"
    Id: "f1a2b3c4-4444-4a22-8333-444444444444"

    [\foo]
    Defines: X := {x_ : x_ is \set | \\forall{y is \set}:then{y is? \set}} is \set
    Documented:
    . called: "foo"
    . written: "X?"
    Id: "b165f407-283d-4d1b-815f-9200da352065"
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(root, &[], &mut event_log);

        assert_eq!(result.files_checked, 1);
        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 1 file").with_origin("mlg_check")]
        );
    }

    #[test]
    fn check_reports_invalid_refined_headings_and_refines_targets() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("invalid-refines.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Describes: X
    Documented:
    . written: "\operatorname{set}"

    [\function:?on{A}:?to{B}]
    Describes: f(x__)
    when: A, B is \set
    Documented:
    . called: "function"
    . written: "f?"

    [\(injective)::function:?on{A}:?to{B}]
    Refines: f(x__) is \function:?on{A}
    when: A, B is \set
    Documented:
    . adjective: "injective"

    [\(surjective)::function:?on{A}:?to{B}]
    Refines: f(x__)
    when: A, B is \set
    Documented:
    . called: "surjective"
    . adjective: "surjective"

    [\(bad)::function]
    Describes: g
    Documented:
    . written: "\operatorname{bad}"
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("invalid-refines.mlg")],
            &mut event_log,
        );
        let messages = user_events(&event_log)
            .into_iter()
            .filter_map(|event| event.as_message().map(|message| message.message.clone()))
            .collect::<Vec<_>>();

        assert_eq!(result.files_checked, 1);
        assert!(messages.iter().any(|message| message
            == "Refines entries must have the form `Refines: <form>`; the refined target is inferred from the heading"));
        assert!(
            messages.iter().any(|message| message
                == "Refined command headings may only be used with Refines entries")
        );
        assert!(messages.iter().any(|message| {
            message == "`Refines` documentation does not accept `called:`; use `adjective:`"
        }));
        assert!(event_log.has_errors());
    }

    #[test]
    fn check_accepts_command_when_requirement_from_given_type_fact() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("type-fact.mlg");

        write_mlg_fixture(
            &file,
            r#"[\real]
    Describes: x
    Documented:
    . written: "\operatorname{real}"

    [\foo{s}]
    Describes: x
    when: s is \real
    Documented:
    . written: "\operatorname{foo}"

    Theorem:
    given: r is \real
    then:
    . \foo{r}
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("type-fact.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 1 file").with_origin("mlg_check")]
        );
    }

    #[test]
    fn check_accepts_disambiguated_binary_operator_branches() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("disambiguates-plus.mlg");

        write_mlg_fixture(
            &file,
            r#"[\real]
    Describes: R
    Documented:
    . written: "\operatorname{real}"

    [\complex]
    Describes: C
    Documented:
    . written: "\operatorname{complex}"

    [\integer]
    Describes: I
    Documented:
    . written: "\operatorname{integer}"

    [a \.complex.+./ b]
    Defines: c is \complex
    when:
    . a is \real
    . b is \complex
    Documented:
    . written: "a? + b?"

    [a \.real.+./ b]
    Defines: c is \real
    when:
    . a is \real
    . b is \integer
    Documented:
    . written: "a? + b?"

    [x_ + y_]
    Disambiguates:
    when:
    . x_ is \real
    . y_ is \complex
    to: x_ \.complex.+./ y_
    when:
    . x_ is \real
    . y_ is \integer
    to: x_ \.real.+./ y_
    Documented:
    . written: "x_? + y_?"

    [op(x_, y_)]
    Disambiguates:
    when:
    . x_ is \real
    . y_ is \integer
    to: x_ \.real.+./ y_
    Documented:
    . written: "op(x_?, y_?)"

    Theorem:
    given:
    . r is \real
    . z is \complex
    . n is \integer
    then:
    . r + z
    . r + n
    . r |op| n
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("disambiguates-plus.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 1 file").with_origin("mlg_check")]
        );
    }

    #[test]
    fn check_accepts_disambiguates_with_else_only() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("disambiguates-else-only.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Describes: X
    Enables:
    . capability: x_ + y_ :=> x_ \.set.+./ y_
    Documented:
    . called: "set"

    [A \.set.+./ B]
    Defines: C is \set
    when: A, B is \set
    Documented:
    . written: "A? + B?"

    [x_ + y_]
    Disambiguates:
    else: x_ :+: y_
    Documented:
    . written: "x_? + y_?"

    Theorem:
    given: A, B is \set
    then: A + B is \set
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("disambiguates-else-only.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 1 file").with_origin("mlg_check")]
        );
    }

    #[test]
    fn check_accepts_type_directed_provided_binary_operators() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("type-directed-minus.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Describes: X
    Enables:
    . capability: x_ - y_ :=> x_ \.set.minus./ y_
    . capability: x_ ** y_ :=> x_ \.set.minus./ y_
    . capability: x_ +_-* y_ :=> x_ \.set.minus./ y_
    . capability: x_ *_free y_ :=> x_ \.set.minus./ y_
    . capability: x_ |minus| y_ :=> x_ \.set.minus./ y_
    Documented:
    . called: "set"

    [A \.set.minus./ B]
    Defines: C := A is \set
    when: A, B is \set
    Documented:
    . called: "set difference of $A?$ and $B?$"
    . written: "A? \backslash B?"

    Theorem:
    given: A, B is \set
    then: A :- B is \set

    Theorem:
    given: A, B is \set
    then: A -: B is \set

    Theorem:
    given: A, B is \set
    then: A :-: B is \set

    Theorem:
    given: A, B is \set
    then: A :** B is \set

    Theorem:
    given: A, B is \set
    then: A **: B is \set

    Theorem:
    given: A, B is \set
    then: A :**: B is \set

    Theorem:
    given: A, B is \set
    then: A :*_free B is \set

    Theorem:
    given: A, B is \set
    then: A :+_-* B is \set

    Theorem:
    given: A, B is \set
    then: A :|minus| B is \set

    Theorem:
    given: A, B is \set
    then: A |minus|: B is \set

    Theorem:
    given: A, B is \set
    then: A :|minus|: B is \set

    Theorem:
    given: A, B is \set
    then: A - B is \set

    [x_ - y_]
    Disambiguates:
    when: x_, y_ is \set
    to: x_ \.set.minus./ y_
    else: x_ :-: y_
    Documented:
    . written: "x_? - y_?"
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("type-directed-minus.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 1 file").with_origin("mlg_check")]
        );
    }

    #[test]
    fn check_accepts_disambiguated_else_operator_results_as_command_arguments() {
        let temp_dir = TestDir::new();
        let file = temp_dir
            .path()
            .join("disambiguated-minus-union-arguments.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Describes: X
    Enables:
    . capability: x_ - y_ :=> x_ \.set.minus./ y_
    Documented:
    . called: "set"

    [A \.set.minus./ B]
    Defines: C is \set
    when: A, B is \set
    Documented:
    . called: "set difference of $A?$ and $B?$"
    . written: "A? \backslash B?"

    [A \.set.union./ B]
    Defines: C is \set
    when: A, B is \set
    Documented:
    . called: "union of $A?$ and $B?$"
    . written: "A? \cup B?"

    [A \.set.symmetric.difference./ B]
    Defines: C := (A - B) \.set.union./ (B - A) is \set
    when: A, B is \set
    Documented:
    . called: "symmetric difference of $A?$ and $B?$"
    . written: "A? \Delta B?"

    [x_ - y_]
    Disambiguates:
    else: x_ :-: y_
    Documented:
    . written: "x_? - y_?"
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("disambiguated-minus-union-arguments.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 1 file").with_origin("mlg_check")]
        );
    }

    #[test]
    fn check_accepts_provided_expression_symbols_with_owner_context() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("provided-symbol-owner-context.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Describes: X
    Enables:
    . capability: x_ != y_ :=> \not{x_ = y_}
    . capability: f(x_) :=> \foo{X, x_}
    . capability: a :=> \some.value{X}
    Documented:
    . called: "set"

    [\not{P}]
    Defines: Q is \\statement
    when: P is \\expression
    Documented:
    . written: "\neg P?"

    [\foo{X, x}]
    Defines: Y is \\expression
    when:
    . X is \set
    . x is \\expression
    Documented:
    . written: "\operatorname{foo}(X?, x?)"

    [\some.value{X}]
    Defines: Y is \\expression
    when: X is \set
    Documented:
    . written: "\operatorname{someValue}(X?)"

    Theorem:
    given: A, B is \set
    then:
    . A :!=: B
    . A.f(B)
    . A.a
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("provided-symbol-owner-context.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 1 file").with_origin("mlg_check")]
        );
    }

    #[test]
    fn check_accepts_callable_owner_capability_functions() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("callable-owner-capability.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Describes: X
    Requires:
    . capability: x_ "in" X :-> \\abstract
      written: "x_? \in X?"
    Documented:
    . called: "set"

    [\relation:from{A}:to{B}]
    Describes: R
    when: A, B is \set
    Requires:
    . capability: z_ "in" R :-> \\abstract
      written: "z_? \in R?"
    Enables:
    . capability: R(a_, b_) :-> (a_, b_) "in" R
      written: "a_? \: R \: b_?"
    Documented:
    . called: "relation from $A?$ to $B?$"
    . written: "R? \subseteq A? \times B?"

    [\needs.specification{P}]
    Describes: x
    when: P is \\specification
    Documented:
    . written: "\operatorname{needsSpecification}(P?)"

    Theorem:
    given:
    . A, B is \set
    . R is \relation:from{A}:to{B}
    . a "in" A
    . b "in" B
    then:
    . R(a, b)
    . \needs.specification{R(a, b)}
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("callable-owner-capability.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 1 file").with_origin("mlg_check")]
        );
    }

    #[test]
    fn check_accepts_provided_operator_when_operand_is_defined_command_result() {
        let temp_dir = TestDir::new();
        let file = temp_dir
            .path()
            .join("defined-command-result-provided-operator.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Describes: X
    Enables:
    . capability: x_ "in" X :-> \\abstract
      written: "x_? \in X?"
    . capability: x_ = y_ :=> x_ \.set.=./ y_
    . capability: x_ != y_ :=> \not{x_ = y_}
    . capability: x_ - y_ :=> x_ \.set.minus./ y_
    Documented:
    . called: "set"

    [X \.set.=./ Y]
    States:
    when: X, Y is \set
    that:
    . forAll: Z "in" X
      then: Z "in" Y
    . forAll: Z "in" Y
      then: Z "in" X
    Documented:
    . written: "X? = Y?"

    [A \.set.minus./ B]
    Defines: C is \set
    when: A, B is \set
    Documented:
    . written: "A? \backslash B?"

    [\not{P}]
    Defines: Q is \\statement
    when: P is \\expression
    Documented:
    . written: "\neg P?"

    [\empty.set]
    Defines: X is \set
    expresses:
    . not:
      . exists: Y is \set
        suchThat: Y "in" X
    Documented:
    . written: "\emptyset"

    [\nonempty.set]
    Describes: X
    extends: X is \set
    satisfies:
    . X != \empty.set
    Documented:
    . called: "non-empty set"
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from(
                "defined-command-result-provided-operator.mlg",
            )],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 1 file").with_origin("mlg_check")]
        );
    }

    #[test]
    fn check_rejects_plain_binary_operators_without_disambiguation() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("unresolved-plain-operators.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Describes: X
    Enables:
    . capability: x_ "in" X :-> \\abstract
      written: "x_? \in X?"
    . capability: x_ = y_ :=> x_ \.set.=./ y_
    . capability: x_ - y_ :=> x_ \.set.minus./ y_
    Documented:
    . called: "set"

    Theorem:
    given: A, B is \set
    then: A + B is \set

    Theorem:
    given: A, B is \set
    then: A - B is \set
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("unresolved-plain-operators.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        let messages = event_log
            .events()
            .iter()
            .filter_map(Event::as_message)
            .filter(|message| message.audience == Audience::User)
            .map(|message| message.message.as_str())
            .collect::<Vec<_>>();

        assert!(messages.contains(
            &"Could not resolve operator `+`: no matching `Disambiguates` entry was found"
        ));
        assert!(messages.contains(
            &"Could not resolve operator `-`: no matching `Disambiguates` entry was found"
        ));
        assert_eq!(messages.last(), Some(&"Found 2 issues."));
    }

    #[test]
    fn check_accepts_plain_equality_and_inequality_without_type_capabilities() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("unresolved-equality.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Describes: X
    Documented:
    . called: "set"

    [\number]
    Describes: n
    Documented:
    . called: "number"

    [\needs.statement{P}]
    Describes: x
    when: P is \\statement
    Documented:
    . written: "\operatorname{needsStatement}(P?)"

    Theorem:
    given:
    . A is \set
    . n is \number
    then:
    . A = n
    . A != n
    . \needs.statement{A = n}
    . \needs.statement{A != n}
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("unresolved-equality.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 1 file").with_origin("mlg_check")]
        );
    }

    #[test]
    fn check_uses_type_defined_plain_equality_and_inequality_when_available() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("defined-equality.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Describes: X
    Enables:
    . capability: x_ = y_ :=> x_ \.set.=./ y_
    . capability: x_ != y_ :=> \not{x_ = y_}
    Documented:
    . called: "set"

    [A \.set.=./ B]
    States:
    when: A, B is \set
    that: A = A
    Documented:
    . written: "A? = B?"

    [\not{P}]
    States:
    when: P is \\statement
    that: P
    Documented:
    . written: "\neg P?"

    [\needs.statement{P}]
    Describes: x
    when: P is \\statement
    Documented:
    . written: "\operatorname{needsStatement}(P?)"

    Theorem:
    given: A, B is \set
    then:
    . \needs.statement{A = B}
    . \needs.statement{A != B}
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("defined-equality.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 1 file").with_origin("mlg_check")]
        );
    }

    #[test]
    fn check_formats_binary_operator_requirement_errors_readably() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("readable-binary-requirement.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Describes: X
    Documented:
    . called: "set"

    [A \.set.union./ B]
    Defines: C is \set
    when: A, B is \set
    Documented:
    . called: "union of $A?$ and $B?$"
    . written: "A? \cup B?"

    Theorem:
    given: A, B is \set
    then: (A - B) \.set.union./ B
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("readable-binary-requirement.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        let expected =
            "Could not establish requirement `A - B is \\set` for command `\\.set.union./`";
        let canonical_file = file.canonicalize().unwrap();
        assert!(user_events(&event_log).iter().any(|event| {
            event.as_message().is_some_and(|message| {
                message.message == expected
                    && message.location.as_ref().is_some_and(|location| {
                        matches!(
                            location,
                            crate::events::EventLocation::File {
                                path,
                                span: Some(_)
                            } if path == &canonical_file
                        )
                    })
            })
        }));
        assert!(!user_events(&event_log).iter().any(|event| {
            event.as_message().is_some_and(|message| {
                message.message.contains("Operator {") || message.message.contains("Subtract(")
            })
        }));
        assert!(event_log.has_errors());
    }

    #[test]
    fn check_accepts_disambiguated_prefix_and_postfix_operator_branches() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("disambiguates-prefix-postfix.mlg");

        write_mlg_fixture(
            &file,
            r#"[\real]
    Describes: R
    Documented:
    . written: "\operatorname{real}"

    [\prefix.real{x}]
    Defines: y is \real
    when: x is \real
    Documented:
    . written: "\operatorname{pre}(x?)"

    [\postfix.real{x}]
    Defines: y is \real
    when: x is \real
    Documented:
    . written: "\operatorname{post}(x?)"

    [f| x_]
    Disambiguates:
    when: x_ is \real
    to: \prefix.real{x_}
    Documented:
    . written: "f| x_?"

    [x_ |f]
    Disambiguates:
    when: x_ is \real
    to: \postfix.real{x_}
    Documented:
    . written: "x_? |f"

    [g(x_)]
    Disambiguates:
    when: x_ is \real
    to: \prefix.real{x_}
    Documented:
    . written: "g(x_?)"

    Theorem:
    given: r is \real
    then:
    . f| r
    . r |f
    . g(r)
    . g| r
    . r |g
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("disambiguates-prefix-postfix.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 1 file").with_origin("mlg_check")]
        );
    }

    #[test]
    fn check_accepts_tuple_targets_in_set_builder_definitions() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("set-builder-tuple-target.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Describes: X
    Enables:
    . capability: x_ "in" X :-> \\abstract
    Documented:
    . written: "\operatorname{set}"

    [A \.set.cross./ B]
    Defines: C := {(a_, b_) : a_ "in" A, b_ "in" B} is \set
    when: A, B is \set
    Documented:
    . called: "Cartesian product of $A?$ and $B?$"
    . written: "A? \times B?"
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("set-builder-tuple-target.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 1 file").with_origin("mlg_check")]
        );
    }

    #[test]
    fn check_accepts_introduced_set_builder_targets_and_definition_predicates() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("cartesian-set-builder.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Describes: X
    Requires:
    . capability: x_ "in" X :-> \\abstract
      written: "x_? \in X?"
    Documented:
    . called: "set"
    Id: "059126b9-dc83-41a2-aa1c-84f8e942f8d6"

    [\ordered.pair:of{a}:and{b}]
    Defines: P is \set
    when: a, b is \\opaque
    Documented:
    . called: "ordered pair of $a?$ and $b?$"
    . written: "(a?, b?)"
    Id: "10faf153-d005-4feb-b620-c31589aefea1"

    [\cartesian.product:of{A}:and{B}]
    Defines: P is \set
    when: A, B is \set
    expresses: P := {z_ ::= (a_, b_) : a_ "in" A; b_ "in" B | z_ := \ordered.pair:of{a_}:and{b_}}
    Documented:
    . called: "cartesian product of $A?$ and $B?$"
    . written: "A? \times B?"
    Id: "64578792-cf4f-4497-9c79-3fc0189a08e4"
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("cartesian-set-builder.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 1 file").with_origin("mlg_check")]
        );
    }

    #[test]
    fn check_reports_command_when_requirement_type_mismatches() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("type-mismatch.mlg");

        write_mlg_fixture(
            &file,
            r#"[\real]
    Describes: x
    Documented:
    . written: "\operatorname{real}"

    [\foo{s}]
    Describes: x
    when: s is \real
    Documented:
    . written: "\operatorname{foo}"

    Theorem:
    then:
    . \foo{r}
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("type-mismatch.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert!(user_events(&event_log).iter().any(|event| {
            event.as_message().is_some_and(|message| {
                message.message
                    == "Could not establish requirement `r is \\real` for command `\\foo`"
            })
        }));
        assert!(event_log.has_errors());
    }

    #[test]
    fn check_accepts_spec_infix_definitions_predicates_and_extensions() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("spec-infix-valid.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Describes: X
    Enables:
    . capability: x_ "in" X :-> \\abstract
    Documented:
    . written: "\operatorname{set}"

    [A \:subset:/ B]
    Describes: A
    when: B is \set
    extends: A is \set
    satisfies:
    . forAll: a "in" A
      then:
      . a "in"? B
    Documented:
    . written: "A? \subseteq B?"

    [\needs.set{s}]
    Describes: x
    when: s is \set
    Documented:
    . written: "\operatorname{needsSet}"

    Theorem:
    given: A, B is \set
    where:
    . A \:subset:/ B
    then:
    . A \:subset?:/ B
    . \needs.set{A}
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("spec-infix-valid.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 1 file").with_origin("mlg_check")]
        );
    }

    #[test]
    fn check_reports_spec_infix_requirement_mismatches() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("spec-infix-requirement.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Describes: X
    Documented:
    . written: "\operatorname{set}"

    [\thing]
    Describes: X
    Documented:
    . written: "\operatorname{thing}"

    [A \:subset:/ B]
    Describes: A
    when: A, B is \set
    Documented:
    . written: "A? \subseteq B?"

    Theorem:
    given:
    . A is \set
    . B is \thing
    then:
    . A \:subset?:/ B
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("spec-infix-requirement.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert!(user_events(&event_log).iter().any(|event| {
            event.as_message().is_some_and(|message| {
                message.message
                    == "Could not establish requirement `B is \\set` for command `\\:subset:/`"
            })
        }));
        assert!(event_log.has_errors());
    }

    #[test]
    fn check_accepts_spec_infix_optional_tail_hidden_witnesses() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("spec-infix-optional-tail.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Describes: X
    Enables:
    . capability: x_ "in" X :-> \\abstract
    Documented:
    . written: "\operatorname{set}"

    [A \:subset:?within{U}:/ B]
    Describes: A
    when:
    . A, B, U is \set
    . B \:subset:/ U
    extends: A is \set
    satisfies:
    . forAll: a "in" A
      then:
      . a "in"? B
    Documented:
    . written: "A? \subseteq B?"

    Theorem:
    given:
    . A, B, U is \set
    . B \:subset:/ U
    where:
    . A \:subset:within{U}:/ B
    then:
    . A \:subset?:/ B
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("spec-infix-optional-tail.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 1 file").with_origin("mlg_check")]
        );
    }

    #[test]
    fn check_accepts_spec_operator_support_inherited_through_extensions() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("inherited-spec-operator.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Describes: X
    Enables:
    . capability: x_ "in" X :-> \\abstract
    Documented:
    . written: "\operatorname{set}"

    [A \:subset:?within{U}:/ B]
    Describes: A
    when:
    . A, B, U is \set
    . B \:subset:/ U
    extends: A is \set
    satisfies:
    . forAll: a "in" A
      then:
      . a "in"? B
    Documented:
    . written: "A? \subseteq B?"

    [P \.and./ Q]
    States:
    when: P, Q is \\statement
    that: P

    [A \.set.intersect:?within{U}./ B]
    Defines: C \:subset:/ U
    when:
    . A, B, U is \set
    . A \:subset:/ U
    . B \:subset:/ U
    expresses: C := {c_ : c_ "in" U | (.c "in"? A.) \.and./ (.c "in"? B.)}
    Documented:
    . written: "A? \cap@[U]{_{U?}}:{} B?"
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("inherited-spec-operator.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 1 file").with_origin("mlg_check")]
        );
    }

    #[test]
    fn check_accepts_builtin_expression_statement_and_specification_requirements() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("builtin-categories.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Describes: X
    Enables:
    . capability: x_ "in" X :-> \\abstract
    Documented:
    . written: "\operatorname{set}"

    [A \:subset:/ B]
    Describes: A
    when: A, B is \set
    extends: A is \set
    Documented:
    . written: "A? \subseteq B?"

    [A \.same.as./ B]
    States:
    when: A, B is \set
    that:
    . A is? \set

    [\needs.expression{x}]
    Describes: y
    when: x is \\expression
    Documented:
    . written: "\operatorname{needsExpression}"

    [\needs.statement{x}]
    Describes: y
    when: x is \\statement
    Documented:
    . written: "\operatorname{needsStatement}"

    [\needs.specification{x}]
    Describes: y
    when: x is \\specification
    Documented:
    . written: "\operatorname{needsSpecification}"

    Theorem:
    given:
    . A, B is \set
    where:
    . A \:subset:/ B
    then:
    . \needs.expression{A}
    . \needs.statement{A is? \set}
    . \needs.statement{A "in"? B}
    . \needs.statement{A \:subset?:/ B}
    . \needs.statement{A \.same.as./ B}
    . \needs.specification{A is \set}
    . \needs.specification{A "in" B}
    . \needs.specification{A \:subset:/ B}
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("builtin-categories.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 1 file").with_origin("mlg_check")]
        );
    }

    #[test]
    fn check_reports_builtin_statement_and_specification_mismatches() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("builtin-category-mismatches.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Describes: X
    Documented:
    . written: "\operatorname{set}"

    [\needs.statement{x}]
    Describes: y
    when: x is \\statement
    Documented:
    . written: "\operatorname{needsStatement}"

    [\needs.specification{x}]
    Describes: y
    when: x is \\specification
    Documented:
    . written: "\operatorname{needsSpecification}"

    [\wrap{x}]
    Describes: y
    when: x is \\expression
    Documented:
    . written: "\operatorname{wrap}"

    Theorem:
    given: A is \set
    then:
    . \needs.statement{A}
    . \needs.statement{\wrap{A is? \set}}
    . \needs.specification{A}
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("builtin-category-mismatches.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        let messages = user_events(&event_log)
            .iter()
            .filter_map(Event::as_message)
            .map(|event| event.message.clone())
            .collect::<Vec<_>>();
        assert!(messages.contains(&String::from(
            "Could not establish requirement `A is \\\\statement` for command `\\needs.statement`"
        )));
        assert!(messages.contains(&String::from(
            "Could not establish requirement `\\wrap{A is? \\set} is \\\\statement` for command `\\needs.statement`"
        )));
        assert!(messages.contains(&String::from(
            "Could not establish requirement `A is \\\\specification` for command `\\needs.specification`"
        )));
        assert!(event_log.has_errors());
    }

    #[test]
    fn check_reports_invalid_spec_infix_headings() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("spec-infix-invalid-heading.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Describes: X
    Documented:
    . written: "\operatorname{set}"

    [A \:wrong:/ B]
    Describes: C
    when: B is \set
    Documented:
    . written: "\operatorname{wrong}"

    [A \:states:/ B]
    States:
    that: A = A
    Documented:
    . written: "\operatorname{states}"
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("spec-infix-invalid-heading.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        let messages = user_events(&event_log)
            .iter()
            .filter_map(Event::as_message)
            .map(|event| event.message.clone())
            .collect::<Vec<_>>();
        assert!(messages.contains(&String::from(
            "Spec-infix Describes heading left operand must match the Describes argument"
        )));
        assert!(messages.contains(&String::from(
            "Spec-infix headings may only be used with Describes entries"
        )));
        assert!(event_log.has_errors());
    }

    #[test]
    fn check_reports_type_argument_mismatches_in_is_statements_and_predicates() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("function-argument-types.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Describes: X
    Enables:
    . capability: x_ "in" X :-> \\abstract
    Documented:
    . written: "\operatorname{set}"

    [\function:on{A}:to{B}]
    Describes: f(x__)
    when: A, B is \set
    satisfies:
    . forAll: x "in" A
      then:
      . existsUnique: y "in" B
        suchThat: f(x) = y
    Documented:
    . called: "function on $A?$ to $B?$"
    . written: "f? \: : \: A? \rightarrow B?"

    Theorem:
    given:
    . X, Y is \set
    . x "in" X
    . g is \function:on{X}:to{Y}
    . h is \function:on{g}:to{x}
    then:
    . g is? \function:on{X}:to{g}
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("function-argument-types.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        let messages = user_events(&event_log)
            .iter()
            .filter_map(Event::as_message)
            .map(|event| event.message.clone())
            .collect::<Vec<_>>();
        assert!(messages.contains(&String::from(
            "Could not establish requirement `g is \\set` for command `\\function:on:to`"
        )));
        assert!(messages.contains(&String::from(
            "Could not establish requirement `x is \\set` for command `\\function:on:to`"
        )));
        let canonical_file = file.canonicalize().unwrap();
        assert!(has_user_error_at(
            &event_log,
            &canonical_file,
            28,
            8,
            "Could not establish requirement `g is \\set` for command `\\function:on:to`"
        ));
        assert!(has_user_error_at(
            &event_log,
            &canonical_file,
            26,
            7,
            "Could not establish requirement `x is \\set` for command `\\function:on:to`"
        ));
        assert!(event_log.has_errors());
    }

    #[test]
    fn check_uses_extends_sections_for_subtype_requirements() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("subtype-requirement.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Describes: X
    Enables:
    . capability: x_ "in" X :-> \\abstract
    Documented:
    . written: "\operatorname{set}"

    [\element.of:group{G ::= (X, *, e)}]
    Describes: x
    when:
    . G is \group
    . X is \set
    . * is \function:on{X}:to{X}
    . e "in" G
    extends: x "in" X
    Documented:
    . written: "x? \in G?"

    [\group]
    Describes: G ::= (X, *, e)
    extends: G is \set via X
    specifies:
    . X is \set
    . * is \function:on{X}:to{X}
    . e "in" G
    Enables:
    . capability: x_ "in" G :-> x_ is \element.of:group{G}
    Documented:
    . written: "\operatorname{group}"

    [\function:on{A}:to{B}]
    Describes: f(x__)
    when: A, B is \set
    Documented:
    . written: "A? \to B?"

    Theorem:
    given:
    . G is \group
    . f is \function:on{G}:to{G}
    then:
    . f is? \function:on{G}:to{G}
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("subtype-requirement.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 1 file").with_origin("mlg_check")]
        );
    }

    #[test]
    fn check_uses_function_type_extends_for_function_calls() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("function-type.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Describes: X
    Enables:
    . capability: x_ "in" X :-> \\abstract
    Documented:
    . written: "\operatorname{set}"

    [\function:on{A}:to{B}]
    Describes: f(x__)
    when: A, B is \set
    extends: f is (_ "in" A) => (_ "in" B)
    Documented:
    . written: "A? \to B?"

    Theorem:
    given:
    . A, B is \set
    . f is \function:on{A}:to{B}
    . y "in" A
    then:
    . f(y) "in" B
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("function-type.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 1 file").with_origin("mlg_check")]
        );
    }

    #[test]
    fn check_reports_function_type_result_mismatches() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("function-type-result.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Describes: X
    Enables:
    . capability: x_ "in" X :-> \\abstract
    Documented:
    . written: "\operatorname{set}"

    [\function:on{A}:to{B}]
    Describes: f(x__)
    when: A, B is \set
    extends: f is (_ "in" A) => (_ "in" B)
    Documented:
    . written: "A? \to B?"

    Theorem:
    given:
    . A, B, C is \set
    . f is \function:on{A}:to{B}
    . y "in" A
    then:
    . f(y) "in" C
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("function-type-result.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert!(user_events(&event_log).iter().any(|event| {
            event.as_message().is_some_and(|message| {
                message.message == "Could not establish function call result `f(y) \"in\" C`"
            })
        }));
        assert!(event_log.has_errors());
    }

    #[test]
    fn check_uses_function_type_is_specs_for_function_calls() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("function-type-is-specs.mlg");

        write_mlg_fixture(
            &file,
            r#"[\real]
    Describes: x
    Documented:
    . written: "\operatorname{real}"

    [\integer]
    Describes: x
    Documented:
    . written: "\operatorname{integer}"

    Theorem:
    given:
    . f is (_ is \real) => (_ is \integer)
    . y is \real
    then:
    . f(y) is \integer
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("function-type-is-specs.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 1 file").with_origin("mlg_check")]
        );
    }

    #[test]
    fn check_rejects_named_function_type_parameters() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("function-type-parameters.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Describes: X
    Documented:
    . written: "\operatorname{set}"

    [\function:on{A}:to{B}]
    Describes: f(x__)
    when: A, B is \set
    extends: f is (x "in" A) => (_ "in" B)
    Documented:
    . written: "A? \to B?"
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("function-type-parameters.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert!(user_events(&event_log).iter().any(|event| {
            event
                .as_message()
                .is_some_and(|message| message.message == "Function type parameters must be `_`")
        }));
        assert!(event_log.has_errors());
    }

    #[test]
    fn check_reports_unrecognized_symbols_in_command_arguments() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("unrecognized-symbol.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Describes: X
    Documented:
    . written: "\operatorname{set}"

    [\function:on{A}:to{B}]
    Describes: f(x__)
    when: A, B is \set
    Documented:
    . written: "A? \to B?"

    Theorem:
    given:
    . X, Y is \set
    . f is \function:on{X}:to{Y}
    then:
    . f is? \function:on{X}:to{Z}
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("unrecognized-symbol.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert!(has_user_error_at(
            &event_log,
            &file.canonicalize().unwrap(),
            18,
            27,
            "Unrecognized symbol `Z`"
        ));
        assert!(event_log.has_errors());
    }

    #[test]
    fn check_uses_local_bindings_when_matching_types() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("type-binding.mlg");

        write_mlg_fixture(
            &file,
            r#"[\real]
    Describes: x
    Documented:
    . written: "\operatorname{real}"

    [\foo{s}]
    Describes: x
    when: s is \real
    Documented:
    . written: "\operatorname{foo}"

    Theorem:
    given: A is \real
    where:
    . A ::= B := B
    then:
    . \foo{B}
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("type-binding.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 1 file").with_origin("mlg_check")]
        );
    }

    #[test]
    fn check_rejects_unintroduced_definition_rhs_symbols() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("definition-rhs-scope.mlg");

        write_mlg_fixture(
            &file,
            r#"[\real]
    Describes: x
    Documented:
    . written: "\operatorname{real}"

    Theorem:
    given: x := y
    then: x = x
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("definition-rhs-scope.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert!(user_events(&event_log).iter().any(|event| {
            event
                .as_message()
                .is_some_and(|message| message.message == "Unrecognized symbol `y`")
        }));
        assert!(event_log.has_errors());
    }

    #[test]
    fn check_rejects_unintroduced_defines_relation_symbols() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("defines-relation-scope.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Describes: X
    Documented:
    . written: "\operatorname{set}"

    [\function:on{A}:to{B}]
    Describes: f(x__) ::= y_
    when: A, B is \set
    specifies:
    . x__ is \\expression
    . y_ is \\opaque
    Documented:
    . written: "f? \: : \: A? \rightarrow B?"

    [\identify.function:on{A}]
    Defines: f(x__) := x__ is \function:on{A}:to{B}
    when: A is \set
    Documented:
    . called: "identity function on $A?$"
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("defines-relation-scope.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert!(user_events(&event_log).iter().any(|event| {
            event
                .as_message()
                .is_some_and(|message| message.message == "Unrecognized symbol `B`")
        }));
        assert!(event_log.has_errors());
    }

    #[test]
    fn check_reports_unintroduced_later_relation_symbols_on_later_entry() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("relation-location-scope.mlg");
        let source = r#"[\set]
    Describes: X
    Documented:
    . written: "\operatorname{set}"

    [\function:?on{A}:?to{B}]
    Describes: f(x__) ::= y_
    when: A, B is \set
    specifies:
    . x__ is \\expression
    . y_ is \\opaque
    Documented:
    . written: "f? \: : \: A? \rightarrow B?"

    [f \.function.compose./ g]
    Defines: h(x__) := f(g(x__)) is \function:on{A}:to{C}
    using: A, B, C is \set
    when:
    . g is \function:on{A}:to{B}
    . f is \function:on{B}:to{C}
    Documented:
    . written: "f? \circ g?"

    [\identify.function:on{A}]
    Defines: f(x__) := x__ is \function:on{A}:to{B}
    when: A is \set
    Documented:
    . called: "identity function on $A?$"
    "#;

        write_mlg_fixture(&file, source).unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("relation-location-scope.mlg")],
            &mut event_log,
        );

        let compose_row = source
            .lines()
            .position(|line| line.contains("Defines: h(x__)"))
            .expect("expected composition row");
        let expected =
            "Could not establish requirement `B is \\set` for command `\\function:on:to`";
        let canonical_file = file.canonicalize().unwrap();

        assert_eq!(result.files_checked, 1);
        assert!(user_events(&event_log).iter().any(|event| {
            event.as_message().is_some_and(|message| {
                message.message == expected
                    && message.location.as_ref().is_some_and(|location| {
                        matches!(
                            location,
                            crate::events::EventLocation::File {
                                path,
                                span: Some(span)
                            } if path == &canonical_file
                                && span.start.row.is_some_and(|row| row > compose_row)
                        )
                    })
            })
        }));
        assert!(!user_events(&event_log).iter().any(|event| {
            event.as_message().is_some_and(|message| {
                message.message == expected
                    && message.location.as_ref().is_some_and(|location| {
                        matches!(
                            location,
                            crate::events::EventLocation::File {
                                path,
                                span: Some(span)
                            } if path == &canonical_file && span.start.row == Some(compose_row)
                        )
                    })
            })
        }));
        assert!(event_log.has_errors());
    }

    #[test]
    fn check_allows_declaration_lhs_symbols_in_definition_rhs() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("definition-rhs-placeholders.mlg");

        write_mlg_fixture(
            &file,
            r#"[\real]
    Describes: x
    Documented:
    . written: "\operatorname{real}"

    Theorem:
    given: f(x_) := x_
    then: f(x_) = f(x_)
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("definition-rhs-placeholders.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 1 file").with_origin("mlg_check")]
        );
    }

    #[test]
    fn check_rejects_unintroduced_member_of_collection_in_assumptions() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("member-of-scope.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Describes: X
    Documented:
    . written: "\operatorname{set}"

    Theorem:
    where: x member_of X
    then: x = x
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("member-of-scope.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert!(user_events(&event_log).iter().any(|event| {
            event
                .as_message()
                .is_some_and(|message| message.message == "Unrecognized symbol `X`")
        }));
        assert!(event_log.has_errors());
    }

    #[test]
    fn check_uses_quantifier_bindings_when_matching_types() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("exists-binding.mlg");

        write_mlg_fixture(
            &file,
            r#"[\real]
    Describes: x
    Documented:
    . written: "\operatorname{real}"

    [\foo{s}]
    Describes: x
    when: s is \real
    Documented:
    . written: "\operatorname{foo}"

    Theorem:
    then:
    . exists: A ::= B := B
      suchThat:
      . A is \real
      . \foo{B}
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("exists-binding.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 1 file").with_origin("mlg_check")]
        );
    }

    #[test]
    fn check_uses_all_quantifier_bindings_in_clause_group_blocks() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("multi-quantifier-bindings.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Describes: X
    Requires:
    . capability: x_ "in" X :-> \\abstract
      written: "x_? \in X?"
    Documented:
    . called: "set"

    Theorem:
    given: A, B is \set
    then:
    . exists:
      . a "in" A
      . b "in" B
      suchThat:
      . a = b
    . existsUnique:
      . c "in" A
      . d "in" B
      suchThat:
      . c != d
    . forAll:
      . e "in" A
      . f "in" B
      then:
      . e = f
    . given:
      . g "in" A
      . h "in" B
      then:
      . g != h
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("multi-quantifier-bindings.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 1 file").with_origin("mlg_check")]
        );
    }

    #[test]
    fn check_accepts_exists_without_such_that_sections() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("exists-without-such-that.mlg");

        write_mlg_fixture(
            &file,
            r#"[\real]
    Describes: x
    Documented:
    . written: "\operatorname{real}"

    Theorem:
    then:
    . exists: x is \real
    . existsUnique: y is \real
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("exists-without-such-that.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 1 file").with_origin("mlg_check")]
        );
    }

    #[test]
    fn check_accepts_optional_command_header_tail_combinations_in_order() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("optional-command-tails.mlg");

        write_mlg_fixture(
            &file,
            r#"[\thing]
    Describes: value
    Documented:
    . written: "\operatorname{thing}"

    [\foo:?baz{A}:?bar{B}]
    Defines: A ::= B "defines" B
    when: A, B is \thing
    Documented:
    . [docs.called]
      written:
      . "\operatorname{foo}"

    Theorem:
    given: a, b is \thing
    then:
    . \foo
    . \foo:baz{a}
    . \foo:bar{b}
    . \foo:baz{a}:bar{b}
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("optional-command-tails.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 1 file").with_origin("mlg_check")]
        );
    }

    #[test]
    fn check_rejects_optional_command_header_tail_references_out_of_order() {
        let temp_dir = TestDir::new();
        let file = temp_dir
            .path()
            .join("optional-command-tails-out-of-order.mlg");

        write_mlg_fixture(
            &file,
            r#"[\foo:?baz{A}:?bar{B}]
    Defines: A ::= B "defines" B
    Documented:
    . [docs.called]
      written:
      . "\operatorname{foo}"

    Theorem:
    then:
    . \foo:bar{2}:baz{1}
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("optional-command-tails-out-of-order.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert!(
            event_log
                .events()
                .iter()
                .filter_map(Event::as_message)
                .any(|event| event
                    .message
                    .contains("Undefined command signature `\\foo:bar:baz`"))
        );
        assert!(event_log.has_errors());
    }

    #[test]
    fn check_accepts_states_that_sections_with_multiple_group_clauses() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("dotted-infix-heading.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Describes: X
    Enables:
    . capability: x_ "in" X :-> \\abstract
    Documented:
    . written: "\operatorname{set}"

    [X \.set.=./ Y]
    States:
    when: X, Y is \set
    that:
    . forAll: Z "in" X
      then: Z "in" Y
    . forAll: Z "in" Y
      then: Z "in" X
    Documented:
    . written: "X? = Y?"

    Theorem:
    given: A, B is \set
    then:
    . A \.set.=./ B
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("dotted-infix-heading.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 1 file").with_origin("mlg_check")]
        );
    }

    #[test]
    fn check_reduces_spec_operator_aliases_to_type_facts() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("spec-reduction.mlg");

        write_mlg_fixture(
            &file,
            r#"[\real]
    Describes: x
    Documented:
    . written: "\operatorname{real}"

    [\reals]
    Describes: R
    Enables:
    . capability: x_ "in" R :-> x is \real
    Documented:
    . written: "\operatorname{reals}"

    [\foo{s}]
    Describes: x
    when: s is \real
    Documented:
    . written: "\operatorname{foo}"

    Theorem:
    given:
    . S is \reals
    . r "in" S
    then:
    . \foo{r}
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("spec-reduction.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 1 file").with_origin("mlg_check")]
        );
    }

    #[test]
    fn check_reduces_collection_membership_to_literal_element_type() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("collection-membership.mlg");

        write_mlg_fixture(
            &file,
            r#"[\real]
    Describes: r
    Documented:
    . written: "\operatorname{real}"

    [\set]
    Describes: X ::= {x__ : ...}
    Enables:
    . capability: x_ "in" X :-> x_ member_of X
    Documented:
    . written: "\operatorname{set}"

    [\needs.real{x}]
    Describes: y
    when: x is \real
    Documented:
    . written: "\operatorname{needsReal}(x?)"

    Theorem:
    given:
    . A := {x_ : x_ is \real} as \set
    . x "in" A
    then: \needs.real{x}

    Theorem:
    given:
    . B := {b_ : b_ is \real} is \set
    . y "in" B
    then: \needs.real{y}

    Theorem:
    given:
    . C := {c_ : c_ is \real} as \set
    . z "in" C
    then: \needs.real{z}

    Theorem:
    given: X := {x_ : x_ is \set} as \set
    then:
    . forAll: x "in" X
      then: x is \set
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("collection-membership.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 1 file").with_origin("mlg_check")]
        );
    }

    #[test]
    fn check_reduces_cast_membership_through_from_capability() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("cast-membership.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Describes: X
    Requires:
    . capability: x_ "in" X :-> \\abstract
    Enables:
    . from: Y ::= {y__ : ...}
      capability: x_ "in" X :-> x_ member_of Y
    Documented:
    . written: "\operatorname{set}"

    [\needs.set{x}]
    Describes: y
    when: x is \set
    Documented:
    . written: "\operatorname{needsSet}(x?)"

    Theorem:
    given: X := {x_ : x_ is \set} as \set
    then:
    . forAll: x "in" X
      then:
      . x is? \set
      . \needs.set{x}
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("cast-membership.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 1 file").with_origin("mlg_check")]
        );
    }

    #[test]
    fn check_reduces_cast_function_outputs_through_from_as() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("cast-function-output.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Describes: X
    Documented:
    . written: "\operatorname{set}"

    [\function]
    Describes: f(x__) ::= y_
    specifies:
    . x__ is \\expression
    . y_ is \\opaque
    Enables:
    . from: P ::= {(p_, q_) : ...}
      as: f(p_) := q_
    Documented:
    . written: "\operatorname{function}"

    [\needs.set{x}]
    Describes: y
    when: x is \set
    Documented:
    . written: "\operatorname{needsSet}(x?)"

    Theorem:
    given:
    . F := {(p_, q_) : p_ is \\expression, q_ is \set} as \function
    . a is \\expression
    then: \needs.set{F(a)}
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("cast-function-output.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 1 file").with_origin("mlg_check")]
        );
    }

    #[test]
    fn check_uses_view_casts_for_resolved_command_requirements() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("view-cast-requirements.mlg");

        write_mlg_fixture(
            &file,
            r#"[\rational]
    Describes: r
    Documented:
    . written: "\operatorname{rational}"

    [\integer]
    Describes: n
    Enables:
    . relation:
      to: r is \rational
      when: n is \integer
      means: n \.embedded.to./ r
      as: \\view
    Documented:
    . written: "\operatorname{integer}"

    [A \.embedded.to./ B]
    States:
    when:
    . A is \integer
    . B is \rational
    that: A is? \integer
    Documented:
    . written: "A? \hookrightarrow B?"

    [A \.rational.+./ B]
    Defines: C is \rational
    when: A, B is \rational
    Documented:
    . written: "A? + B?"

    [\needs.rational{x}]
    Describes: y
    when: x is \rational
    Documented:
    . written: "\operatorname{needsRational}(x?)"

    Theorem:
    given: n is \integer
    then: \needs.rational{n}

    Theorem:
    given: n, m is \integer
    then: n \.rational.+./ m is? \rational
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("view-cast-requirements.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 1 file").with_origin("mlg_check")]
        );
    }

    #[test]
    fn check_accepts_explicit_as_cast_for_view_requirements() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("explicit-view-cast.mlg");

        write_mlg_fixture(
            &file,
            r#"[\rational]
    Describes: r
    Documented:
    . written: "\operatorname{rational}"

    [\integer]
    Describes: n
    Enables:
    . relation:
      to: r is \rational
      when: n is \integer
      as: \\view
    Documented:
    . written: "\operatorname{integer}"

    [\needs.rational{x}]
    Describes: y
    when: x is \rational
    Documented:
    . written: "\operatorname{needsRational}(x?)"

    Theorem:
    given: n is \integer
    then:
    . \needs.rational{n as \rational}
    . (n as \rational) is? \rational
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("explicit-view-cast.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 1 file").with_origin("mlg_check")]
        );
    }

    #[test]
    fn check_hard_cast_uses_abstraction_relationships() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("hard-cast-abstraction.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Describes: X
    Documented:
    . written: "\operatorname{set}"

    [\natural]
    Describes: n
    Enables:
    . relation:
      to: n is \set
      as: \\abstraction
    Documented:
    . written: "\operatorname{natural}"

    [\needs.set{x}]
    Describes: y
    when: x is \set
    Documented:
    . written: "\operatorname{needsSet}(x?)"

    Theorem:
    given: n is \natural
    then:
    . \needs.set{n as! \set}
    . (n as! \set) is? \set
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("hard-cast-abstraction.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 1 file").with_origin("mlg_check")]
        );
    }

    #[test]
    fn check_plain_cast_does_not_use_abstraction_relationships() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("plain-cast-no-abstraction.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Describes: X
    Documented:
    . written: "\operatorname{set}"

    [\natural]
    Describes: n
    Enables:
    . relation:
      to: n is \set
      as: \\abstraction
    Documented:
    . written: "\operatorname{natural}"

    [\needs.set{x}]
    Describes: y
    when: x is \set
    Documented:
    . written: "\operatorname{needsSet}(x?)"

    Theorem:
    given: n is \natural
    then: \needs.set{n as \set}
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("plain-cast-no-abstraction.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        let events = user_events(&event_log);
        assert!(events.iter().any(|event| {
            matches!(event, Event::Message(message) if
                message.message.contains("Could not establish cast `n as \\set`")
            )
        }));
        assert!(event_log.has_errors());
    }

    #[test]
    fn check_does_not_use_view_casts_for_operator_resolution() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("view-does-not-resolve-operators.mlg");

        write_mlg_fixture(
            &file,
            r#"[\rational]
    Describes: r
    Enables:
    . capability: x_ + y_ :=> x_ \.rational.+./ y_
    Documented:
    . written: "\operatorname{rational}"

    [\integer]
    Describes: n
    Enables:
    . relation:
      to: r is \rational
      when: n is \integer
      as: \\view
    Documented:
    . written: "\operatorname{integer}"

    [A \.rational.+./ B]
    Defines: C is \rational
    when: A, B is \rational
    Documented:
    . written: "A? + B?"

    Theorem:
    given: n, m is \integer
    then: n + m is? \rational
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("view-does-not-resolve-operators.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        let events = user_events(&event_log);
        assert!(events.iter().any(|event| {
            matches!(event, Event::Message(message) if
                message.message.contains("Could not resolve operator `+`")
            )
        }));
        assert!(event_log.has_errors());
    }

    #[test]
    fn check_reports_relation_as_with_unknown_marker() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("relation-as-marker.mlg");

        write_mlg_fixture(
            &file,
            r#"[\rational]
    Describes: r
    Documented:
    . written: "\operatorname{rational}"

    [\integer]
    Describes: n
    Enables:
    . relation:
      to: r is \rational
      when: n is \integer
      as: \\something.else
    Documented:
    . written: "\operatorname{integer}"
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("relation-as-marker.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        let events = user_events(&event_log);
        assert!(events.iter().any(|event| {
            matches!(event, Event::Message(message) if
                message
                    .message
                    .contains("`as:` entries must be `\\\\view` or `\\\\abstraction`")
            )
        }));
        assert!(event_log.has_errors());
    }

    #[test]
    fn check_does_not_use_view_casts_for_disambiguates_branches() {
        let temp_dir = TestDir::new();
        let file = temp_dir
            .path()
            .join("view-does-not-match-disambiguates.mlg");

        write_mlg_fixture(
            &file,
            r#"[\rational]
    Describes: r
    Documented:
    . written: "\operatorname{rational}"

    [\integer]
    Describes: n
    Enables:
    . relation:
      to: r is \rational
      when: n is \integer
      as: \\view
    Documented:
    . written: "\operatorname{integer}"

    [A \.rational.+./ B]
    Defines: C is \rational
    when: A, B is \rational
    Documented:
    . written: "A? + B?"

    [x_ + y_]
    Disambiguates:
    when: x_, y_ is \rational
    to: x_ \.rational.+./ y_
    Documented:
    . written: "x_? + y_?"

    Theorem:
    given: n, m is \integer
    then: n + m is? \rational
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("view-does-not-match-disambiguates.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        let events = user_events(&event_log);
        assert!(events.iter().any(|event| {
            matches!(event, Event::Message(message) if
                message
                    .message
                    .contains("Could not disambiguate operator `+`")
            )
        }));
        assert!(event_log.has_errors());
    }

    #[test]
    fn check_does_not_reduce_opaque_member_of_through_cast_literal() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("opaque-member-of-cast.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Describes: X
    Requires:
    . capability: x_ "in" X :-> x_ member_of X
    Documented:
    . written: "\operatorname{set}"

    [A \.set.minus./ B]
    Defines: C is \set
    when: A, B is \set
    Documented:
    . written: "A? \setminus B?"

    Theorem:
    given: X := {x_ : x_ is \set} as \set
    then:
    . forAll: x, y "in" X
      then: x \.set.minus./ y is? \set
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("opaque-member-of-cast.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        let events = user_events(&event_log);
        assert!(events.iter().any(|event| {
            matches!(event, Event::Message(message) if
                message.message == "Could not establish requirement `x is \\set` for command `\\.set.minus./`"
            )
        }));
        assert!(events.iter().any(|event| {
            matches!(event, Event::Message(message) if
                message.message == "Could not establish requirement `y is \\set` for command `\\.set.minus./`"
            )
        }));
        assert!(event_log.has_errors());
    }

    #[test]
    fn check_from_capability_does_not_hide_command_requirement_mismatch() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("from-capability-mismatch.mlg");

        write_mlg_fixture(
            &file,
            r#"[\function]
    Describes: f
    Documented:
    . written: "\operatorname{function}"

    [\set]
    Describes: X
    Requires:
    . capability: x_ "in" X :-> \\abstract
    Enables:
    . capability: x_ - y_ :=> x_ \.set.minus./ y_
    . from: Y ::= {y_ : ...}
      capability: x_ "in" X :-> x_ is \function
    Documented:
    . written: "\operatorname{set}"

    [A \.set.minus./ B]
    Defines: C is \set
    when: A, B is \set
    Documented:
    . written: "A? \setminus B?"

    Theorem:
    given: X := {x_ : x_ is \set} as \set
    then:
    . forAll: x, y "in" X
      then: x \.set.minus./ y is? \set
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("from-capability-mismatch.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        let events = user_events(&event_log);
        assert!(events.iter().any(|event| {
            matches!(event, Event::Message(message) if
                message.message == "Could not establish requirement `x is \\set` for command `\\.set.minus./`"
            )
        }));
        assert!(events.iter().any(|event| {
            matches!(event, Event::Message(message) if
                message.message == "Could not establish requirement `y is \\set` for command `\\.set.minus./`"
            )
        }));
        assert!(event_log.has_errors());
    }

    #[test]
    fn check_treats_membership_in_unstructured_collection_as_opaque() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("collection-membership-opaque.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Describes: X ::= {x__ : ...}
    Enables:
    . capability: x_ "in" X :-> x_ member_of X
    Documented:
    . written: "\operatorname{set}"

    [\needs.opaque{x}]
    Describes: y
    when: x is \\opaque
    Documented:
    . written: "\operatorname{needsOpaque}(x?)"

    Theorem:
    given:
    . A is \set
    . x "in" A
    then: \needs.opaque{x}
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("collection-membership-opaque.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 1 file").with_origin("mlg_check")]
        );
    }

    #[test]
    fn check_opaque_requirements_accept_any_declared_value() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("opaque-requirement.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Describes: X
    Documented:
    . written: "\operatorname{set}"

    [\accepts.opaque{X}]
    Describes: Y
    when: X is \\opaque
    Documented:
    . written: "\operatorname{acceptsOpaque}(X?)"

    Theorem:
    given: A is \set
    then: \accepts.opaque{A}
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("opaque-requirement.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 1 file").with_origin("mlg_check")]
        );
    }

    #[test]
    fn check_opaque_facts_do_not_establish_concrete_types() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("opaque-does-not-establish-set.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Describes: X
    Documented:
    . written: "\operatorname{set}"

    [\requires.set{X}]
    Describes: Y
    when: X is \set
    Documented:
    . written: "\operatorname{requiresSet}(X?)"

    Theorem:
    given: A is \\opaque
    then: \requires.set{A}
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("opaque-does-not-establish-set.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert!(user_events(&event_log).iter().any(|event| {
            event.as_message().is_some_and(|message| {
                message.message
                    == "Could not establish requirement `A is \\set` for command `\\requires.set`"
            })
        }));
        assert!(event_log.has_errors());
    }

    #[test]
    fn check_matches_spec_requirements_without_reducing_to_type_facts() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("direct-spec.mlg");

        write_mlg_fixture(
            &file,
            r#"[\group]
    Describes: G
    Enables:
    . capability: x_ "in" G :-> \\abstract
    Documented:
    . written: "\operatorname{group}"

    [\foo{G}:with{x}]
    Describes: y
    when:
    . G is \group
    . x "in" G
    Documented:
    . written: "\operatorname{foo}"

    Theorem:
    given:
    . H is \group
    . y "in" H
    . z is \foo{H}:with{y}
    then:
    . z = z
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("direct-spec.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 1 file").with_origin("mlg_check")]
        );
    }

    #[test]
    fn check_rejects_spec_assumption_without_provided_operator() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("missing-spec-provider.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Describes: X
    Documented:
    . written: "\operatorname{set}"

    [\function:?on{A}:?to{B}]
    Describes: f(x__)
    when: A, B is \set
    Documented:
    . written: "\operatorname{function}"

    [\group]
    Describes: G ::= (X, *, e)
    specifies:
    . X is \set
    . * is \function:on{X}:to{X}
    . e "in" G
    Documented:
    . written: "\operatorname{group}"
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("missing-spec-provider.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert!(user_events(&event_log).iter().any(|event| {
            event.as_message().is_some_and(|message| {
                message
                    .message
                    .contains("Could not validate spec fact `e \"in\" G`")
            })
        }));
        assert!(event_log.has_errors());
    }

    #[test]
    fn check_validates_definition_when_sections_against_parameters() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("definition-when-parameters.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Describes: X
    Documented:
    . written: "\operatorname{set}"

    [\group]
    Describes: G ::= (X, *, e)
    when:
    . X is \set
    . Y is \set
    . e := X
    Documented:
    . written: "\operatorname{group}"
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("definition-when-parameters.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        let messages = user_events(&event_log)
            .iter()
            .filter_map(|event| event.as_message().map(|message| message.message.clone()))
            .collect::<Vec<_>>();
        assert!(
            messages.iter().any(|message| message.contains(
                "`when:` requirement for `X` is not allowed because `X` is not a parameter"
            )),
            "{messages:#?}"
        );
        assert!(
            messages.iter().any(|message| message.contains(
                "`when:` requirement for `Y` is not allowed because `Y` is not a parameter"
            )),
            "{messages:#?}"
        );
        assert!(
            messages
                .iter()
                .any(|message| message.contains("Missing specification for target symbol `X`")),
            "{messages:#?}"
        );
        assert!(
            messages
                .iter()
                .any(|message| message.contains("Missing specification for target symbol `*`")),
            "{messages:#?}"
        );
        assert!(
            messages
                .iter()
                .any(|message| message.contains("Missing specification for target symbol `e`")),
            "{messages:#?}"
        );
        assert!(
            messages.iter().any(|message| message.contains(
                "`when:` clauses only support `<subject> is <type>` or `<subject> \"op\" <target>` requirements"
            )),
            "{messages:#?}"
        );
        assert!(event_log.has_errors());
    }

    #[test]
    fn check_rejects_describes_when_for_non_header_target_symbols() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("describes-target-when-symbols.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Describes: X
    Enables:
    . capability: x_ "in" X :-> \\abstract
    Documented:
    . written: "\operatorname{set}"

    [\function:on{A}:to{B}]
    Describes: f(x__)
    when: A, B is \set
    Documented:
    . written: "\operatorname{function}"

    [\element.of:group{G}]
    Describes: x
    when: G is \group
    Documented:
    . written: "x? \in G?"

    [\group]
    Describes: G ::= (X, *, e)
    when:
    . X is \set
    . * is \function:on{X}:to{X}
    . e "in" G
    extends: G is \set via X
    Enables:
    . capability: x_ "in" G :-> x_ is \element.of:group{G}
    Documented:
    . written: "\operatorname{group}"
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("describes-target-when-symbols.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        let messages = user_events(&event_log)
            .iter()
            .filter_map(|event| event.as_message().map(|message| message.message.clone()))
            .collect::<Vec<_>>();
        for subject in ["X", "*", "e"] {
            assert!(
                messages.iter().any(|message| message.contains(&format!(
                    "`when:` requirement for `{subject}` is not allowed because `{subject}` is not a parameter"
                ))),
                "{messages:#?}"
            );
        }
        for subject in ["*", "e"] {
            assert!(
                messages.iter().any(|message| {
                    message.contains(&format!(
                        "Missing specification for target symbol `{subject}`"
                    ))
                }),
                "{messages:#?}"
            );
        }
        assert!(event_log.has_errors());
    }

    #[test]
    fn check_does_not_require_described_subjects_or_optional_tail_parameters_in_when() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("optional-when-parameters.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Describes: X
    Documented:
    . written: "\operatorname{set}"

    [A \:subset:?within{U}:/ B]
    Describes: A
    when:
    . A is \set
    . U is \set
    . B is \set
    extends: A is \set
    Documented:
    . written: "A? \subset B?"

    [A \.combine:?using{U}./ B]
    Defines: C is \set
    when: A, B is \set
    Documented:
    . written: "A? \star B?"
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("optional-when-parameters.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 1 file").with_origin("mlg_check")]
        );
    }

    #[test]
    fn check_uses_nominal_typing_for_describes_type_requirements() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("nominal-describes-type.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Describes: X
    Documented:
    . written: "\operatorname{set}"

    [\function:?on{A}:?to{B}]
    Describes: f(x__)
    when: A, B is \set
    Documented:
    . written: "\operatorname{function}"

    [\group]
    Describes: G ::= (X, *, e)
    specifies:
    . X is \set
    . * is \function:on{X}:to{X}
    . e "in" G
    Enables:
    . capability: x_ "in" G :-> x_ is \element.of:group{G}
    Documented:
    . written: "\operatorname{group}"

    [\element.of:group{G}]
    Describes: x
    when: G is \group
    Documented:
    . called: "element of group $G?$"
    . written: "x? \in G?"
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("nominal-describes-type.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 1 file").with_origin("mlg_check")]
        );
    }

    #[test]
    fn check_validates_describes_type_expression_arguments() {
        let temp_dir = TestDir::new();
        let file = temp_dir
            .path()
            .join("describes-type-expression-arguments.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Describes: X
    Documented:
    . written: "\operatorname{set}"

    [\thing]
    Describes: value
    Documented:
    . written: "\operatorname{thing}"

    [\element.of:group{G}]
    Describes: x
    when: G is \set
    Documented:
    . written: "x? \in G?"

    Theorem:
    given: A, x is \thing
    then:
    . x is \element.of:group{A}
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("describes-type-expression-arguments.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert!(user_events(&event_log).iter().any(|event| {
            event.as_message().is_some_and(|message| {
                message.message
                    == "Could not establish requirement `A is \\set` for command `\\element.of:group`"
            })
        }));
        assert!(event_log.has_errors());
    }

    #[test]
    fn check_validates_spec_operator_alias_target_requirements() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("spec-alias-target-requirement.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Describes: X
    Documented:
    . written: "\operatorname{set}"

    [\function:?on{A}:?to{B}]
    Describes: f(x__)
    when: A, B is \set
    Documented:
    . written: "\operatorname{function}"

    [\group]
    Describes: G ::= (X, *, e)
    specifies:
    . X is \set
    . * is \function:on{X}:to{X}
    . e "in" G
    Enables:
    . capability: x_ "in" G :-> x_ is \element.of:group{G}
    Documented:
    . written: "\operatorname{group}"

    [\element.of:group{G}]
    Describes: x
    when: G is \set
    Documented:
    . called: "element of group $G?$"
    . written: "x? \in G?"
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("spec-alias-target-requirement.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert!(user_events(&event_log).iter().any(|event| {
            event.as_message().is_some_and(|message| {
                message.message
                    == "Could not establish requirement `G is \\set` for command `\\element.of:group`"
            })
        }));
        assert!(event_log.has_errors());
    }

    #[test]
    fn check_accepts_spec_operator_alias_target_requirements_via_extends() {
        let temp_dir = TestDir::new();
        let file = temp_dir
            .path()
            .join("spec-alias-target-requirement-extends.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Describes: X
    Documented:
    . written: "\operatorname{set}"

    [\function:?on{A}:?to{B}]
    Describes: f(x__)
    when: A, B is \set
    Documented:
    . written: "\operatorname{function}"

    [\group]
    Describes: G ::= (X, *, e)
    extends: G is \set
    specifies:
    . X is \set
    . * is \function:on{X}:to{X}
    . e "in" G
    Enables:
    . capability: x_ "in" G :-> x_ is \element.of:group{G}
    Documented:
    . written: "\operatorname{group}"

    [\element.of:group{G}]
    Describes: x
    when: G is \set
    Documented:
    . called: "element of group $G?$"
    . written: "x? \in G?"
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("spec-alias-target-requirement-extends.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 1 file").with_origin("mlg_check")]
        );
    }

    #[test]
    fn check_reports_command_references_with_wrong_curly_argument_count() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("wrong-curly-count.mlg");

        write_mlg_fixture(
            &file,
            r#"[\some.function{A}(x, y)]
    Defines: A "defines" B
    Documented:
    . [docs.called]
      written:
      . "\operatorname{someFunction}"

    Theorem:
    then:
    . f is \some.function{A, B}(x, y)
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("wrong-curly-count.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert!(has_user_error_at(
            &event_log,
            &file.canonicalize().unwrap(),
            10,
            7,
            "Command signature `\\some.function` expects argument shape `{1}(2)` but found `{2}(2)`"
        ));
        assert!(event_log.has_errors());
    }

    #[test]
    fn check_reports_command_references_with_wrong_paren_argument_count() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("wrong-paren-count.mlg");

        write_mlg_fixture(
            &file,
            r#"[\some.function{A}(x, y)]
    Defines: A "defines" B
    Documented:
    . [docs.called]
      written:
      . "\operatorname{someFunction}"

    Theorem:
    then:
    . f is \some.function{A}(x, y, z)
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("wrong-paren-count.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert!(has_user_error_at(
            &event_log,
            &file.canonicalize().unwrap(),
            10,
            7,
            "Command signature `\\some.function` expects argument shape `{1}(2)` but found `{1}(3)`"
        ));
        assert!(event_log.has_errors());
    }

    #[test]
    fn check_requires_defines_describes_and_refines_to_have_documented_called_or_written() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("documented-rendering.mlg");

        write_mlg_fixture(
            &file,
            r#"[\missing.rendering]
    Defines: A "defines" B

    [\describes.missing]
    Describes: A

    [\called.only]
    Describes: A
    Documented:
    . [docs.called]
      called:
    . "called only"

    [\refines.missing]
    Refines: A
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("documented-rendering.mlg")],
            &mut event_log,
        );
        let canonical_file = file.canonicalize().unwrap();

        assert_eq!(result.files_checked, 1);
        assert!(has_user_error_at(
            &event_log,
            &canonical_file,
            0,
            1,
            "Defines entries must include either a `called:` or `written:` item in `Documented:`"
        ));
        assert!(has_user_error_at(
            &event_log,
            &canonical_file,
            4,
            1,
            "Describes entries must include either a `called:` or `written:` item in `Documented:`"
        ));
        assert!(has_user_error_at(
            &event_log,
            &canonical_file,
            16,
            1,
            "Refines entries must include an `adjective:` item in `Documented:`"
        ));
        assert!(event_log.has_errors());
    }

    #[test]
    fn check_accepts_documented_called_or_written() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("documented-valid.mlg");

        write_mlg_fixture(
            &file,
            r#"[\written.only]
    Defines: A is \\opaque
    Documented:
    . [docs.written]
      written:
      . "written only"

    [\called.only]
    Describes: A
    Documented:
    . [docs.called]
      called:
      . "called only"

    [\called.and.written]
    Describes: A
    Documented:
    . [docs.called]
      called:
      . "called and written"
      written:
      . "written"
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("documented-valid.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 1 file").with_origin("mlg_check")]
        );
    }

    #[test]
    fn check_accepts_command_using_context_suffix() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("using-context.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
Describes: X
Documented:
. called: "set"
Id: "0d50b7b0-30b6-4bb1-9fa9-6ac3fcb435f0"

[\ordered.pair]
Describes: p
using: A, B is \set
Documented:
. called: "ordered pair"
Id: "c48e8057-c05f-458d-b7ad-09df94d4e9a4"

[\ordered.pair:of{A}:and{B}]
Defines: p is \ordered.pair#using{A := A; B := B}
when: A, B is \set
Documented:
. called: "ordered pair of $A?$ and $B?$"
Id: "9890d56c-448e-41d1-99cb-9dfbd33f1643"

[\uses.given.context]
Theorem:
given: A is \set
then: A is? \set
Id: "0b75f789-b51c-4741-bdcf-9d1ea2a39ced"

Theorem:
given: X is \set
then: \uses.given.context#given{A := X}
Id: "c812728f-5e16-4774-a62d-00c911127a75"
"#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("using-context.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 1 file").with_origin("mlg_check")]
        );
    }

    #[test]
    fn check_accepts_relationship_enables_with_hard_cast_assumptions() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("relationships.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
Describes: X
Documented:
. called: "set"
Id: "0d50b7b0-30b6-4bb1-9fa9-6ac3fcb435f0"

[\pair]
Describes: P
Documented:
. called: "pair"
Id: "7e446cf6-995e-45aa-9b05-e07bf4be82e1"

[\set.theoretic.pair:of{a}:and{b}]
Defines: P is \set
when: a, b is \set
Documented:
. called: "set-theoretic pair of $a?$ and $b?$"
Id: "9f79d83e-8423-4343-b547-e391b3305994"

[\pair:on{a}:and{b}]
Defines: P is \pair
when: a, b is \set
Enables:
. relation:
  to: \set.theoretic.pair:of{a0}:and{b0}
  when:
  . a0 := a is! \set
  . b0 := b is! \set
  as: \\abstraction
  by: "\some.theorem"
Documented:
. written: "(a?, b?)"
Id: "a95d2ea7-d1fd-41a5-b55c-b6c18c0d05b7"
"#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("relationships.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 1 file").with_origin("mlg_check")]
        );
    }

    #[test]
    fn check_accepts_defines_expansion_symbols_bound_by_definitions() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("defines-bindings.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
Describes: X
Documented:
. called: "set"
Id: "b977c5dd-d79e-426c-8cc8-b028a716c47a"

[\foo:of{a}:and{b}]
Defines: Z ::= (x, y) := (a, b) is \set
when: a, b is \set
Documented:
. called: "foo"
Id: "5800ef12-bed3-427b-985f-ae871a6080ff"

[\foo2:of{a}:and{b}]
Defines: Z ::= (x, y) is \set
when: a, b is \set
expresses:
. x := a
. y := b
Documented:
. called: "foo2"
Id: "5800ef12-bed3-427b-985f-ae871a6080f1"
"#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("defines-bindings.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 1 file").with_origin("mlg_check")]
        );
    }

    #[test]
    fn check_reports_duplicate_defines_expansion_symbol_bindings() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("defines-duplicate-bindings.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
Describes: X
Documented:
. called: "set"
Id: "b977c5dd-d79e-426c-8cc8-b028a716c47a"

[\foo:of{a}:and{b}]
Defines: Z ::= (x, y) is \set
when: a, b is \set
expresses:
. x := a
. x := b
. y := b
Documented:
. called: "foo"
Id: "5800ef12-bed3-427b-985f-ae871a6080ff"
"#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("defines-duplicate-bindings.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        let events = user_events(&event_log);
        assert!(events.iter().any(|event| {
            matches!(event, Event::Message(message) if
                message
                    .message
                    .contains("Duplicate definition for target symbol `x`")
            )
        }));
        assert!(event_log.has_errors());
    }
}

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
        .map(|relative| relative.display().to_string())
}

fn absolute_path(path: &Path, cwd: &Path) -> String {
    let path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        cwd.join(path)
    };

    path.canonicalize().unwrap_or(path).display().to_string()
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
            7,
            1,
            &format!(
                "Duplicate command signature `\\function` in Theorem; previously defined as Defines in {}:1:2",
                canonical_file.display()
            )
        ));
        assert!(event_log.has_errors());
    }

    #[test]
    fn check_reports_references_to_undefined_command_signatures() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("undefined.mlg");

        write_mlg_fixture(
            &file,
            r#"[\function:on{A}:to{B}]
    Defines: A ::= B "defines" B
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
            9,
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
            9,
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
    Defines: A "defines" B
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
    Provides:
    . symbol: x_ "in" X :-> \\abstract
    Documented:
    . written: "\operatorname{set}"

    [\function:on{A}:to{B}]
    Describes: f(x__)
    when: A, B is \set
    Documented:
    . written: "A? \to B?"

    [\(bounded)::function:on{A}:to{B}]
    Refines: f(x__) is \function:on{A}:to{B}
    Documented:
    . adjective: "bounded"
    . written: "\operatorname{bounded}"

    [\(continuous)::function:on{A}:to{B}]
    Refines: f(x__) is \function:on{A}:to{B}
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
    Provides:
    . symbol: x_ "in" X :-> \\abstract
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
    Refines: f(x__) is \function:?on{A}:?to{B}
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
    Refines: f(x__) is \function:?on{A}:?to{B}
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
    Refines: f(x__) is \function
    Documented:
    . adjective: "injective"

    [\(surjective)::function]
    Refines: f(x__) is \function
    Documented:
    . adjective: "surjective"

    [\(bijective)::function]
    Refines: f(x__) is \function
    extends: f is \(injective, surjective)::[[f]]
    Documented:
    . adjective: "bijective"

    [\uses.injective{f}]
    States:
    when: f is? \(injective)::bounded.function
    that: f = f
    Documented:
    . written: "\operatorname{usesInjective}"

    Theorem:
    given: f is \bounded.function
    where: f is? \(bijective)::bounded.function
    then: \uses.injective{f}
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
    fn check_omits_optional_expression_tails_when_arguments_are_not_defined() {
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
    Refines: f(x__) is \function:?on{A}:?to{B}
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
            == "Refines target must exactly match the command after `::` in the refined heading"));
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

    Theorem:
    given:
    . r is \real
    . z is \complex
    . n is \integer
    then:
    . r + z
    . r + n
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

    Theorem:
    given: r is \real
    then:
    . f| r
    . r |f
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
    Provides:
    . symbol: x_ "in" X :-> \\abstract
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
                message.message == "Could not prove requirement `r is \\real` for command `\\foo`"
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
    Provides:
    . symbol: x_ "in" X :-> \\abstract
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
    given: B is \set
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
    when: B is \set
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
                    == "Could not prove requirement `B is \\set` for command `\\:subset:/`"
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
    Provides:
    . symbol: x_ "in" X :-> \\abstract
    Documented:
    . written: "\operatorname{set}"

    [A \:subset:?within{U}:/ B]
    Describes: A
    when:
    . U is \set
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
    . U is \set
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
    Provides:
    . symbol: x_ "in" X :-> \\abstract
    Documented:
    . written: "\operatorname{set}"

    [A \:subset:?within{U}:/ B]
    Describes: A
    when:
    . U is \set
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
    Provides:
    . symbol: x_ "in" X :-> \\abstract
    Documented:
    . written: "\operatorname{set}"

    [A \:subset:/ B]
    Describes: A
    when: B is \set
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
    . \needs.expression{A + B}
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
            "Could not prove requirement `A is \\\\statement` for command `\\needs.statement`"
        )));
        assert!(messages.contains(&String::from(
            "Could not prove requirement `\\wrap{A is? \\set} is \\\\statement` for command `\\needs.statement`"
        )));
        assert!(messages.contains(&String::from(
            "Could not prove requirement `A is \\\\specification` for command `\\needs.specification`"
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
    Provides:
    . symbol: x_ "in" X :-> \\abstract
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
            "Could not prove requirement `g is \\set` for command `\\function:on:to`"
        )));
        assert!(messages.contains(&String::from(
            "Could not prove requirement `x is \\set` for command `\\function:on:to`"
        )));
        let canonical_file = file.canonicalize().unwrap();
        assert!(has_user_error_at(
            &event_log,
            &canonical_file,
            26,
            8,
            "Could not prove requirement `g is \\set` for command `\\function:on:to`"
        ));
        assert!(has_user_error_at(
            &event_log,
            &canonical_file,
            24,
            7,
            "Could not prove requirement `x is \\set` for command `\\function:on:to`"
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
    Provides:
    . symbol: x_ "in" X :-> \\abstract
    Documented:
    . written: "\operatorname{set}"

    [\element.of:group{G ::= (X, *, e)}]
    Describes: x
    when:
    . G is \group
    . X is \set
    extends: x "in" X
    Documented:
    . written: "x? \in G?"

    [\group]
    Describes: G ::= (X, *, e)
    when: X is \set
    extends: G is \set via X
    Provides:
    . symbol: x_ "in" G :-> x_ is \element.of:group{G}
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
    Provides:
    . symbol: x_ "in" X :-> \\abstract
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
    Provides:
    . symbol: x_ "in" X :-> \\abstract
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
                message.message == "Could not prove function call result `f(y) \"in\" C`"
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
            16,
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
    fn check_accepts_optional_command_header_tail_combinations_in_order() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("optional-command-tails.mlg");

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
    . \foo
    . \foo:baz{1}
    . \foo:bar{2}
    . \foo:baz{1}:bar{2}
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
    Provides:
    . symbol: x_ "in" X :-> \\abstract
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
    Provides:
    . symbol: x_ "in" R :-> x is \real
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
    fn check_matches_spec_requirements_without_reducing_to_type_facts() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("direct-spec.mlg");

        write_mlg_fixture(
            &file,
            r#"[\group]
    Describes: G
    Provides:
    . symbol: x_ "in" G :-> \\abstract
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
    Documented:
    . written: "\operatorname{function}"

    [\group]
    Describes: G ::= (X, *, e)
    when:
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
    Documented:
    . written: "\operatorname{function}"

    [\group]
    Describes: G ::= (X, *, e)
    when:
    . X is \set
    . * is \function:on{X}:to{X}
    . e "in" G
    Provides:
    . symbol: x_ "in" G :-> x_ is \element.of:group{G}
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
                    == "Could not prove requirement `A is \\set` for command `\\element.of:group`"
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
    Documented:
    . written: "\operatorname{function}"

    [\group]
    Describes: G ::= (X, *, e)
    when:
    . X is \set
    . * is \function:on{X}:to{X}
    . e "in" G
    Provides:
    . symbol: x_ "in" G :-> x_ is \element.of:group{G}
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
                    == "Could not prove requirement `G is \\set` for command `\\element.of:group`"
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
    Documented:
    . written: "\operatorname{function}"

    [\group]
    Describes: G ::= (X, *, e)
    when:
    . X is \set
    . * is \function:on{X}:to{X}
    . e "in" G
    extends: G is \set
    Provides:
    . symbol: x_ "in" G :-> x_ is \element.of:group{G}
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
            9,
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
            9,
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
    Refines: A is \called.only
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
            3,
            1,
            "Describes entries must include either a `called:` or `written:` item in `Documented:`"
        ));
        assert!(has_user_error_at(
            &event_log,
            &canonical_file,
            13,
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
    Defines: A "defines" B
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
}

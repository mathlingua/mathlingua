use crate::backend::collection::{SourceCollection, find_collection_root};
use crate::backend::config::load_config;
use crate::backend::semantic::DocumentTypeInfo;
use crate::events::{Audience, EventLocation, EventLog, EventLogListener, Level, MarkerRange};
use crate::mlg::format::format_collection;
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
    /// The types resolved for each line of the file named by
    /// [`check_collecting_type_info`]'s `type_info_for`; empty otherwise.
    pub type_info: DocumentTypeInfo,
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
    check_collecting_type_info(cwd, paths, listener, None)
}

/// Checks the collection and, when `type_info_for` names one of its files, also
/// resolves the type of every expression on each of that file's lines.
///
/// The language server uses this so a save produces both the diagnostics and the
/// type information it serves until the next save, from a single check.
pub fn check_collecting_type_info(
    cwd: &Path,
    paths: &[PathBuf],
    listener: Option<Box<dyn EventLogListener>>,
    type_info_for: Option<&Path>,
) -> CheckResult {
    let mut event_log = EventLog::new();
    if let Some(listener) = listener {
        event_log.add_boxed_listener(listener);
    }

    let starting_event_count = event_log.events().len();
    let (summary, type_info) =
        check_in_collecting_type_info(cwd, paths, &mut event_log, type_info_for);
    let successful = no_errors_since(&event_log, starting_event_count);

    CheckResult {
        event_log,
        successful,
        files_checked: summary.files_checked,
        marker_range: summary.marker_range,
        type_info,
    }
}

#[cfg(test)]
pub(super) fn check_in(cwd: &Path, paths: &[PathBuf], event_log: &mut EventLog) -> CheckSummary {
    check_in_collecting_type_info(cwd, paths, event_log, None).0
}

fn check_in_collecting_type_info(
    cwd: &Path,
    paths: &[PathBuf],
    event_log: &mut EventLog,
    type_info_for: Option<&Path>,
) -> (CheckSummary, DocumentTypeInfo) {
    let begin = event_log.begin_marker("check_in", Some(ORIGIN));
    let starting_event_count = event_log.events().len();

    event_log.system_debug(
        Some(ORIGIN),
        format!("Checking {} explicit path(s)", paths.len()),
    );

    format_before_checking(cwd, event_log);

    let mut collection = SourceCollection::load(cwd, event_log, ORIGIN);
    let diagnostic_filter = collection.diagnostic_filter(cwd, paths, event_log, ORIGIN);
    let files_checked = diagnostic_filter.selected_file_count(&collection);

    let type_info =
        collection.run_check_passes_filtered(event_log, ORIGIN, &diagnostic_filter, type_info_for);

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

    (
        CheckSummary {
            files_checked,
            marker_range: MarkerRange::new(begin, end),
        },
        type_info,
    )
}

/// Run `mlg format` over the collection before checking it, unless the config
/// turns that off with `"formatOnCheck": false`.
///
/// Formatting rewrites the very files about to be parsed, so it has to happen
/// before the collection is loaded — a check of the pre-format source would
/// report positions that no longer exist by the time the user looks at them.
///
/// Whole-collection, like `mlg format` itself: even a check narrowed to a few
/// paths reads the whole collection to resolve them, and formatting only the
/// named files would leave the collection in a half-formatted state that
/// depends on which paths were last checked.
///
/// Outside a collection there is no config and no root to format, so this is a
/// no-op — `mlg check` still works on loose files.
fn format_before_checking(cwd: &Path, event_log: &mut EventLog) {
    let start = cwd.canonicalize().unwrap_or_else(|_| cwd.to_path_buf());
    let Some(root) = find_collection_root(&start) else {
        return;
    };
    if !load_config(&root).format_on_check() {
        return;
    }

    // Silent when nothing changed: the interesting news is the check result,
    // and a "Nothing to format" on every run would bury it.
    match format_collection(&root, event_log, ORIGIN) {
        Some(0) | None => {}
        Some(1) => event_log.user_log(Some(ORIGIN), "Formatted 1 file"),
        Some(count) => event_log.user_log(Some(ORIGIN), format!("Formatted {count} files")),
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
    use crate::events::{Audience, Event, EventLocation, EventLog, Level};
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

    /// Asserts a clean check whose summary reads `summary`.
    ///
    /// Unlike comparing the whole event list, this tolerates the `Formatted N
    /// files` that `mlg check` reports when a fixture is not already in
    /// canonical form — several fixtures separate their items by one blank line
    /// where the formatter wants two, which has nothing to do with what those
    /// tests are about.
    pub(super) fn assert_checked_cleanly(event_log: &EventLog, summary: &str) {
        let events = user_events(event_log);
        assert!(
            !event_log.has_errors(),
            "expected a clean check: {events:#?}"
        );
        assert_eq!(
            events.last(),
            Some(&Event::user_log(summary).with_origin("mlg_check")),
            "expected the check summary last: {events:#?}"
        );
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
    fn check_accepts_valid_mathlingua_syntax_in_text_code_fences() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("intro.mlg");

        // The fenced example is syntactically valid, so it passes even though its
        // `\function:on:to` is defined nowhere — fences are syntax-checked only.
        write_mlg_fixture(
            &file,
            r#"
            Text: "
            Example:

            ```mlg
            [\function:on{A}:to{B}]
            Declares: f(x__) ::= y_
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
    fn check_reports_syntax_error_in_text_code_fence() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("intro.mlg");

        // `Bogus:` is not a recognized top-level item, so the fenced code fails to
        // parse and its syntax error is surfaced by `mlg check`.
        write_mlg_fixture(
            &file,
            r#"Text: "
Example:

```mlg
Bogus: \"not a real item\"
```
"
Id: "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa"
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
        assert!(
            user_events(&event_log).iter().any(|event| event
                .as_message()
                .is_some_and(|message| message.message.contains("mlg` code block"))),
            "expected a fenced-code syntax error: {:#?}",
            user_events(&event_log)
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
                "mlg.json is missing required field \"margin\"",
                "mlg.json is missing required field \"formatOnCheck\"",
                "mlg.json is missing required field \"outputDir\"",
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
    fn check_diagnostics_schema_declares_report_shape() {
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
    Declares: value
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
        assert_checked_cleanly(&event_log, "Checked 1 file");
    }

    /// A file the formatter will rewrite: its two top-level items are separated
    /// by one blank line where the canonical form wants two.
    const UNFORMATTED_SOURCE: &str = "Title: \"A\"\nId: \"aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa\"\n\nTitle: \"B\"\nId: \"bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb\"\n";

    const FORMATTED_SOURCE: &str = "Title: \"A\"\nId: \"aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa\"\n\n\nTitle: \"B\"\nId: \"bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb\"\n";

    /// A collection root holding `UNFORMATTED_SOURCE`, configured with `config`.
    fn unformatted_collection(root: &Path, config: &str) -> PathBuf {
        let file = root.join("unformatted.mlg");
        fs::create_dir_all(root).unwrap();
        fs::write(root.join("mlg.json"), config).unwrap();
        fs::write(&file, UNFORMATTED_SOURCE).unwrap();
        file
    }

    #[test]
    fn check_formats_the_collection_by_default() {
        let temp_dir = TestDir::new();
        let root = temp_dir.path().join("repo");
        let file = unformatted_collection(&root, &default_config_contents());

        let mut event_log = EventLog::new();
        check_in(&root, &[], &mut event_log);

        assert_eq!(fs::read_to_string(&file).unwrap(), FORMATTED_SOURCE);
        assert!(
            user_events(&event_log)
                .iter()
                .filter_map(|event| event.as_message())
                .any(|message| message.message == "Formatted 1 file"),
            "the rewrite must be reported: {:#?}",
            user_events(&event_log)
        );
    }

    #[test]
    fn check_leaves_the_collection_alone_when_format_on_check_is_false() {
        let temp_dir = TestDir::new();
        let root = temp_dir.path().join("repo");
        let file = unformatted_collection(
            &root,
            r#"{"name": "a", "version": "1", "margin": 80, "formatOnCheck": false, "outputDir": "docs"}"#,
        );

        let mut event_log = EventLog::new();
        check_in(&root, &[], &mut event_log);

        assert_eq!(fs::read_to_string(&file).unwrap(), UNFORMATTED_SOURCE);
        assert_checked_cleanly(&event_log, "Checked 1 file");
    }

    #[test]
    fn check_formats_the_whole_collection_even_when_given_one_path() {
        // Formatting only the named file would leave the collection in a
        // half-formatted state that depends on which paths were last checked.
        let temp_dir = TestDir::new();
        let root = temp_dir.path().join("repo");
        let unnamed = unformatted_collection(&root, &default_config_contents());
        let named = root.join("named.mlg");
        fs::write(
            &named,
            "Title: \"C\"\nId: \"cccccccc-cccc-4ccc-8ccc-cccccccccccc\"\n",
        )
        .unwrap();

        let mut event_log = EventLog::new();
        check_in(&root, &[PathBuf::from("named.mlg")], &mut event_log);

        assert_eq!(fs::read_to_string(&unnamed).unwrap(), FORMATTED_SOURCE);
    }

    #[test]
    fn check_says_nothing_about_formatting_when_nothing_changed() {
        let temp_dir = TestDir::new();
        let root = temp_dir.path().join("repo");
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("mlg.json"), default_config_contents()).unwrap();
        fs::write(root.join("formatted.mlg"), FORMATTED_SOURCE).unwrap();

        let mut event_log = EventLog::new();
        check_in(&root, &[], &mut event_log);

        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 1 file").with_origin("mlg_check")]
        );
    }

    #[test]
    fn check_outside_a_collection_does_not_format() {
        // There is no root to format and no config to consult, and `mlg check`
        // still has to work on loose files.
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("loose.mlg");
        fs::write(&file, UNFORMATTED_SOURCE).unwrap();

        let mut event_log = EventLog::new();
        check_in(
            temp_dir.path(),
            &[PathBuf::from("loose.mlg")],
            &mut event_log,
        );

        assert_eq!(fs::read_to_string(&file).unwrap(), UNFORMATTED_SOURCE);
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
    fn check_accepts_conjecture_as_an_unproved_statement() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("conjecture.mlg");

        write_mlg_fixture(
            &file,
            r#"[\identity.conjecture]
Conjecture:
given: x is \\expression
then: x = x
Documented:
. called: "Identity conjecture"
"#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("conjecture.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 1 file").with_origin("mlg_check")]
        );
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
specifies: a = b
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
    fn check_reports_undeclared_symbol_in_relation_specifies() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("relation-scope.mlg");

        write_mlg_fixture(
            &file,
            r#"Relation:
between: a is \\expression
and: b is \\expression
specifies: c = c
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
    fn check_accepts_topic_with_within_related_and_documented() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("topics.mlg");

        write_mlg_fixture(
            &file,
            r##"[#analysis]
Topic: "The study of limits, continuity, and convergence."

[#real.analysis]
Topic: "Analysis over the real numbers."
within: "#analysis"
Related:
. to: "#complex.analysis"
  specifies: "Closely connected subjects."
Documented:
. called: "Real Analysis"
"##,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("topics.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 1 file").with_origin("mlg_check")]
        );
    }

    #[test]
    fn check_accepts_relation_between_topics() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("topic-relation.mlg");

        write_mlg_fixture(
            &file,
            r##"[#real.analysis]
Topic: "Analysis over the real numbers."

[#complex.analysis]
Topic: "Analysis over the complex numbers."

Relation:
between: "#real.analysis"
and: "#complex.analysis"
specifies: "Complex analysis extends real analysis."
"##,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("topic-relation.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 1 file").with_origin("mlg_check")]
        );
    }

    #[test]
    fn check_accepts_relation_between_quoted_signatures_with_text_specifies() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("signature-relation.mlg");

        // Quoted `"\..."` subjects are references, not usages, so `\sin`/`\cos`
        // need not be defined; a prose `specifies:` is recorded, not checked, so its
        // `c` is not reported as an undeclared symbol.
        write_mlg_fixture(
            &file,
            r##"Relation:
between: "\sin"
and: "\cos"
specifies: "c makes them cofunctions."
"##,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("signature-relation.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 1 file").with_origin("mlg_check")]
        );
    }

    #[test]
    fn check_accepts_equivalent_over_matching_declares() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("equivalent.mlg");

        write_mlg_fixture(
            &file,
            r#"[\aaa{a}]
Declares: S
when: a is \\expression
Documented:
. called: "aaa"

[\bbb{a}]
Declares: T
when: a is \\expression
Documented:
. called: "bbb"

[\eq{a}]
Equivalent:
when:
. a is \\expression
to:
. \aaa{a}
. \bbb{a}
"#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("equivalent.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert!(
            !event_log.has_errors(),
            "expected a clean check: {:#?}",
            user_events(&event_log)
        );
    }

    #[test]
    fn check_reports_equivalent_mixed_member_kinds() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("equivalent-mixed.mlg");

        write_mlg_fixture(
            &file,
            r#"[\aaa{a}]
Declares: S
when: a is \\expression
Documented:
. called: "aaa"

[\bbb{a}]
Defines: b is \\expression
when: a is \\expression
Documented:
. called: "bbb"

[\eq{a}]
Equivalent:
when:
. a is \\expression
to:
. \aaa{a}
. \bbb{a}
"#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        check_in(
            temp_dir.path(),
            &[PathBuf::from("equivalent-mixed.mlg")],
            &mut event_log,
        );

        assert!(
            user_events(&event_log).iter().any(|event| event
                .as_message()
                .is_some_and(|message| message.message.contains("must be the same kind"))),
            "expected a mixed-kind error: {:#?}",
            user_events(&event_log)
        );
    }

    #[test]
    fn check_reports_equivalent_target_shape_mismatch() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("equivalent-shape.mlg");

        write_mlg_fixture(
            &file,
            r#"[\aaa{a}]
Declares: S
when: a is \\expression
Documented:
. called: "aaa"

[\bbb{a}]
Declares: f(x__)
when: a is \\expression
Documented:
. called: "bbb"

[\eq{a}]
Equivalent:
when:
. a is \\expression
to:
. \aaa{a}
. \bbb{a}
"#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        check_in(
            temp_dir.path(),
            &[PathBuf::from("equivalent-shape.mlg")],
            &mut event_log,
        );

        assert!(
            user_events(&event_log).iter().any(|event| event
                .as_message()
                .is_some_and(|message| message.message.contains("different shapes"))),
            "expected a target-shape mismatch error: {:#?}",
            user_events(&event_log)
        );
    }

    #[test]
    fn check_reports_equivalent_defines_type_mismatch() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("equivalent-deftype.mlg");

        write_mlg_fixture(
            &file,
            r#"[\aaa{a}]
Defines: b is \\expression
when: a is \\expression
Documented:
. called: "aaa"

[\bbb{a}]
Defines: c is \\statement
when: a is \\expression
Documented:
. called: "bbb"

[\eq{a}]
Equivalent:
when:
. a is \\expression
to:
. \aaa{a}
. \bbb{a}
"#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        check_in(
            temp_dir.path(),
            &[PathBuf::from("equivalent-deftype.mlg")],
            &mut event_log,
        );

        assert!(
            user_events(&event_log)
                .iter()
                .any(|event| event.as_message().is_some_and(|message| message
                    .message
                    .contains("define values of different types"))),
            "expected a Defines type-identity error: {:#?}",
            user_events(&event_log)
        );
    }

    #[test]
    fn check_reports_equivalent_when_incompatible_with_member() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("equivalent-when.mlg");

        write_mlg_fixture(
            &file,
            r#"[\aaa{a}]
Declares: S
when: a is \\statement
Documented:
. called: "aaa"

[\bbb{a}]
Declares: T
when: a is \\statement
Documented:
. called: "bbb"

[\eq{a}]
Equivalent:
when:
. a is \\expression
to:
. \aaa{a}
. \bbb{a}
"#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        check_in(
            temp_dir.path(),
            &[PathBuf::from("equivalent-when.mlg")],
            &mut event_log,
        );

        assert!(
            user_events(&event_log)
                .iter()
                .any(|event| event.as_message().is_some_and(|message| message
                    .message
                    .contains("Could not establish requirement")
                    && message.message.contains("\\aaa"))),
            "expected an unsatisfied-requirement error for a member: {:#?}",
            user_events(&event_log)
        );
    }

    #[test]
    fn check_reports_equivalent_to_command_using_non_parameter() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("equivalent-param.mlg");

        write_mlg_fixture(
            &file,
            r#"[\aaa{a}]
Declares: S
when: a is \\expression
Documented:
. called: "aaa"

[\bbb{a}]
Declares: T
when: a is \\expression
Documented:
. called: "bbb"

[\eq{a}]
Equivalent:
when:
. a is \\expression
to:
. \aaa{a}
. \bbb{z}
"#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        check_in(
            temp_dir.path(),
            &[PathBuf::from("equivalent-param.mlg")],
            &mut event_log,
        );

        assert!(
            user_events(&event_log)
                .iter()
                .any(|event| event.as_message().is_some_and(|message| message
                    .message
                    .contains("is not a parameter of the `Equivalent:` header"))),
            "expected a parameter-exactness error: {:#?}",
            user_events(&event_log)
        );
    }

    #[test]
    fn check_reports_undefined_reference_in_equivalently_clause() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("equivalently-clause.mlg");

        write_mlg_fixture(
            &file,
            r#"Theorem:
given: x is \\expression
then:
. equivalently:
  . x is \nonexistent
  . x = x
"#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        check_in(
            temp_dir.path(),
            &[PathBuf::from("equivalently-clause.mlg")],
            &mut event_log,
        );

        assert!(
            user_events(&event_log)
                .iter()
                .any(|event| event.as_message().is_some_and(|message| message
                    .message
                    .contains("Undefined command signature")
                    && message.message.contains("\\nonexistent"))),
            "expected an undefined-command error inside the equivalently clause: {:#?}",
            user_events(&event_log)
        );
    }

    #[test]
    fn check_accepts_requirement_satisfied_through_equivalence() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("equivalence-use.mlg");

        write_mlg_fixture(
            &file,
            r#"[\aaa{a}]
Declares: S
when: a is \\expression
Documented:
. called: "aaa"

[\bbb{a}]
Declares: T
when: a is \\expression
Documented:
. called: "bbb"

[\eq{a}]
Equivalent:
when:
. a is \\expression
to:
. \aaa{a}
. \bbb{a}

[\uses{p}{a}]
Declares: W
when:
. a is \\expression
. p is \aaa{a}
Documented:
. called: "uses"

Theorem:
given:
. z is \\expression
. y is \bbb{z}
then:
. \uses{y}{z} = \uses{y}{z}
"#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("equivalence-use.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert!(
            !event_log.has_errors(),
            "expected `y is \\bbb{{z}}` to satisfy the required `y is \\aaa{{z}}` via \
             equivalence: {:#?}",
            user_events(&event_log)
        );
    }

    #[test]
    fn check_rejects_equivalence_with_mismatched_actual() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("equivalence-mismatch.mlg");

        write_mlg_fixture(
            &file,
            r#"[\aaa{a}]
Declares: S
when: a is \\expression
Documented:
. called: "aaa"

[\bbb{a}]
Declares: T
when: a is \\expression
Documented:
. called: "bbb"

[\eq{a}]
Equivalent:
when:
. a is \\expression
to:
. \aaa{a}
. \bbb{a}

[\uses{p}{a}]
Declares: W
when:
. a is \\expression
. p is \aaa{a}
Documented:
. called: "uses"

Theorem:
given:
. z is \\expression
. w is \\expression
. y is \bbb{w}
then:
. \uses{y}{z} = \uses{y}{z}
"#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        check_in(
            temp_dir.path(),
            &[PathBuf::from("equivalence-mismatch.mlg")],
            &mut event_log,
        );

        assert!(
            user_events(&event_log)
                .iter()
                .any(|event| event.as_message().is_some_and(|message| message
                    .message
                    .contains("Could not establish requirement")
                    && message.message.contains("\\aaa"))),
            "expected `y is \\bbb{{w}}` NOT to satisfy the required `y is \\aaa{{z}}` \
             (mismatched actual): {:#?}",
            user_events(&event_log)
        );
    }

    #[test]
    fn check_resolves_class_capability_on_equivalent_header_typed_value() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("equivalence-capability.mlg");

        write_mlg_fixture(
            &file,
            r#"[\setA]
Declares: X
Enables:
. capability: x_ "belongsA" X :-> \\abstract
Documented:
. called: "setA"

[\setB]
Declares: Y
Enables:
. capability: x_ "belongsA" Y :-> \\abstract
Documented:
. called: "setB"

[\eqset]
Equivalent:
to:
. \setA
. \setB

Theorem:
given:
. p is \\expression
. A is \eqset
then:
. p "belongsA" A
"#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("equivalence-capability.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert!(
            !event_log.has_errors(),
            "expected the class capability to resolve on an `\\eqset`-typed value: {:#?}",
            user_events(&event_log)
        );
    }

    #[test]
    fn check_does_not_resolve_class_capability_on_unrelated_value() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("equivalence-capability-neg.mlg");

        write_mlg_fixture(
            &file,
            r#"[\setA]
Declares: X
Enables:
. capability: x_ "belongsA" X :-> \\abstract
Documented:
. called: "setA"

[\setB]
Declares: Y
Enables:
. capability: x_ "belongsA" Y :-> \\abstract
Documented:
. called: "setB"

[\eqset]
Equivalent:
to:
. \setA
. \setB

[\other]
Declares: Z
Documented:
. called: "other"

Theorem:
given:
. p is \\expression
. C is \other
then:
. p "belongsA" C
"#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        check_in(
            temp_dir.path(),
            &[PathBuf::from("equivalence-capability-neg.mlg")],
            &mut event_log,
        );

        assert!(
            user_events(&event_log).iter().any(|event| event
                .as_message()
                .is_some_and(|message| message.message.contains("\"belongsA\""))),
            "expected `\\other` NOT to resolve the class capability `belongsA`: {:#?}",
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
    when: A, B is \\anything
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
    Declares: value
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
    . y is? \foo{C, D}(z)
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
    fn check_resolves_mapping_parameter_command_overloads() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("mapping-parameter-overloads.mlg");

        write_mlg_fixture(
            &file,
            r#"[\integral{f(x_, y_)}:d{f.x_}]
    Axiom:
    then: f is? \\anything

    [\integral{f(x_, y_)}:d{f.y_}]
    Axiom:
    then: f is? \\anything

    [\integral{f(x_[i_:=1...n])}:d{f.x1?_, f.x2?_}]
    Axiom:
    then: f is? \\anything

    [\integral{f(x_[i_:=1...n])}:d{f.x_[i_[j_:=1...m]]}]
    Axiom:
    then: f is? \\anything

    Theorem:
    then:
    . \integral[x_, y_ is \\anything]{x_}:d{x_}
    . \integral[x_, y_ is \\anything]{y_}:d{y_}
    . \integral[x_, y_, z_ is \\anything]{x_}:d{x_, y_}
    . \integral[x_, y_, z_ is \\anything]{x_}:d{x_, y_, z_}
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("mapping-parameter-overloads.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 1 file").with_origin("mlg_check")]
        );
    }

    #[test]
    fn check_does_not_require_mapping_parameter_placeholders_in_when() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("mapping-parameter-when.mlg");

        write_mlg_fixture(
            &file,
            r#"[\exact.selector{f(x_, y_)}:d{f.x_}]
    States:
    when: f is \\anything
    that: f is? \\anything
    Documented:
    . called: "exact selector"

    [\arbitrary.selector{f(x_, y_)}:d{f.u?_}]
    States:
    when: f is \\anything
    that: f is? \\anything
    Documented:
    . called: "arbitrary selector"

    [\variadic.selector{f(x_[i_:=1...n])}:d{f.x_[i_[j_:=1...m]]}]
    States:
    when: f is \\anything
    that: f is? \\anything
    Documented:
    . called: "variadic selector"
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("mapping-parameter-when.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 1 file").with_origin("mlg_check")]
        );
    }

    #[test]
    fn check_accepts_mapping_literal_for_structural_command_requirement() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("mapping-literal-requirement.mlg");

        write_mlg_fixture(
            &file,
            r#"[\natural]
    Declares: n
    Enables:
    . capability: x_ + y_ :=> x_ \.natural.+./ y_
    Documented:
    . called: "natural"

    [x_ \.natural.+./ y_]
    Defines: z is \natural
    when: x_, y_ is \natural
    Documented:
    . written: "x_? + y_?"

    [\function]
    Declares: f(x__) ::= y_
    specifies:
    . x__ is \\expression
    . y_ is \\anything
    Documented:
    . called: "function"

    [\d{f(x_, y_)}:d{f.x_}]
    Defines: g(x_, y_) is \function
    when: f is \function
    Documented:
    . written: "d"

    Theorem:
    then: \d[x_, y_ is \natural]{x_ + y_}:d{x_}
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("mapping-literal-requirement.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 1 file").with_origin("mlg_check")]
        );
    }

    #[test]
    fn check_accepts_mapping_literal_for_collection_function_requirement() {
        let temp_dir = TestDir::new();
        let file = temp_dir
            .path()
            .join("collection-function-literal-requirement.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Declares: X
    Requires:
    . capability: x_ "in" X :-> \\abstract
    Enables:
    . from: Y ::= {y__ : ...}
      capability: x_ "in" X :<->: x_ member_of Y
    Documented:
    . called: "set"

    [\naturals.set]
    Defines: N := \set@{n_ : n_ is \natural}
    Documented:
    . called: "naturals"

    [\function:?on{A}:?to{B}]
    Declares: f(x__) ::= y_
    when: A, B is \set
    specifies:
    . x__ "in" A
    . y_ "in" B
    Documented:
    . called: "function"

    [A \.set.cross./ B]
    Defines: X := \set@{(a_, b_) : a_ "in" A; b_ "in" B}
    when: A, B is \set
    Documented:
    . called: "cross"

    [\binary.operation:on{X}]
    Declares: x_ * y_ is \function:on{X \.set.cross./ X}:to{X}
    when: X is \set
    Documented:
    . called: "binary operation"

    [n_ \.natural.+./ m_]
    Defines: n_ + m_ is \binary.operation:on{\naturals.set}
    Documented:
    . written: "n_? + m_?"

    [\natural]
    Declares: n
    Enables:
    . capability: x_ + y_ :=> x_ \.natural.+./ y_
    Documented:
    . called: "natural"

    [\d{f(x_, y_)}:d{f.x_}]
    Defines: g(x_, y_) is \function:on{\naturals.set}:to{\naturals.set}
    when: f is \function:on{\naturals.set}:to{\naturals.set}
    Documented:
    . written: "d"

    Theorem:
    then: \d[x_, y_ is \natural]{x_ + y_}:d{x_}
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("collection-function-literal-requirement.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 1 file").with_origin("mlg_check")]
        );
    }

    #[test]
    fn check_accepts_tuple_and_set_literals_for_structural_command_requirements() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("collection-literal-requirements.mlg");

        write_mlg_fixture(
            &file,
            r#"[\natural]
    Declares: n
    Documented:
    . called: "natural"

    [\natural.pair]
    Declares: P ::= (a, b)
    specifies:
    . a, b is \natural
    Documented:
    . called: "natural pair"

    [\natural.set]
    Declares: N ::= {n__ : ...}
    specifies:
    . n__ is \natural
    Documented:
    . called: "natural set"

    [\take.pair{P}]
    Defines: result is \\anything
    when: P is \natural.pair
    Documented:
    . called: "take pair"

    [\take.set{N}]
    Defines: result is \\anything
    when: N is \natural.set
    Documented:
    . called: "take set"

    Theorem:
    given: x, y is \natural
    then:
    . \take.pair{(x, y)}
    . \take.set{{n_ : n_ is \natural}}
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("collection-literal-requirements.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 1 file").with_origin("mlg_check")]
        );
    }

    #[test]
    fn check_rejects_literal_components_with_wrong_structural_types() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("literal-requirement-mismatches.mlg");

        write_mlg_fixture(
            &file,
            r#"[\natural]
    Declares: n
    Documented:
    . called: "natural"

    [\real]
    Declares: r
    Documented:
    . called: "real"

    [\natural.function]
    Declares: f(x_) ::= y_
    specifies:
    . x_, y_ is \natural
    Documented:
    . called: "natural function"

    [\natural.pair]
    Declares: P ::= (a, b)
    specifies:
    . a, b is \natural
    Documented:
    . called: "natural pair"

    [\natural.set]
    Declares: N ::= {n__ : ...}
    specifies:
    . n__ is \natural
    Documented:
    . called: "natural set"

    [\take.function{f}]
    Defines: result is \\anything
    when: f is \natural.function
    Documented:
    . called: "take function"

    [\take.pair{P}]
    Defines: result is \\anything
    when: P is \natural.pair
    Documented:
    . called: "take pair"

    [\take.set{N}]
    Defines: result is \\anything
    when: N is \natural.set
    Documented:
    . called: "take set"

    Theorem:
    given: r is \real
    then:
    . \take.function{(x_ is \real) => x_}
    . \take.pair{(r, r)}
    . \take.set{{r_ : r_ is \real}}
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        check_in(
            temp_dir.path(),
            &[PathBuf::from("literal-requirement-mismatches.mlg")],
            &mut event_log,
        );

        for signature in ["\\take.function", "\\take.pair", "\\take.set"] {
            assert!(
                user_events(&event_log)
                    .iter()
                    .any(|event| event.as_message().is_some_and(|message| message
                        .message
                        .contains("Could not establish requirement")
                        && message.message.contains(signature))),
                "expected a literal type mismatch for {signature}: {:#?}",
                user_events(&event_log)
            );
        }
    }

    #[test]
    fn check_rejects_a_selected_name_that_is_not_a_mapping_parameter() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("invalid-mapping-parameter.mlg");

        write_mlg_fixture(
            &file,
            r#"[\integral{f(x_, y_)}:d{f.x_}]
    Axiom:
    then: f is? \\anything

    Theorem:
    given: c is \\anything
    then: \integral[x_, y_ is \\anything]{x_}:d{c}
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        check_in(
            temp_dir.path(),
            &[PathBuf::from("invalid-mapping-parameter.mlg")],
            &mut event_log,
        );

        assert!(event_log.events().iter().any(|event| {
            event.as_message().is_some_and(|message| {
                message.message == "Undefined command signature `\\integral:d`"
            })
        }));
    }

    #[test]
    fn check_accepts_variadic_commands_and_independent_lengths() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("variadic-valid.mlg");

        write_mlg_fixture(
            &file,
            r#"[\foo{x...n}:bar{y...n}]
    States:
    when:
    . x[1...n] is \\statement
    . y[1...n] is \\statement
    that:
    . x[1...n] = y[1...n]
    . \\map{x[1...i_...n]}:to{x[i_]}
    Documented:
    . called: "foo"

    [\foo2{x...n}:bar2{y...m}]
    States:
    when:
    . x[1...n] is \\statement
    . y[1...m] is \\statement
    that: x[1...n] = x[1...n]
    Documented:
    . called: "foo two"

    Theorem:
    given: P, Q, R, S is \\statement
    then:
    . \foo{P, Q}:bar{R, S}
    . \foo2{P, Q}:bar2{R}
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("variadic-valid.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert!(
            !event_log.has_errors(),
            "expected variadic references to type-check: {:#?}",
            event_log.events()
        );
    }

    #[test]
    fn check_uses_specify_types_as_numeric_literal_fallbacks() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("specified-numeric-literals.mlg");

        write_mlg_fixture(
            &file,
            r#"[\real]
    Declares: x
    Documented:
    . called: "real"

    [\whole]
    Declares: x
    Documented:
    . called: "whole"

    [\natural]
    Declares: x
    Documented:
    . called: "natural"

    [\integer]
    Declares: x
    Documented:
    . called: "integer"

    Specify:
    . decimal:
      is: \real
    . zeroOrPositiveInt:
      is: \whole
    . positiveInt:
      is: \natural
    . int:
      is: \integer

    [\accept.real{x}]
    States:
    when: x is \real
    that: x = x
    Documented:
    . called: "accept real"

    [\accept.whole{x}]
    States:
    when: x is \whole
    that: x = x
    Documented:
    . called: "accept whole"

    [\accept.natural{x}]
    States:
    when: x is \natural
    that: x = x
    Documented:
    . called: "accept natural"

    [\accept.integer{x}]
    States:
    when: x is \integer
    that: x = x
    Documented:
    . called: "accept integer"

    Theorem:
    then:
    . \accept.real{1.2}
    . \accept.whole{0}
    . \accept.natural{1}
    . \accept.integer{-1}
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("specified-numeric-literals.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert!(
            !event_log.has_errors(),
            "expected specified numeric literal types to satisfy requirements: {:#?}",
            user_events(&event_log)
        );
    }

    #[test]
    fn check_types_variadic_indices_from_specify_and_accepts_computed_indices() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("specified-variadic-indices.mlg");

        write_mlg_fixture(
            &file,
            r#"[\whole]
    Declares: x
    Documented:
    . called: "whole"

    [\natural]
    Declares: x
    Documented:
    . called: "natural"

    Specify:
    . zeroOrPositiveInt:
      is: \whole
    . positiveInt:
      is: \natural

    [\mul{x, y}]
    Defines: z is \natural
    when: x, y is \natural
    Documented:
    . called: "multiply"

    [\accept.whole{x}]
    States:
    when: x is \whole
    that: x = x
    Documented:
    . called: "accept whole"

    [\one.based{x[i_ := 1...n]}]
    States:
    when: x[1...n] is \\statement
    that: x[\mul{i, i}]
    Documented:
    . called: "one based"

    [\zero.based{x[i_ := 0...n]}]
    States:
    when: x[0...n] is \\statement
    that: \accept.whole{i}
    Documented:
    . called: "zero based"

    [\two.dimensional{x[(i_, j_) := (1,1)...(m,n)]}]
    States:
    when: x[..., ...] is \\statement
    that: x[\mul{i, i}, j]
    Documented:
    . called: "two dimensional"

    Theorem:
    given: P, Q is \\statement
    then:
    . \one.based{P, Q}
    . \zero.based{P, Q}
    . \two.dimensional{P, Q; Q, P}
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("specified-variadic-indices.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert!(
            !event_log.has_errors(),
            "expected configured and computed variadic indices to type-check: {:#?}",
            user_events(&event_log)
        );
    }

    #[test]
    fn check_rejects_a_computed_variadic_index_with_the_wrong_specified_type() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("invalid-specified-variadic-index.mlg");

        write_mlg_fixture(
            &file,
            r#"[\real]
    Declares: x
    Documented:
    . called: "real"

    [\natural]
    Declares: x
    Documented:
    . called: "natural"

    Specify:
    . positiveInt:
      is: \natural

    [\real.value{x}]
    Defines: z is \real
    when: x is \natural
    Documented:
    . called: "real value"

    [\one.based{x[i_ := 1...n]}]
    States:
    when: x[1...n] is \\statement
    that: x[\real.value{i}]
    Documented:
    . called: "one based"
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        check_in(
            temp_dir.path(),
            &[PathBuf::from("invalid-specified-variadic-index.mlg")],
            &mut event_log,
        );

        assert!(user_events(&event_log).iter().any(|event| {
            event.as_message().is_some_and(|message| {
                message
                    .message
                    .contains("Could not establish index requirement")
                    && message.message.contains("is \\natural")
                    && message.message.contains("variadic parameter `x`")
            })
        }));
    }

    #[test]
    fn check_prefers_a_scoped_numeric_definition_over_specify_fallback() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("scoped-numeric-definition.mlg");

        write_mlg_fixture(
            &file,
            r#"[\real]
    Declares: x
    Documented:
    . called: "real"

    [\natural]
    Declares: x
    Documented:
    . called: "natural"

    Specify:
    . positiveInt:
      is: \natural

    [\accept.natural{x}]
    States:
    when: x is \natural
    that: x = x
    Documented:
    . called: "accept natural"

    Theorem:
    given: 1 is \real
    then: \accept.natural{1}
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        check_in(
            temp_dir.path(),
            &[PathBuf::from("scoped-numeric-definition.mlg")],
            &mut event_log,
        );

        assert!(user_events(&event_log).iter().any(|event| {
            event.as_message().is_some_and(|message| {
                message.message
                    == "Could not establish requirement `1 is \\natural` for command `\\accept.natural`"
            })
        }));
    }

    #[test]
    fn check_accepts_rectangular_two_dimensional_variadic_arguments() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("variadic-2d-valid.mlg");

        write_mlg_fixture(
            &file,
            r#"[\matrix.statement{x[(i_, j_) := (1,1)...(m,n)]}]
    States:
    when: x[i, j] is \\statement
    that: x[i, j] = x[i, j]
    Documented:
    . called: "matrix statement"

    Theorem:
    given: P, Q, R, S is \\statement
    then: \matrix.statement{P, Q; R, S}
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("variadic-2d-valid.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert!(
            !event_log.has_errors(),
            "expected the 2D variadic reference to type-check: {:#?}",
            event_log.events()
        );
    }

    #[test]
    fn check_expands_whole_two_dimensional_selection_requirements_per_cell() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("variadic-2d-whole-selection.mlg");

        write_mlg_fixture(
            &file,
            r#"[\matrix]
    Declares: X
    Documented:
    . called: "matrix"

    [\matrix:of{x[(i_, j_) := (1,1)...(m,n)]}]
    Defines: X is \matrix
    when: x[..., ...] is \\statement
    Documented:
    . written: "\left [ x?{{...\:...}...\\} \right ]"

    Theorem:
    given: P, Q, R, S, T, U is \\statement
    then: \matrix:of{P, Q, R; S, T, U}
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("variadic-2d-whole-selection.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert!(
            !event_log.has_errors(),
            "expected the whole 2D selection to be checked per cell: {:#?}",
            event_log.events()
        );
    }

    #[test]
    fn check_accepts_computed_cells_for_two_dimensional_variadic_requirements() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("variadic-2d-computed-cell.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Declares: X
    Requires:
    . capability: x_ "in" X :-> \\abstract
    Enables:
    . from: Y ::= {y__ : ...}
      capability: x_ "in" X :<->: x_ member_of Y
    Documented:
    . called: "set"

    [\naturals.set]
    Defines: N := \set@{n_ : n_ is \natural}
    Documented:
    . called: "naturals"

    [\function:?on{A}:?to{B}]
    Declares: f(x__) ::= y_
    when: A, B is \set
    specifies:
    . x__ "in" A
    . y_ "in" B
    Documented:
    . called: "function"

    [A \.set.cross./ B]
    Defines: X := \set@{(a_, b_) : a_ "in" A; b_ "in" B}
    when: A, B is \set
    Documented:
    . called: "cross"

    [\binary.operation:on{X}]
    Declares: x_ * y_ is \function:on{X \.set.cross./ X}:to{X}
    when: X is \set
    Documented:
    . called: "binary operation"

    [n_ \.natural.+./ m_]
    Defines: n_ + m_ is \binary.operation:on{\naturals.set}
    Documented:
    . written: "n_? + m_?"

    [\natural]
    Declares: n
    Enables:
    . capability: x_ + y_ :=> x_ \.natural.+./ y_
    Documented:
    . called: "natural"

    [\matrix]
    Declares: X
    Documented:
    . called: "matrix"

    [\matrix:of{x[(i_, j_) := (1,1)...(m,n)]}]
    Defines: X is \matrix
    when: x[..., ...] is \natural
    Documented:
    . written: "matrix"

    Theorem:
    given: a, b, c, x, y, z is \natural
    then: \matrix:of{a, b, c; x, y, z + z}
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("variadic-2d-computed-cell.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 1 file").with_origin("mlg_check")]
        );
    }

    #[test]
    fn check_rejects_ragged_and_flat_arguments_for_two_dimensional_parameters() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("variadic-2d-invalid.mlg");

        write_mlg_fixture(
            &file,
            r#"[\matrix.statement{x[(i_, j_) := (1,1)...(m,n)]}]
    States:
    when: x[i, j] is \\statement
    that: x[i, j] = x[i, j]
    Documented:
    . called: "matrix statement"

    Theorem:
    given: P, Q, R, S is \\statement
    then:
    . \matrix.statement{P, Q; R}
    . \matrix.statement{P, Q, R, S}
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        check_in(
            temp_dir.path(),
            &[PathBuf::from("variadic-2d-invalid.mlg")],
            &mut event_log,
        );

        let shape_errors = user_events(&event_log)
            .iter()
            .filter(|event| {
                event
                    .as_message()
                    .is_some_and(|message| message.message.contains("expects argument shape"))
            })
            .count();
        assert_eq!(shape_errors, 2, "events: {:#?}", event_log.events());
    }

    #[test]
    fn check_rejects_mismatched_shared_variadic_lengths() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("variadic-mismatch.mlg");

        write_mlg_fixture(
            &file,
            r#"[\foo{x...n}:bar{y...n}]
    States:
    when:
    . x[1...n] is \\statement
    . y[1...n] is \\statement
    that: x[1...n] = y[1...n]
    Documented:
    . called: "foo"

    Theorem:
    given: P, Q, R is \\statement
    then: \foo{P, Q}:bar{R}
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("variadic-mismatch.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert!(
            event_log.events().iter().any(|event| {
                event.as_message().is_some_and(|message| {
                    message.message.contains("expects argument shape")
                        && message.message.contains("but found")
                })
            }),
            "expected a shared variadic length error: {:#?}",
            event_log.events()
        );
        assert!(event_log.has_errors());
    }

    #[test]
    fn check_rejects_variadic_slices_with_unsupported_operators() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("variadic-operator.mlg");

        write_mlg_fixture(
            &file,
            r#"[\foo{x...}]
    States:
    when: x... is \\statement
    that: x... + 1
    Documented:
    . called: "foo"
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        check_in(
            temp_dir.path(),
            &[PathBuf::from("variadic-operator.mlg")],
            &mut event_log,
        );

        assert!(
            event_log.events().iter().any(|event| {
                event.as_message().is_some_and(|message| {
                    message
                        .message
                        .contains("variadic slices only support the binary operators `=` and `!=`")
                })
            }),
            "expected an unsupported variadic operator error: {:#?}",
            event_log.events()
        );
    }

    #[test]
    fn check_accepts_command_references_that_omit_defined_paren_arguments() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("optional-parens.mlg");

        write_mlg_fixture(
            &file,
            r#"[\thing]
    Declares: value
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
    . f is? \some.function{X}
    . g is? \some.function{X}(a, b)
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
    fn check_accepts_callable_command_headings_with_placeholder_parameters() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("callable-heading.mlg");

        write_mlg_fixture(
            &file,
            r#"[\natural]
    Declares: N
    Documented:
    . called: "natural"

    [\natural.constructor]
    Declares: succ(n_) ::= m_
    specifies:
    . n_ is \natural
    . m_ is \natural
    Documented:
    . called: "natural constructor"

    [\natural.succ(n_)]
    Defines: succ(n_) is \natural.constructor
    Documented:
    . called: "successor of $n?$"
    . written: "n?+\!\!+"
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("callable-heading.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 1 file").with_origin("mlg_check")]
        );
    }

    #[test]
    fn check_accepts_infix_command_headings_with_placeholder_operands() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("operator-heading.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Declares: X
    Requires:
    . capability: x_ "in" X :-> \\abstract
    Documented:
    . called: "set"

    [A \.set.cross./ B]
    Defines: X := \set@{(a_, b_) : a_ "in" A; b_ "in" B}
    when: A, B is \set
    Documented:
    . called: "Cartesian product"

    [\function:?on{A}:?to{B}]
    Declares: f(x__) ::= y_
    when: A, B is \set
    specifies:
    . x__ "in" A
    . y_ "in" B
    Documented:
    . called: "function"

    [\naturals.set]
    Defines: N is \set
    Documented:
    . called: "natural numbers"

    [\binary.operation:on{X}]
    Declares: x_ * y_ is \function:on{X \.set.cross./ X}:to{X}
    when: X is \set
    Documented:
    . called: "binary operation on $X?$"

    [n_ \.natural.+./ m_]
    Defines: n_ + m_ ::= p_ is \binary.operation:on{\naturals.set}
    expresses: p_ := n_
    Documented:
    . called: "natural addition"

    Theorem:
    given: n, m "in" \naturals.set
    then: n \.natural.+./ m "in" \naturals.set
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("operator-heading.mlg")],
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
    Declares: X
    Enables:
    . capability: x_ "in" X :-> \\abstract
    Documented:
    . written: "\operatorname{set}"

    [\function:on{A}:to{B}]
    Declares: f(x__)
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
    fn check_accepts_refined_spec_infix_definitions_and_references() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("refined-spec-infix.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Declares: X
    Documented:
    . written: "set"

    [A \:subset:/ B]
    Declares: A
    when: B is \set
    Documented:
    . written: "A? \subset B?"

    [A \:(nonempty)::subset:/ B]
    Refines: A
    when: B is \set
    satisfies:
    . A \:subset?:/ B
    Documented:
    . adjective: "nonempty"
    . written: "A? \subset_{+} B?"

    Theorem:
    given:
    . X is \set
    . X' \:(nonempty)::subset:/ X
    then: X' \:subset?:/ X
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("refined-spec-infix.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 1 file").with_origin("mlg_check")]
        );
    }

    #[test]
    fn check_resolves_implicit_refined_spec_infix_through_extended_type() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("implicit-refined-spec-infix.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Declares: X
    Documented:
    . called: "set"

    [\(nonempty)::set]
    Refines: X
    Documented:
    . adjective: "nonempty"

    [A \:subset:/ B]
    Declares: A is \set
    when: B is \set
    Documented:
    . written: "A? \subset B?"

    Theorem:
    given:
    . X is \set
    . X' \:(nonempty)::subset:/ X
    then: X' is? \(nonempty)::set
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("implicit-refined-spec-infix.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 1 file").with_origin("mlg_check")]
        );
    }

    #[test]
    fn check_accepts_placeholder_spec_capability_target() {
        // A capability may map a placeholder spec on the left of `:->` to a spec that
        // reuses that placeholder on the right: `x_ "in" A :-> x_ "in" B` says that
        // `x "in" A` implies `x "in" B`. This must parse and check cleanly.
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("subset.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Declares: X
    Enables:
    . capability: x_ "in" X :-> \\abstract
    Documented:
    . written: "\operatorname{set}"

    [A \:subset:/ B]
    Declares: A is \set
    when: B is \set
    Requires:
    . capability: x_ "in" A :-> x_ "in" B
    Documented:
    . written: "A? \subseteq B?"
    . called: "$A?$ is a subset of $B?$"
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("subset.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 1 file").with_origin("mlg_check")]
        );
    }

    #[test]
    fn check_requires_defines_to_state_its_type() {
        // A `Defines:` value must state the type it defines — either `... is <type>`
        // or a top-level `\ty@value` build. A bare `X := {…}` is rejected even though
        // the `member_of` capability would let the collection literal infer `\set`.
        let set = "[\\set]\nDeclares: X\nRequires:\n. capability: x_ \"in\" X :-> \\\\abstract\nEnables:\n. from: Y ::= {y__ : ...}\n  capability: x_ \"in\" X :-> x_ member_of Y\nDocumented:\n. called: \"set\"\n\n";

        let accepted = [
            // Explicit `is`.
            r#"X := {(a_, b_) : a_ "in" A; b_ "in" B} is \set"#,
            // Top-level build (sugar for the `is`).
            r#"X := \set@{(a_, b_) : a_ "in" A; b_ "in" B}"#,
        ];
        for defines in accepted {
            let temp_dir = TestDir::new();
            let file = temp_dir.path().join("cross.mlg");
            write_mlg_fixture(
                &file,
                &format!(
                    "{set}[A \\.set.cross./ B]\nDefines: {defines}\nwhen: A, B is \\set\nDocumented:\n. called: \"cross\"\n"
                ),
            )
            .unwrap();
            let mut event_log = EventLog::new();
            check_in(
                temp_dir.path(),
                &[PathBuf::from("cross.mlg")],
                &mut event_log,
            );
            assert!(
                !event_log.has_errors(),
                "expected `{defines}` to check, got: {:#?}",
                user_events(&event_log)
            );
        }

        // A bare definition states no type and is rejected.
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("cross.mlg");
        write_mlg_fixture(
            &file,
            &format!(
                "{set}[A \\.set.cross./ B]\nDefines: X := {{(a_, b_) : a_ \"in\" A; b_ \"in\" B}}\nwhen: A, B is \\set\nDocumented:\n. called: \"cross\"\n"
            ),
        )
        .unwrap();
        let mut event_log = EventLog::new();
        check_in(
            temp_dir.path(),
            &[PathBuf::from("cross.mlg")],
            &mut event_log,
        );
        assert!(
            event_log.events().iter().any(|event| {
                event.as_message().is_some_and(|message| {
                    message.level == Level::Error && message.message.contains("must state its type")
                })
            }),
            "expected a bare `Defines:` to be rejected, got: {:#?}",
            user_events(&event_log)
        );
    }

    #[test]
    fn check_accepts_command_build_literals() {
        // `\cmd@<literal>` builds a value of the command's type inline. Here a set is
        // built from a collection literal and used where a `\set` is expected.
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("build.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Declares: X
    Enables:
    . capability: x_ "in" X :-> \\abstract
    Documented:
    . written: "\operatorname{set}"

    Theorem:
    given: X := \set@{x_ : x_ is \set}
    then: X is? \set
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("build.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 1 file").with_origin("mlg_check")]
        );
    }

    #[test]
    fn check_rejects_an_unbuildable_command_build_literal() {
        // A build must be establishable by construction; here `n` is an expression, so
        // `\set@n` cannot build a set and is reported.
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("bad-build.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Declares: X
    Enables:
    . capability: x_ "in" X :-> \\abstract
    Documented:
    . written: "\operatorname{set}"

    Theorem:
    given: n is \\expression
    then: \set@n is? \set
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        check_in(
            temp_dir.path(),
            &[PathBuf::from("bad-build.mlg")],
            &mut event_log,
        );

        assert!(
            event_log.events().iter().any(|event| {
                event.as_message().is_some_and(|message| {
                    message.level == Level::Error
                        && message.message.contains("Could not build `\\set@n`")
                })
            }),
            "expected an unbuildable-literal error, got: {:#?}",
            user_events(&event_log)
        );
    }

    #[test]
    fn check_accepts_inferred_parameters() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("inferred-parameters.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Declares: X
    Enables:
    . capability: x_ "in" X :-> \\abstract
    Documented:
    . written: "\operatorname{set}"

    [\function:on{A}:to{B}]
    Declares: f(x__)
    when: A, B is \set
    Documented:
    . written: "A? \to B?"

    Theorem:
    given:
    . f is \function:on{A?}:to{B?}
    . x "in" A
    then:
    . x "in" A
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("inferred-parameters.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 1 file").with_origin("mlg_check")]
        );
    }

    #[test]
    fn check_reports_inferred_parameter_type_misuse() {
        // `A?` locks `A`'s type to `\set` (the type its `\function` argument
        // position requires). Using `A` where a `\group` is required must then be
        // rejected by requirement checking.
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("inferred-misuse.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Declares: X
    Enables:
    . capability: x_ "in" X :-> \\abstract
    Documented:
    . written: "\operatorname{set}"

    [\group]
    Declares: X
    Documented:
    . written: "\operatorname{group}"

    [\function:on{A}:to{B}]
    Declares: f(x__)
    when: A, B is \set
    Documented:
    . written: "A? \to B?"

    [\needs.group{G}]
    Declares: x
    when: G is \group
    Documented:
    . written: "\operatorname{needsGroup}"

    Theorem:
    given:
    . f is \function:on{A?}:to{B?}
    then:
    . \needs.group{A}
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("inferred-misuse.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert!(
            user_events(&event_log).iter().any(|event| {
                event.as_message().is_some_and(|message| {
                    message.message
                        == "Could not establish requirement `A is \\group` for command `\\needs.group`"
                })
            }),
            "expected a requirement mismatch because `A` is a `\\set`, not a `\\group`, got {:#?}",
            user_events(&event_log)
        );
    }

    #[test]
    fn check_reports_reintroduced_inferred_parameter() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("inferred-reintroduced.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Declares: X
    Enables:
    . capability: x_ "in" X :-> \\abstract
    Documented:
    . written: "\operatorname{set}"

    [\function:on{A}:to{B}]
    Declares: f(x__)
    when: A, B is \set
    Documented:
    . written: "A? \to B?"

    Theorem:
    given:
    . A is \set
    . f is \function:on{A?}:to{B?}
    then:
    . A is \set
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("inferred-reintroduced.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert!(
            user_events(&event_log).iter().any(|event| {
                event.as_message().is_some_and(|message| {
                    message.message == "Inferred parameter `A` is already introduced"
                })
            }),
            "expected a re-introduction error, got {:#?}",
            user_events(&event_log)
        );
    }

    #[test]
    fn check_accepts_refines_adjectives_and_optional_expression_tails() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("refined-adjective.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Declares: X
    Enables:
    . capability: x_ "in" X :-> \\abstract
    Documented:
    . called: "set"
    . written: "\operatorname{set}"

    [\function:?on{A}:?to{B}]
    Declares: f(x__)
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
    fn check_uses_declares_function_signature_specifies() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("function-signature-specifies.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Declares: X
    Enables:
    . capability: x_ "in" X :-> \\abstract
    Documented:
    . written: "\operatorname{set}"

    [\function:?on{A}:?to{B}]
    Declares: f(x__) ::= y_
    when: A, B is \set
    specifies:
    . x__ "in" A
    . y_ "in" B
    Documented:
    . called: "function"
    . written: "f?"

    [\ternary.function:?on{A}:?to{B}]
    Declares: g(x_, y_, z_) ::= w_
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
    fn check_requires_used_optional_declares_parameters_in_when() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("used-optional-declares-parameter.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Declares: X
    Enables:
    . capability: x_ "in" X :-> \\abstract
    Documented:
    . written: "\operatorname{set}"

    [\function:?on{A}:?to{B}]
    Declares: f(x__) ::= y_
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
            &[PathBuf::from("used-optional-declares-parameter.mlg")],
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
    fn check_applies_refines_specifies_to_dynamic_refined_base() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("dynamic-refined-base.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Declares: X
    Documented:
    . written: "\operatorname{set}"

    [\function]
    Declares: f(x__)
    Documented:
    . written: "\operatorname{function}"

    [\bounded.function]
    Declares: f(x__) is \function
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
    specifies: f is \(injective, surjective)::[[f]]
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
    fn check_accepts_implicitly_marker_restating_inherited_refinement() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("implicitly-marker.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Declares: X
    Documented:
    . written: "\operatorname{set}"

    [\function]
    Declares: f(x__)
    Documented:
    . written: "\operatorname{function}"

    [\bounded.function]
    Declares: f(x__) is \function
    Documented:
    . written: "\operatorname{boundedFunction}"

    [\(injective)::function]
    Refines: f(x__)
    Documented:
    . adjective: "injective"

    [\(injective)::bounded.function]
    Refines: f(x__)
    implicitly:
    specifies: f is \(injective)::function
    Documented:
    . adjective: "injective"
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("implicitly-marker.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 1 file").with_origin("mlg_check")]
        );
    }

    #[test]
    fn check_rejects_implicitly_marker_not_naming_parent_refinement() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("implicitly-wrong-parent.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Declares: X
    Documented:
    . written: "\operatorname{set}"

    [\function]
    Declares: f(x__)
    Documented:
    . written: "\operatorname{function}"

    [\bounded.function]
    Declares: f(x__) is \function
    Documented:
    . written: "\operatorname{boundedFunction}"

    [\(injective)::function]
    Refines: f(x__)
    Documented:
    . adjective: "injective"

    [\(injective)::bounded.function]
    Refines: f(x__)
    implicitly:
    specifies: f is \(injective)::[[f]]
    Documented:
    . adjective: "injective"
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        check_in(
            temp_dir.path(),
            &[PathBuf::from("implicitly-wrong-parent.mlg")],
            &mut event_log,
        );

        assert!(
            user_events(&event_log)
                .iter()
                .any(|event| event.as_message().is_some_and(|message| message
                    .message
                    .contains("name the parent type's refinement"))),
            "expected a parent-refinement error: {:#?}",
            user_events(&event_log)
        );
    }

    #[test]
    fn check_rejects_implicitly_marker_that_adds_properties() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("implicitly-extra.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Declares: X
    Documented:
    . written: "\operatorname{set}"

    [\function]
    Declares: f(x__)
    Documented:
    . written: "\operatorname{function}"

    [\bounded.function]
    Declares: f(x__) is \function
    Documented:
    . written: "\operatorname{boundedFunction}"

    [\(injective)::function]
    Refines: f(x__)
    Documented:
    . adjective: "injective"

    [\(injective)::bounded.function]
    Refines: f(x__)
    implicitly:
    specifies: f is \(injective)::[[f]]
    satisfies:
    . forAll: x__
      then: f(x__) = f(x__)
    Documented:
    . adjective: "injective"
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        check_in(
            temp_dir.path(),
            &[PathBuf::from("implicitly-extra.mlg")],
            &mut event_log,
        );

        assert!(
            user_events(&event_log)
                .iter()
                .any(|event| event.as_message().is_some_and(|message| message
                    .message
                    .contains("must contain only the inherited"))),
            "expected an implicitly-marker error: {:#?}",
            user_events(&event_log)
        );
    }

    #[test]
    fn check_accepts_explicitly_marker_that_adds_properties() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("explicitly-marker.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Declares: X
    Documented:
    . written: "\operatorname{set}"

    [\function]
    Declares: f(x__)
    Documented:
    . written: "\operatorname{function}"

    [\bounded.function]
    Declares: f(x__) is \function
    Documented:
    . written: "\operatorname{boundedFunction}"

    [\(injective)::function]
    Refines: f(x__)
    Documented:
    . adjective: "injective"

    [\(injective)::bounded.function]
    Refines: f(x__)
    explicitly:
    specifies: f is \(injective)::[[f]]
    satisfies:
    . forAll: x__
      then: f(x__) = f(x__)
    Documented:
    . adjective: "injective"
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("explicitly-marker.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 1 file").with_origin("mlg_check")]
        );
    }

    #[test]
    fn check_rejects_explicitly_marker_without_added_properties() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("explicitly-trivial.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Declares: X
    Documented:
    . written: "\operatorname{set}"

    [\function]
    Declares: f(x__)
    Documented:
    . written: "\operatorname{function}"

    [\bounded.function]
    Declares: f(x__) is \function
    Documented:
    . written: "\operatorname{boundedFunction}"

    [\(injective)::function]
    Refines: f(x__)
    Documented:
    . adjective: "injective"

    [\(injective)::bounded.function]
    Refines: f(x__)
    explicitly:
    specifies: f is \(injective)::[[f]]
    Documented:
    . adjective: "injective"
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        check_in(
            temp_dir.path(),
            &[PathBuf::from("explicitly-trivial.mlg")],
            &mut event_log,
        );

        assert!(
            user_events(&event_log).iter().any(|event| event
                .as_message()
                .is_some_and(|message| message.message.contains("must add at least one property"))),
            "expected an explicitly-marker error: {:#?}",
            user_events(&event_log)
        );
    }

    #[test]
    fn check_rejects_refinement_marker_with_arguments() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("marker-with-args.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Declares: X
    Documented:
    . written: "\operatorname{set}"

    [\function]
    Declares: f(x__)
    Documented:
    . written: "\operatorname{function}"

    [\bounded.function]
    Declares: f(x__) is \function
    Documented:
    . written: "\operatorname{boundedFunction}"

    [\(injective)::bounded.function]
    Refines: f(x__)
    implicitly: f
    specifies: f is \(injective)::[[f]]
    Documented:
    . adjective: "injective"
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        check_in(
            temp_dir.path(),
            &[PathBuf::from("marker-with-args.mlg")],
            &mut event_log,
        );

        assert!(
            user_events(&event_log).iter().any(|event| event
                .as_message()
                .is_some_and(|message| message.message.contains("takes no arguments"))),
            "expected a marker-argument error: {:#?}",
            user_events(&event_log)
        );
    }

    #[test]
    fn check_rejects_both_refinement_markers() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("marker-both.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Declares: X
    Documented:
    . written: "\operatorname{set}"

    [\function]
    Declares: f(x__)
    Documented:
    . written: "\operatorname{function}"

    [\bounded.function]
    Declares: f(x__) is \function
    Documented:
    . written: "\operatorname{boundedFunction}"

    [\(injective)::bounded.function]
    Refines: f(x__)
    implicitly:
    explicitly:
    specifies: f is \(injective)::[[f]]
    Documented:
    . adjective: "injective"
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        check_in(
            temp_dir.path(),
            &[PathBuf::from("marker-both.mlg")],
            &mut event_log,
        );

        assert!(
            user_events(&event_log)
                .iter()
                .any(|event| event.as_message().is_some_and(|message| message
                    .message
                    .contains("at most one of `implicitly:` or `explicitly:`"))),
            "expected a mutual-exclusivity error: {:#?}",
            user_events(&event_log)
        );
    }

    #[test]
    fn check_rejects_refinement_marker_on_non_subtype_base() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("marker-non-subtype.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Declares: X
    Documented:
    . written: "\operatorname{set}"

    [\function]
    Declares: f(x__)
    Documented:
    . written: "\operatorname{function}"

    [\(injective)::function]
    Refines: f(x__)
    implicitly:
    specifies: f is \(injective)::[[f]]
    Documented:
    . adjective: "injective"
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        check_in(
            temp_dir.path(),
            &[PathBuf::from("marker-non-subtype.mlg")],
            &mut event_log,
        );

        assert!(
            user_events(&event_log).iter().any(|event| event
                .as_message()
                .is_some_and(|message| message.message.contains("subtype of another type"))),
            "expected a non-subtype marker error: {:#?}",
            user_events(&event_log)
        );
    }

    #[test]
    fn check_locates_errors_outside_fenced_examples() {
        // A ```mlg fence embedded in a `Text:` value contains a definition that
        // looks just like the real one below but *has* its `when:`. The real
        // definition is missing `when:`, so the checker must anchor the
        // missing-`when:` error to the real definition, never to the earlier
        // look-alike inside the prose fence.
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("fence-location.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Declares: X
    Enables:
    . capability: x_ "in" X :-> \\abstract
    Documented:
    . written: "\operatorname{set}"

    Text: "Example:
           ```mlg
           [\foo:bar{A}]
           Declares: X
           when: A is \set
           specifies:
           . X \"in\" A
           ```"
    Id: "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa"

    [\foo:bar{A}]
    Declares: X
    specifies:
    . X "in" A
    Documented:
    . written: "\operatorname{foo}"
    Id: "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb"
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        check_in(
            temp_dir.path(),
            &[PathBuf::from("fence-location.mlg")],
            &mut event_log,
        );

        let source = fs::read_to_string(&file).unwrap();
        let lines = source.lines().collect::<Vec<_>>();
        let real_def_line = lines
            .iter()
            .rposition(|line| line.contains("[\\foo:bar"))
            .expect("real definition present");
        let fence_line = lines
            .iter()
            .position(|line| line.contains("[\\foo:bar"))
            .expect("fenced look-alike present");
        assert_ne!(real_def_line, fence_line, "fixture needs both occurrences");

        let error_row = user_events(&event_log)
            .iter()
            .find_map(|event| {
                let message = event.as_message()?;
                if !message
                    .message
                    .contains("Missing `when:` requirement for parameter `A`")
                {
                    return None;
                }
                match message.location.as_ref()? {
                    EventLocation::File { span, .. } => span.as_ref()?.start.row,
                    EventLocation::InMemory { span, .. } => span.as_ref()?.start.row,
                }
            })
            .expect("a located missing-`when:` error");

        assert!(
            error_row >= real_def_line,
            "missing-`when:` error at row {error_row} should be at/after the real definition \
             (line {real_def_line}), not inside the fenced example (line {fence_line})"
        );
    }

    #[test]
    fn check_keeps_item_errors_within_the_item() {
        // A definition with several failing spec facts is followed by another
        // item whose body contains the same `"in"`/symbol text. Every error must
        // be anchored inside the definition, never spilling onto the next item.
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("item-window.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Declares: X
    Enables:
    . capability: x_ "in" X :-> \\abstract
    Documented:
    . written: "\operatorname{set}"

    [\fun:?on{A}:?to{B}]
    Declares: f(x__) ::= y_
    specifies:
    . x__ "in" A
    . y_ "in" B
    satisfies:
    . forAll: x "in" A
      then:
      . existsUnique: y "in" B
        suchThat: f(x) = y
    Documented:
    . written: "\operatorname{fun}"

    [\pair:on{A}:to{B}]
    Defines: P := \set@{(a_, b_) : a_ "in" A; b_ "in" B}
    when: A, B is \set
    Documented:
    . written: "P"
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        check_in(
            temp_dir.path(),
            &[PathBuf::from("item-window.mlg")],
            &mut event_log,
        );

        let source = fs::read_to_string(&file).unwrap();
        let next_item_line = source
            .lines()
            .position(|line| line.contains("[\\pair"))
            .expect("next item present");

        let error_rows = user_events(&event_log)
            .iter()
            .filter_map(|event| {
                let message = event.as_message()?;
                if message.level != Level::Error {
                    return None;
                }
                match message.location.as_ref()? {
                    EventLocation::File { span, .. } => span.as_ref()?.start.row,
                    EventLocation::InMemory { span, .. } => span.as_ref()?.start.row,
                }
            })
            .collect::<Vec<_>>();

        assert!(!error_rows.is_empty(), "expected located errors");
        assert!(
            error_rows.iter().all(|&row| row < next_item_line),
            "every error row {error_rows:?} must precede the next item (line {next_item_line})"
        );
    }

    #[test]
    fn check_infers_refines_operator_type_from_refined_base() {
        // The `\(associative)::binary.operation:on{X}` refinement uses `*` in its
        // `satisfies:` without respecifying it: `*`'s type (a function) is
        // inherited from the base `\binary.operation:on{X}`. The operator must
        // resolve and `*` must count as a specified target symbol.
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("refines-operator.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Declares: X
    Requires:
    . capability: x_ "in" X :-> \\abstract
    Documented:
    . written: "\operatorname{set}"

    [A \.set.cross./ B]
    Defines: X := \set@{(a_, b_) : a_ "in" A; b_ "in" B}
    when: A, B is \set
    Documented:
    . written: "A? \times B?"

    [\function:?on{A}:?to{B}]
    Declares: f(x__) ::= y_
    when: A, B is \set
    specifies:
    . x__ "in" A
    . y_ "in" B
    Documented:
    . written: "f?"

    [\binary.operation:on{X}]
    Declares: x_ * y_ is \function:on{X \.set.cross./ X}:to{X}
    when: X is \set
    Documented:
    . written: "\operatorname{binop}"

    [\(associative)::binary.operation:on{X}]
    Refines: x_ * y_
    when: X is \set
    satisfies:
    . forAll: a, b, c "in" X
      then: (a * b) * c = a * (b * c)
    Documented:
    . adjective: "associative"
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("refines-operator.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 1 file").with_origin("mlg_check")]
        );
    }

    #[test]
    fn check_treats_a_declares_target_relation_and_an_extends_section_as_equivalent() {
        // `Declares: M ::= (X, *) is \set via X` and the same target with a
        // separate `extends: M is \set via X` say the same thing: both give the
        // subtype implication and both type the `via` symbol.
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("extends-equivalence.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Declares: X
    Requires:
    . capability: x_ "in" X :-> \\abstract
    Documented:
    . called: "set"

    [\magma]
    Declares: M ::= (X, *) is \set via X
    specifies: * is \set
    Documented:
    . called: "magma"

    [\magma.too]
    Declares: M ::= (X, *)
    extends: M is \set via X
    specifies: * is \set
    Documented:
    . called: "magma too"

    Theorem:
    given: M ::= (X, *) is \magma
    then:
    . M is? \set
    . X is? \set

    Theorem:
    given: M ::= (X, *) is \magma.too
    then:
    . M is? \set
    . X is? \set
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("extends-equivalence.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 1 file").with_origin("mlg_check")]
        );
    }

    #[test]
    fn check_rejects_a_specifies_item_that_respecifies_an_extended_symbol() {
        // `via X` already states `X is \set`, so repeating it in `specifies:` states
        // the same symbol's type twice.
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("duplicate-specification.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Declares: X
    Requires:
    . capability: x_ "in" X :-> \\abstract
    Documented:
    . called: "set"

    [\function:on{A}:to{B}]
    Declares: f(x__) is (_ "in" A) -> (_ "in" B)
    when: A, B is \set
    Documented:
    . called: "function"

    [\group]
    Declares: G ::= (X, *, e) is \set via X
    specifies:
    . X is \set
    . * is \function:on{X}:to{X}
    . e "in" G
    Documented:
    . called: "group"
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        check_in(
            temp_dir.path(),
            &[PathBuf::from("duplicate-specification.mlg")],
            &mut event_log,
        );

        assert!(
            user_events(&event_log).iter().any(|event| {
                event.as_message().is_some_and(|message| {
                    message.message
                        == "Duplicate specification for target symbol `X`; it is already specified by the `Declares:` target"
                })
            }),
            "{:#?}",
            user_events(&event_log)
        );
    }

    #[test]
    fn check_accepts_several_extends_clauses_reaching_the_same_symbol() {
        // `extends:` exists so one definition can extend several types, so two
        // clauses may name the same subject and reach the same component
        // through different views. That is one specification, not two.
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("shared-via-component.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Declares: X
    Requires:
    . capability: x_ "in" X :-> \\abstract
    Documented:
    . called: "set"

    [\left.pair]
    Declares: L ::= (U, V)
    specifies:
    . U is \set
    . V is \set
    Documented:
    . called: "left pair"

    [\right.pair]
    Declares: R ::= (S, T)
    specifies:
    . S is \set
    . T is \set
    Documented:
    . called: "right pair"

    [\triple]
    Declares: X ::= (A, B, C)
    extends:
    . X is \left.pair via (A, B)
    . X is \right.pair via (B, C)
    Documented:
    . called: "triple"
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("shared-via-component.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 1 file").with_origin("mlg_check")]
        );
    }

    #[test]
    fn check_applies_every_extends_clause_of_a_declares_group() {
        // An `extends:` section may name several types, each through a
        // different `via` view of the target's tuple — the case a single `is`
        // on the `Declares:` target cannot express.
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("multi-extends.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Declares: X
    Requires:
    . capability: x_ "in" X :-> \\abstract
    Documented:
    . called: "set"

    [\left.pair]
    Declares: L ::= (U, V)
    specifies:
    . U is \set
    . V is \set
    Documented:
    . called: "left pair"

    [\right.pair]
    Declares: R ::= (S, T)
    specifies:
    . S is \set
    . T is \set
    Documented:
    . called: "right pair"

    [\triple]
    Declares: X ::= (A, B, C)
    extends:
    . X is \left.pair via (A, B)
    . X is \right.pair via (B, C)
    Documented:
    . called: "triple"

    Theorem:
    given: X ::= (A, B, C) is \triple
    then:
    . X is? \left.pair
    . X is? \right.pair
    . A is? \set
    . B is? \set
    . C is? \set
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("multi-extends.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 1 file").with_origin("mlg_check")]
        );
    }

    /// The shared prelude for the abstract-declaration tests: a set, a function
    /// type, and a value to realize the naturals with.
    const REALIZES_PRELUDE: &str = r#"[\set]
    Declares: X
    Requires:
    . capability: x_ "in" X :-> \\abstract
    Documented:
    . called: "set"

    [\function:on{A}:to{B}]
    Declares: f(x__) is (_ "in" A) -> (_ "in" B)
    when: A, B is \set
    Documented:
    . called: "function"

    [\empty.set]
    Defines: E := \set@{x : ...}
    Documented:
    . called: "empty set"

    [\naturals]
    Defines: Nb ::= (N, 0, succ(n_))
    abstractly:
    specifies:
    . N is \set
    . 0 "in" N
    . succ is \function:on{N}:to{N}
    Documented:
    . called: "naturals"

"#;

    /// `goldens/examples/` is a complete MathLingua collection that exercises
    /// every language feature and checks cleanly. It is a living document:
    /// this test runs the real checker over it, so an example cannot drift from
    /// what the implementation accepts, and asserts every top-level item kind
    /// appears, so a new kind cannot be added without an example.
    ///
    /// The collection is copied to a temporary directory first because a check
    /// rewrites its input — it formats each file and writes in any missing
    /// `Id:` — and a test should not modify the source tree.
    #[test]
    fn check_accepts_the_golden_examples_collection() {
        let source = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/goldens/examples"));
        let temp_dir = TestDir::new();
        let root = temp_dir.path().join("examples");
        copy_tree(source, &root);

        let mut event_log = EventLog::new();
        let result = check_in(&root, &[], &mut event_log);

        assert_eq!(
            user_events(&event_log),
            [
                Event::user_log(format!("Checked {} files", result.files_checked))
                    .with_origin("mlg_check")
            ],
            "goldens/examples must check cleanly"
        );

        let mut seen = std::collections::BTreeSet::new();
        for entry in fs::read_dir(root.join("content")).expect("expected a content directory") {
            let path = entry.expect("expected a directory entry").path();
            let text = fs::read_to_string(&path).expect("expected a readable example");
            for line in text.lines() {
                if let Some(label) = line.split(':').next()
                    && !label.is_empty()
                    && line.starts_with(label)
                    && line[label.len()..].starts_with(':')
                {
                    seen.insert(label.to_owned());
                }
            }
        }
        for kind in [
            "Axiom",
            "Conjecture",
            "Defines",
            "Declares",
            "Disambiguates",
            "Equivalent",
            "Person",
            "Realizes",
            "Refines",
            "Relation",
            "Resource",
            "SectionTitle",
            "Specify",
            "States",
            "SubsectionTitle",
            "Text",
            "TextAxiom",
            "TextConjecture",
            "TextDefinition",
            "TextTheorem",
            "Theorem",
            "Title",
            "Topic",
            "Writing",
        ] {
            assert!(
                seen.contains(kind),
                "goldens/examples must show a `{kind}:` item"
            );
        }
    }

    /// Recursively copies a directory, for tests that need to run a command
    /// that writes to its input.
    fn copy_tree(from: &Path, to: &Path) {
        fs::create_dir_all(to).expect("expected to create the destination");
        for entry in fs::read_dir(from).expect("expected a readable directory") {
            let entry = entry.expect("expected a directory entry");
            let target = to.join(entry.file_name());
            if entry.file_type().expect("expected a file type").is_dir() {
                copy_tree(&entry.path(), &target);
            } else {
                fs::copy(entry.path(), &target).expect("expected to copy a file");
            }
        }
    }

    #[test]
    fn check_accepts_a_realization_of_an_abstract_declaration() {
        // A `Realizes:` supplies a value for every symbol its declaration left
        // abstract, and the realized components keep the declaration's types —
        // so destructuring either one proves the same facts.
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("realizes.mlg");

        write_mlg_fixture(
            &file,
            &format!(
                r#"{REALIZES_PRELUDE}    [\von.neumann.naturals]
    Realizes: Nb := \naturals
    specifies:
    . N := \empty.set
    . 0 := \empty.set
    . succ(n_) := \empty.set
    Documented:
    . called: "von Neumann naturals"

    Theorem:
    given: Nb ::= (N, 0, succ(n_)) := \naturals
    then:
    . N is? \set
    . 0 "in"? N

    Theorem:
    given: Nb ::= (N, 0, succ(n_)) := \von.neumann.naturals
    then:
    . N is? \set
    . 0 "in"? N
    "#
            ),
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("realizes.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 1 file").with_origin("mlg_check")]
        );
    }

    #[test]
    fn check_reports_an_abstract_symbol_a_realization_omits() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("incomplete-realization.mlg");

        write_mlg_fixture(
            &file,
            &format!(
                r#"{REALIZES_PRELUDE}    [\partial.naturals]
    Realizes: Nb := \naturals
    specifies:
    . N := \empty.set
    Documented:
    . called: "partial naturals"
    "#
            ),
        )
        .unwrap();

        let mut event_log = EventLog::new();
        check_in(
            temp_dir.path(),
            &[PathBuf::from("incomplete-realization.mlg")],
            &mut event_log,
        );

        let messages = user_events(&event_log);
        for symbol in ["0", "succ"] {
            assert!(
                messages.iter().any(|event| {
                    event.as_message().is_some_and(|message| {
                        message.message
                            == format!(
                                "Missing realization for abstract symbol `{symbol}`; a `Realizes:` must supply every symbol its declaration leaves abstract"
                            )
                    })
                }),
                "expected `{symbol}` to be reported: {messages:#?}"
            );
        }
    }

    #[test]
    fn check_requires_a_realizes_target_to_name_an_abstract_declaration() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("wrong-realization.mlg");

        write_mlg_fixture(
            &file,
            &format!(
                r#"{REALIZES_PRELUDE}    [\not.a.declaration]
    Realizes: Nb := \set
    Documented:
    . called: "not a declaration"
    "#
            ),
        )
        .unwrap();

        let mut event_log = EventLog::new();
        check_in(
            temp_dir.path(),
            &[PathBuf::from("wrong-realization.mlg")],
            &mut event_log,
        );

        assert!(
            user_events(&event_log).iter().any(|event| {
                event.as_message().is_some_and(|message| {
                    message.message
                        == "`Realizes:` must name a `Defines:` marked `abstractly:`; `\\set` is a `Declares:`"
                })
            }),
            "{:#?}",
            user_events(&event_log)
        );
    }

    #[test]
    fn check_requires_a_concrete_defines_specifies_item_to_supply_a_value() {
        // Without `abstractly:`, a `specifies:` item that only states a type leaves
        // the symbol undefined; `expresses:` is the indirect way to define it.
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("concrete-specifies.mlg");

        write_mlg_fixture(
            &file,
            &format!(
                r#"{REALIZES_PRELUDE}    [\concrete.pair]
    Defines: P ::= (A, B)
    specifies:
    . A := \empty.set
    . B is \set
    Documented:
    . called: "concrete pair"
    "#
            ),
        )
        .unwrap();

        let mut event_log = EventLog::new();
        check_in(
            temp_dir.path(),
            &[PathBuf::from("concrete-specifies.mlg")],
            &mut event_log,
        );

        let messages = user_events(&event_log);
        assert!(
            messages.iter().any(|event| {
                event.as_message().is_some_and(|message| {
                    message.message
                        == "`B` states a specification but no value; define it with `:=`, define it in `expresses:`, or mark this `Defines:` `abstractly:`"
                })
            }),
            "{messages:#?}"
        );
        // `A := ...` binds `A`, and binding every component binds the subject,
        // so the undefined `B` is the only complaint.
        assert!(
            !messages.iter().any(|event| {
                event
                    .as_message()
                    .is_some_and(|message| message.message.contains("Missing definition"))
            }),
            "{messages:#?}"
        );
    }

    #[test]
    fn check_accepts_refined_command_in_specifies() {
        // A `specifies:` item (like a `Declares:` target) may name a refined command
        // as the type: `* is \(associative)::binary.operation:on{X}`.
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("specifies-refined.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Declares: X
    Requires:
    . capability: x_ "in" X :-> \\abstract
    Documented:
    . written: "\operatorname{set}"

    [A \.set.cross./ B]
    Defines: X := \set@{(a_, b_) : a_ "in" A; b_ "in" B}
    when: A, B is \set
    Documented:
    . written: "A? \times B?"

    [\function:?on{A}:?to{B}]
    Declares: f(x__) ::= y_
    when: A, B is \set
    specifies:
    . x__ "in" A
    . y_ "in" B
    Documented:
    . written: "f?"

    [\binary.operation:on{X}]
    Declares: x_ * y_ is \function:on{X \.set.cross./ X}:to{X}
    when: X is \set
    Documented:
    . written: "\operatorname{binop}"

    [\(associative)::binary.operation:on{X}]
    Refines: x_ * y_
    when: X is \set
    satisfies:
    . forAll: a, b, c "in" X
      then: (a * b) * c = a * (b * c)
    Documented:
    . adjective: "associative"

    [\semigroup]
    Declares: S ::= (X, *) is \set via X
    specifies: * is \(associative)::binary.operation:on{X}
    Documented:
    . written: "\operatorname{semigroup}"
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("specifies-refined.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 1 file").with_origin("mlg_check")]
        );
    }

    #[test]
    fn check_accepts_refinement_of_an_inherited_component_specification() {
        // `\magma` already specifies `*` as a binary operation. The
        // `semigroup` `specifies:` item adds the `associative` refinement to that
        // same base type, so it is additive rather than a duplicate specification.
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("refined-inherited-component.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Declares: X
    Requires:
    . capability: x_ "in" X :-> \\abstract
    Documented:
    . written: "\operatorname{set}"

    [A \.set.cross./ B]
    Defines: X := \set@{(a_, b_) : a_ "in" A; b_ "in" B}
    when: A, B is \set
    Documented:
    . written: "A? \times B?"

    [\function:?on{A}:?to{B}]
    Declares: f(x__) ::= y_
    when: A, B is \set
    specifies:
    . x__ "in" A
    . y_ "in" B
    Documented:
    . written: "f?"

    [\binary.operation:on{X}]
    Declares: x_ * y_ is \function:on{X \.set.cross./ X}:to{X}
    when: X is \set
    Documented:
    . written: "\operatorname{binop}"

    [\magma]
    Declares: M ::= (X, *) is \set via X
    specifies: * is \binary.operation:on{M}
    Documented:
    . written: "\operatorname{magma}"

    [\(associative)::binary.operation:on{X}]
    Refines: x_ * y_
    when: X is \set
    satisfies:
    . forAll: a, b, c "in" X
      then: (a * b) * c = a * (b * c)
    Documented:
    . adjective: "associative"

    [\semigroup]
    Declares: S ::= (X, *) is \magma via (X, *)
    specifies: * is \(associative)::binary.operation:on{X}
    Documented:
    . written: "\operatorname{semigroup}"
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("refined-inherited-component.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 1 file").with_origin("mlg_check")]
        );
    }

    #[test]
    fn check_rejects_a_different_type_for_an_inherited_component() {
        // Refinement is the exception to the single-specification rule, not a
        // license to replace the inherited binary-operation type with a
        // different type such as `\function`.
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("different-inherited-component.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Declares: X
    Requires:
    . capability: x_ "in" X :-> \\abstract
    Documented:
    . written: "\operatorname{set}"

    [\function:?on{A}:?to{B}]
    Declares: f(x__) ::= y_
    when: A, B is \set
    specifies:
    . x__ "in" A
    . y_ "in" B
    Documented:
    . written: "f?"

    [\binary.operation:on{X}]
    Declares: x_ * y_ is \function:on{X}:to{X}
    when: X is \set
    Documented:
    . written: "\operatorname{binop}"

    [\magma]
    Declares: M ::= (X, *) is \set via X
    specifies: * is \binary.operation:on{M}
    Documented:
    . written: "\operatorname{magma}"

    [\bad.magma]
    Declares: B ::= (X, *) is \magma via (X, *)
    specifies: * is \function:on{X}:to{X}
    Documented:
    . written: "\operatorname{bad}"
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        check_in(
            temp_dir.path(),
            &[PathBuf::from("different-inherited-component.mlg")],
            &mut event_log,
        );

        assert!(
            user_events(&event_log).iter().any(|event| {
                event.as_message().is_some_and(|message| {
                    message.message
                        == "Duplicate specification for target symbol `*`; it is already specified by the `Declares:` target"
                })
            }),
            "{:#?}",
            user_events(&event_log)
        );
    }

    #[test]
    fn check_reports_undefined_command_in_spec_capability_target() {
        // The `:->` reduction target of an `Enables:` `capability:` references
        // `\grp.element:of`, which is defined nowhere. It must be reported.
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("cap-target-undefined.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Declares: X
    Requires:
    . capability: x_ "in" X :-> \\abstract
    Documented:
    . written: "\operatorname{set}"

    [\grp:of{G}]
    Declares: Z
    when: G is \set
    Enables:
    . capability: x_ "in" Z :-> x_ is \grp.element:of{G}
      written: "x_? \in Z?"
    Documented:
    . written: "\operatorname{grp}"
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        check_in(
            temp_dir.path(),
            &[PathBuf::from("cap-target-undefined.mlg")],
            &mut event_log,
        );

        assert!(
            user_events(&event_log)
                .iter()
                .any(|event| event.as_message().is_some_and(|message| {
                    message
                        .message
                        .contains("Undefined command signature `\\grp.element:of`")
                })),
            "expected an undefined-command error for the capability target: {:#?}",
            user_events(&event_log)
        );
    }

    #[test]
    fn check_reports_undefined_command_in_expression_capability_target() {
        // The `:=>` reduction target of an operator capability references
        // `\undefined.elt:of`, which is defined nowhere. It must be reported.
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("expr-cap-target-undefined.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Declares: X
    Requires:
    . capability: x_ "in" X :-> \\abstract
    Documented:
    . written: "\operatorname{set}"

    [A \.set.cross./ B]
    Defines: X := \set@{(a_, b_) : a_ "in" A; b_ "in" B}
    when: A, B is \set
    Documented:
    . written: "A? \times B?"

    [\function:?on{A}:?to{B}]
    Declares: f(x__) ::= y_
    when: A, B is \set
    specifies:
    . x__ "in" A
    . y_ "in" B
    Documented:
    . written: "f?"

    [\binary.operation:on{X}]
    Declares: x_ * y_ is \function:on{X \.set.cross./ X}:to{X}
    when: X is \set
    Documented:
    . written: "\operatorname{binop}"

    [\magma]
    Declares: M ::= (X, *) is \set via X
    specifies:
    . * is \binary.operation:on{M}
    Documented:
    . written: "\operatorname{magma}"

    [\magma.element:of{M ::= (X, *)}]
    Declares: x "in" X
    when: M is \magma
    Enables:
    . capability: x_ [*] y_ :=> x_ is \undefined.elt:of{M}
      written: "x_? *? y_?"
    Documented:
    . written: "\operatorname{elt}"
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        check_in(
            temp_dir.path(),
            &[PathBuf::from("expr-cap-target-undefined.mlg")],
            &mut event_log,
        );

        assert!(
            user_events(&event_log)
                .iter()
                .any(|event| event.as_message().is_some_and(|message| {
                    message
                        .message
                        .contains("Undefined command signature `\\undefined.elt:of`")
                })),
            "expected an undefined-command error for the `:=>` target: {:#?}",
            user_events(&event_log)
        );
    }

    #[test]
    fn check_accepts_member_access_and_member_call_capabilities() {
        // An `Enables:` `capability:` may use a member-access left-hand side —
        // `x.self` (member access) and `x.twin(a_)` (member call), where `x` is
        // the described subject — and uses of `p.self` / `p.twin(q)` resolve.
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("member-capability.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Declares: X
    Requires:
    . capability: x_ "in" X :-> \\abstract
    Documented:
    . written: "\operatorname{set}"

    [\elt:of{X}]
    Declares: x "in" X
    when: X is \set
    Enables:
    . capability: x.self :=> x
      written: "x?"
    . capability: x.twin(a_) :=> a_
      written: "a_?"
    Documented:
    . written: "\operatorname{elt}"

    Theorem:
    given:
    . X is \set
    . p is \elt:of{X}
    . q "in" X
    then:
    . p.self "in"? X
    . (p.self).self "in"? X
    . p.twin(q) "in"? X
    Id: "aaaaaaaa-1111-4aaa-8aaa-aaaaaaaaaaaa"
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("member-capability.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 1 file").with_origin("mlg_check")]
        );
    }

    #[test]
    fn check_resolves_members_on_grouped_operator_and_member_results() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("grouped-member-capability.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Declares: X
    Requires:
    . capability: x_ "in" X :-> \\abstract
    Documented:
    . called: "set"

    [A \.cross./ B]
    Defines: X := \set@{(a_, b_) : a_ "in" A; b_ "in" B}
    when: A, B is \set
    Documented:
    . called: "cross"

    [\function:on{A}:to{B}]
    Declares: f(x__) ::= y_
    when: A, B is \set
    specifies:
    . x__ "in" A
    . y_ "in" B
    Documented:
    . called: "function"

    [\binary.operation:on{X}]
    Declares: x_ * y_ is \function:on{X \.cross./ X}:to{X}
    when: X is \set
    Documented:
    . called: "binary operation"

    [\magma]
    Declares: M ::= (X, *) is \set via X
    specifies: * is \binary.operation:on{M}
    Enables:
    . capability: x_ "in" M :<->: x_ is \magma.element:of{M}
    Documented:
    . called: "magma"

    [\magma.element:of{M ::= (X, *)}]
    Declares: x "in" X
    when: M is \magma
    Enables:
    . capability: x_ [*] y_ :=> x_ |M.*| y_
      written: "x_? *? y_?"
    Documented:
    . called: "magma element"

    [\group]
    Declares: G ::= (X, *, e) is \magma via (X, *)
    specifies: e "in" X
    Enables:
    . capability: x_ "in" G :<->: x_ is \group.element:of{G}
    Documented:
    . called: "group"

    [\group.element:of{G}]
    Declares: x is \magma.element:of{G}
    when: G is \group
    Enables:
    . capability: x.inv :=> \group.inverse:of{x}:in{G}
      written: "x+?^{-1}"
    Documented:
    . called: "group element"

    [\group.inverse:of{x}:in{G}]
    Defines: y "in" G
    when:
    . G is \group
    . x "in" G
    Documented:
    . called: "group inverse"

    Theorem:
    given:
    . G is \group
    . x, y "in" G
    then: (x * y).inv = y.inv * x.inv

    Theorem:
    given:
    . G ::= (X, *, e) is \group
    . x "in" G
    then: (x.inv).inv = x
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("grouped-member-capability.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 1 file").with_origin("mlg_check")]
        );
    }

    #[test]
    fn check_resolves_members_from_spec_facts_and_instantiates_owner_type_parameters() {
        // `p "in" D` reduces to `p is \element:of{D}`, which both makes `.copy`
        // available and binds the capability owner's `C` parameter to `D` in
        // the target `\copy:of{p}:in{D}`.
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("parameterized-member-capability.mlg");

        write_mlg_fixture(
            &file,
            r#"[\container]
    Declares: C
    Requires:
    . capability: x_ "in" C :-> x_ is \element:of{C}
    Documented:
    . written: "\operatorname{container}"

    [\element:of{C}]
    Declares: x "in" C
    when: C is \container
    Enables:
    . capability: x.copy :=> \copy:of{x}:in{C}
      written: "x?^{\prime}"
    Documented:
    . written: "\operatorname{element}"

    [\copy:of{x}:in{C}]
    Defines: y "in" C
    when:
    . C is \container
    . x "in" C
    Documented:
    . written: "x+?^{\prime}"

    Theorem:
    given:
    . D is \container
    . p "in" D
    then: p.copy "in"? D
    Id: "aaaaaaaa-1111-4aaa-8aaa-aaaaaaaaaaaa"
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("parameterized-member-capability.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 1 file").with_origin("mlg_check")]
        );
    }

    #[test]
    fn check_rejects_member_capability_with_non_subject_owner() {
        // The owner of a member capability must be exactly the described subject.
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("member-owner.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Declares: X
    Requires:
    . capability: x_ "in" X :-> \\abstract
    Documented:
    . written: "\operatorname{set}"

    [\elt:of{X}]
    Declares: x "in" X
    when: X is \set
    Enables:
    . capability: z.self :=> x
      written: "x?"
    Documented:
    . written: "\operatorname{elt}"
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        check_in(
            temp_dir.path(),
            &[PathBuf::from("member-owner.mlg")],
            &mut event_log,
        );

        assert!(
            user_events(&event_log)
                .iter()
                .any(|event| event.as_message().is_some_and(|message| {
                    message
                        .message
                        .contains("Member capability owner `z` must be the described item `x`")
                })),
            "expected a member-owner error: {:#?}",
            user_events(&event_log)
        );
    }

    #[test]
    fn check_accepts_subscripted_operators_and_destructuring_spec_infix_heading() {
        // A subscripted operator (`*_1`) is a valid operator name, and a
        // spec-infix `Declares` heading whose left operand destructures
        // (`H ::= (X1, *_1, e1) \:sub:/ …`) matches its `Declares:` argument.
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("subgroup.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Declares: X
    Requires:
    . capability: x_ "in" X :-> \\abstract
    Documented:
    . written: "\operatorname{set}"

    [A \:subset:/ B]
    Declares: A is \set
    when: B is \set
    Requires:
    . capability: x_ "in" A :-> x_ "in" B
    Documented:
    . called: "subset of $B?$"

    [\op]
    Declares: f(x__)
    Documented:
    . written: "\operatorname{op}"

    [\binary.operation:on{X}]
    Declares: x_ * y_ is \op
    when: X is \set
    Documented:
    . called: "binary operation on $X?$"

    [\grp]
    Declares: G ::= (X, *, e) is \set via X
    specifies:
    . * is \binary.operation:on{X}
    . e "in" X
    Documented:
    . called: "grp"

    [H ::= (X1, *_1, e1) \:sub:/ G ::= (X, *, e)]
    Declares: H ::= (X1, *_1, e1)
    when: G is \grp
    specifies:
    . X1 \:subset:/ X
    . *_1 is \binary.operation:on{X1}
    . e1 "in" X1
    Documented:
    . called: "subgroup"
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("subgroup.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 1 file").with_origin("mlg_check")]
        );
    }

    #[test]
    fn check_accepts_pointwise_tuple_pattern_operator_definitions() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("direct-product.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
Declares: X
Requires:
. capability: x_ "in" X :-> \\abstract
Enables:
. from: Y ::= {y__ : ...}
  capability: x_ "in" X :-> x_ member_of Y
Documented:
. called: "set"

[\function:on{A}:to{B}]
Declares: f(x__) ::= y_
when: A, B is \set
specifies:
. x__ "in" A
. y_ "in" B
Documented:
. called: "function"

[A \.cross./ B]
Defines: X := \set@{(a_, b_) : a_ "in" A; b_ "in" B}
when: A, B is \set
Documented:
. called: "cross"

[\binary.operation:on{X}]
Declares: x_ * y_ is \function:on{X \.cross./ X}:to{X}
when: X is \set
Documented:
. called: "binary operation"

[\group]
Declares: G ::= (X, *, e) is \set via X
specifies:
. * is \binary.operation:on{G}
. e "in" X
Documented:
. called: "group"

[G1 ::= (X1, *_1, e1) \.direct.product./ G2 ::= (X2, *_2, e2)]
Defines: G3 ::= (X3, *_3, e3) is \group
when: G1, G2 is \group
expresses:
. X3 := {(x1, x2) : x1 "in" X1; x2 "in" X2}
. (a1_, a2_) *_3 (b1_, b2_) := (a1_ *_1 b1_, a2_ *_2 b2_)
. e3 := (e1, e2)
Documented:
. called: "direct product"
"#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("direct-product.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 1 file").with_origin("mlg_check")]
        );
    }

    #[test]
    fn check_infers_optional_parameters_from_argument_type() {
        // `\uses:of{g}` requires `g is \fn:on{A?}:to{B?}`; the `?` parameters `A`
        // and `B` are solved from the passed argument's type — here `\op:on{X}`,
        // which extends `\fn:on{X}:to{X}` — so the later `S \:subset:/ A`
        // requirement resolves to `Z \:subset:/ X`.
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("infer.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Declares: X
    Requires:
    . capability: x_ "in" X :-> \\abstract
    Documented:
    . written: "\operatorname{set}"

    [A \:subset:/ B]
    Declares: A is \set
    when: B is \set
    Requires:
    . capability: x_ "in" A :-> x_ "in" B
    Documented:
    . called: "subset of $B?$"

    [\fn:on{A}:to{B}]
    Declares: f(x__) ::= y_
    when: A, B is \set
    Documented:
    . written: "f?"

    [\op:on{X}]
    Declares: p(x__) ::= y_ is \fn:on{X}:to{X}
    when: X is \set
    Documented:
    . written: "\operatorname{op}"

    [\uses:of{g(x__)}:sub{S}]
    Declares: h
    when:
    . g is \fn:on{A?}:to{B?}
    . S \:subset:/ A
    Documented:
    . called: "uses"

    Theorem:
    given:
    . X, Z is \set
    . Z \:subset:/ X
    . r is \op:on{X}
    . q is \uses:of{r}:sub{Z}
    then: q is? \uses:of{r}:sub{Z}
    Id: "aaaaaaaa-1111-4aaa-8aaa-aaaaaaaaaaaa"
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("infer.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 1 file").with_origin("mlg_check")]
        );
    }

    #[test]
    fn check_have_asserting_group_establishes_specification() {
        // A `have:`/`asserting:` group lets an explicit assertion establish a
        // specification the checker cannot reach on its own: `\wrap:of{P}:in{Q}`
        // requires `P \:subset:/ Q`, which the `asserting:` item provides.
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("have.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Declares: X
    Requires:
    . capability: x_ "in" X :-> \\abstract
    Documented:
    . written: "\operatorname{set}"

    [A \:subset:/ B]
    Declares: A is \set
    when: B is \set
    Requires:
    . capability: x_ "in" A :-> x_ "in" B
    Documented:
    . called: "subset of $B?$"

    [\wrap:of{A}:in{B}]
    Defines: w "in" B
    when:
    . A is \set
    . B is \set
    . A \:subset:/ B
    Documented:
    . called: "wrap"

    [\pair.thm]
    Theorem:
    given: P, Q is \set
    then: P is? \set
    Id: "cccccccc-1111-4ccc-8ccc-cccccccccccc"

    [\thing:on{P}:and{Q}]
    Declares: t
    when: P, Q is \set
    specifies:
    . have: t is \wrap:of{P}:in{Q}
      asserting: P \:subset?:/ Q
      because: P is? \set
      by: \pair.thm#given{P := P; Q := Q}
    Documented:
    . called: "thing"
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("have.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 1 file").with_origin("mlg_check")]
        );
    }

    #[test]
    fn check_accepts_have_groups_without_asserting_sections() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("have-without-asserting.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Declares: X
    Requires:
    . capability: x_ "in" X :-> \\abstract
    Documented:
    . called: "set"

    [\thing]
    Declares: t
    using: x is \set
    specifies:
    . (.x is \set.)[:known:]
    Documented:
    . called: "thing"
    Justification:
    . [known]
      have: x is \set
      because: x is? \set

    Theorem:
    given: x is \set
    then:
    . have: x is? \set
      because: x is? \set
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("have-without-asserting.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 1 file").with_origin("mlg_check")]
        );
    }

    #[test]
    fn check_have_asserting_group_requires_the_assertion_to_establish_have() {
        // If the `asserting:` items do not establish the `have:` item, the
        // requirement is still reported.
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("have-insufficient.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Declares: X
    Requires:
    . capability: x_ "in" X :-> \\abstract
    Documented:
    . written: "\operatorname{set}"

    [A \:subset:/ B]
    Declares: A is \set
    when: B is \set
    Requires:
    . capability: x_ "in" A :-> x_ "in" B
    Documented:
    . called: "subset of $B?$"

    [\wrap:of{A}:in{B}]
    Defines: w "in" B
    when:
    . A is \set
    . B is \set
    . A \:subset:/ B
    Documented:
    . called: "wrap"

    [\thing:on{P}:and{Q}]
    Declares: t
    when: P, Q is \set
    specifies:
    . have: t is \wrap:of{P}:in{Q}
      asserting: P is? \set
    Documented:
    . called: "thing"
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        check_in(
            temp_dir.path(),
            &[PathBuf::from("have-insufficient.mlg")],
            &mut event_log,
        );

        assert!(
            user_events(&event_log).iter().any(|event| event
                .as_message()
                .is_some_and(|message| message.message.contains("`P \\:subset:/ Q`"))),
            "expected the unestablished requirement to be reported: {:#?}",
            user_events(&event_log)
        );
    }

    #[test]
    fn check_treats_text_placeholder_groups_as_opaque() {
        // `Text*` placeholders are opaque prose: their markdown/LaTeX body is never
        // parsed as MathLingua, so `\group` (undefined here) raises no diagnostic.
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("placeholder.mlg");

        write_mlg_fixture(
            &file,
            r#"TextTheorem: "In every $\group$ the identity is unique."
    Documented:
    . called: "Uniqueness of identity"
    . notes: "Turn this into a structured Theorem once \group exists."
    Id: "11111111-1111-4111-8111-111111111111"

    TextDefinition: "A **prime** has exactly two divisors."
    Id: "22222222-2222-4222-8222-222222222222"
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("placeholder.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 1 file").with_origin("mlg_check")]
        );
    }

    #[test]
    fn check_labeled_specification_is_established_by_a_justification_entry() {
        // A labeled specification `(.t is \wrap:of{P}:in{Q}.)[:1:]` is established by
        // the `Justification:` entry `[1]`, whose `have:` restates it and whose
        // `asserting:` provides the `P \:subset:/ Q` the requirement needs.
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("justification.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Declares: X
    Requires:
    . capability: x_ "in" X :-> \\abstract
    Documented:
    . written: "\operatorname{set}"

    [A \:subset:/ B]
    Declares: A is \set
    when: B is \set
    Requires:
    . capability: x_ "in" A :-> x_ "in" B
    Documented:
    . called: "subset of $B?$"

    [\wrap:of{A}:in{B}]
    Defines: w "in" B
    when:
    . A is \set
    . B is \set
    . A \:subset:/ B
    Documented:
    . called: "wrap"

    [\thing:on{P}:and{Q}]
    Declares: t
    when: P, Q is \set
    specifies:
    . (.t is \wrap:of{P}:in{Q}.)[:1:]
    Documented:
    . called: "thing"
    Justification:
    . [1]
      have:
      . t is \wrap:of{P}:in{Q}
      asserting:
      . P \:subset?:/ Q
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("justification.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 1 file").with_origin("mlg_check")]
        );
    }

    #[test]
    fn check_reports_unreferenced_justification_entry() {
        // Every `Justification:` entry must justify some labeled specification; an
        // entry no label references is reported.
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("justification-unused.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Declares: X
    Requires:
    . capability: x_ "in" X :-> \\abstract
    Documented:
    . written: "\operatorname{set}"

    [\thing]
    Declares: t
    using:
    . z is \set
    specifies:
    . z is \set
    Documented:
    . called: "thing"
    Justification:
    . [1]
      have:
      . z is \set
      asserting:
      . z is \set
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        check_in(
            temp_dir.path(),
            &[PathBuf::from("justification-unused.mlg")],
            &mut event_log,
        );

        assert!(
            user_events(&event_log)
                .iter()
                .any(|event| event.as_message().is_some_and(|message| message
                    .message
                    .contains("`Justification:` entry `[1]` is not referenced"))),
            "expected the unreferenced justification entry to be reported: {:#?}",
            user_events(&event_log)
        );
    }

    #[test]
    fn check_establishes_labeled_declaration_in_satisfies() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("justification-satisfies.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Declares: X
    Requires:
    . capability: x_ "in" X :-> \\abstract
    Documented:
    . written: "\operatorname{set}"

    [\thing]
    Declares: t
    using:
    . x is \set
    . y is \set
    . z is \set
    satisfies:
    . (.y := x.)[:1:]
    . z := (.x.)[:2:]
    Documented:
    . called: "thing"
    Justification:
    . [1]
      have: y := x
      asserting: x is \set
    . [2]
      have: x
      asserting: x is \set
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("justification-satisfies.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 1 file").with_origin("mlg_check")]
        );
    }

    #[test]
    fn check_resolves_backtick_stropped_operator_as_a_value() {
        // A backtick-stropped operator `` `*` `` refers to the bound operator `*`,
        // so it resolves as a value with `*`'s type and can be invoked in function
        // form as `` `*`(a, b) ``.
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("stropped-operator.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Declares: X
    Requires:
    . capability: x_ "in" X :-> \\abstract
    Documented:
    . written: "\operatorname{set}"

    [A \.set.cross./ B]
    Defines: X := \set@{(a_, b_) : a_ "in" A; b_ "in" B}
    when: A, B is \set
    Documented:
    . written: "A? \times B?"

    [\fn:on{A}:to{B}]
    Declares: f(x__) ::= y_
    when: A, B is \set
    specifies:
    . x__ "in" A
    . y_ "in" B
    Documented:
    . written: "f?"

    [\op:on{X}]
    Declares: x_ * y_ is \fn:on{X \.set.cross./ X}:to{X}
    when: X is \set
    Documented:
    . written: "\operatorname{op}"

    [\magma]
    Declares: M ::= (X, *) is \set via X
    specifies:
    . * is \op:on{X}
    Documented:
    . written: "\operatorname{magma}"

    Theorem:
    given:
    . M ::= (X, *) is \magma
    . a, b "in" X
    then:
    . `*` is? \op:on{X}
    . `*`(a, b) = a * b
    Id: "aaaaaaaa-1111-4aaa-8aaa-aaaaaaaaaaaa"
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("stropped-operator.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 1 file").with_origin("mlg_check")]
        );
    }

    #[test]
    fn check_establishes_spec_requirement_from_providing_capability() {
        // `\grp` provides `x_ "in" G :<->: x_ is \grp.elt:of{G}`, so membership and
        // being an element are equivalent. A command requiring `x "in" G` must
        // therefore be satisfiable by a value known only to be `\grp.elt:of{G}`.
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("spec-requirement.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Declares: X
    Requires:
    . capability: x_ "in" X :-> \\abstract
    Documented:
    . written: "\operatorname{set}"

    [\grp]
    Declares: G is \set
    Enables:
    . capability: x_ "in" G :<->: x_ is \grp.elt:of{G}
      written: "x_? \in G?"
    Documented:
    . written: "\operatorname{grp}"

    [\grp.elt:of{G}]
    Declares: x "in" G
    when: G is \grp
    Documented:
    . written: "\operatorname{elt}"

    [\op:of{x}:in{G}]
    Defines: y is \grp.elt:of{G}
    when:
    . G is \grp
    . x "in" G
    Documented:
    . written: "\operatorname{op}"

    Theorem:
    given:
    . G is \grp
    . x is \grp.elt:of{G}
    then: \op:of{x}:in{G} is? \grp.elt:of{G}
    Id: "aaaaaaaa-1111-4aaa-8aaa-aaaaaaaaaaaa"
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("spec-requirement.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 1 file").with_origin("mlg_check")]
        );
    }

    #[test]
    fn check_does_not_reverse_one_way_spec_operator_capability() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("one-way-spec-requirement.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Declares: X
    Requires:
    . capability: x_ "in" X :-> \\abstract
    Documented:
    . called: "set"

    [\foo]
    Declares: X is \set
    Enables:
    . capability: x_ "in" X :-> x_ is \bar
    Documented:
    . called: "foo"

    [\bar]
    Declares: x
    Documented:
    . called: "bar"

    [\needs.member{x}:of{X}]
    Declares: y
    when:
    . X is \foo
    . x "in" X
    Documented:
    . called: "needs member"

    Theorem:
    given:
    . X is \foo
    . x is \bar
    then: \needs.member{x}:of{X}
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        check_in(
            temp_dir.path(),
            &[PathBuf::from("one-way-spec-requirement.mlg")],
            &mut event_log,
        );

        assert!(user_events(&event_log).iter().any(|event| {
            event.as_message().is_some_and(|message| {
                message
                    .message
                    .contains("Could not establish requirement `x \"in\" X`")
            })
        }));
    }

    #[test]
    fn check_iff_spec_operator_requires_and_produces_every_target() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("multi-target-iff-spec.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Declares: X
    Requires:
    . capability: x_ "in" X :-> \\abstract
    Documented:
    . called: "set"

    [\both]
    Declares: X is \set
    Enables:
    . capability: x_ "in" X :<->: x_ is \foo; x_ is \bar
    Documented:
    . called: "both"

    [\foo]
    Declares: x
    Documented:
    . called: "foo"

    [\bar]
    Declares: x
    Documented:
    . called: "bar"

    [\needs.both{x}]
    Declares: y
    when:
    . x is \foo
    . x is \bar
    Documented:
    . called: "needs both"

    [\needs.member{x}:of{X}]
    Declares: y
    when:
    . X is \both
    . x "in" X
    Documented:
    . called: "needs member"

    Theorem:
    given:
    . X is \both
    . x "in" X
    then: \needs.both{x}

    Theorem:
    given:
    . X is \both
    . y is \foo
    . y is \bar
    then: \needs.member{y}:of{X}

    Theorem:
    given:
    . X is \both
    . z is \foo
    then: \needs.member{z}:of{X}
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        check_in(
            temp_dir.path(),
            &[PathBuf::from("multi-target-iff-spec.mlg")],
            &mut event_log,
        );

        let errors = user_events(&event_log)
            .iter()
            .filter_map(Event::as_message)
            .filter(|event| event.message.contains("Could not establish requirement"))
            .map(|event| event.message.clone())
            .collect::<Vec<_>>();
        assert_eq!(
            errors,
            ["Could not establish requirement `z \"in\" X` for command `\\needs.member:of`"]
        );
    }

    #[test]
    fn check_set_builder_reverse_membership_requires_its_condition() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("conditioned-set-membership.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Declares: X ::= {x__ : ...}
    Enables:
    . capability: x_ "in" X :<->: x_ member_of X
    Documented:
    . called: "set"

    [\foo]
    Declares: x
    Documented:
    . called: "foo"

    [\bar]
    Declares: x
    Documented:
    . called: "bar"

    [\needs.member{x}:of{X}]
    Declares: y
    when:
    . X is \set
    . x "in" X
    Documented:
    . called: "needs member"

    [\needs.bar{x}]
    Declares: y
    when: x is \bar
    Documented:
    . called: "needs bar"

    Theorem:
    given:
    . A := {a_ : a_ is \foo} is \set
    . x is \foo
    then: \needs.member{x}:of{A}

    Theorem:
    given:
    . B := {b_ : b_ is \foo | b_ is \bar} is \set
    . y is \foo
    then: \needs.member{y}:of{B}

    Theorem:
    given:
    . C := {c_ : c_ is \foo | c_ is \bar} is \set
    . z is \foo
    . z is \bar
    then: \needs.member{z}:of{C}

    Theorem:
    given:
    . D := {d_ : d_ is \foo | d_ is \bar} is \set
    . w "in" D
    then: \needs.bar{w}

    Theorem:
    given:
    . q is \foo
    . E := {e_ : e_ is \foo | q} is \set
    . v is \foo
    then: \needs.member{v}:of{E}
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        check_in(
            temp_dir.path(),
            &[PathBuf::from("conditioned-set-membership.mlg")],
            &mut event_log,
        );

        let errors = user_events(&event_log)
            .iter()
            .filter_map(Event::as_message)
            .filter(|event| event.message.contains("Could not establish requirement"))
            .map(|event| event.message.clone())
            .collect::<Vec<_>>();
        assert_eq!(errors.len(), 2, "unexpected errors: {errors:?}");
        assert!(errors[0].contains("could not establish set condition(s): `y is \\bar`"));
        assert!(errors[1].contains("could not establish set condition(s): `q`"));
    }

    #[test]
    fn check_uses_requires_capabilities_for_type_provided_specs() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("requires-capability.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Declares: X
    Requires:
    . capability: x_ "in" X :-> \\abstract
      written: "x_? \in X?"
    Documented:
    . written: "\operatorname{set}"

    Theorem:
    given:
    . X is \set
    . x is \\anything
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
    Declares: n
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
    fn check_builtin_type_predicate_recognizes_declares_only() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("builtin-type.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Declares: X
    Documented:
    . written: "\operatorname{set}"

    [\sqrt]
    Defines: y is \set
    Documented:
    . written: "\sqrt{}"

    Theorem:
    then: \set is? \\type
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
    Declares: X
    Documented:
    . written: "\operatorname{set}"

    [\tag:?on{A}]
    Declares: x
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
    Declares: X
    Enables:
    . capability: x_ "in" X :-> \\abstract
    Documented:
    . written: "\operatorname{set}"

    [\function:?on{A}:?to{B}]
    Declares: f(x__)
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
        assert_checked_cleanly(&event_log, "Checked 1 file");
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
    Declares: X
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
        assert_checked_cleanly(&event_log, "Checked 1 file");
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
    Declares: x
    Documented:
    . called: "real"
    Id: "f1a2b3c4-1111-4a22-8333-111111111111"

    [\natural]
    Declares: n
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
        assert_checked_cleanly(&event_log, "Checked 1 file");
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
    Declares: S
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
        assert_checked_cleanly(&event_log, "Checked 1 file");
    }

    #[test]
    fn check_reports_invalid_refined_headings_and_refines_targets() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("invalid-refines.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Declares: X
    Documented:
    . written: "\operatorname{set}"

    [\function:?on{A}:?to{B}]
    Declares: f(x__)
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
    Declares: g
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
            == "`Refines:` must have the form `Refines: <form>` or `Refines: <matching form> ::= <matching expansion>`; the refined target is inferred from the heading"));
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
    fn check_accepts_refines_destructuring_matching_the_base_declares_target() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("refines-destructuring.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Declares: X
    Requires:
    . capability: x_ "in" X :-> \\abstract
    Documented:
    . called: "set"

    [A \.cross./ B]
    Defines: X := \set@{(a_, b_) : a_ "in" A; b_ "in" B}
    when: A, B is \set
    Documented:
    . called: "cross"

    [\function:on{A}:to{B}]
    Declares: f(x__) ::= y_
    when: A, B is \set
    specifies:
    . x__ "in" A
    . y_ "in" B
    Documented:
    . called: "function"

    [\binary.operation:on{X}]
    Declares: x_ * y_ is \function:on{X \.cross./ X}:to{X}
    when: X is \set
    Documented:
    . called: "binary operation"

    [\group]
    Declares: G ::= (X, *, e) is \set via X
    specifies:
    . * is \binary.operation:on{G}
    . e "in" G
    Documented:
    . called: "group"

    [\(abelian)::group]
    Refines: G ::= (X, *, e)
    satisfies:
    . forAll: x, y "in" G
      then: x * y = y * x
    Documented:
    . adjective: "abelian"
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("refines-destructuring.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 1 file").with_origin("mlg_check")]
        );
    }

    #[test]
    fn check_accepts_full_and_abbreviated_refines_function_targets() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("refines-function-targets.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Declares: X
    Requires:
    . capability: x_ "in" X :-> \\abstract
    Documented:
    . called: "set"

    [\function:?on{A}:?to{B}]
    Declares: f(x__) ::= y_
    when: A, B is \set
    specifies:
    . x__ "in" A
    . y_ "in" B
    Documented:
    . called: "function"

    [\(full)::function:?on{A}:?to{B}]
    Refines: f(x__) ::= y_
    when: A, B is \set
    Documented:
    . adjective: "full"

    [\(subject)::function:?on{A}:?to{B}]
    Refines: f(x__)
    when: A, B is \set
    Documented:
    . adjective: "subject"

    [\(name)::function:?on{A}:?to{B}]
    Refines: f
    when: A, B is \set
    Documented:
    . adjective: "name"
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("refines-function-targets.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 1 file").with_origin("mlg_check")]
        );
    }

    #[test]
    fn check_rejects_refines_destructuring_that_does_not_match_the_base() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("refines-destructuring-mismatch.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Declares: X
    Documented:
    . called: "set"

    [\group]
    Declares: G ::= (X, *, e) is \set via X
    specifies:
    . * is \set
    . e is \set
    Documented:
    . called: "group"

    [\(bad)::group]
    Refines: G ::= (X, operation, e)
    Documented:
    . adjective: "bad"
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        check_in(
            temp_dir.path(),
            &[PathBuf::from("refines-destructuring-mismatch.mlg")],
            &mut event_log,
        );

        assert!(user_events(&event_log).iter().any(|event| {
            event.as_message().is_some_and(|message| {
                message.message
                    == "`Refines:` destructuring has shape (value, value, value), but the base `Declares:` target has shape (value, operator, value)"
            })
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
    Declares: x
    Documented:
    . written: "\operatorname{real}"

    [\foo{s}]
    Declares: x
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
    Declares: R
    Documented:
    . written: "\operatorname{real}"

    [\complex]
    Declares: C
    Documented:
    . written: "\operatorname{complex}"

    [\integer]
    Declares: I
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
    fn check_accepts_destructuring_components_and_symbolic_operator_application() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("structure.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
            Declares: X
            Enables:
            . capability: x_ "in" X :-> \\abstract
            Documented:
            . called: "set"

            [A \.cross./ B]
            Defines: X := \set@{(a_, b_) : a_ "in" A; b_ "in" B}
            when: A, B is \set
            Documented:
            . written: "A? \times B?"

            [\fn:on{A}:to{B}]
            Declares: f(x__) ::= y_
            when: A, B is \set
            specifies:
            . x__ "in" A
            . y_ "in" B
            Documented:
            . called: "fn"

            [\op:on{X}]
            Declares: x_ * y_ is \fn:on{X \.cross./ X}:to{X}
            when: X is \set
            Documented:
            . called: "op"

            [\elt:of{M ::= (X, *)}]
            Declares: x "in" X
            when: M is \structure
            Enables:
            . capability: x_ * y_ :=> x_ |M.*| y_
              written: "x_? * y_?"
            Documented:
            . called: "elt"

            [\structure]
            Declares: M ::= (X, *) is \set via X
            specifies:
            . * is \op:on{M}
            Enables:
            . capability: x_ "in" M :-> x_ is \elt:of{M}
              written: "x_? \in M?"
            Documented:
            . called: "structure"

            [\pointed.structure]
            Declares: S ::= (X, *, e) is \structure via (X, *)
            specifies:
            . e "in" X
            Documented:
            . called: "pointed structure"

            Theorem:
            given:
            . M ::= (X, *) is \structure
            . x "in" M
            then: x * x "in" M

            Theorem:
            given:
            . M ::= (X, *) is \structure
            . x "in" M
            then: x |M.*| x "in" M

            Theorem:
            given:
            . M ::= (Y, +) is \structure
            . y "in" M
            then: y * y "in" M
            "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("structure.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 1 file").with_origin("mlg_check")]
        );
    }

    #[test]
    fn check_accesses_components_directly_from_defined_objects() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("direct-components.mlg");

        write_mlg_fixture(
            &file,
            r#"[\thing]
    Declares: x
    Documented:
    . called: "thing"

    [\object]
    Defines: O ::= (first, second, next(x_))
    abstractly:
    specifies:
    . first is \thing
    . second is \thing
    . next is (_ is \thing) -> (_ is \thing)
    Documented:
    . called: "object"

    Theorem:
    then:
    . \object..first is? \thing
    . \object..second is? \thing
    . \object..next(\object..first) is? \thing
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("direct-components.mlg")],
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
    Declares: X
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
    then: A + B is? \set
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
    Declares: X
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
    then: A :- B is? \set

    Theorem:
    given: A, B is \set
    then: A -: B is? \set

    Theorem:
    given: A, B is \set
    then: A :-: B is? \set

    Theorem:
    given: A, B is \set
    then: A :** B is? \set

    Theorem:
    given: A, B is \set
    then: A **: B is? \set

    Theorem:
    given: A, B is \set
    then: A :**: B is? \set

    Theorem:
    given: A, B is \set
    then: A :*_free B is? \set

    Theorem:
    given: A, B is \set
    then: A :+_-* B is? \set

    Theorem:
    given: A, B is \set
    then: A :|minus| B is? \set

    Theorem:
    given: A, B is \set
    then: A |minus|: B is? \set

    Theorem:
    given: A, B is \set
    then: A :|minus|: B is? \set

    Theorem:
    given: A, B is \set
    then: A - B is? \set

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
    Declares: X
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
    Declares: X
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
    Declares: X
    Requires:
    . capability: x_ "in" X :-> \\abstract
      written: "x_? \in X?"
    Documented:
    . called: "set"

    [\relation:from{A}:to{B}]
    Declares: R
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
    Declares: x
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
    Declares: X
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
    Declares: X is \set
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
    Declares: X
    Enables:
    . capability: x_ "in" X :-> \\abstract
      written: "x_? \in X?"
    Documented:
    . called: "set"

    Theorem:
    given: A, B is \set
    then: A + B is? \set

    Theorem:
    given: A, B is \set
    then: A * B is? \set
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

        // A plain operator with neither a `Disambiguates` entry nor a
        // provided-symbol capability owned by the operand type is unresolved.
        assert!(messages.contains(
            &"Could not resolve operator `+`: no matching `Disambiguates` entry was found"
        ));
        assert!(messages.contains(
            &"Could not resolve operator `*`: no matching `Disambiguates` entry was found"
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
    Declares: X
    Documented:
    . called: "set"

    [\number]
    Declares: n
    Documented:
    . called: "number"

    [\needs.statement{P}]
    Declares: x
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
    Declares: X
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
    Declares: x
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
    Declares: X
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
    Declares: R
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
    Declares: X
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
    fn check_accepts_command_expression_targets_in_set_builder_definitions() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("set-builder-expression-target.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Declares: X
    Enables:
    . capability: x_ "in" X :-> \\abstract
    Documented:
    . called: "set"

    [\class:of{x}:over{X}]
    Defines: C is \set
    when:
    . X is \set
    . x "in" X
    Documented:
    . called: "class of $x?$ over $X?$"

    [\classes:of{X}]
    Defines: I := \set@{ \class:of{x_}:over{X} : x_ "in" X }
    when: X is \set
    Documented:
    . called: "classes of $X?$"
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("set-builder-expression-target.mlg")],
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
    Declares: X
    Requires:
    . capability: x_ "in" X :-> \\abstract
      written: "x_? \in X?"
    Documented:
    . called: "set"
    Id: "059126b9-dc83-41a2-aa1c-84f8e942f8d6"

    [\ordered.pair:of{a}:and{b}]
    Defines: P is \set
    when: a, b is \\anything
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
    Declares: x
    Documented:
    . written: "\operatorname{real}"

    [\foo{s}]
    Declares: x
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
    Declares: X
    Enables:
    . capability: x_ "in" X :-> \\abstract
    Documented:
    . written: "\operatorname{set}"

    [A \:subset:/ B]
    Declares: A is \set
    when: B is \set
    satisfies:
    . forAll: a "in" A
      then:
      . a "in"? B
    Documented:
    . written: "A? \subseteq B?"

    [\needs.set{s}]
    Declares: x
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
    fn check_rejects_specifications_in_statement_clauses() {
        // Specifications (`is`, non-predicate `\:...:/`) introduce symbols and are
        // only allowed in binding positions
        // (`exists:`/`given:`/`forAll:`/`let:`). In a statement position
        // (`if:`/`then:`/`iff:`/`that:`), the predicate forms (`is?`,
        // `\:...?:/`) must be used instead.
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("spec-in-statement.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Declares: X
    Enables:
    . capability: x_ "in" X :-> \\abstract
    Documented:
    . written: "\operatorname{set}"

    [A \:subset:/ B]
    Declares: A is \set
    when: B is \set
    Documented:
    . written: "A? \subseteq B?"

    Theorem:
    given: A, B is \set
    then:
    . A \:subset:/ B
    . if:
      . A is \set
      then:
      . A \:subset?:/ B
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("spec-in-statement.mlg")],
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

        assert!(
            messages.contains(
                &"An infix specification (`\\:...:/`) introduces a symbol and is only allowed in `exists:`, `given:`, `forAll:`, or `let:`; use the predicate form (`\\:...?:/`) here"
            ),
            "expected the infix-specification rejection, got {messages:#?}"
        );
        assert!(
            messages.contains(
                &"An `is` specification introduces a symbol and is only allowed in `exists:`, `given:`, `forAll:`, or `let:`; use the statement form `is?` here"
            ),
            "expected the `is`-specification rejection, got {messages:#?}"
        );
    }

    #[test]
    fn check_reports_spec_infix_requirement_mismatches() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("spec-infix-requirement.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Declares: X
    Documented:
    . written: "\operatorname{set}"

    [\thing]
    Declares: X
    Documented:
    . written: "\operatorname{thing}"

    [A \:subset:/ B]
    Declares: A is \set
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
    Declares: X
    Enables:
    . capability: x_ "in" X :-> \\abstract
    Documented:
    . written: "\operatorname{set}"

    [A \:subset:?within{U}:/ B]
    Declares: A is \set
    when:
    . B, U is \set
    . B \:subset:/ U
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
    Declares: X
    Enables:
    . capability: x_ "in" X :-> \\abstract
    Documented:
    . written: "\operatorname{set}"

    [A \:subset:?within{U}:/ B]
    Declares: A is \set
    when:
    . B, U is \set
    . B \:subset:/ U
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
    Documented:
    . written: "P? \land Q?"

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
    Declares: X
    Enables:
    . capability: x_ "in" X :-> \\abstract
    Documented:
    . written: "\operatorname{set}"

    [A \:subset:/ B]
    Declares: A is \set
    when: B is \set
    Documented:
    . written: "A? \subseteq B?"

    [A \.same.as./ B]
    States:
    when: A, B is \set
    that:
    . A is? \set
    Documented:
    . written: "A? \equiv B?"

    [\needs.expression{x}]
    Declares: y
    when: x is \\expression
    Documented:
    . written: "\operatorname{needsExpression}"

    [\needs.statement{x}]
    Declares: y
    when: x is \\statement
    Documented:
    . written: "\operatorname{needsStatement}"

    [\needs.specification{x}]
    Declares: y
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
    Declares: X
    Documented:
    . written: "\operatorname{set}"

    [\needs.statement{x}]
    Declares: y
    when: x is \\statement
    Documented:
    . written: "\operatorname{needsStatement}"

    [\needs.specification{x}]
    Declares: y
    when: x is \\specification
    Documented:
    . written: "\operatorname{needsSpecification}"

    [\wrap{x}]
    Declares: y
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
    Declares: X
    Documented:
    . written: "\operatorname{set}"

    [A \:wrong:/ B]
    Declares: C
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
            "Spec-infix Declares heading left operand must match the Declares argument"
        )));
        assert!(messages.contains(&String::from(
            "Spec-infix headings may only be used with Declares entries"
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
    Declares: X
    Enables:
    . capability: x_ "in" X :-> \\abstract
    Documented:
    . written: "\operatorname{set}"

    [\function:on{A}:to{B}]
    Declares: f(x__)
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
    Declares: X
    Enables:
    . capability: x_ "in" X :-> \\abstract
    Documented:
    . written: "\operatorname{set}"

    [\element.of:group{G ::= (X, *, e)}]
    Declares: x "in" X
    when:
    . G is \group
    . X is \set
    . * is \function:on{X}:to{X}
    . e "in" G
    Documented:
    . written: "x? \in G?"

    [\group]
    Declares: G ::= (X, *, e) is \set via X
    specifies:
    . * is \function:on{X}:to{X}
    . e "in" G
    Enables:
    . capability: x_ "in" G :-> x_ is \element.of:group{G}
    Documented:
    . written: "\operatorname{group}"

    [\function:on{A}:to{B}]
    Declares: f(x__)
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
    Declares: X
    Enables:
    . capability: x_ "in" X :-> \\abstract
    Documented:
    . written: "\operatorname{set}"

    [\function:on{A}:to{B}]
    Declares: f(x__) is (_ "in" A) -> (_ "in" B)
    when: A, B is \set
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
    fn check_treats_a_function_declaration_alias_as_an_additional_mapping_name() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("aliased-mapping.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Declares: A
    Enables:
    . capability: a_ "in" A :-> \\abstract
    Documented:
    . written: "\operatorname{set}"

    [\sequence:on{A}]
    Declares: X ::= x(i_) is (_ "in" A) -> (_ "in" A)
    when: A is \set
    Documented:
    . writing: x(i)
      as: "x?_{i?}"
    . writing: x(i_)
      as: "\left\{x?_{i_?}\right\}"
    . called: "sequence"

    [\pointed.sequence:on{A}]
    Declares: X ::= x(i_) ::= y_
    when: A is \set
    specifies:
    . i_ "in" A
    . y_ "in" A
    Documented:
    . writing: x(i)
      as: "x?_{i?}"
    . writing: x(i_)
      as: "\left\{x?_{i_?}\right\}"
    . called: "pointed sequence"

    Theorem:
    given:
    . A is \set
    . x(i_) is \sequence:on{A}
    . i "in" A
    then: x(i) "in" A

    Theorem:
    given:
    . A is \set
    . x(i_) is \pointed.sequence:on{A}
    . i "in" A
    then: x(i) "in" A
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("aliased-mapping.mlg")],
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
    Declares: X
    Enables:
    . capability: x_ "in" X :-> \\abstract
    Documented:
    . written: "\operatorname{set}"

    [\function:on{A}:to{B}]
    Declares: f(x__) is (_ "in" A) -> (_ "in" B)
    when: A, B is \set
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
    Declares: x
    Documented:
    . written: "\operatorname{real}"

    [\integer]
    Declares: x
    Documented:
    . written: "\operatorname{integer}"

    Theorem:
    given:
    . f is (_ is \real) -> (_ is \integer)
    . y is \real
    then:
    . f(y) is? \integer
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
    fn check_uses_compact_spec_literal_function_types() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("compact-function-types.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Declares: X
    Enables:
    . capability: x_ "in" X :-> \\abstract
    Documented:
    . written: "\operatorname{set}"

    [\real]
    Declares: r
    Documented:
    . written: "\operatorname{real}"

    [\integer]
    Declares: z
    Documented:
    . written: "\operatorname{integer}"

    Theorem:
    given:
    . f is (_ is \real) -> (_ is \integer)
    . y is \real
    then:
    . f(y) is? \integer

    Theorem:
    given:
    . f is (_ is \real) -> (_ is \integer)
    . y is \real
    then:
    . f(y) is? \integer

    Theorem:
    given:
    . A, B is \set
    . f is (_ "in" A) -> (_ "in" B)
    . y "in" A
    then:
    . f(y) "in" B

    Theorem:
    given:
    . A is \set
    . f is (_ is \real, _ "in" A) -> (_ is \integer)
    . x is \real
    . y "in" A
    then:
    . f(x, y) is? \integer

    Theorem:
    given:
    . f is (_ is \real, _ is \real, _ is \real) -> (_ is \integer)
    . x, y, z is \real
    then:
    . f(x, y, z) is? \integer

    Theorem:
    given:
    . A is \set
    . a is \\anything
    . f := (x_, y_) => x_ is (_ is \real, _ "in" A) -> (_ is \real)
    then: a is? \\anything

    Theorem:
    given: a is \\anything
    then:
    . let: x_ * y_ is (_ is \\anything, _ is \\anything) -> (_ is \\anything)
      then: a is? \\anything
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("compact-function-types.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 1 file").with_origin("mlg_check")]
        );
    }

    #[test]
    fn check_uses_whole_function_spec_literal_declaration() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("whole-function-declaration.mlg");

        write_mlg_fixture(
            &file,
            r#"[\real]
    Declares: r
    Documented:
    . written: "\operatorname{real}"

    [\real.function]
    Declares: f(x_) ::= y_
    specifies:
    . f is (_ is \real) -> (_ is \real)
    Documented:
    . written: "f?"

    Theorem:
    given:
    . f is \real.function
    . x is \real
    then: f(x) is? \real
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("whole-function-declaration.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 1 file").with_origin("mlg_check")]
        );
    }

    #[test]
    fn check_instantiates_tuple_and_set_spec_literal_types() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("structural-literal-types.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Declares: X
    Enables:
    . capability: x_ "in" X :-> \\abstract
    Documented:
    . written: "\operatorname{set}"

    [\natural]
    Declares: n
    Documented:
    . written: "\operatorname{natural}"

    Theorem:
    given:
    . A is \set
    . (x, y) is (_ is \natural, _ "in" A)
    then:
    . x is? \natural
    . y "in" A

    Theorem:
    given:
    . {x : ...} is {_ is \natural : ...}
    then: x is? \natural

    Theorem:
    given:
    . A is \set
    . {(x, y) : ...} is {(_ is \natural, _ "in" A) : ...}
    then:
    . x is? \natural
    . y "in" A
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("structural-literal-types.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 1 file").with_origin("mlg_check")]
        );
    }

    #[test]
    fn check_accepts_spec_literal_function_type_definition_requirements() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("natural-constructor.mlg");

        write_mlg_fixture(
            &file,
            r#"[\natural.0]
    Defines: 0 is \natural
    Documented:
    . written: "0"

    [\natural.succ(n_)]
    Defines: succ(n_) is (_ is \natural) -> (_ is \natural)
    Documented:
    . called: "successor of $n?$"
    . written: "n?+\!\!+"

    [\natural]
    Declares: n
    Requires:
    . definition: \natural.0 is \natural
    . definition: \natural.succ is (_ is \natural) -> (_ is \natural)
    Documented:
    . called: "naturals"
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("natural-constructor.mlg")],
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
    Declares: X
    Documented:
    . written: "\operatorname{set}"

    [\function:on{A}:to{B}]
    Declares: f(x__) is (x "in" A) -> (_ "in" B)
    when: A, B is \set
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
            event.as_message().is_some_and(|message| {
                message.message == "Function type specs must use `_` as their subject"
            })
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
    Declares: X
    Documented:
    . written: "\operatorname{set}"

    [\function:on{A}:to{B}]
    Declares: f(x__)
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
    Declares: x
    Documented:
    . written: "\operatorname{real}"

    [\foo{s}]
    Declares: x
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
    Declares: x
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
    fn check_rejects_unintroduced_specifies_relation_symbols() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("defines-relation-scope.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Declares: X
    Documented:
    . written: "\operatorname{set}"

    [\function:on{A}:to{B}]
    Declares: f(x__) ::= y_
    when: A, B is \set
    specifies:
    . x__ is \\expression
    . y_ is \\anything
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
    Declares: X
    Documented:
    . written: "\operatorname{set}"

    [\function:?on{A}:?to{B}]
    Declares: f(x__) ::= y_
    when: A, B is \set
    specifies:
    . x__ is \\expression
    . y_ is \\anything
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
    Declares: x
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
    Declares: X
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
    Declares: x
    Documented:
    . written: "\operatorname{real}"

    [\foo{s}]
    Declares: x
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
    Declares: X
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
    fn check_uses_let_bindings_and_where_assumptions_inside_the_then_clause() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("let-binding.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Declares: X
    Requires:
    . capability: x_ "in" X :-> \\abstract
      written: "x_? \\in X?"
    Documented:
    . called: "set"

    Theorem:
    given: X is \set
    then:
    . let: n "in" X
      where: m := n
      then: m = n
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("let-binding.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 1 file").with_origin("mlg_check")]
        );
    }

    #[test]
    fn check_keeps_let_bindings_local_to_the_then_clause() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("let-binding-scope.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Declares: X
    Requires:
    . capability: x_ "in" X :-> \\abstract
      written: "x_? \\in X?"
    Documented:
    . called: "set"

    Theorem:
    given: X is \set
    then:
    . let: n "in" X
      then: n = n
    . n = n
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("let-binding-scope.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert!(user_events(&event_log).iter().any(|event| {
            event
                .as_message()
                .is_some_and(|message| message.message == "Unrecognized symbol `n`")
        }));
        assert!(event_log.has_errors());
    }

    #[test]
    fn check_accepts_exists_without_such_that_sections() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("exists-without-such-that.mlg");

        write_mlg_fixture(
            &file,
            r#"[\real]
    Declares: x
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
    Declares: value
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
    Declares: X
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
    Declares: x
    Documented:
    . written: "\operatorname{real}"

    [\reals]
    Declares: R
    Enables:
    . capability: x_ "in" R :-> x is \real
    Documented:
    . written: "\operatorname{reals}"

    [\foo{s}]
    Declares: x
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
    Declares: r
    Documented:
    . written: "\operatorname{real}"

    [\set]
    Declares: X ::= {x__ : ...}
    Enables:
    . capability: x_ "in" X :-> x_ member_of X
    Documented:
    . written: "\operatorname{set}"

    [\needs.real{x}]
    Declares: y
    when: x is \real
    Documented:
    . written: "\operatorname{needsReal}(x?)"

    Theorem:
    given:
    . A := {x_ : x_ is \real} is \set
    . x "in" A
    then: \needs.real{x}

    Theorem:
    given:
    . B := {b_ : b_ is \real} is \set
    . y "in" B
    then: \needs.real{y}

    Theorem:
    given:
    . C := {c_ : c_ is \real} is \set
    . z "in" C
    then: \needs.real{z}

    Theorem:
    given: X := {x_ : x_ is \set} is \set
    then:
    . forAll: x "in" X
      then: x is? \set
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
    fn check_resolves_plain_operator_from_destructured_infix_collection_membership() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("product-member-operator.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Declares: X
    Requires:
    . capability: x_ "in" X :-> \\abstract
    Enables:
    . from: Y ::= {y__ : ...}
      capability: x_ "in" X :-> x_ member_of Y
    Documented:
    . called: "set"

    [A \.cross./ B]
    Defines: X := \set@{(a_, b_) : a_ "in" A; b_ "in" B}
    when: A, B is \set
    Documented:
    . called: "product"

    [\natural]
    Declares: n
    Enables:
    . capability: x_ + y_ :=> x_ \.natural.+./ y_
    Documented:
    . called: "natural"

    [\naturals]
    Defines: N := \set@{n_ : n_ is \natural}
    Documented:
    . called: "naturals"

    [a_ \.natural.+./ b_]
    Defines: c is \natural
    when: a_, b_ is \natural
    Documented:
    . written: "a_? + b_?"

    Theorem:
    given: (n1, n2) "in" (\naturals \.cross./ \naturals)
    then: n1 + n2 is? \natural
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("product-member-operator.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 1 file").with_origin("mlg_check")]
        );
    }

    #[test]
    fn check_reduces_spec_literal_set_membership_to_element_type() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("spec-literal-membership.mlg");

        write_mlg_fixture(
            &file,
            r#"[\real]
    Declares: r
    Documented:
    . written: "\operatorname{real}"

    [\set]
    Declares: X ::= {x__ : ...}
    Enables:
    . capability: x_ "in" X :-> x_ member_of X
    Documented:
    . written: "\operatorname{set}"

    [\needs.real{x}]
    Declares: y
    when: x is \real
    Documented:
    . written: "\operatorname{needsReal}(x?)"

    [\set:where{spec}]
    Defines: X := {x_ : x_ satisfies spec} is \set
    when: spec is \\specification
    Documented:
    . written: "\operatorname{setWhere}(spec?)"

    Theorem:
    given:
    . y "in" \set:where{_ is \real}
    then: \needs.real{y}
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("spec-literal-membership.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 1 file").with_origin("mlg_check")]
        );
    }

    #[test]
    fn check_reduces_type_parameter_set_membership_to_element_type() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("type-parameter-membership.mlg");

        write_mlg_fixture(
            &file,
            r#"[\real]
    Declares: r
    Documented:
    . written: "\operatorname{real}"

    [\set]
    Declares: X ::= {x__ : ...}
    Enables:
    . capability: x_ "in" X :-> x_ member_of X
    Documented:
    . written: "\operatorname{set}"

    [\needs.real{x}]
    Declares: y
    when: x is \real
    Documented:
    . written: "\operatorname{needsReal}(x?)"

    [\set:of{T}]
    Defines: X := {x_ : x_ is T} is \set
    when: T is \\type
    Documented:
    . written: "\operatorname{setOf}(T?)"

    Theorem:
    given:
    . z "in" \set:of{\real}
    then: \needs.real{z}
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("type-parameter-membership.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 1 file").with_origin("mlg_check")]
        );
    }

    #[test]
    fn check_accepts_mapping_literal_with_spec() {
        // The body is checked with the parameter bound to its spec's type.
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("mapping-literal.mlg");

        write_mlg_fixture(
            &file,
            r#"[\real]
    Declares: r
    Documented:
    . written: "\operatorname{real}"

    Theorem:
    given:
    . a is \\anything
    . f := (x_ is \real) => x_
    then: a is? \\anything
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("mapping-literal.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 1 file").with_origin("mlg_check")]
        );
    }

    #[test]
    fn check_reports_undeclared_symbol_in_mapping_body() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("mapping-body-error.mlg");

        write_mlg_fixture(
            &file,
            r#"[\real]
    Declares: r
    Documented:
    . written: "\operatorname{real}"

    Theorem:
    given:
    . a is \\anything
    . f := (x_ is \real) => undeclared
    then: a is? \\anything
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("mapping-body-error.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert!(
            user_events(&event_log).iter().any(|event| {
                event
                    .as_message()
                    .is_some_and(|message| message.message == "Unrecognized symbol `undeclared`")
            }),
            "expected an unrecognized-symbol error in the mapping body, got {:#?}",
            user_events(&event_log)
        );
    }

    #[test]
    fn check_reports_bare_mapping_without_known_type() {
        // A bare-parameter mapping outside an `is`-typed context has no spec to bind.
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("mapping-bare.mlg");

        write_mlg_fixture(
            &file,
            r#"[\real]
    Declares: r
    Documented:
    . written: "\operatorname{real}"

    Theorem:
    given:
    . a is \\anything
    . f := x_ => x_
    then: a is? \\anything
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("mapping-bare.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert!(
            user_events(&event_log).iter().any(|event| {
                event
                    .as_message()
                    .is_some_and(|message| message.message.contains("needs a spec"))
            }),
            "expected a 'needs a spec' error for the bare mapping, got {:#?}",
            user_events(&event_log)
        );
    }

    #[test]
    fn check_infers_bare_mapping_parameter_from_is_type() {
        // `f := x_ => x_ is (_ is \real) -> (_ is \real)` — the bare parameter's
        // spec is inferred from the declared function type.
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("mapping-inferred.mlg");

        write_mlg_fixture(
            &file,
            r#"[\real]
    Declares: r
    Documented:
    . written: "\operatorname{real}"

    Theorem:
    given:
    . a is \\anything
    . f := x_ => x_ is (_ is \real) -> (_ is \real)
    then: a is? \\anything
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("mapping-inferred.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 1 file").with_origin("mlg_check")]
        );
    }

    #[test]
    fn check_accepts_collection_literal_argument_sugar() {
        // `\collect{x_ : ...}` (sugar) checks the same as the explicit
        // double-brace `\collect{{x_ : ...}}`.
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("collection-literal-sugar.mlg");

        write_mlg_fixture(
            &file,
            r#"[\real]
    Declares: r
    Documented:
    . written: "\operatorname{real}"

    [\collect{S}]
    Declares: y
    when: S is \\expression
    Documented:
    . written: "\operatorname{collect}(S?)"

    Theorem:
    given:
    . a is \\anything
    then:
    . a is? \collect{x_ : x_ is \real}
    . a is? \collect{{x_ : x_ is \real}}
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("collection-literal-sugar.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 1 file").with_origin("mlg_check")]
        );
    }

    #[test]
    fn check_applies_satisfies_of_concrete_spec_literal() {
        // In a local set literal, `x_ satisfies (_ is \real)` reduces to
        // `x_ is \real`, so members are known to be reals.
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("satisfies-literal.mlg");

        write_mlg_fixture(
            &file,
            r#"[\real]
    Declares: r
    Documented:
    . written: "\operatorname{real}"

    [\set]
    Declares: X ::= {x__ : ...}
    Enables:
    . capability: x_ "in" X :-> x_ member_of X
    Documented:
    . written: "\operatorname{set}"

    [\needs.real{x}]
    Declares: y
    when: x is \real
    Documented:
    . written: "\operatorname{needsReal}(x?)"

    Theorem:
    given:
    . A := {x_ : x_ satisfies (_ is \real)} is \set
    . y "in" A
    then: \needs.real{y}
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("satisfies-literal.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 1 file").with_origin("mlg_check")]
        );
    }

    #[test]
    fn check_reports_wrong_element_type_from_spec_literal_set() {
        // `y "in" \set:where{_ is \real}` establishes `y is \real`, so requiring
        // `y is \set` must fail — the reduction must not over-derive.
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("spec-literal-negative.mlg");

        write_mlg_fixture(
            &file,
            r#"[\real]
    Declares: r
    Documented:
    . written: "\operatorname{real}"

    [\set]
    Declares: X ::= {x__ : ...}
    Enables:
    . capability: x_ "in" X :-> x_ member_of X
    Documented:
    . written: "\operatorname{set}"

    [\needs.set{x}]
    Declares: y
    when: x is \set
    Documented:
    . written: "\operatorname{needsSet}(x?)"

    [\set:where{spec}]
    Defines: X := {x_ : x_ satisfies spec} is \set
    when: spec is \\specification
    Documented:
    . written: "\operatorname{setWhere}(spec?)"

    Theorem:
    given:
    . y "in" \set:where{_ is \real}
    then: \needs.set{y}
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("spec-literal-negative.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert!(
            user_events(&event_log).iter().any(|event| {
                event.as_message().is_some_and(|message| {
                    message.message
                        == "Could not establish requirement `y is \\set` for command `\\needs.set`"
                })
            }),
            "expected a requirement mismatch: `y` is a `\\real`, not a `\\set`, got {:#?}",
            user_events(&event_log)
        );
    }

    #[test]
    fn check_rejects_type_target_for_spec_operator_literal() {
        // `_ "in" \real` uses a `Declares:` type as a spec-operator target; only
        // values (`Defines:`) are allowed there.
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("spec-literal-bad-target.mlg");

        write_mlg_fixture(
            &file,
            r#"[\real]
    Declares: r
    Documented:
    . written: "\operatorname{real}"

    [\set]
    Declares: X ::= {x__ : ...}
    Enables:
    . capability: x_ "in" X :-> x_ member_of X
    Documented:
    . written: "\operatorname{set}"

    [\set:where{spec}]
    Defines: X := {x_ : x_ satisfies spec} is \set
    when: spec is \\specification
    Documented:
    . written: "\operatorname{setWhere}(spec?)"

    Theorem:
    given:
    . y "in" \set:where{_ "in" \real}
    then: y is \set
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("spec-literal-bad-target.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert!(
            user_events(&event_log).iter().any(|event| {
                event.as_message().is_some_and(|message| {
                    message.message
                        == "the target of a spec operator must be a value, not the type `\\real`"
                })
            }),
            "expected a spec-operator-target rejection, got {:#?}",
            user_events(&event_log)
        );
    }

    #[test]
    fn check_reduces_cast_membership_through_from_capability() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("cast-membership.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Declares: X
    Requires:
    . capability: x_ "in" X :-> \\abstract
    Enables:
    . from: Y ::= {y__ : ...}
      capability: x_ "in" X :-> x_ member_of Y
    Documented:
    . written: "\operatorname{set}"

    [\needs.set{x}]
    Declares: y
    when: x is \set
    Documented:
    . written: "\operatorname{needsSet}(x?)"

    Theorem:
    given: X := {x_ : x_ is \set} is \set
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
    Declares: X
    Documented:
    . written: "\operatorname{set}"

    [\function]
    Declares: f(x__) ::= y_
    specifies:
    . x__ is \\expression
    . y_ is \\anything
    Enables:
    . from: P ::= {(p_, q_) : ...}
      as: f(p_) := q_
    Documented:
    . written: "\operatorname{function}"

    [\needs.set{x}]
    Declares: y
    when: x is \set
    Documented:
    . written: "\operatorname{needsSet}(x?)"

    Theorem:
    given:
    . F := {(p_, q_) : p_ is \\expression, q_ is \set} is \function
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
    Declares: r
    Documented:
    . written: "\operatorname{rational}"

    [\as.rational{x}]
    Defines: r is \rational
    when: x is \integer
    Documented:
    . written: "\operatorname{asRational}(x?)"

    [\integer]
    Declares: n
    Enables:
    . view:
      as: r := \as.rational{n} is \rational
      signifies: n \.embedded.to./ r
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
    Declares: y
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
    fn check_uses_views_declared_on_defined_values() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("defined-value-view.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Declares: X
    Requires:
    . capability: x_ "in" X :-> \\abstract
    Documented:
    . called: "set"

    [\function:on{A}:to{B}]
    Declares: f(x__) is (_ "in" A) -> (_ "in" B)
    when: A, B is \set
    Documented:
    . called: "function"

    [\naturals]
    Defines: Nb ::= (N, 0, S(n_))
    abstractly:
    specifies:
    . N is \set
    . 0 "in" N
    . S is \function:on{N}:to{N}
    Enables:
    . view:
      as: X := N is \set
    Documented:
    . called: "naturals"

    Theorem:
    then:
    . \naturals is? \set
    . \function:on{\naturals}:to{\naturals} is? \\type
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("defined-value-view.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 1 file").with_origin("mlg_check")]
        );
    }

    #[test]
    fn check_uses_refined_capabilities_declared_on_defined_values() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("defined-value-refined-capability.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Declares: X
    Requires:
    . capability: x_ "in" X :-> \\abstract
    Documented:
    . called: "set"

    [\function:on{A}:to{B}]
    Declares: f(x__) ::= y_
    when: A, B is \set
    specifies:
    . x__ "in" A
    . y_ "in" B
    Documented:
    . called: "function"

    [\(inductive)::set]
    Refines: I
    Documented:
    . adjective: "inductive"

    [\omega]
    Defines: omega is \set
    Enables:
    . capability: X_ "in" omega :-> X_ is \(inductive)::set
    Documented:
    . written: "\omega"

    [\set.successor:of{X}]
    Defines: Y is \set
    when: X is \set
    Documented:
    . called: "set successor"

    [\S(n_)]
    Defines: S(n_) is \function:on{\omega}:to{\omega}
    expresses: S(n_) := \set.successor:of{n_}
    Documented:
    . written: "S(n_?)"
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("defined-value-refined-capability.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 1 file").with_origin("mlg_check")]
        );
    }

    #[test]
    fn check_allows_refined_declaration_in_justification_have() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("von-neumann-omega.mlg");

        write_mlg_fixture(
            &file,
            r#"
[\set]
Declares: X ::= {x__ : ...}
Enables:
. capability: x_ "in" X :-> x_ member_of X
Documented:
. called: "set"

[\(inductive)::set]
Refines: I
Documented:
. adjective: "inductive"

[\axiom.of.infinity]
Axiom:
then:
. exists: A is \(inductive)::set
Documented:
. called: "axiom of infinity"

[\von.neumann.omega]
Defines: omega is \set
expresses:
. let:
  . (.A is \(inductive)::set.)[:1:]
  . X is \set
  then:
  . have: X "in"? omega
    iff:
    . X "in"? A
    . forAll: I is \(inductive)::set
      then: X "in"? I
Enables:
. capability: X_ "in" omega :-> X_ is \set
Documented:
. written: "\omega"
Justification:
. [1]
  have: A is \(inductive)::set
  by: \axiom.of.infinity
Id: "c13f4641-0ed5-4ad7-b309-8ec13b4c6b77"
"#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("von-neumann-omega.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 1 file").with_origin("mlg_check")]
        );
    }

    #[test]
    fn check_follows_named_view_components_for_membership_facts() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("named-view-component.mlg");

        write_mlg_fixture(
            &file,
            r#"[\von.neumann.natural]
    Declares: n
    Documented:
    . called: "von Neumann natural"

    [\set]
    Declares: X ::= {x__ : ...}
    Enables:
    . capability: x_ "in" X :-> x_ member_of X
    Documented:
    . called: "set"

    [\naturals]
    Defines: Nb ::= (N, Z)
    abstractly:
    specifies:
    . N := \set@{n_ : n_ is \von.neumann.natural} is \set
    . Z is \von.neumann.natural
    Enables:
    . view:
      as: X := N is \set
    Documented:
    . called: "naturals"

    [\needs.natural{x}]
    Declares: y
    when: x is \von.neumann.natural
    Documented:
    . called: "needs natural"

    Theorem:
    given: x "in" \naturals
    then: \needs.natural{x}
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("named-view-component.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 1 file").with_origin("mlg_check")]
        );
    }

    #[test]
    fn check_accepts_soft_build_cast_for_view_requirements() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("explicit-view-cast.mlg");

        write_mlg_fixture(
            &file,
            r#"[\rational]
    Declares: r
    Documented:
    . written: "\operatorname{rational}"

    [\integer]
    Declares: n
    Enables:
    . view:
      as: r := n is \rational
    Documented:
    . written: "\operatorname{integer}"

    [\needs.rational{x}]
    Declares: y
    when: x is \rational
    Documented:
    . written: "\operatorname{needsRational}(x?)"

    Theorem:
    given: n is \integer
    then:
    . \needs.rational{\rational@n}
    . (\rational@n) is? \rational
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
    fn check_does_not_use_view_casts_for_operator_resolution() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("view-does-not-resolve-operators.mlg");

        write_mlg_fixture(
            &file,
            r#"[\rational]
    Declares: r
    Enables:
    . capability: x_ + y_ :=> x_ \.rational.+./ y_
    Documented:
    . written: "\operatorname{rational}"

    [\integer]
    Declares: n
    Enables:
    . view:
      as: r := n is \rational
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
    fn check_rejects_removed_relation_enables_group() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("relation-represents-marker.mlg");

        write_mlg_fixture(
            &file,
            r#"[\rational]
    Declares: r
    Documented:
    . written: "\operatorname{rational}"

    [\integer]
    Declares: n
    Enables:
    . relation:
      to: r is \rational
      when: n is \integer
      represents: \\something.else
    Documented:
    . written: "\operatorname{integer}"
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("relation-represents-marker.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        let events = user_events(&event_log);
        assert!(events.iter().any(|event| {
            matches!(event, Event::Message(message) if
                message.message.contains("Unexpected enables group `relation`")
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
    Declares: r
    Documented:
    . written: "\operatorname{rational}"

    [\integer]
    Declares: n
    Enables:
    . view:
      as: r := n is \rational
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
    Declares: X
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
    given: X := {x_ : x_ is \set} is \set
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
    Declares: f
    Documented:
    . written: "\operatorname{function}"

    [\set]
    Declares: X
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
    given: X := {x_ : x_ is \set} is \set
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
    fn check_treats_membership_in_unstructured_collection_as_anything() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("collection-membership-anything.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Declares: X ::= {x__ : ...}
    Enables:
    . capability: x_ "in" X :-> x_ member_of X
    Documented:
    . written: "\operatorname{set}"

    [\needs.anything{x}]
    Declares: y
    when: x is \\anything
    Documented:
    . written: "\operatorname{needsAnything}(x?)"

    Theorem:
    given:
    . A is \set
    . x "in" A
    then: \needs.anything{x}
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("collection-membership-anything.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 1 file").with_origin("mlg_check")]
        );
    }

    #[test]
    fn check_anything_requirements_accept_any_declared_value() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("anything-requirement.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Declares: X
    Documented:
    . written: "\operatorname{set}"

    [\accepts.anything{X}]
    Declares: Y
    when: X is \\anything
    Documented:
    . written: "\operatorname{acceptsAnything}(X?)"

    Theorem:
    given: A is \set
    then: \accepts.anything{A}
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("anything-requirement.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 1 file").with_origin("mlg_check")]
        );
    }

    #[test]
    fn check_opaque_no_longer_has_builtin_anything_semantics() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("obsolete-opaque-requirement.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Declares: X
    Documented:
    . written: "\operatorname{set}"

    [\accepts.obsolete{X}]
    Declares: Y
    when: X is \\opaque
    Documented:
    . written: "\operatorname{acceptsObsolete}(X?)"

    Theorem:
    given: A is \set
    then: \accepts.obsolete{A}
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("obsolete-opaque-requirement.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert!(user_events(&event_log).iter().any(|event| {
            event.as_message().is_some_and(|message| {
                message.message
                    == "Could not establish requirement `A is \\\\opaque` for command `\\accepts.obsolete`"
            })
        }));
        assert!(event_log.has_errors());
    }

    #[test]
    fn check_anything_facts_do_not_establish_concrete_types() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("anything-does-not-establish-set.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Declares: X
    Documented:
    . written: "\operatorname{set}"

    [\requires.set{X}]
    Declares: Y
    when: X is \set
    Documented:
    . written: "\operatorname{requiresSet}(X?)"

    Theorem:
    given: A is \\anything
    then: \requires.set{A}
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("anything-does-not-establish-set.mlg")],
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
    Declares: G
    Enables:
    . capability: x_ "in" G :-> \\abstract
    Documented:
    . written: "\operatorname{group}"

    [\foo{G}:with{x}]
    Declares: y
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
    Declares: X
    Documented:
    . written: "\operatorname{set}"

    [\function:?on{A}:?to{B}]
    Declares: f(x__)
    when: A, B is \set
    Documented:
    . written: "\operatorname{function}"

    [\group]
    Declares: G ::= (X, *, e)
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
    Declares: X
    Documented:
    . written: "\operatorname{set}"

    [\group]
    Declares: G ::= (X, *, e)
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
    fn check_rejects_declares_when_for_non_header_target_symbols() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("declares-target-when-symbols.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Declares: X
    Enables:
    . capability: x_ "in" X :-> \\abstract
    Documented:
    . written: "\operatorname{set}"

    [\function:on{A}:to{B}]
    Declares: f(x__)
    when: A, B is \set
    Documented:
    . written: "\operatorname{function}"

    [\element.of:group{G}]
    Declares: x
    when: G is \group
    Documented:
    . written: "x? \in G?"

    [\group]
    Declares: G ::= (X, *, e) is \set via X
    when:
    . X is \set
    . * is \function:on{X}:to{X}
    . e "in" G
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
            &[PathBuf::from("declares-target-when-symbols.mlg")],
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
    fn check_rejects_a_when_requirement_on_the_described_spec_infix_subject() {
        // A spec-infix heading is sugar for a command whose left operand is the
        // symbol being defined, so `when:` may constrain only the other
        // operands; what `A` is belongs on the `Declares:` target.
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("described-subject-when.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Declares: X
    Documented:
    . written: "\operatorname{set}"

    [A \:subset:/ B]
    Declares: A is \set
    when: A, B is \set
    Documented:
    . written: "A? \subseteq B?"
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        check_in(
            temp_dir.path(),
            &[PathBuf::from("described-subject-when.mlg")],
            &mut event_log,
        );

        assert!(
            user_events(&event_log).iter().any(|event| {
                event.as_message().is_some_and(|message| {
                    message.message
                        == "`when:` requirement for `A` is not allowed because `A` is what this definition describes; state its type on the definition's target instead"
                })
            }),
            "{:#?}",
            user_events(&event_log)
        );
    }

    #[test]
    fn check_omits_described_subjects_and_optional_tail_parameters_from_when() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("optional-when-parameters.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Declares: X
    Documented:
    . written: "\operatorname{set}"

    [A \:subset:?within{U}:/ B]
    Declares: A is \set
    when:
    . U is \set
    . B is \set
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
    fn check_uses_nominal_typing_for_declares_type_requirements() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("nominal-declares-type.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Declares: X
    Documented:
    . written: "\operatorname{set}"

    [\function:?on{A}:?to{B}]
    Declares: f(x__)
    when: A, B is \set
    Documented:
    . written: "\operatorname{function}"

    [\group]
    Declares: G ::= (X, *, e)
    specifies:
    . X is \set
    . * is \function:on{X}:to{X}
    . e "in" G
    Enables:
    . capability: x_ "in" G :-> x_ is \element.of:group{G}
    Documented:
    . written: "\operatorname{group}"

    [\element.of:group{G}]
    Declares: x
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
            &[PathBuf::from("nominal-declares-type.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 1 file").with_origin("mlg_check")]
        );
    }

    #[test]
    fn check_validates_declares_type_expression_arguments() {
        let temp_dir = TestDir::new();
        let file = temp_dir
            .path()
            .join("declares-type-expression-arguments.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Declares: X
    Documented:
    . written: "\operatorname{set}"

    [\thing]
    Declares: value
    Documented:
    . written: "\operatorname{thing}"

    [\element.of:group{G}]
    Declares: x
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
            &[PathBuf::from("declares-type-expression-arguments.mlg")],
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
    Declares: X
    Documented:
    . written: "\operatorname{set}"

    [\function:?on{A}:?to{B}]
    Declares: f(x__)
    when: A, B is \set
    Documented:
    . written: "\operatorname{function}"

    [\group]
    Declares: G ::= (X, *, e)
    specifies:
    . X is \set
    . * is \function:on{X}:to{X}
    . e "in" G
    Enables:
    . capability: x_ "in" G :-> x_ is \element.of:group{G}
    Documented:
    . written: "\operatorname{group}"

    [\element.of:group{G}]
    Declares: x
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
    Declares: X
    Documented:
    . written: "\operatorname{set}"

    [\function:?on{A}:?to{B}]
    Declares: f(x__)
    when: A, B is \set
    Documented:
    . written: "\operatorname{function}"

    [\group]
    Declares: G ::= (X, *, e) is \set
    specifies:
    . X is \set
    . * is \function:on{X}:to{X}
    . e "in" G
    Enables:
    . capability: x_ "in" G :-> x_ is \element.of:group{G}
    Documented:
    . written: "\operatorname{group}"

    [\element.of:group{G}]
    Declares: x
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
    fn check_requires_defines_declares_and_refines_to_have_documented_called_or_written() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("documented-rendering.mlg");

        write_mlg_fixture(
            &file,
            r#"[\missing.rendering]
    Defines: A "defines" B

    [\describes.missing]
    Declares: A

    [\called.only]
    Declares: A
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
            "Declares entries must include either a `called:` or `written:` item in `Documented:`"
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
    fn check_requires_states_to_have_documented_called_or_written() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("states-rendering.mlg");

        // A `States:` group, like `Declares:`/`Defines:`, must render via `called:`
        // or `written:`. This one omits `Documented:` entirely, so it must error.
        write_mlg_fixture(
            &file,
            r#"[\set]
    Declares: X
    Enables:
    . capability: x_ "in" X :-> \\abstract
    Documented:
    . written: "\operatorname{set}"

    [x \.not.in./ X]
    States:
    when: x, X is \set
    that:
    . not: x "in"? X
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("states-rendering.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert!(
            event_log.events().iter().any(|event| {
                event.as_message().is_some_and(|message| {
                    message.message
                        == "States entries must include either a `called:` or `written:` item in `Documented:`"
                })
            }),
            "expected a States called/written diagnostic, got: {:#?}",
            user_events(&event_log)
        );
    }

    #[test]
    fn check_accepts_states_with_documented_written() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("states-valid.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Declares: X
    Enables:
    . capability: x_ "in" X :-> \\abstract
    Documented:
    . written: "\operatorname{set}"

    [x \.not.in./ X]
    States:
    when: x, X is \set
    that:
    . not: x "in"? X
    Documented:
    . written: "x? \notin X?"
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("states-valid.mlg")],
            &mut event_log,
        );

        assert_eq!(result.files_checked, 1);
        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 1 file").with_origin("mlg_check")]
        );
    }

    #[test]
    fn check_accepts_documented_called_or_written() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("documented-valid.mlg");

        write_mlg_fixture(
            &file,
            r#"[\written.only]
    Defines: A is \\anything
    Documented:
    . [docs.written]
      written:
      . "written only"

    [\called.only]
    Declares: A
    Documented:
    . [docs.called]
      called:
      . "called only"

    [\called.and.written]
    Declares: A
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
Declares: X
Documented:
. called: "set"
Id: "0d50b7b0-30b6-4bb1-9fa9-6ac3fcb435f0"

[\ordered.pair]
Declares: p
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
    fn check_accepts_complex_view_expression() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("relationships.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
Declares: X
Documented:
. called: "set"
Id: "0d50b7b0-30b6-4bb1-9fa9-6ac3fcb435f0"

[\pair]
Declares: P
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
. view:
  as: p := \set.theoretic.pair:of{a}:and{b} is \set
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
Declares: X
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
Declares: X
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

    #[test]
    fn check_accepts_piecewise_with_else_if_sections() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("piecewise_test.mlg");

        write_mlg_fixture(
            &file,
            r#"[\piecewise.test]
Theorem:
given: x is \\expression
then:
. piecewise:
  if: x = x
  then: x = x
  elseIf: x = x
  then: x = x
  else: x = x
Documented:
. called: "Piecewise test theorem"
"#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        check_in(
            temp_dir.path(),
            &[PathBuf::from("piecewise_test.mlg")],
            &mut event_log,
        );

        assert!(
            !event_log.has_errors(),
            "unexpected check errors: {:#?}",
            user_events(&event_log)
        );
    }

    #[test]
    fn check_follows_specifies_assignments_in_capabilities_for_requirements() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("von_neumann_capability.mlg");

        write_mlg_fixture(
            &file,
            r#"
[\set]
Declares: X
Requires:
. capability: x_ "in" X :-> \\abstract
Documented:
. called: "set"

[\empty.set]
Defines: X is \set
Documented:
. called: "empty set"

[\von.neumann.omega]
Defines: omega is \set
Enables:
. capability: X_ "in" omega :-> X_ is \set
Documented:
. called: "Von Neumann Omega"

[\von.neumann.S(n_)]
Defines: S(n_) is \function:on{\von.neumann.omega}:to{\von.neumann.omega}
expresses: S(n_) := \set.successor:of{n_}
Documented:
. called: "Von Neumann Successor"

[\set.successor:of{X}]
Defines: Y is \set
when: X is \set
Documented:
. called: "The set successor of $X?$"

[\natural]
Declares: n
Documented:
. called: "natural"

[\function:?on{A}:?to{B}]
Declares: f(x__) ::= y_
when: A, B is \set
specifies:
. x__ "in" A
. y_ "in" B
Documented:
. called: "function"

[\naturals]
Defines: Nb ::= (N, 0, S(n_))
abstractly:
specifies:
. N is \set
. 0 "in" N
. S is \function:on{N}:to{N}
Documented:
. called: "the naturals"

[\von.neumann.naturals]
Realizes: Nb ::= (N, 0, S(n_)) := \naturals
specifies:
. N := \von.neumann.omega
. 0 := \empty.set
. S(n_) := \von.neumann.S(n_)
Enables:
. capability: x_ "in" Nb :<->: x_ "in" N
Documented:
. called: "Von Neumann Naturals"

[\to.von.neumann.natural{n}]
Defines: m "in" \von.neumann.naturals
when: n is \natural
expresses:
. piecewise:
  if: n = 0
  then: m = \empty.set
  else:
  . let: k is \natural
    then: m = \set.successor:of{\to.von.neumann.natural{k}}
Documented:
. called: "Von Neumann Numeral"
"#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        check_in(
            temp_dir.path(),
            &[PathBuf::from("von_neumann_capability.mlg")],
            &mut event_log,
        );

        assert!(
            !event_log.has_errors(),
            "unexpected check errors: {:#?}",
            user_events(&event_log)
        );
    }

    #[test]
    fn check_accepts_specifies_with_walrus_in_declares_and_defines() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("specifies_walrus.mlg");

        write_mlg_fixture(
            &file,
            r#"
[\set]
Declares: X
Documented:
. called: "set"

[\empty.set]
Defines: X is \set
Documented:
. called: "empty set"

[\group_like]
Declares: G ::= (X, e)
specifies:
. X is \set
. e := \empty.set
Documented:
. called: "group like"

[\concrete_group]
Defines: G ::= (X, e) := \group_like
specifies:
. X := \empty.set
. e := \empty.set
Documented:
. called: "concrete group"
"#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        check_in(
            temp_dir.path(),
            &[PathBuf::from("specifies_walrus.mlg")],
            &mut event_log,
        );

        assert!(
            !event_log.has_errors(),
            "unexpected check errors: {:#?}",
            user_events(&event_log)
        );
    }

    #[test]
    fn check_allows_spec_with_direct_component_target() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("direct_component_spec.mlg");

        write_mlg_fixture(
            &file,
            r#"
[\set]
Declares: X
Requires:
. capability: x_ "in" X :-> \\abstract
Documented:
. called: "set"

[\function:?on{A}:?to{B}]
Declares: f(x__) ::= y_
when: A, B is \set
specifies:
. x__ "in" A
. y_ "in" B
Documented:
. called: "function"

[\naturals]
Defines: Nb ::= (N, 0, S(n_))
abstractly:
specifies:
. N is \set
. 0 "in" N
. S is \function:on{N}:to{N}
Documented:
. called: "the naturals"

[\natural]
Declares: n "in" \naturals..N
Documented:
. called: "natural"
"#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        check_in(
            temp_dir.path(),
            &[PathBuf::from("direct_component_spec.mlg")],
            &mut event_log,
        );

        assert!(
            !event_log.has_errors(),
            "unexpected check errors: {:#?}",
            user_events(&event_log)
        );
    }
}

#[cfg(test)]
mod type_info_tests {
    use super::check_collecting_type_info;
    use super::tests::TestDir;
    use crate::backend::config::default_config_contents;
    use crate::backend::semantic::{DocumentTypeInfo, TypeEntry};
    use std::fs;
    use std::path::Path;

    const SOURCE: &str = r#"[\set]
Declares: X
Documented:
. called: "set"


[\element:of{X}]
Declares: x
when: X is \set
Documented:
. called: "element"


Theorem:
given:
. X is \set
. x is \element:of{X}
then:
. x = x
"#;

    /// The entries recorded for the line whose text is exactly `line`.
    fn entries_for<'a>(info: &'a DocumentTypeInfo, source: &str, line: &str) -> &'a [TypeEntry] {
        let row = source
            .lines()
            .position(|candidate| candidate.trim() == line)
            .unwrap_or_else(|| panic!("no line `{line}` in:\n{source}"));
        info.get(&row)
            .unwrap_or_else(|| panic!("no type info on row {row} (`{line}`)"))
    }

    fn rendered(entries: &[TypeEntry]) -> Vec<(usize, &str, Vec<&str>)> {
        entries
            .iter()
            .map(|entry| {
                (
                    entry.depth,
                    entry.text.as_str(),
                    entry.types.iter().map(String::as_str).collect(),
                )
            })
            .collect()
    }

    fn check_fixture(root: &Path) -> (DocumentTypeInfo, String) {
        fs::write(root.join("mlg.json"), default_config_contents()).unwrap();
        let file = root.join("theory.mlg");
        fs::write(&file, SOURCE).unwrap();

        let result = check_collecting_type_info(root, &[], None, Some(&file));
        assert!(result.successful, "fixture should check cleanly");
        // `check` formats before checking, so read the file back for row numbers.
        (result.type_info, fs::read_to_string(&file).unwrap())
    }

    #[test]
    fn type_info_records_every_sub_expression_of_a_checked_line() {
        let temp_dir = TestDir::new();
        let (info, source) = check_fixture(temp_dir.path());

        assert_eq!(
            rendered(entries_for(&info, &source, r". x = x")),
            vec![
                (0, "x = x", vec![r"is \\statement"]),
                (1, "x", vec![r"is \element:of{X}"]),
                (1, "x", vec![r"is \element:of{X}"]),
            ]
        );
    }

    #[test]
    fn type_info_records_assumed_lines_that_are_never_checked() {
        let temp_dir = TestDir::new();
        let (info, source) = check_fixture(temp_dir.path());

        // A `given:` line is assumed rather than checked, so it reaches the
        // recorder through a different path than a `then:` line does.
        assert_eq!(
            rendered(entries_for(&info, &source, r". x is \element:of{X}")),
            vec![(
                0,
                r"x is \element:of{X}",
                vec![r"asserts x is \element:of{X}"]
            )]
        );
    }

    #[test]
    fn type_info_is_empty_for_a_check_that_asked_about_no_file() {
        let temp_dir = TestDir::new();
        let root = temp_dir.path();
        fs::write(root.join("mlg.json"), default_config_contents()).unwrap();
        fs::write(root.join("theory.mlg"), SOURCE).unwrap();

        let result = check_collecting_type_info(root, &[], None, None);

        assert!(result.successful);
        assert!(result.type_info.is_empty());
    }
}


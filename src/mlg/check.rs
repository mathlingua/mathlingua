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
mod tests {
    use super::check_in;
    use crate::events::{Audience, Event, EventLog, Level};
    use crate::mlg::collection::{find_collection_root, resolve_source_files};
    use crate::mlg::config::default_config_contents;
    use std::fs;
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

    #[test]
    fn check_without_arguments_uses_collection_content_from_a_nested_directory() {
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
    fn check_without_arguments_errors_when_not_in_a_collection() {
        let temp_dir = TestDir::new();

        let mut event_log = EventLog::new();
        let result = check_in(temp_dir.path(), &[], &mut event_log);
        let user_events = user_events(&event_log);

        assert_eq!(result.files_checked, 0);
        assert_eq!(event_log.issue_count(), 1);
        assert_eq!(
            user_events[0].as_message().unwrap().message,
            "Not inside a Mathlingua collection and no paths were provided"
        );
        assert_eq!(
            user_events[1],
            Event::user_log("Found 1 issue.").with_origin("mlg_check")
        );
        assert!(event_log.has_errors());
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
    fn resolve_source_files_collects_explicit_files_and_directories() {
        let temp_dir = TestDir::new();
        let docs = temp_dir.path().join("docs");
        let nested = docs.join("nested");
        let extra = temp_dir.path().join("extra.mlg");

        fs::create_dir_all(&nested).unwrap();
        fs::write(docs.join("a.mlg"), "Defines: A\n").unwrap();
        fs::write(nested.join("b.mlg"), "Defines: B\n").unwrap();
        fs::write(&extra, "Defines: C\n").unwrap();

        let mut event_log = EventLog::new();
        let files = resolve_source_files(
            temp_dir.path(),
            &[PathBuf::from("docs"), PathBuf::from("extra.mlg")],
            &mut event_log,
            "mlg_check",
        );

        assert!(user_events(&event_log).is_empty());
        assert_eq!(files.len(), 3);
    }

    #[test]
    fn finds_collection_root_in_ancestor_directories() {
        let temp_dir = TestDir::new();
        let root = temp_dir.path().join("repo");
        let nested = root.join("content/logic");

        fs::create_dir_all(&nested).unwrap();
        fs::write(root.join("mlg.json"), default_config_contents()).unwrap();

        let discovered = find_collection_root(&nested).expect("expected collection root");

        assert_eq!(discovered, root);
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

        fs::write(
            &file,
            r#"[\function{A, B}]
    Defines: A "defines" B
    Documented:
    . [docs.called]
      called:
      . "function"

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

        fs::write(
            &file,
            r#"[\function:on{A}:to{B}]
    Defines: A "defines" B
    Documented:
    . [docs.called]
      called:
      . "function"

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

        fs::write(
            &file,
            r#"[\foo{A, B}(x)]
    Defines: A "defines" B
    Documented:
    . [docs.called]
      called:
      . "foo"

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

        fs::write(
            &file,
            r#"[\thing]
    Describes: value
    Documented:
    . called: "thing"

    [\foo{A, B}(x)]
    Defines: A "defines" B
    Documented:
    . [docs.called]
      called:
      . "foo"

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

        fs::write(
            &file,
            r#"[\thing]
    Describes: value
    Documented:
    . called: "thing"

    [\some.function{A}(x, y)]
    Defines: A "defines" B
    Documented:
    . [docs.called]
      called:
      . "some function"

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

        fs::write(
            &file,
            r#"[\set]
    Describes: X
    Documented:
    . called: "set"

    [\function:on{A}:to{B}]
    Describes: f(x__)
    when: A, B is \set
    Documented:
    . called: "Function on $A?$ to $B?$"

    [\(bounded)::function:on{A}:to{B}]
    Refines: f(x__) is \function:on{A}:to{B}
    Documented:
    . called: "bounded"

    [\(continuous)::function:on{A}:to{B}]
    Refines: f(x__) is \function:on{A}:to{B}
    Documented:
    . called: "continuous"

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
    fn check_accepts_command_when_requirement_from_given_type_fact() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("type-fact.mlg");

        fs::write(
            &file,
            r#"[\real]
    Describes: x
    Documented:
    . called: "real"

    [\foo{s}]
    Describes: x
    when: s is \real
    Documented:
    . called: "foo"

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
    fn check_reports_command_when_requirement_type_mismatches() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("type-mismatch.mlg");

        fs::write(
            &file,
            r#"[\real]
    Describes: x
    Documented:
    . called: "real"

    [\foo{s}]
    Describes: x
    when: s is \real
    Documented:
    . called: "foo"

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
    fn check_reports_type_argument_mismatches_in_is_statements_and_predicates() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("function-argument-types.mlg");

        fs::write(
            &file,
            r#"[\set]
    Describes: X
    Provides:
    . symbol: x_ "in" X :-> \\abstract
    Documented:
    . called: "set"

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

        fs::write(
            &file,
            r#"[\set]
    Describes: X
    Provides:
    . symbol: x_ "in" X :-> \\abstract
    Documented:
    . called: "set"

    [\element.of:group{G := (X, *, e)}]
    Describes: x
    when: G is \group
    extends: x "in" X
    Documented:
    . called: "element of group"

    [\group]
    Describes: G := (X, *, e)
    extends: G is \set via X
    Provides:
    . symbol: x_ "in" G :-> x_ is \element.of:group{G}
    Documented:
    . called: "group"

    [\function:on{A}:to{B}]
    Describes: f(x__)
    when: A, B is \set
    Documented:
    . called: "function"

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
    fn check_reports_unrecognized_symbols_in_command_arguments() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("unrecognized-symbol.mlg");

        fs::write(
            &file,
            r#"[\set]
    Describes: X
    Documented:
    . called: "set"

    [\function:on{A}:to{B}]
    Describes: f(x__)
    when: A, B is \set
    Documented:
    . called: "function"

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

        fs::write(
            &file,
            r#"[\real]
    Describes: x
    Documented:
    . called: "real"

    [\foo{s}]
    Describes: x
    when: s is \real
    Documented:
    . called: "foo"

    Theorem:
    given: A is \real
    where:
    . A := B
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
    fn check_uses_quantifier_bindings_when_matching_types() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("exists-binding.mlg");

        fs::write(
            &file,
            r#"[\real]
    Describes: x
    Documented:
    . called: "real"

    [\foo{s}]
    Describes: x
    when: s is \real
    Documented:
    . called: "foo"

    Theorem:
    then:
    . exists: A := B
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
    fn check_reduces_spec_operator_aliases_to_type_facts() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("spec-reduction.mlg");

        fs::write(
            &file,
            r#"[\real]
    Describes: x
    Documented:
    . called: "real"

    [\reals]
    Describes: R
    Provides:
    . symbol: x_ "in" R :-> x is \real
    Documented:
    . called: "reals"

    [\foo{s}]
    Describes: x
    when: s is \real
    Documented:
    . called: "foo"

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

        fs::write(
            &file,
            r#"[\group]
    Describes: G
    Documented:
    . called: "group"

    [\foo{G}:with{x}]
    Describes: y
    when:
    . G is \group
    . x "in" G
    Documented:
    . called: "foo"

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
    fn check_reports_command_references_with_wrong_curly_argument_count() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("wrong-curly-count.mlg");

        fs::write(
            &file,
            r#"[\some.function{A}(x, y)]
    Defines: A "defines" B
    Documented:
    . [docs.called]
      called:
      . "some function"

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

        fs::write(
            &file,
            r#"[\some.function{A}(x, y)]
    Defines: A "defines" B
    Documented:
    . [docs.called]
      called:
      . "some function"

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
    fn check_requires_defines_describes_and_refines_to_have_documented_called() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("documented-called.mlg");

        fs::write(
            &file,
            r#"[\missing.called]
    Defines: A "defines" B

    [\written.only]
    Describes: A
    Documented:
    . [docs.written]
      written:
      . "written only"

    [\refines.missing]
    Refines: A is \missing.called
    "#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("documented-called.mlg")],
            &mut event_log,
        );
        let canonical_file = file.canonicalize().unwrap();

        assert_eq!(result.files_checked, 1);
        assert!(has_user_error_at(
            &event_log,
            &canonical_file,
            0,
            1,
            "Defines entries must include a `called:` item in `Documented:`"
        ));
        assert!(has_user_error_at(
            &event_log,
            &canonical_file,
            3,
            1,
            "Describes entries must include a `called:` item in `Documented:`"
        ));
        assert!(has_user_error_at(
            &event_log,
            &canonical_file,
            10,
            1,
            "Refines entries must include a `called:` item in `Documented:`"
        ));
        assert!(event_log.has_errors());
    }

    #[test]
    fn check_accepts_documented_called_with_or_without_written() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("documented-valid.mlg");

        fs::write(
            &file,
            r#"[\called.only]
    Defines: A "defines" B
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

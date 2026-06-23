use crate::backend::collection::SourceCollection;
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
    use super::check_in;
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
    . called: "thing"

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

        write_mlg_fixture(
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

        write_mlg_fixture(
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

        write_mlg_fixture(
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

        write_mlg_fixture(
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

        write_mlg_fixture(
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

        write_mlg_fixture(
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

        write_mlg_fixture(
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

        write_mlg_fixture(
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

        write_mlg_fixture(
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
    fn check_uses_function_type_extends_for_function_calls() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("function-type.mlg");

        write_mlg_fixture(
            &file,
            r#"[\set]
    Describes: X
    Documented:
    . called: "set"

    [\function:on{A}:to{B}]
    Describes: f(x__)
    when: A, B is \set
    extends: f is (_ "in" A) => (_ "in" B)
    Documented:
    . called: "function"

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
    Documented:
    . called: "set"

    [\function:on{A}:to{B}]
    Describes: f(x__)
    when: A, B is \set
    extends: f is (_ "in" A) => (_ "in" B)
    Documented:
    . called: "function"

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
    . called: "real"

    [\integer]
    Describes: x
    Documented:
    . called: "integer"

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
    . called: "set"

    [\function:on{A}:to{B}]
    Describes: f(x__)
    when: A, B is \set
    extends: f is (x "in" A) => (_ "in" B)
    Documented:
    . called: "function"
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

        write_mlg_fixture(
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

        write_mlg_fixture(
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

        write_mlg_fixture(
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

        write_mlg_fixture(
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

        write_mlg_fixture(
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

        write_mlg_fixture(
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

        write_mlg_fixture(
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

        write_mlg_fixture(
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

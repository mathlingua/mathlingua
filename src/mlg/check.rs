use crate::environment::current_working_directory;
use crate::events::{Audience, Event, EventLog, Level, MarkerRange};
use crate::frontend::structural::parse_document;
use crate::mlg::collection::resolve_source_files;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

const ORIGIN: &str = "mlg_check";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckResult {
    pub files_checked: usize,
    pub marker_range: MarkerRange,
}

pub fn check(paths: &[PathBuf], event_log: &mut EventLog) -> io::Result<CheckResult> {
    let Some(cwd) = current_working_directory(event_log) else {
        return Err(io::Error::other(
            "Failed to determine the current working directory",
        ));
    };

    Ok(check_in(&cwd, paths, event_log))
}

pub fn check_in(cwd: &Path, paths: &[PathBuf], event_log: &mut EventLog) -> CheckResult {
    let begin = event_log.begin_marker("check_in", Some(ORIGIN));
    let starting_event_count = event_log.events().len();

    event_log.system_debug(
        Some(ORIGIN),
        format!("Checking {} explicit path(s)", paths.len()),
    );

    let files = resolve_source_files(cwd, paths, event_log, ORIGIN);
    let files_checked = files.len();

    for file in files {
        parse_source_file(&file, event_log);
    }

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

fn parse_source_file(path: &Path, event_log: &mut EventLog) {
    event_log.system_debug(Some(ORIGIN), format!("Parsing {}", path.display()));

    let source = match fs::read_to_string(path) {
        Ok(source) => source,
        Err(error) => {
            event_log.user_error_at_path(
                Some(ORIGIN),
                path.to_path_buf(),
                format!("Failed to read file: {error}"),
            );
            return;
        }
    };

    let mut file_event_log = EventLog::new();
    let _ = parse_document(&source, &mut file_event_log);

    for event in file_event_log.events() {
        event_log.push(event.clone().with_file_path(path.to_path_buf()));
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

// =============================================================================

#[cfg(test)]
mod tests {
    use super::check_in;
    use crate::events::{Audience, Event, EventLog, Level};
    use crate::mlg::collection::{find_collection_root, resolve_source_files};
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    static NEXT_TEST_DIR_ID: AtomicUsize = AtomicUsize::new(0);

    fn user_events(event_log: &EventLog) -> Vec<Event> {
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

    #[test]
    fn check_without_arguments_uses_collection_content_from_a_nested_directory() {
        let temp_dir = TestDir::new();
        let root = temp_dir.path().join("repo");
        let nested_cwd = root.join("content/algebra");

        fs::create_dir_all(&nested_cwd).unwrap();
        fs::write(root.join("mlg.json"), "{}\n").unwrap();
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
        fs::write(root.join("mlg.json"), "{}\n").unwrap();

        let mut event_log = EventLog::new();
        let result = check_in(&root, &[], &mut event_log);

        assert_eq!(result.files_checked, 0);
        assert_eq!(
            user_events(&event_log),
            [Event::user_log("Checked 0 files").with_origin("mlg_check")]
        );
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
        fs::write(root.join("mlg.json"), "{}\n").unwrap();

        let discovered = find_collection_root(&nested).expect("expected collection root");

        assert_eq!(discovered, root);
    }

    #[test]
    fn check_result_markers_bound_the_check_events() {
        let temp_dir = TestDir::new();
        let root = temp_dir.path().join("repo");

        fs::create_dir_all(root.join("content")).unwrap();
        fs::write(root.join("mlg.json"), "{}\n").unwrap();

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

    struct TestDir {
        path: PathBuf,
    }

    impl TestDir {
        fn new() -> Self {
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

        fn path(&self) -> &Path {
            &self.path
        }
    }

    impl Drop for TestDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }
}

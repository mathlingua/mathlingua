use super::{find_collection_root, render_diagnostic, resolve_source_files, run_in};
use crate::diagnostics::{Diagnostic, DiagnosticTracker, Location, Severity};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static NEXT_TEST_DIR_ID: AtomicUsize = AtomicUsize::new(0);

#[test]
fn check_without_arguments_uses_collection_content_from_a_nested_directory() {
    let temp_dir = TestDir::new();
    let root = temp_dir.path().join("repo");
    let nested_cwd = root.join("content/algebra");

    fs::create_dir_all(&nested_cwd).unwrap();
    fs::write(root.join("mlg.json"), "{}\n").unwrap();
    fs::write(root.join("content/sets.mlg"), "Defines: A\n").unwrap();
    fs::write(nested_cwd.join("groups.mlg"), "Defines: G\n").unwrap();

    let result = run_in(&nested_cwd, &[]);

    assert_eq!(result.summary.files_checked, 2);
    assert!(result.diagnostics.diagnostics().is_empty());
}

#[test]
fn check_without_arguments_errors_when_not_in_a_collection() {
    let temp_dir = TestDir::new();

    let result = run_in(temp_dir.path(), &[]);

    assert_eq!(result.diagnostics.diagnostics().len(), 1);
    assert_eq!(
        result.diagnostics.diagnostics()[0].message,
        "Not inside a Mathlingua collection and no paths were provided"
    );
    assert!(result.diagnostics.has_errors());
}

#[test]
fn check_with_directory_argument_processes_mlg_files_recursively() {
    let temp_dir = TestDir::new();
    let docs = temp_dir.path().join("docs/logic");

    fs::create_dir_all(&docs).unwrap();
    fs::write(docs.join("intro.mlg"), "Defines: f(x_)\n").unwrap();
    fs::write(docs.join("notes.txt"), "ignore me").unwrap();

    let result = run_in(temp_dir.path(), &[PathBuf::from("docs")]);

    assert_eq!(result.summary.files_checked, 1);
    assert!(result.diagnostics.diagnostics().is_empty());
}

#[test]
fn check_with_empty_content_directory_succeeds() {
    let temp_dir = TestDir::new();
    let root = temp_dir.path().join("repo");

    fs::create_dir_all(root.join("content")).unwrap();
    fs::write(root.join("mlg.json"), "{}\n").unwrap();

    let result = run_in(&root, &[]);

    assert_eq!(result.summary.files_checked, 0);
    assert!(result.diagnostics.diagnostics().is_empty());
}

#[test]
fn check_reports_proto_diagnostics_for_invalid_files() {
    let temp_dir = TestDir::new();
    let file = temp_dir.path().join("broken.mlg");

    fs::write(&file, "Defines: 'f(x_)'\n").unwrap();

    let result = run_in(temp_dir.path(), &[PathBuf::from("broken.mlg")]);
    let diagnostics = result.diagnostics.diagnostics();

    assert_eq!(diagnostics.len(), 1);
    assert_eq!(
        diagnostics[0].location.path,
        Some(file.canonicalize().unwrap())
    );
    assert_eq!(diagnostics[0].location.row, Some(0));
    assert_eq!(
        diagnostics[0].message,
        "Single-quoted formulations are not allowed"
    );
    assert!(result.diagnostics.has_errors());
}

#[test]
fn check_rejects_non_mlg_files_when_given_explicitly() {
    let temp_dir = TestDir::new();
    let file = temp_dir.path().join("notes.txt");

    fs::write(&file, "not mathlingua").unwrap();

    let result = run_in(temp_dir.path(), &[PathBuf::from("notes.txt")]);
    let diagnostics = result.diagnostics.diagnostics();

    assert_eq!(diagnostics.len(), 1);
    assert_eq!(
        diagnostics[0],
        Diagnostic::path_error(file.canonicalize().unwrap(), "Not a .mlg file")
    );
    assert!(result.diagnostics.has_errors());
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

    let mut diagnostics = DiagnosticTracker::new();
    let files = resolve_source_files(
        temp_dir.path(),
        &[PathBuf::from("docs"), PathBuf::from("extra.mlg")],
        &mut diagnostics,
    );

    assert!(diagnostics.diagnostics().is_empty());
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
fn renders_issues_relative_to_the_working_directory_when_possible() {
    let cwd = Path::new("/repo");
    let diagnostic = Diagnostic {
        message: "Unexpected header: [duplicate]".to_string(),
        severity: Severity::Error,
        location: Location::at_path_and_row("/repo/content/example.mlg", 3),
    };

    assert_eq!(
        render_diagnostic(cwd, &diagnostic),
        "content/example.mlg:4: error: Unexpected header: [duplicate]"
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

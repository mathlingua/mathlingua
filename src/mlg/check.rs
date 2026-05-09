use crate::constants::{CONFIG_FILE, CONTENT_DIR};
use crate::diagnostics::{DiagnosticTracker, Level};
use crate::frontend::structural::parse_document;
use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct CheckResult {
    pub files_checked: usize,
}

pub fn check(paths: &[PathBuf], diagnostics: &mut DiagnosticTracker) -> io::Result<CheckResult> {
    let cwd = match env::current_dir() {
        Ok(cwd) => cwd,
        Err(error) => {
            diagnostics.global_error(format!(
                "Failed to determine the current working directory: {error}"
            ));
            return Err(error);
        }
    };

    Ok(check_in(&cwd, paths, diagnostics))
}

pub fn check_in(cwd: &Path, paths: &[PathBuf], diagnostics: &mut DiagnosticTracker) -> CheckResult {
    let starting_diagnostic_count = diagnostics.diagnostics().len();
    let files = resolve_source_files(cwd, paths, diagnostics);
    let files_checked = files.len();

    for file in files {
        parse_source_file(&file, diagnostics);
    }

    let new_diagnostics = &diagnostics.diagnostics()[starting_diagnostic_count..];
    let has_new_errors = new_diagnostics
        .iter()
        .any(|diagnostic| diagnostic.level == Level::Error);
    let new_issue_count = new_diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.level != Level::Log)
        .count();

    if has_new_errors {
        diagnostics.log(format!(
            "Found {}.",
            format_diagnostic_count(new_issue_count)
        ));
    } else {
        diagnostics.log(render_check_success(files_checked));
    }

    CheckResult { files_checked }
}

fn resolve_source_files(
    cwd: &Path,
    paths: &[PathBuf],
    diagnostics: &mut DiagnosticTracker,
) -> Vec<PathBuf> {
    let mut files = BTreeSet::new();

    if paths.is_empty() {
        let Some(root) = find_collection_root(cwd) else {
            diagnostics.path_error(
                cwd.to_path_buf(),
                "Not inside a Mathlingua collection and no paths were provided",
            );
            return Vec::new();
        };

        collect_source_files(root.join(CONTENT_DIR), &mut files, diagnostics);
    } else {
        for path in paths {
            if let Some(resolved_path) = resolve_input_path(cwd, path, diagnostics) {
                collect_source_files(resolved_path, &mut files, diagnostics);
            }
        }
    }

    files.into_iter().collect()
}

fn resolve_input_path(
    cwd: &Path,
    path: &Path,
    diagnostics: &mut DiagnosticTracker,
) -> Option<PathBuf> {
    let joined = if path.is_absolute() {
        path.to_path_buf()
    } else {
        cwd.join(path)
    };

    match joined.canonicalize() {
        Ok(path) => Some(path),
        Err(error) => {
            diagnostics.path_error(joined, format!("Failed to resolve path: {error}"));
            None
        }
    }
}

fn find_collection_root(start: &Path) -> Option<PathBuf> {
    start
        .ancestors()
        .find(|directory| directory.join(CONFIG_FILE).is_file())
        .map(Path::to_path_buf)
}

fn collect_source_files(
    target: PathBuf,
    files: &mut BTreeSet<PathBuf>,
    diagnostics: &mut DiagnosticTracker,
) {
    match fs::metadata(&target) {
        Ok(metadata) if metadata.is_dir() => {
            collect_directory_source_files(&target, files, diagnostics)
        }
        Ok(metadata) if metadata.is_file() => {
            if is_mathlingua_source_file(&target) {
                files.insert(target);
            } else {
                diagnostics.path_error(target, "Not a .mlg file");
            }
        }
        Ok(_) => diagnostics.path_error(target, "Unsupported filesystem entry"),
        Err(error) => diagnostics.path_error(target, format!("Failed to read path: {error}")),
    }
}

fn collect_directory_source_files(
    directory: &Path,
    files: &mut BTreeSet<PathBuf>,
    diagnostics: &mut DiagnosticTracker,
) {
    let entries = match read_directory_entries(directory) {
        Ok(entries) => entries,
        Err(error) => {
            diagnostics.path_error(
                directory.to_path_buf(),
                format!("Failed to read directory: {error}"),
            );
            return;
        }
    };

    for entry in entries {
        let path = entry.path();

        if path.is_dir() {
            collect_directory_source_files(&path, files, diagnostics);
        } else if path.is_file() && is_mathlingua_source_file(&path) {
            files.insert(path);
        }
    }
}

fn read_directory_entries(directory: &Path) -> io::Result<Vec<fs::DirEntry>> {
    let mut entries = fs::read_dir(directory)?.collect::<Result<Vec<_>, io::Error>>()?;
    entries.sort_by(|left, right| left.path().cmp(&right.path()));
    Ok(entries)
}

fn is_mathlingua_source_file(path: &Path) -> bool {
    path.extension()
        .and_then(|value| value.to_str())
        .map(|extension| extension.eq_ignore_ascii_case("mlg"))
        .unwrap_or(false)
}

fn parse_source_file(path: &Path, diagnostics: &mut DiagnosticTracker) {
    let source = match fs::read_to_string(path) {
        Ok(source) => source,
        Err(error) => {
            diagnostics.path_error(path.to_path_buf(), format!("Failed to read file: {error}"));
            return;
        }
    };

    let mut file_diagnostics = DiagnosticTracker::new();
    let _ = parse_document(&source, &mut file_diagnostics);

    for diagnostic in file_diagnostics.diagnostics() {
        diagnostics.push(diagnostic.clone().with_path(path.to_path_buf()));
    }
}

fn render_check_success(files_checked: usize) -> String {
    if files_checked == 1 {
        "Checked 1 file".to_string()
    } else {
        format!("Checked {files_checked} files")
    }
}

fn format_diagnostic_count(diagnostic_count: usize) -> String {
    if diagnostic_count == 1 {
        "1 diagnostic".to_string()
    } else {
        format!("{diagnostic_count} diagnostics")
    }
}

// =============================================================================

#[cfg(test)]
mod tests {
    use super::{check_in, find_collection_root, resolve_source_files};
    use crate::diagnostics::{
        ColorMode, Diagnostic, DiagnosticFormatter, DiagnosticTracker, Level, Location,
    };
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
        fs::write(root.join("content/sets.mlg"), "Title: \"Sets\"\n").unwrap();
        fs::write(nested_cwd.join("groups.mlg"), "Title: \"Groups\"\n").unwrap();

        let mut diagnostics = DiagnosticTracker::new();
        let result = check_in(&nested_cwd, &[], &mut diagnostics);

        assert_eq!(result.files_checked, 2);
        assert_eq!(
            diagnostics.diagnostics(),
            [Diagnostic::log("Checked 2 files")]
        );
    }

    #[test]
    fn check_without_arguments_errors_when_not_in_a_collection() {
        let temp_dir = TestDir::new();

        let mut diagnostics = DiagnosticTracker::new();
        let result = check_in(temp_dir.path(), &[], &mut diagnostics);

        assert_eq!(result.files_checked, 0);
        assert_eq!(diagnostics.issue_count(), 1);
        assert_eq!(
            diagnostics.diagnostics()[0].message,
            "Not inside a Mathlingua collection and no paths were provided"
        );
        assert_eq!(
            diagnostics.diagnostics()[1],
            Diagnostic::log("Found 1 diagnostic.")
        );
        assert!(diagnostics.has_errors());
    }

    #[test]
    fn check_with_directory_argument_processes_mlg_files_recursively() {
        let temp_dir = TestDir::new();
        let docs = temp_dir.path().join("docs/logic");

        fs::create_dir_all(&docs).unwrap();
        fs::write(docs.join("intro.mlg"), "Title: \"Intro\"\n").unwrap();
        fs::write(docs.join("notes.txt"), "ignore me").unwrap();

        let mut diagnostics = DiagnosticTracker::new();
        let result = check_in(temp_dir.path(), &[PathBuf::from("docs")], &mut diagnostics);

        assert_eq!(result.files_checked, 1);
        assert_eq!(
            diagnostics.diagnostics(),
            [Diagnostic::log("Checked 1 file")]
        );
    }

    #[test]
    fn check_with_empty_content_directory_succeeds() {
        let temp_dir = TestDir::new();
        let root = temp_dir.path().join("repo");

        fs::create_dir_all(root.join("content")).unwrap();
        fs::write(root.join("mlg.json"), "{}\n").unwrap();

        let mut diagnostics = DiagnosticTracker::new();
        let result = check_in(&root, &[], &mut diagnostics);

        assert_eq!(result.files_checked, 0);
        assert_eq!(
            diagnostics.diagnostics(),
            [Diagnostic::log("Checked 0 files")]
        );
    }

    #[test]
    fn check_reports_proto_diagnostics_for_invalid_files() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("broken.mlg");

        fs::write(&file, "Defines: 'f(x_)'\n").unwrap();

        let mut diagnostics = DiagnosticTracker::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("broken.mlg")],
            &mut diagnostics,
        );
        let diagnostics = diagnostics.diagnostics();

        assert_eq!(result.files_checked, 1);
        assert!(
            diagnostics
                .iter()
                .filter(|diagnostic| diagnostic.level != Level::Log)
                .all(|diagnostic| {
                    diagnostic.location.path == Some(file.canonicalize().unwrap())
                })
        );
        assert!(diagnostics.iter().any(|diagnostic| {
            diagnostic.location.row == Some(0)
                && diagnostic.message == "Single-quoted formulations are not allowed"
        }));
        let issue_count = diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.level != Level::Log)
            .count();
        assert_eq!(
            diagnostics.last(),
            Some(&Diagnostic::log(format!(
                "Found {} diagnostic{}.",
                issue_count,
                if issue_count == 1 { "" } else { "s" }
            )))
        );
    }

    #[test]
    fn check_reports_structural_and_formulation_diagnostics_for_invalid_files() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("broken-structural.mlg");

        fs::write(&file, "[\\function]\nDefines: x |plus|\n").unwrap();

        let mut diagnostics = DiagnosticTracker::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("broken-structural.mlg")],
            &mut diagnostics,
        );
        let diagnostics = diagnostics.diagnostics();

        assert_eq!(result.files_checked, 1);
        assert!(
            diagnostics
                .iter()
                .filter(|diagnostic| diagnostic.level != Level::Log)
                .all(|diagnostic| {
                    diagnostic.location.path == Some(file.canonicalize().unwrap())
                })
        );
        assert!(diagnostics.iter().any(|diagnostic| {
            diagnostic
                .message
                .starts_with("Invalid Defines formulation:")
        }));
        let issue_count = diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.level != Level::Log)
            .count();
        assert!(diagnostics.iter().any(|diagnostic| {
            diagnostic.level == Level::Log
                && diagnostic.message
                    == format!(
                        "Found {} diagnostic{}.",
                        issue_count,
                        if issue_count == 1 { "" } else { "s" }
                    )
        }));
    }

    #[test]
    fn check_rejects_non_mlg_files_when_given_explicitly() {
        let temp_dir = TestDir::new();
        let file = temp_dir.path().join("notes.txt");

        fs::write(&file, "not mathlingua").unwrap();

        let mut diagnostics = DiagnosticTracker::new();
        let result = check_in(
            temp_dir.path(),
            &[PathBuf::from("notes.txt")],
            &mut diagnostics,
        );
        let diagnostics = diagnostics.diagnostics();

        assert_eq!(result.files_checked, 0);
        assert_eq!(diagnostics.len(), 2);
        assert_eq!(
            diagnostics[0],
            Diagnostic::path_error(file.canonicalize().unwrap(), "Not a .mlg file")
        );
        assert_eq!(diagnostics[1], Diagnostic::log("Found 1 diagnostic."));
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
        let formatter = DiagnosticFormatter::new()
            .with_base_path(cwd)
            .with_color_mode(ColorMode::Never);
        let diagnostic = Diagnostic {
            message: "Unexpected header: [duplicate]".to_string(),
            level: Level::Error,
            location: Location::at_path_and_row("/repo/content/example.mlg", 3),
        };

        assert_eq!(
            formatter.format(&diagnostic),
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
}

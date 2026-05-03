use crate::constants::{CONFIG_FILE, CONTENT_DIR};
use crate::diagnostics::{Diagnostic, DiagnosticTracker, Severity};
use crate::proto::Parser as ProtoParser;
use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process;

pub fn run(paths: &[PathBuf]) {
    let cwd = match env::current_dir() {
        Ok(cwd) => cwd,
        Err(error) => {
            eprintln!("Failed to determine the current working directory: {error}");
            process::exit(1);
        }
    };

    let result = run_in(&cwd, paths);

    for diagnostic in result.diagnostics.diagnostics() {
        eprintln!("{}", render_diagnostic(&cwd, diagnostic));
    }

    if result.diagnostics.has_errors() {
        eprintln!(
            "Found {}.",
            format_diagnostic_count(result.diagnostics.diagnostics().len())
        );
        process::exit(1);
    }

    println!("{}", render_success(&result.summary));
}

#[derive(Debug)]
struct CheckRunResult {
    summary: CheckSummary,
    diagnostics: DiagnosticTracker,
}

#[derive(Debug, PartialEq, Eq)]
struct CheckSummary {
    files_checked: usize,
}

fn run_in(cwd: &Path, paths: &[PathBuf]) -> CheckRunResult {
    let mut diagnostics = DiagnosticTracker::new();
    let files = resolve_source_files(cwd, paths, &mut diagnostics);
    let files_checked = files.len();

    for file in files {
        parse_source_file(&file, &mut diagnostics);
    }

    CheckRunResult {
        summary: CheckSummary { files_checked },
        diagnostics,
    }
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
    {
        let mut parser = ProtoParser::new(&source, &mut file_diagnostics);
        let _ = parser.parse();
    }

    for diagnostic in file_diagnostics.diagnostics() {
        diagnostics.push(diagnostic.clone().with_path(path.to_path_buf()));
    }
}

fn render_success(summary: &CheckSummary) -> String {
    if summary.files_checked == 1 {
        "Checked 1 file".to_string()
    } else {
        format!("Checked {} files", summary.files_checked)
    }
}

fn render_diagnostic(cwd: &Path, diagnostic: &Diagnostic) -> String {
    let severity = match diagnostic.severity {
        Severity::Error => "error",
        Severity::Warning => "warning",
    };

    match (&diagnostic.location.path, diagnostic.location.row) {
        (Some(path), Some(row)) => {
            format!(
                "{}:{}: {severity}: {}",
                display_path(cwd, path),
                row + 1,
                diagnostic.message
            )
        }
        (Some(path), None) => {
            format!(
                "{}: {severity}: {}",
                display_path(cwd, path),
                diagnostic.message
            )
        }
        (None, Some(row)) => format!("line {}: {severity}: {}", row + 1, diagnostic.message),
        (None, None) => format!("{severity}: {}", diagnostic.message),
    }
}

fn display_path(cwd: &Path, path: &Path) -> String {
    path.strip_prefix(cwd)
        .map(|relative| {
            if relative.as_os_str().is_empty() {
                ".".to_string()
            } else {
                relative.display().to_string()
            }
        })
        .unwrap_or_else(|_| path.display().to_string())
}

fn format_diagnostic_count(diagnostic_count: usize) -> String {
    if diagnostic_count == 1 {
        "1 diagnostic".to_string()
    } else {
        format!("{diagnostic_count} diagnostics")
    }
}

#[cfg(test)]
mod tests;

use serde_json::Value;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

const CASES_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/mlg_check_cases");
static NEXT_TEMP_DIR_ID: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug)]
struct CheckCase {
    source_file: PathBuf,
    description: String,
    input: String,
    expected_output: String,
}

fn main() {
    let fixture_files = fixture_files(Path::new(CASES_DIR));
    assert!(
        !fixture_files.is_empty(),
        "expected mlg check fixture files"
    );

    let mut failures = Vec::new();
    let mut passed = 0usize;
    for path in &fixture_files {
        let contents = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("could not read {}: {error}", path.display()));
        let case = parse_case(path, &contents);
        let display_path = path.strip_prefix(CASES_DIR).unwrap_or(path);

        print!("test {} ... ", display_path.display());
        io::stdout().flush().expect("could not flush test status");
        match run_case(&case) {
            Ok(()) => {
                passed += 1;
                println!("ok");
            }
            Err(failure) => {
                println!("FAILED");
                failures.push(failure);
            }
        }
    }

    println!();
    if failures.is_empty() {
        println!(
            "test result: ok. {passed} passed; 0 failed; {} total",
            fixture_files.len()
        );
    } else {
        eprintln!(
            "test result: FAILED. {passed} passed; {} failed; {} total\n\n{}",
            failures.len(),
            fixture_files.len(),
            failures.join("\n\n")
        );
        std::process::exit(1);
    }
}

fn fixture_files(root: &Path) -> Vec<PathBuf> {
    fn visit(directory: &Path, files: &mut Vec<PathBuf>) {
        let entries = fs::read_dir(directory)
            .unwrap_or_else(|error| panic!("could not read {}: {error}", directory.display()));
        for entry in entries {
            let path = entry
                .expect("could not read fixture directory entry")
                .path();
            if path.is_dir() {
                visit(&path, files);
            } else if path.extension().is_some_and(|extension| extension == "txt") {
                files.push(path);
            }
        }
    }

    let mut files = Vec::new();
    visit(root, &mut files);
    files.sort();
    files
}

fn parse_case(path: &Path, contents: &str) -> CheckCase {
    const BEGIN_DESCRIPTION: &str = "== begin:description ==";
    const END_DESCRIPTION: &str = "== end:description ==";
    const BEGIN_INPUT: &str = "== begin:input ==";
    const END_INPUT: &str = "== end:input ==";
    const BEGIN_OUTPUT: &str = "== begin:output ==";
    const END_OUTPUT: &str = "== end:output ==";

    let lines = contents.split_inclusive('\n').collect::<Vec<_>>();
    let mut cursor = 0usize;
    expect_marker(path, &lines, &mut cursor, BEGIN_DESCRIPTION);
    let description = take_section(path, &lines, &mut cursor, END_DESCRIPTION);
    expect_marker(path, &lines, &mut cursor, BEGIN_INPUT);
    let input = take_section(path, &lines, &mut cursor, END_INPUT);
    expect_marker(path, &lines, &mut cursor, BEGIN_OUTPUT);
    let expected_output = take_section(path, &lines, &mut cursor, END_OUTPUT);

    while cursor < lines.len() && lines[cursor].trim().is_empty() {
        cursor += 1;
    }
    assert_eq!(
        cursor,
        lines.len(),
        "{} contains content after its one test case",
        path.display()
    );

    let description = description.trim().to_owned();
    assert!(
        !description.is_empty(),
        "{} has an empty description",
        path.display()
    );
    assert!(
        !input.trim().is_empty(),
        "{} has empty input",
        path.display()
    );

    CheckCase {
        source_file: path.to_path_buf(),
        description,
        input,
        expected_output: expected_output.trim_end().to_owned(),
    }
}

fn expect_marker(path: &Path, lines: &[&str], cursor: &mut usize, marker: &str) {
    assert_eq!(
        lines.get(*cursor).map(|line| line.trim_end()),
        Some(marker),
        "{}:{}: expected `{marker}`",
        path.display(),
        *cursor + 1
    );
    *cursor += 1;
}

fn take_section(path: &Path, lines: &[&str], cursor: &mut usize, end_marker: &str) -> String {
    let mut value = String::new();
    while let Some(line) = lines.get(*cursor) {
        if line.trim_end() == end_marker {
            *cursor += 1;
            return value;
        }
        value.push_str(line);
        *cursor += 1;
    }
    panic!("{}: missing `{end_marker}`", path.display());
}

fn run_case(case: &CheckCase) -> Result<(), String> {
    let temp_dir = TempDir::new();
    let source_path = temp_dir.path.join("case.mlg");
    fs::write(&source_path, &case.input).map_err(|error| format_case(case, error))?;

    let output = Command::new(env!("CARGO_BIN_EXE_mlg"))
        .current_dir(&temp_dir.path)
        .args(["check", "--json", "case.mlg"])
        .output()
        .map_err(|error| format_case(case, format!("could not run mlg: {error}")))?;

    if !output.stderr.is_empty() {
        return Err(format_case(
            case,
            format!(
                "mlg --json wrote unexpected stderr:\n{}",
                String::from_utf8_lossy(&output.stderr)
            ),
        ));
    }

    let report: Value = serde_json::from_slice(&output.stdout).map_err(|error| {
        format_case(
            case,
            format!(
                "mlg did not write a JSON diagnostic report: {error}\nstdout:\n{}",
                String::from_utf8_lossy(&output.stdout)
            ),
        )
    })?;
    let actual_output =
        render_diagnostics(&report, &temp_dir.path).map_err(|error| format_case(case, error))?;
    let successful = report
        .get("successful")
        .and_then(Value::as_bool)
        .ok_or_else(|| format_case(case, "report omitted boolean `successful`"))?;
    let expected_success = case.expected_output.is_empty();

    if output.status.success() != successful {
        return Err(format_case(
            case,
            format!(
                "process success ({}) disagreed with report success ({successful})",
                output.status.success()
            ),
        ));
    }
    if successful != expected_success {
        return Err(format_case(
            case,
            format!(
                "expected success {expected_success}, got {successful}\nexpected output:\n{}\nactual output:\n{}",
                display_output(&case.expected_output),
                display_output(&actual_output)
            ),
        ));
    }
    if actual_output != case.expected_output {
        return Err(format_case(
            case,
            format!(
                "output differed\nexpected:\n{}\nactual:\n{}",
                display_output(&case.expected_output),
                display_output(&actual_output)
            ),
        ));
    }

    Ok(())
}

/// Render the stable, user-visible part of each diagnostic. File locations are
/// intentionally omitted because generated `Id` sections can shift line
/// numbers away from the source text shown in a fixture.
fn render_diagnostics(report: &Value, invocation_root: &Path) -> Result<String, String> {
    let canonical_root = invocation_root
        .canonicalize()
        .unwrap_or_else(|_| invocation_root.into());
    let diagnostics = report
        .get("diagnostics")
        .and_then(Value::as_array)
        .ok_or_else(|| "report omitted array `diagnostics`".to_owned())?;
    diagnostics
        .iter()
        .map(|diagnostic| {
            let level = diagnostic
                .get("level")
                .and_then(Value::as_str)
                .ok_or_else(|| "diagnostic omitted string `level`".to_owned())?;
            let message = diagnostic
                .get("message")
                .and_then(Value::as_str)
                .ok_or_else(|| "diagnostic omitted string `message`".to_owned())?;
            let message = message
                .replace(&format!("{}/", canonical_root.display()), "")
                .replace(&format!("{}/", invocation_root.display()), "");
            Ok(format!("{level}: {message}"))
        })
        .collect::<Result<Vec<_>, String>>()
        .map(|lines| lines.join("\n"))
}

fn format_case(case: &CheckCase, detail: impl std::fmt::Display) -> String {
    format!(
        "{} — {}\n{}",
        case.source_file.display(),
        case.description,
        detail
    )
}

fn display_output(output: &str) -> &str {
    if output.is_empty() { "<blank>" } else { output }
}

struct TempDir {
    path: PathBuf,
}

impl TempDir {
    fn new() -> Self {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is before Unix epoch")
            .as_nanos();
        let id = NEXT_TEMP_DIR_ID.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "mlg-check-e2e-{}-{unique}-{id}",
            std::process::id()
        ));
        fs::create_dir(&path).expect("could not create mlg check test directory");
        Self { path }
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

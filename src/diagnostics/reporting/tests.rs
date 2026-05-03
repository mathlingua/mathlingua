use super::format_diagnostic;
use crate::diagnostics::{Diagnostic, Location, Severity};
use std::path::Path;

#[test]
fn formats_file_diagnostics_relative_to_the_base_path() {
    let base = Path::new("/repo");
    let diagnostic = Diagnostic {
        message: "Unexpected header: [duplicate]".to_string(),
        severity: Severity::Error,
        location: Location::at_path_and_row("/repo/content/example.mlg", 3),
    };

    assert_eq!(
        format_diagnostic(base, &diagnostic, false),
        "content/example.mlg:4: error: Unexpected header: [duplicate]"
    );
}

#[test]
fn formats_path_only_diagnostics() {
    let base = Path::new("/repo");
    let diagnostic = Diagnostic::path_error("/repo/content", "Not a .mlg file");

    assert_eq!(
        format_diagnostic(base, &diagnostic, false),
        "content: error: Not a .mlg file"
    );
}

#[test]
fn formats_line_only_diagnostics() {
    let base = Path::new("/repo");
    let diagnostic = Diagnostic::warning(1, "Unused statement");

    assert_eq!(
        format_diagnostic(base, &diagnostic, false),
        "line 2: warning: Unused statement"
    );
}

#[test]
fn adds_ansi_colors_when_enabled() {
    let base = Path::new("/repo");
    let diagnostic = Diagnostic::file_error("/repo/content/example.mlg", 0, "Unexpected token");

    let rendered = format_diagnostic(base, &diagnostic, true);

    assert!(rendered.contains("\x1b[1;36mcontent/example.mlg\x1b[0m"));
    assert!(rendered.contains("\x1b[2m1\x1b[0m"));
    assert!(rendered.contains("\x1b[1;31merror\x1b[0m"));
}

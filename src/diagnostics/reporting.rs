use super::{Diagnostic, Severity};
use std::io::{self, IsTerminal, Write};
use std::path::Path;

pub fn print_diagnostics_to_stderr(base: &Path, diagnostics: &[Diagnostic]) -> io::Result<()> {
    let use_color = std::io::stderr().is_terminal();
    let mut stderr = std::io::stderr().lock();

    for diagnostic in diagnostics {
        writeln!(stderr, "{}", format_diagnostic(base, diagnostic, use_color))?;
    }

    Ok(())
}

pub fn format_diagnostic(base: &Path, diagnostic: &Diagnostic, use_color: bool) -> String {
    let severity = match diagnostic.severity {
        Severity::Error => style_label("error", Style::Red, use_color),
        Severity::Warning => style_label("warning", Style::Yellow, use_color),
    };

    match (&diagnostic.location.path, diagnostic.location.row) {
        (Some(path), Some(row)) => format!(
            "{}:{}: {severity}: {}",
            style_label(&display_path(base, path), Style::Cyan, use_color),
            style_label(&(row + 1).to_string(), Style::Dim, use_color),
            diagnostic.message
        ),
        (Some(path), None) => format!(
            "{}: {severity}: {}",
            style_label(&display_path(base, path), Style::Cyan, use_color),
            diagnostic.message
        ),
        (None, Some(row)) => format!(
            "line {}: {severity}: {}",
            style_label(&(row + 1).to_string(), Style::Dim, use_color),
            diagnostic.message
        ),
        (None, None) => format!("{severity}: {}", diagnostic.message),
    }
}

fn display_path(base: &Path, path: &Path) -> String {
    path.strip_prefix(base)
        .map(|relative| {
            if relative.as_os_str().is_empty() {
                ".".to_string()
            } else {
                relative.display().to_string()
            }
        })
        .unwrap_or_else(|_| path.display().to_string())
}

#[derive(Clone, Copy)]
enum Style {
    Red,
    Yellow,
    Cyan,
    Dim,
}

fn style_label(text: &str, style: Style, use_color: bool) -> String {
    if !use_color {
        return text.to_string();
    }

    let code = match style {
        Style::Red => "1;31",
        Style::Yellow => "1;33",
        Style::Cyan => "1;36",
        Style::Dim => "2",
    };

    format!("\x1b[{code}m{text}\x1b[0m")
}

// =============================================================================

#[cfg(test)]
mod tests {
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
}

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

#[cfg(test)]
mod tests;

use super::data::{Diagnostic, Severity};
use std::io::IsTerminal;
use std::path::Path;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ColorMode {
    #[default]
    Auto,
    Always,
    Never,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct DiagnosticFormatter<'a> {
    base_path: Option<&'a Path>,
    color_mode: ColorMode,
}

impl<'a> DiagnosticFormatter<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_base_path(mut self, base_path: &'a Path) -> Self {
        self.base_path = Some(base_path);
        self
    }

    pub fn with_color_mode(mut self, color_mode: ColorMode) -> Self {
        self.color_mode = color_mode;
        self
    }

    pub fn format(&self, diagnostic: &Diagnostic) -> String {
        self.format_with_color(diagnostic, self.should_use_color())
    }

    pub fn format_all(&self, diagnostics: &[Diagnostic]) -> String {
        let use_color = self.should_use_color();

        diagnostics
            .iter()
            .map(|diagnostic| self.format_with_color(diagnostic, use_color))
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn format_with_color(&self, diagnostic: &Diagnostic, use_color: bool) -> String {
        let severity = match diagnostic.severity {
            Severity::Error => style_label("error", Style::Red, use_color),
            Severity::Warning => style_label("warning", Style::Yellow, use_color),
        };

        match (&diagnostic.location.path, diagnostic.location.row) {
            (Some(path), Some(row)) => format!(
                "{}:{}: {severity}: {}",
                style_label(&self.display_path(path), Style::Cyan, use_color),
                style_label(&(row + 1).to_string(), Style::Dim, use_color),
                diagnostic.message
            ),
            (Some(path), None) => format!(
                "{}: {severity}: {}",
                style_label(&self.display_path(path), Style::Cyan, use_color),
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

    fn display_path(&self, path: &Path) -> String {
        self.base_path
            .and_then(|base| path.strip_prefix(base).ok())
            .map(|relative| {
                if relative.as_os_str().is_empty() {
                    ".".to_string()
                } else {
                    relative.display().to_string()
                }
            })
            .unwrap_or_else(|| path.display().to_string())
    }

    fn should_use_color(&self) -> bool {
        match self.color_mode {
            ColorMode::Auto => std::io::stderr().is_terminal(),
            ColorMode::Always => true,
            ColorMode::Never => false,
        }
    }
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
    use super::{ColorMode, DiagnosticFormatter};
    use crate::diagnostics::{Diagnostic, Location, Severity};
    use std::path::Path;

    #[test]
    fn formats_file_diagnostics_relative_to_the_base_path() {
        let formatter = DiagnosticFormatter::new()
            .with_base_path(Path::new("/repo"))
            .with_color_mode(ColorMode::Never);
        let diagnostic = Diagnostic {
            message: "Unexpected header: [duplicate]".to_string(),
            severity: Severity::Error,
            location: Location::at_path_and_row("/repo/content/example.mlg", 3),
        };

        assert_eq!(
            formatter.format(&diagnostic),
            "content/example.mlg:4: error: Unexpected header: [duplicate]"
        );
    }

    #[test]
    fn formats_path_only_diagnostics() {
        let formatter = DiagnosticFormatter::new()
            .with_base_path(Path::new("/repo"))
            .with_color_mode(ColorMode::Never);
        let diagnostic = Diagnostic::path_error("/repo/content", "Not a .mlg file");

        assert_eq!(
            formatter.format(&diagnostic),
            "content: error: Not a .mlg file"
        );
    }

    #[test]
    fn formats_line_only_diagnostics() {
        let formatter = DiagnosticFormatter::new().with_color_mode(ColorMode::Never);
        let diagnostic = Diagnostic::warning(1, "Unused statement");

        assert_eq!(
            formatter.format(&diagnostic),
            "line 2: warning: Unused statement"
        );
    }

    #[test]
    fn adds_ansi_colors_when_enabled() {
        let formatter = DiagnosticFormatter::new()
            .with_base_path(Path::new("/repo"))
            .with_color_mode(ColorMode::Always);
        let diagnostic = Diagnostic::file_error("/repo/content/example.mlg", 0, "Unexpected token");

        let rendered = formatter.format(&diagnostic);

        assert!(rendered.contains("\x1b[1;36mcontent/example.mlg\x1b[0m"));
        assert!(rendered.contains("\x1b[2m1\x1b[0m"));
        assert!(rendered.contains("\x1b[1;31merror\x1b[0m"));
    }

    #[test]
    fn formats_multiple_diagnostics_with_newline_separators() {
        let formatter = DiagnosticFormatter::new().with_color_mode(ColorMode::Never);
        let diagnostics = [
            Diagnostic::error(0, "Unexpected token"),
            Diagnostic::warning(1, "Unused statement"),
        ];

        assert_eq!(
            formatter.format_all(&diagnostics),
            "line 1: error: Unexpected token\nline 2: warning: Unused statement"
        );
    }
}

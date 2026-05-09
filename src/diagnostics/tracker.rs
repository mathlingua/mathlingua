use super::data::{Diagnostic, Level};
use super::formatter::{ColorMode, DiagnosticFormatter};
use std::io::{self, Write};
use std::path::PathBuf;

#[derive(Debug)]
pub struct DiagnosticTracker {
    diagnostics: Vec<Diagnostic>,
    formatter: DiagnosticFormatter,
    live: bool,
}

impl Default for DiagnosticTracker {
    fn default() -> Self {
        Self {
            diagnostics: Vec::new(),
            formatter: DiagnosticFormatter::new(),
            live: false,
        }
    }
}

impl DiagnosticTracker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_live(mut self, live: bool) -> Self {
        self.live = live;
        self
    }

    pub fn with_base_path(mut self, base_path: impl Into<PathBuf>) -> Self {
        self.formatter.set_base_path(base_path);
        self
    }

    pub fn with_color_mode(mut self, color_mode: ColorMode) -> Self {
        self.formatter.set_color_mode(color_mode);
        self
    }

    pub fn set_live(&mut self, live: bool) {
        self.live = live;
    }

    pub fn set_base_path(&mut self, base_path: impl Into<PathBuf>) {
        self.formatter.set_base_path(base_path);
    }

    pub fn set_color_mode(&mut self, color_mode: ColorMode) {
        self.formatter.set_color_mode(color_mode);
    }

    pub fn push(&mut self, diagnostic: Diagnostic) {
        if self.live {
            let _ = self.write_diagnostic(&diagnostic);
        }

        self.diagnostics.push(diagnostic);
    }

    pub fn error(&mut self, row: usize, message: impl Into<String>) {
        self.push(Diagnostic::error(row, message));
    }

    pub fn warning(&mut self, row: usize, message: impl Into<String>) {
        self.push(Diagnostic::warning(row, message));
    }

    pub fn global_error(&mut self, message: impl Into<String>) {
        self.push(Diagnostic::global_error(message));
    }

    pub fn global_warning(&mut self, message: impl Into<String>) {
        self.push(Diagnostic::global_warning(message));
    }

    pub fn log(&mut self, message: impl Into<String>) {
        self.push(Diagnostic::log(message));
    }

    pub fn path_error(&mut self, path: impl Into<PathBuf>, message: impl Into<String>) {
        self.push(Diagnostic::path_error(path, message));
    }

    pub fn path_warning(&mut self, path: impl Into<PathBuf>, message: impl Into<String>) {
        self.push(Diagnostic::path_warning(path, message));
    }

    pub fn file_error(&mut self, path: impl Into<PathBuf>, row: usize, message: impl Into<String>) {
        self.push(Diagnostic::file_error(path, row, message));
    }

    pub fn file_warning(
        &mut self,
        path: impl Into<PathBuf>,
        row: usize,
        message: impl Into<String>,
    ) {
        self.push(Diagnostic::file_warning(path, row, message));
    }

    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    pub fn issue_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.level != Level::Log)
            .count()
    }

    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|diagnostic| diagnostic.level == Level::Error)
    }

    pub fn print(&self) -> io::Result<()> {
        for diagnostic in &self.diagnostics {
            self.write_diagnostic(diagnostic)?;
        }

        Ok(())
    }

    fn write_diagnostic(&self, diagnostic: &Diagnostic) -> io::Result<()> {
        let rendered = self.formatter.format(diagnostic);

        if diagnostic.level == Level::Log {
            write_line(std::io::stdout().lock(), &rendered)
        } else {
            write_line(std::io::stderr().lock(), &rendered)
        }
    }
}

fn write_line(mut writer: impl Write, message: &str) -> io::Result<()> {
    writer.write_all(message.as_bytes())?;
    writer.write_all(b"\n")?;
    writer.flush()
}

// =============================================================================

#[cfg(test)]
mod tests {
    use super::DiagnosticTracker;
    use crate::diagnostics::{ColorMode, Diagnostic, Level, Location};

    fn sample_diagnostic() -> Diagnostic {
        Diagnostic {
            message: "unexpected token".to_string(),
            level: Level::Error,
            location: Location {
                path: None,
                row: Some(7),
            },
        }
    }

    #[test]
    fn stores_added_diagnostics() {
        let mut tracker = DiagnosticTracker::new();
        let diagnostic = sample_diagnostic();

        tracker.push(diagnostic.clone());

        let mut expected = sample_diagnostic();
        expected.message.push('!');

        assert_eq!(tracker.diagnostics(), [diagnostic]);
        assert_ne!(tracker.diagnostics(), [expected]);
    }

    #[test]
    fn returns_a_slice_of_the_internal_collection() {
        let mut tracker = DiagnosticTracker::new();
        tracker.push(sample_diagnostic());

        let mut diagnostics = tracker.diagnostics().to_vec();
        diagnostics.clear();

        assert_eq!(tracker.diagnostics().len(), 1);
    }

    #[test]
    fn supports_warning_diagnostics() {
        let diagnostic = Diagnostic::warning(3, "unused statement");

        let mut tracker = DiagnosticTracker::new();
        tracker.push(diagnostic.clone());

        assert_eq!(tracker.diagnostics(), [diagnostic]);
    }

    #[test]
    fn reports_errors_only_for_error_diagnostics() {
        let mut tracker = DiagnosticTracker::new();
        tracker.push(Diagnostic::warning(3, "unused statement"));
        tracker.push(Diagnostic::log("Checked 1 file"));

        assert!(!tracker.has_errors());

        tracker.push(Diagnostic::error(4, "unexpected token"));

        assert!(tracker.has_errors());
    }

    #[test]
    fn counts_only_errors_and_warnings_as_issues() {
        let mut tracker = DiagnosticTracker::new();
        tracker.push(Diagnostic::warning(3, "unused statement"));
        tracker.push(Diagnostic::log("Checked 1 file"));
        tracker.push(Diagnostic::error(4, "unexpected token"));

        assert_eq!(tracker.issue_count(), 2);
    }

    #[test]
    fn supports_builder_configuration() {
        let tracker = DiagnosticTracker::new()
            .with_live(true)
            .with_color_mode(ColorMode::Never)
            .with_base_path("/repo");

        assert!(!tracker.has_errors());
        assert_eq!(tracker.issue_count(), 0);
    }
}

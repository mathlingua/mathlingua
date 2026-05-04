use super::data::{Diagnostic, Severity};
use std::path::PathBuf;

#[derive(Debug, Default)]
pub struct DiagnosticTracker {
    diagnostics: Vec<Diagnostic>,
}

impl DiagnosticTracker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, diagnostic: Diagnostic) {
        self.diagnostics.push(diagnostic);
    }

    pub fn error(&mut self, row: usize, message: impl Into<String>) {
        self.push(Diagnostic::error(row, message));
    }

    pub fn warning(&mut self, row: usize, message: impl Into<String>) {
        self.push(Diagnostic::warning(row, message));
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

    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity == Severity::Error)
    }
}

// =============================================================================

#[cfg(test)]
mod tests {
    use super::DiagnosticTracker;
    use crate::diagnostics::{Diagnostic, Location, Severity};

    fn sample_diagnostic() -> Diagnostic {
        Diagnostic {
            message: "unexpected token".to_string(),
            severity: Severity::Error,
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

        assert!(!tracker.has_errors());

        tracker.push(Diagnostic::error(4, "unexpected token"));

        assert!(tracker.has_errors());
    }
}

use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Severity {
    Error,
    #[allow(dead_code)]
    Warning,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Location {
    pub row: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Diagnostic {
    pub message: String,
    pub severity: Severity,
    pub location: Location,
}

impl Diagnostic {
    pub fn new(message: impl Into<String>, severity: Severity, row: usize) -> Self {
        Self {
            message: message.into(),
            severity,
            location: Location { row },
        }
    }

    pub fn error(row: usize, message: impl Into<String>) -> Self {
        Self::new(message, Severity::Error, row)
    }

    #[allow(dead_code)]
    pub fn warning(row: usize, message: impl Into<String>) -> Self {
        Self::new(message, Severity::Warning, row)
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let severity = match self.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
        };

        write!(
            f,
            "{severity} at line {}: {}",
            self.location.row + 1,
            self.message
        )
    }
}

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

    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity == Severity::Error)
    }
}

#[cfg(test)]
mod tests;

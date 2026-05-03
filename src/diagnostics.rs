use std::fmt;
use std::path::PathBuf;

pub mod reporting;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Location {
    pub path: Option<PathBuf>,
    pub row: Option<usize>,
}

impl Location {
    pub fn new(path: Option<PathBuf>, row: Option<usize>) -> Self {
        Self { path, row }
    }

    pub fn at_row(row: usize) -> Self {
        Self::new(None, Some(row))
    }

    pub fn at_path(path: impl Into<PathBuf>) -> Self {
        Self::new(Some(path.into()), None)
    }

    pub fn at_path_and_row(path: impl Into<PathBuf>, row: usize) -> Self {
        Self::new(Some(path.into()), Some(row))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Diagnostic {
    pub message: String,
    pub severity: Severity,
    pub location: Location,
}

impl Diagnostic {
    pub fn new(message: impl Into<String>, severity: Severity, location: Location) -> Self {
        Self {
            message: message.into(),
            severity,
            location,
        }
    }

    pub fn error(row: usize, message: impl Into<String>) -> Self {
        Self::new(message, Severity::Error, Location::at_row(row))
    }

    pub fn warning(row: usize, message: impl Into<String>) -> Self {
        Self::new(message, Severity::Warning, Location::at_row(row))
    }

    pub fn path_error(path: impl Into<PathBuf>, message: impl Into<String>) -> Self {
        Self::new(message, Severity::Error, Location::at_path(path))
    }

    pub fn path_warning(path: impl Into<PathBuf>, message: impl Into<String>) -> Self {
        Self::new(message, Severity::Warning, Location::at_path(path))
    }

    pub fn file_error(path: impl Into<PathBuf>, row: usize, message: impl Into<String>) -> Self {
        Self::new(
            message,
            Severity::Error,
            Location::at_path_and_row(path, row),
        )
    }

    pub fn file_warning(path: impl Into<PathBuf>, row: usize, message: impl Into<String>) -> Self {
        Self::new(
            message,
            Severity::Warning,
            Location::at_path_and_row(path, row),
        )
    }

    pub fn with_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.location.path = Some(path.into());
        self
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let severity = match self.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
        };

        match (&self.location.path, self.location.row) {
            (Some(path), Some(row)) => {
                write!(
                    f,
                    "{severity} at {}:{}: {}",
                    path.display(),
                    row + 1,
                    self.message
                )
            }
            (Some(path), None) => write!(f, "{severity} at {}: {}", path.display(), self.message),
            (None, Some(row)) => write!(f, "{severity} at line {}: {}", row + 1, self.message),
            (None, None) => write!(f, "{severity}: {}", self.message),
        }
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

#[cfg(test)]
mod tests;

use std::fmt;
use std::path::PathBuf;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Level {
    Error,
    Warning,
    Log,
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
    pub level: Level,
    pub location: Location,
}

impl Diagnostic {
    pub fn new(message: impl Into<String>, level: Level, location: Location) -> Self {
        Self {
            message: message.into(),
            level,
            location,
        }
    }

    pub fn error(row: usize, message: impl Into<String>) -> Self {
        Self::new(message, Level::Error, Location::at_row(row))
    }

    pub fn warning(row: usize, message: impl Into<String>) -> Self {
        Self::new(message, Level::Warning, Location::at_row(row))
    }

    pub fn global_error(message: impl Into<String>) -> Self {
        Self::new(message, Level::Error, Location::default())
    }

    pub fn global_warning(message: impl Into<String>) -> Self {
        Self::new(message, Level::Warning, Location::default())
    }

    pub fn log(message: impl Into<String>) -> Self {
        Self::new(message, Level::Log, Location::default())
    }

    pub fn path_error(path: impl Into<PathBuf>, message: impl Into<String>) -> Self {
        Self::new(message, Level::Error, Location::at_path(path))
    }

    pub fn path_warning(path: impl Into<PathBuf>, message: impl Into<String>) -> Self {
        Self::new(message, Level::Warning, Location::at_path(path))
    }

    pub fn file_error(path: impl Into<PathBuf>, row: usize, message: impl Into<String>) -> Self {
        Self::new(message, Level::Error, Location::at_path_and_row(path, row))
    }

    pub fn file_warning(path: impl Into<PathBuf>, row: usize, message: impl Into<String>) -> Self {
        Self::new(
            message,
            Level::Warning,
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
        if self.level == Level::Log {
            return write!(f, "{}", self.message);
        }

        let level = match self.level {
            Level::Error => "error",
            Level::Warning => "warning",
            Level::Log => unreachable!("log diagnostics return early"),
        };

        match (&self.location.path, self.location.row) {
            (Some(path), Some(row)) => {
                write!(
                    f,
                    "{level} at {}:{}: {}",
                    path.display(),
                    row + 1,
                    self.message
                )
            }
            (Some(path), None) => write!(f, "{level} at {}: {}", path.display(), self.message),
            (None, Some(row)) => write!(f, "{level} at line {}: {}", row + 1, self.message),
            (None, None) => write!(f, "{level}: {}", self.message),
        }
    }
}

// =============================================================================

#[cfg(test)]
mod tests {
    use super::Diagnostic;

    #[test]
    fn formats_display_using_one_based_line_numbers() {
        let diagnostic = Diagnostic::error(2, "unexpected token");

        assert_eq!(diagnostic.to_string(), "error at line 3: unexpected token");
    }

    #[test]
    fn supports_file_scoped_diagnostics() {
        let diagnostic = Diagnostic::file_error("content/example.mlg", 4, "unexpected token");

        assert_eq!(
            diagnostic.to_string(),
            "error at content/example.mlg:5: unexpected token"
        );
    }

    #[test]
    fn supports_path_scoped_diagnostics_without_line_numbers() {
        let diagnostic = Diagnostic::path_warning("content", "directory skipped");

        assert_eq!(
            diagnostic.to_string(),
            "warning at content: directory skipped"
        );
    }

    #[test]
    fn renders_log_diagnostics_as_plain_messages() {
        let diagnostic = Diagnostic::log("Created content/");

        assert_eq!(diagnostic.to_string(), "Created content/");
    }
}

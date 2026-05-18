use std::path::PathBuf;

/// Position inside a diagnostic source.
///
/// Positions are zero-based internally.  A position may be row/column based,
/// byte-offset based, or partially specified depending on what a parser can
/// recover.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct EventPosition {
    /// Zero-based row, when available.
    pub row: Option<usize>,
    /// Zero-based column, when available.
    pub column: Option<usize>,
    /// Zero-based byte offset, when available.
    pub offset: Option<usize>,
}

impl EventPosition {
    /// Creates a position from optional row, column, and offset components.
    pub fn new(row: Option<usize>, column: Option<usize>, offset: Option<usize>) -> Self {
        Self {
            row,
            column,
            offset,
        }
    }

    /// Creates a row-only position.
    pub fn at_row(row: usize) -> Self {
        Self::new(Some(row), None, None)
    }

    /// Creates a row-and-column position.
    pub fn at_row_and_column(row: usize, column: usize) -> Self {
        Self::new(Some(row), Some(column), None)
    }

    /// Creates a byte-offset position.
    pub fn at_offset(offset: usize) -> Self {
        Self::new(None, None, Some(offset))
    }
}

/// Span covered by a diagnostic event.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EventSpan {
    /// Start of the span.
    pub start: EventPosition,
    /// Optional end of the span; absent means a point span.
    pub end: Option<EventPosition>,
}

impl EventSpan {
    /// Creates a span from a start position and optional end position.
    pub fn new(start: EventPosition, end: Option<EventPosition>) -> Self {
        Self { start, end }
    }

    /// Creates a point span.
    pub fn point(start: EventPosition) -> Self {
        Self::new(start, None)
    }

    /// Creates a row-only point span.
    pub fn row(row: usize) -> Self {
        Self::point(EventPosition::at_row(row))
    }

    /// Creates a row-and-column point span.
    pub fn row_and_column(row: usize, column: usize) -> Self {
        Self::point(EventPosition::at_row_and_column(row, column))
    }

    /// Creates a byte-offset point span.
    pub fn offset(offset: usize) -> Self {
        Self::point(EventPosition::at_offset(offset))
    }
}

/// Location attached to an event.
///
/// Events can point either at an in-memory snippet or at a filesystem path.
/// File paths may include optional spans for precise diagnostics.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EventLocation {
    /// Location inside an in-memory source with an optional display name.
    InMemory {
        /// Human-readable name for the in-memory source.
        name: Option<String>,
        /// Optional span inside the in-memory source.
        span: Option<EventSpan>,
    },
    /// Location inside a file on disk.
    File {
        /// Filesystem path associated with the diagnostic.
        path: PathBuf,
        /// Optional span inside the file.
        span: Option<EventSpan>,
    },
}

impl EventLocation {
    /// Creates an in-memory location.
    pub fn in_memory(name: Option<impl Into<String>>, span: Option<EventSpan>) -> Self {
        Self::InMemory {
            name: name.map(Into::into),
            span,
        }
    }

    /// Creates a file location.
    pub fn file(path: impl Into<PathBuf>, span: Option<EventSpan>) -> Self {
        Self::File {
            path: path.into(),
            span,
        }
    }

    /// Creates an unnamed in-memory row location.
    pub fn in_memory_row(row: usize) -> Self {
        Self::in_memory(None::<String>, Some(EventSpan::row(row)))
    }

    /// Creates an unnamed in-memory row-and-column location.
    pub fn in_memory_row_and_column(row: usize, column: usize) -> Self {
        Self::in_memory(None::<String>, Some(EventSpan::row_and_column(row, column)))
    }

    /// Creates a file-path-only location.
    pub fn file_path(path: impl Into<PathBuf>) -> Self {
        Self::file(path, None)
    }

    /// Creates a file row location.
    pub fn file_row(path: impl Into<PathBuf>, row: usize) -> Self {
        Self::file(path, Some(EventSpan::row(row)))
    }

    /// Creates a file row-and-column location.
    pub fn file_row_and_column(path: impl Into<PathBuf>, row: usize, column: usize) -> Self {
        Self::file(path, Some(EventSpan::row_and_column(row, column)))
    }

    /// Converts any location into a file location while preserving its span.
    pub fn with_file_path(self, path: impl Into<PathBuf>) -> Self {
        let path = path.into();

        match self {
            Self::InMemory { span, .. } => Self::File { path, span },
            Self::File { span, .. } => Self::File { path, span },
        }
    }
}

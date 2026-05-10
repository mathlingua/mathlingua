use std::path::PathBuf;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct EventPosition {
    pub row: Option<usize>,
    pub column: Option<usize>,
    pub offset: Option<usize>,
}

impl EventPosition {
    pub fn new(row: Option<usize>, column: Option<usize>, offset: Option<usize>) -> Self {
        Self {
            row,
            column,
            offset,
        }
    }

    pub fn at_row(row: usize) -> Self {
        Self::new(Some(row), None, None)
    }

    pub fn at_row_and_column(row: usize, column: usize) -> Self {
        Self::new(Some(row), Some(column), None)
    }

    pub fn at_offset(offset: usize) -> Self {
        Self::new(None, None, Some(offset))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EventSpan {
    pub start: EventPosition,
    pub end: Option<EventPosition>,
}

impl EventSpan {
    pub fn new(start: EventPosition, end: Option<EventPosition>) -> Self {
        Self { start, end }
    }

    pub fn point(start: EventPosition) -> Self {
        Self::new(start, None)
    }

    pub fn row(row: usize) -> Self {
        Self::point(EventPosition::at_row(row))
    }

    pub fn row_and_column(row: usize, column: usize) -> Self {
        Self::point(EventPosition::at_row_and_column(row, column))
    }

    pub fn offset(offset: usize) -> Self {
        Self::point(EventPosition::at_offset(offset))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EventLocation {
    InMemory {
        name: Option<String>,
        span: Option<EventSpan>,
    },
    File {
        path: PathBuf,
        span: Option<EventSpan>,
    },
}

impl EventLocation {
    pub fn in_memory(name: Option<impl Into<String>>, span: Option<EventSpan>) -> Self {
        Self::InMemory {
            name: name.map(Into::into),
            span,
        }
    }

    pub fn file(path: impl Into<PathBuf>, span: Option<EventSpan>) -> Self {
        Self::File {
            path: path.into(),
            span,
        }
    }

    pub fn in_memory_row(row: usize) -> Self {
        Self::in_memory(None::<String>, Some(EventSpan::row(row)))
    }

    pub fn in_memory_row_and_column(row: usize, column: usize) -> Self {
        Self::in_memory(None::<String>, Some(EventSpan::row_and_column(row, column)))
    }

    pub fn file_path(path: impl Into<PathBuf>) -> Self {
        Self::file(path, None)
    }

    pub fn file_row(path: impl Into<PathBuf>, row: usize) -> Self {
        Self::file(path, Some(EventSpan::row(row)))
    }

    pub fn file_row_and_column(path: impl Into<PathBuf>, row: usize, column: usize) -> Self {
        Self::file(path, Some(EventSpan::row_and_column(row, column)))
    }

    pub fn with_file_path(self, path: impl Into<PathBuf>) -> Self {
        let path = path.into();

        match self {
            Self::InMemory { span, .. } => Self::File { path, span },
            Self::File { span, .. } => Self::File { path, span },
        }
    }
}

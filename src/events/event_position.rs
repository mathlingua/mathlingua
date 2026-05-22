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

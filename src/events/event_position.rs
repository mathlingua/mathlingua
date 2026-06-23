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

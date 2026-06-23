use super::EventPosition;

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

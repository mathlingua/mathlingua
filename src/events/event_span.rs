use super::EventPosition;

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

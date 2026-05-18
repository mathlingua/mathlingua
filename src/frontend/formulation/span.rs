/// Byte span of a parsed formulation token or AST node.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Span {
    /// Inclusive start byte offset.
    pub start: usize,
    /// Exclusive end byte offset.
    pub end: usize,
}

impl Span {
    /// Creates a span from start and end byte offsets.
    pub const fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }
}

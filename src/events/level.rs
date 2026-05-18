/// Severity or purpose of an event message.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Level {
    /// Informational message.
    Log,
    /// Non-fatal warning.
    Warning,
    /// Error that should generally cause a failing command.
    Error,
    /// Debugging detail.
    Debug,
}

impl Level {
    /// Returns all message levels in stable display/filter order.
    pub const fn all() -> [Self; 4] {
        [Self::Log, Self::Warning, Self::Error, Self::Debug]
    }
}

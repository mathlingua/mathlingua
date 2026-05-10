#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Level {
    Log,
    Warning,
    Error,
    Debug,
}

impl Level {
    pub const fn all() -> [Self; 4] {
        [Self::Log, Self::Warning, Self::Error, Self::Debug]
    }
}

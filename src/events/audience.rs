/// Intended audience for an event emitted by the application.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Audience {
    /// User-facing output suitable for normal command-line display.
    User,
    /// Internal output useful for debugging or instrumentation.
    System,
}

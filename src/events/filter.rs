use super::{Audience, Event, Level};

/// Controls ANSI color rendering for console diagnostics.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ColorMode {
    /// Use color only when the output stream is a terminal.
    #[default]
    Auto,
    /// Always emit ANSI color codes.
    Always,
    /// Never emit ANSI color codes.
    Never,
}

/// Determines which events should be rendered by an event listener.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EventFilter {
    /// Audiences allowed through the filter.
    audiences: Vec<Audience>,
    /// Message levels allowed through the filter.
    levels: Vec<Level>,
    /// Whether marker events are included.
    include_markers: bool,
}

impl Default for EventFilter {
    /// Creates the default user-facing filter.
    ///
    /// By default all message levels are shown, only user-audience messages are
    /// shown, and instrumentation markers are hidden.
    fn default() -> Self {
        Self {
            audiences: vec![Audience::User],
            levels: Level::all().to_vec(),
            include_markers: false,
        }
    }
}

impl EventFilter {
    /// Creates a new event filter with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Replaces the allowed audiences.
    pub fn with_audiences(mut self, audiences: Vec<Audience>) -> Self {
        self.audiences = audiences;
        self
    }

    /// Replaces the allowed message levels.
    pub fn with_levels(mut self, levels: Vec<Level>) -> Self {
        self.levels = levels;
        self
    }

    /// Configures whether marker events should pass through the filter.
    pub fn include_markers(mut self, include_markers: bool) -> Self {
        self.include_markers = include_markers;
        self
    }

    /// Returns true when an event should be rendered by a listener.
    pub(crate) fn matches(&self, event: &Event) -> bool {
        match event {
            Event::Message(message) => {
                self.audiences.contains(&message.audience) && self.levels.contains(&message.level)
            }
            Event::Marker(_) => self.include_markers,
        }
    }
}

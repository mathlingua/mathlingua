use std::path::PathBuf;

use super::{Audience, EventLocation, Level};

/// User- or system-facing message event.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MessageEvent {
    /// Human-readable message body.
    pub message: String,
    /// Optional source location associated with the message.
    pub location: Option<EventLocation>,
    /// Message severity/purpose.
    pub level: Level,
    /// Intended audience for the message.
    pub audience: Audience,
    /// Optional subsystem that emitted the message.
    pub origin: Option<String>,
}

impl MessageEvent {
    /// Creates a message event from all of its components.
    pub fn new(
        message: impl Into<String>,
        level: Level,
        audience: Audience,
        location: Option<EventLocation>,
        origin: Option<String>,
    ) -> Self {
        Self {
            message: message.into(),
            location,
            level,
            audience,
            origin,
        }
    }

    /// Returns a copy of this event with an origin set.
    pub fn with_origin(mut self, origin: impl Into<String>) -> Self {
        self.origin = Some(origin.into());
        self
    }

    /// Returns a copy of this event with an optional origin set.
    pub fn with_origin_option(mut self, origin: Option<&str>) -> Self {
        self.origin = origin.map(str::to_owned);
        self
    }

    /// Returns a copy of this event associated with a file path.
    ///
    /// In-memory locations are converted to file locations while preserving their
    /// span.  Existing file locations keep their span and replace the path.
    pub fn with_file_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.location = Some(match self.location.take() {
            Some(location) => location.with_file_path(path),
            None => EventLocation::file_path(path),
        });
        self
    }
}

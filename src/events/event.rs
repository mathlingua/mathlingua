use std::path::PathBuf;

use super::{EventLocation, Level, MarkerEvent, MarkerId, MarkerPhase, MessageEvent};

/// Intended audience for an event emitted by the application.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Audience {
    /// User-facing output suitable for normal command-line display.
    User,
    /// Internal output useful for debugging or instrumentation.
    System,
}

/// Event emitted by MathLingua subsystems.
///
/// Most events are user/system messages.  Marker events provide lightweight
/// instrumentation and can be used to slice the log around a single operation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Event {
    /// A diagnostic or informational message.
    Message(MessageEvent),
    /// Begin/end marker used for event-log ranges.
    Marker(MarkerEvent),
}

impl Event {
    /// Creates a message event from explicit message metadata.
    pub fn message(
        message: impl Into<String>,
        level: Level,
        audience: Audience,
        location: Option<EventLocation>,
    ) -> Self {
        Self::Message(MessageEvent::new(message, level, audience, location, None))
    }

    /// Creates a user-facing informational message.
    pub fn user_log(message: impl Into<String>) -> Self {
        Self::message(message, Level::Log, Audience::User, None)
    }

    /// Creates a user-facing warning message.
    pub fn user_warning(message: impl Into<String>) -> Self {
        Self::message(message, Level::Warning, Audience::User, None)
    }

    /// Creates a user-facing error message.
    pub fn user_error(message: impl Into<String>) -> Self {
        Self::message(message, Level::Error, Audience::User, None)
    }

    /// Creates a user-facing debug message.
    pub fn user_debug(message: impl Into<String>) -> Self {
        Self::message(message, Level::Debug, Audience::User, None)
    }

    /// Creates a user warning located at an in-memory row.
    pub fn user_warning_at_row(row: usize, message: impl Into<String>) -> Self {
        Self::message(
            message,
            Level::Warning,
            Audience::User,
            Some(EventLocation::in_memory_row(row)),
        )
    }

    /// Creates a user error located at an in-memory row.
    pub fn user_error_at_row(row: usize, message: impl Into<String>) -> Self {
        Self::message(
            message,
            Level::Error,
            Audience::User,
            Some(EventLocation::in_memory_row(row)),
        )
    }

    /// Creates a user warning associated with a file path.
    pub fn user_path_warning(path: impl Into<PathBuf>, message: impl Into<String>) -> Self {
        Self::message(
            message,
            Level::Warning,
            Audience::User,
            Some(EventLocation::file_path(path)),
        )
    }

    /// Creates a user error associated with a file path.
    pub fn user_path_error(path: impl Into<PathBuf>, message: impl Into<String>) -> Self {
        Self::message(
            message,
            Level::Error,
            Audience::User,
            Some(EventLocation::file_path(path)),
        )
    }

    /// Creates a user warning associated with a file row.
    pub fn user_file_warning(
        path: impl Into<PathBuf>,
        row: usize,
        message: impl Into<String>,
    ) -> Self {
        Self::message(
            message,
            Level::Warning,
            Audience::User,
            Some(EventLocation::file_row(path, row)),
        )
    }

    /// Creates a user error associated with a file row.
    pub fn user_file_error(
        path: impl Into<PathBuf>,
        row: usize,
        message: impl Into<String>,
    ) -> Self {
        Self::message(
            message,
            Level::Error,
            Audience::User,
            Some(EventLocation::file_row(path, row)),
        )
    }

    /// Creates a system informational message.
    pub fn system_log(message: impl Into<String>) -> Self {
        Self::message(message, Level::Log, Audience::System, None)
    }

    /// Creates a system warning message.
    pub fn system_warning(message: impl Into<String>) -> Self {
        Self::message(message, Level::Warning, Audience::System, None)
    }

    /// Creates a system error message.
    pub fn system_error(message: impl Into<String>) -> Self {
        Self::message(message, Level::Error, Audience::System, None)
    }

    /// Creates a system debug message.
    pub fn system_debug(message: impl Into<String>) -> Self {
        Self::message(message, Level::Debug, Audience::System, None)
    }

    /// Creates a marker event.
    pub fn marker(
        id: MarkerId,
        label: impl Into<String>,
        phase: MarkerPhase,
        origin: Option<&str>,
    ) -> Self {
        Self::Marker(MarkerEvent::new(
            id,
            label,
            phase,
            origin.map(str::to_owned),
        ))
    }

    /// Returns this event with an origin set.
    pub fn with_origin(mut self, origin: impl Into<String>) -> Self {
        let origin = origin.into();

        match &mut self {
            Self::Message(event) => event.origin = Some(origin.clone()),
            Self::Marker(event) => event.origin = Some(origin),
        }

        self
    }

    /// Returns this event with an optional origin set.
    pub fn with_origin_option(mut self, origin: Option<&str>) -> Self {
        if let Some(origin) = origin {
            self = self.with_origin(origin);
        }

        self
    }

    /// Returns this event associated with a file path when it is a message.
    pub fn with_file_path(mut self, path: impl Into<PathBuf>) -> Self {
        if let Self::Message(event) = &mut self {
            *event = event.clone().with_file_path(path);
        }

        self
    }

    /// Returns the message payload when this event is a message.
    pub fn as_message(&self) -> Option<&MessageEvent> {
        match self {
            Self::Message(event) => Some(event),
            Self::Marker(_) => None,
        }
    }

    /// Returns the marker payload when this event is a marker.
    pub fn as_marker(&self) -> Option<&MarkerEvent> {
        match self {
            Self::Marker(event) => Some(event),
            Self::Message(_) => None,
        }
    }
}

// ===============================[ tests ]=====================================

#[cfg(test)]
mod tests {
    use super::Event;
    use crate::events::{Audience, EventLocation, EventPosition, EventSpan, Level};

    #[test]
    fn attaches_a_file_path_to_in_memory_events() {
        let event = Event::user_error_at_row(2, "unexpected token")
            .with_origin("proto_parser")
            .with_file_path("content/example.mlg");

        let message = event.as_message().expect("expected message event");
        assert_eq!(message.origin.as_deref(), Some("proto_parser"));
        assert_eq!(
            message.location,
            Some(EventLocation::file_row("content/example.mlg", 2))
        );
    }

    #[test]
    fn preserves_explicit_locations() {
        let event = Event::message(
            "Bad token",
            Level::Error,
            Audience::User,
            Some(EventLocation::InMemory {
                name: Some("snippet".to_string()),
                span: Some(EventSpan::new(
                    EventPosition::at_row_and_column(3, 4),
                    Some(EventPosition::at_row_and_column(3, 7)),
                )),
            }),
        );

        let message = event.as_message().expect("expected message event");
        assert_eq!(
            message.location.as_ref().unwrap(),
            &EventLocation::InMemory {
                name: Some("snippet".to_string()),
                span: Some(EventSpan::new(
                    EventPosition::at_row_and_column(3, 4),
                    Some(EventPosition::at_row_and_column(3, 7)),
                )),
            }
        );
    }
}

use std::path::PathBuf;

use super::{EventLocation, Level, MarkerEvent, MarkerId, MarkerPhase, MessageEvent};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Audience {
    User,
    System,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Event {
    Message(MessageEvent),
    Marker(MarkerEvent),
}

impl Event {
    pub fn message(
        message: impl Into<String>,
        level: Level,
        audience: Audience,
        location: Option<EventLocation>,
    ) -> Self {
        Self::Message(MessageEvent::new(message, level, audience, location, None))
    }

    pub fn user_log(message: impl Into<String>) -> Self {
        Self::message(message, Level::Log, Audience::User, None)
    }

    pub fn user_warning(message: impl Into<String>) -> Self {
        Self::message(message, Level::Warning, Audience::User, None)
    }

    pub fn user_error(message: impl Into<String>) -> Self {
        Self::message(message, Level::Error, Audience::User, None)
    }

    pub fn user_debug(message: impl Into<String>) -> Self {
        Self::message(message, Level::Debug, Audience::User, None)
    }

    pub fn user_warning_at_row(row: usize, message: impl Into<String>) -> Self {
        Self::message(
            message,
            Level::Warning,
            Audience::User,
            Some(EventLocation::in_memory_row(row)),
        )
    }

    pub fn user_error_at_row(row: usize, message: impl Into<String>) -> Self {
        Self::message(
            message,
            Level::Error,
            Audience::User,
            Some(EventLocation::in_memory_row(row)),
        )
    }

    pub fn user_path_warning(path: impl Into<PathBuf>, message: impl Into<String>) -> Self {
        Self::message(
            message,
            Level::Warning,
            Audience::User,
            Some(EventLocation::file_path(path)),
        )
    }

    pub fn user_path_error(path: impl Into<PathBuf>, message: impl Into<String>) -> Self {
        Self::message(
            message,
            Level::Error,
            Audience::User,
            Some(EventLocation::file_path(path)),
        )
    }

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

    pub fn system_log(message: impl Into<String>) -> Self {
        Self::message(message, Level::Log, Audience::System, None)
    }

    pub fn system_warning(message: impl Into<String>) -> Self {
        Self::message(message, Level::Warning, Audience::System, None)
    }

    pub fn system_error(message: impl Into<String>) -> Self {
        Self::message(message, Level::Error, Audience::System, None)
    }

    pub fn system_debug(message: impl Into<String>) -> Self {
        Self::message(message, Level::Debug, Audience::System, None)
    }

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

    pub fn with_origin(mut self, origin: impl Into<String>) -> Self {
        let origin = origin.into();

        match &mut self {
            Self::Message(event) => event.origin = Some(origin.clone()),
            Self::Marker(event) => event.origin = Some(origin),
        }

        self
    }

    pub fn with_origin_option(mut self, origin: Option<&str>) -> Self {
        if let Some(origin) = origin {
            self = self.with_origin(origin);
        }

        self
    }

    pub fn with_file_path(mut self, path: impl Into<PathBuf>) -> Self {
        if let Self::Message(event) = &mut self {
            *event = event.clone().with_file_path(path);
        }

        self
    }

    pub fn as_message(&self) -> Option<&MessageEvent> {
        match self {
            Self::Message(event) => Some(event),
            Self::Marker(_) => None,
        }
    }

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

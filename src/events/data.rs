use std::fmt;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static NEXT_MARKER_ID: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EventAudience {
    User,
    System,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EventLevel {
    Log,
    Warning,
    Error,
    Debug,
}

impl EventLevel {
    pub const fn all() -> [Self; 4] {
        [Self::Log, Self::Warning, Self::Error, Self::Debug]
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct EventPosition {
    pub row: Option<usize>,
    pub column: Option<usize>,
    pub offset: Option<usize>,
}

impl EventPosition {
    pub fn new(row: Option<usize>, column: Option<usize>, offset: Option<usize>) -> Self {
        Self {
            row,
            column,
            offset,
        }
    }

    pub fn at_row(row: usize) -> Self {
        Self::new(Some(row), None, None)
    }

    pub fn at_row_and_column(row: usize, column: usize) -> Self {
        Self::new(Some(row), Some(column), None)
    }

    pub fn at_offset(offset: usize) -> Self {
        Self::new(None, None, Some(offset))
    }
}

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

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EventLocation {
    InMemory {
        name: Option<String>,
        span: Option<EventSpan>,
    },
    File {
        path: PathBuf,
        span: Option<EventSpan>,
    },
}

impl EventLocation {
    pub fn in_memory(name: Option<impl Into<String>>, span: Option<EventSpan>) -> Self {
        Self::InMemory {
            name: name.map(Into::into),
            span,
        }
    }

    pub fn file(path: impl Into<PathBuf>, span: Option<EventSpan>) -> Self {
        Self::File {
            path: path.into(),
            span,
        }
    }

    pub fn in_memory_row(row: usize) -> Self {
        Self::in_memory(None::<String>, Some(EventSpan::row(row)))
    }

    pub fn in_memory_row_and_column(row: usize, column: usize) -> Self {
        Self::in_memory(None::<String>, Some(EventSpan::row_and_column(row, column)))
    }

    pub fn file_path(path: impl Into<PathBuf>) -> Self {
        Self::file(path, None)
    }

    pub fn file_row(path: impl Into<PathBuf>, row: usize) -> Self {
        Self::file(path, Some(EventSpan::row(row)))
    }

    pub fn file_row_and_column(path: impl Into<PathBuf>, row: usize, column: usize) -> Self {
        Self::file(path, Some(EventSpan::row_and_column(row, column)))
    }

    pub fn with_file_path(self, path: impl Into<PathBuf>) -> Self {
        let path = path.into();

        match self {
            Self::InMemory { span, .. } => Self::File { path, span },
            Self::File { span, .. } => Self::File { path, span },
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct EventMarkerId(String);

impl EventMarkerId {
    pub fn new() -> Self {
        let mut bytes = [0u8; 16];
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let counter = NEXT_MARKER_ID.fetch_add(1, Ordering::Relaxed) as u128;
        let raw = now ^ counter.rotate_left(17);

        bytes.copy_from_slice(&raw.to_le_bytes());
        bytes[6] = (bytes[6] & 0x0f) | 0x40;
        bytes[8] = (bytes[8] & 0x3f) | 0x80;

        Self(format_uuid(bytes))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for EventMarkerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EventMarkerPhase {
    Begin,
    End,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MessageEvent {
    pub message: String,
    pub location: Option<EventLocation>,
    pub level: EventLevel,
    pub audience: EventAudience,
    pub origin: Option<String>,
}

impl MessageEvent {
    pub fn new(
        message: impl Into<String>,
        level: EventLevel,
        audience: EventAudience,
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

    pub fn with_origin(mut self, origin: impl Into<String>) -> Self {
        self.origin = Some(origin.into());
        self
    }

    pub fn with_origin_option(mut self, origin: Option<&str>) -> Self {
        self.origin = origin.map(str::to_owned);
        self
    }

    pub fn with_file_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.location = Some(match self.location.take() {
            Some(location) => location.with_file_path(path),
            None => EventLocation::file_path(path),
        });
        self
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MarkerEvent {
    pub id: EventMarkerId,
    pub label: String,
    pub phase: EventMarkerPhase,
    pub origin: Option<String>,
}

impl MarkerEvent {
    pub fn new(
        id: EventMarkerId,
        label: impl Into<String>,
        phase: EventMarkerPhase,
        origin: Option<String>,
    ) -> Self {
        Self {
            id,
            label: label.into(),
            phase,
            origin,
        }
    }

    pub fn with_origin_option(mut self, origin: Option<&str>) -> Self {
        self.origin = origin.map(str::to_owned);
        self
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Event {
    Message(MessageEvent),
    Marker(MarkerEvent),
}

impl Event {
    pub fn message(
        message: impl Into<String>,
        level: EventLevel,
        audience: EventAudience,
        location: Option<EventLocation>,
    ) -> Self {
        Self::Message(MessageEvent::new(message, level, audience, location, None))
    }

    pub fn user_log(message: impl Into<String>) -> Self {
        Self::message(message, EventLevel::Log, EventAudience::User, None)
    }

    pub fn user_warning(message: impl Into<String>) -> Self {
        Self::message(message, EventLevel::Warning, EventAudience::User, None)
    }

    pub fn user_error(message: impl Into<String>) -> Self {
        Self::message(message, EventLevel::Error, EventAudience::User, None)
    }

    pub fn user_debug(message: impl Into<String>) -> Self {
        Self::message(message, EventLevel::Debug, EventAudience::User, None)
    }

    pub fn user_warning_at_row(row: usize, message: impl Into<String>) -> Self {
        Self::message(
            message,
            EventLevel::Warning,
            EventAudience::User,
            Some(EventLocation::in_memory_row(row)),
        )
    }

    pub fn user_error_at_row(row: usize, message: impl Into<String>) -> Self {
        Self::message(
            message,
            EventLevel::Error,
            EventAudience::User,
            Some(EventLocation::in_memory_row(row)),
        )
    }

    pub fn user_path_warning(path: impl Into<PathBuf>, message: impl Into<String>) -> Self {
        Self::message(
            message,
            EventLevel::Warning,
            EventAudience::User,
            Some(EventLocation::file_path(path)),
        )
    }

    pub fn user_path_error(path: impl Into<PathBuf>, message: impl Into<String>) -> Self {
        Self::message(
            message,
            EventLevel::Error,
            EventAudience::User,
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
            EventLevel::Warning,
            EventAudience::User,
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
            EventLevel::Error,
            EventAudience::User,
            Some(EventLocation::file_row(path, row)),
        )
    }

    pub fn system_log(message: impl Into<String>) -> Self {
        Self::message(message, EventLevel::Log, EventAudience::System, None)
    }

    pub fn system_warning(message: impl Into<String>) -> Self {
        Self::message(message, EventLevel::Warning, EventAudience::System, None)
    }

    pub fn system_error(message: impl Into<String>) -> Self {
        Self::message(message, EventLevel::Error, EventAudience::System, None)
    }

    pub fn system_debug(message: impl Into<String>) -> Self {
        Self::message(message, EventLevel::Debug, EventAudience::System, None)
    }

    pub fn marker(
        id: EventMarkerId,
        label: impl Into<String>,
        phase: EventMarkerPhase,
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
        match &mut self {
            Self::Message(event) => event.origin = Some(origin.into()),
            Self::Marker(event) => event.origin = Some(origin.into()),
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EventRange {
    pub begin: MarkerEvent,
    pub end: MarkerEvent,
}

impl EventRange {
    pub fn new(begin: MarkerEvent, end: MarkerEvent) -> Self {
        Self { begin, end }
    }
}

fn format_uuid(bytes: [u8; 16]) -> String {
    let hex = bytes
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<Vec<_>>();

    format!(
        "{}{}{}{}-{}{}-{}{}-{}{}-{}{}{}{}{}{}",
        hex[0],
        hex[1],
        hex[2],
        hex[3],
        hex[4],
        hex[5],
        hex[6],
        hex[7],
        hex[8],
        hex[9],
        hex[10],
        hex[11],
        hex[12],
        hex[13],
        hex[14],
        hex[15]
    )
}

// =============================================================================

#[cfg(test)]
mod tests {
    use super::{
        Event, EventAudience, EventLevel, EventLocation, EventMarkerId, EventMarkerPhase,
        EventPosition, EventSpan,
    };

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
    fn creates_uuid_shaped_marker_ids() {
        let id = EventMarkerId::new().to_string();

        assert_eq!(id.len(), 36);
        assert_eq!(&id[8..9], "-");
        assert_eq!(&id[13..14], "-");
        assert_eq!(&id[18..19], "-");
        assert_eq!(&id[23..24], "-");
    }

    #[test]
    fn stores_marker_metadata() {
        let marker = Event::marker(
            EventMarkerId::new(),
            "parse_document",
            EventMarkerPhase::Begin,
            Some("structural_parser"),
        );
        let marker = marker.as_marker().expect("expected marker event");

        assert_eq!(marker.label, "parse_document");
        assert_eq!(marker.phase, EventMarkerPhase::Begin);
        assert_eq!(marker.origin.as_deref(), Some("structural_parser"));
    }

    #[test]
    fn preserves_explicit_locations() {
        let event = Event::message(
            "Bad token",
            EventLevel::Error,
            EventAudience::User,
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

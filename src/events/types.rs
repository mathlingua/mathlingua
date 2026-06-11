use std::fmt;
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Audience {
    User,
    System,
}

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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MarkerRange {
    pub begin: MarkerEvent,
    pub end: MarkerEvent,
}

impl MarkerRange {
    pub fn new(begin: MarkerEvent, end: MarkerEvent) -> Self {
        Self { begin, end }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MarkerPhase {
    Begin,
    End,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MarkerEvent {
    pub id: MarkerId,
    pub label: String,
    pub phase: MarkerPhase,
    pub origin: Option<String>,
}

impl MarkerEvent {
    pub fn new(
        id: MarkerId,
        label: impl Into<String>,
        phase: MarkerPhase,
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

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct MarkerId(String);

impl MarkerId {
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for MarkerId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for MarkerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MessageEvent {
    pub message: String,
    pub location: Option<EventLocation>,
    pub level: Level,
    pub audience: Audience,
    pub origin: Option<String>,
}

impl MessageEvent {
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
    use crate::events::{
        Audience, Event, EventLocation, EventPosition, EventSpan, Level, MarkerEvent, MarkerId,
        MarkerPhase, MarkerRange,
    };
    use std::collections::HashSet;

    fn begin_end_pair(label: &str) -> (MarkerEvent, MarkerEvent) {
        let id = MarkerId::new();
        let begin = MarkerEvent::new(id.clone(), label, MarkerPhase::Begin, None);
        let end = MarkerEvent::new(id, label, MarkerPhase::End, None);
        (begin, end)
    }

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

    #[test]
    fn stores_marker_metadata() {
        let id = MarkerId::new();
        let marker = MarkerEvent::new(
            id.clone(),
            "parse_document",
            MarkerPhase::Begin,
            Some("structural_parser".to_string()),
        );

        assert_eq!(marker.id, id);
        assert_eq!(marker.label, "parse_document");
        assert_eq!(marker.phase, MarkerPhase::Begin);
        assert_eq!(marker.origin.as_deref(), Some("structural_parser"));
    }

    #[test]
    fn accepts_label_as_string_slice_or_owned() {
        let from_slice = MarkerEvent::new(MarkerId::new(), "scan", MarkerPhase::End, None);
        let from_owned =
            MarkerEvent::new(MarkerId::new(), "scan".to_string(), MarkerPhase::End, None);

        assert_eq!(from_slice.label, from_owned.label);
    }

    #[test]
    fn with_origin_option_replaces_existing_origin() {
        let marker = MarkerEvent::new(
            MarkerId::new(),
            "typecheck",
            MarkerPhase::Begin,
            Some("old".to_string()),
        )
        .with_origin_option(Some("new"));

        assert_eq!(marker.origin.as_deref(), Some("new"));
    }

    #[test]
    fn with_origin_option_clears_origin_when_none() {
        let marker = MarkerEvent::new(
            MarkerId::new(),
            "typecheck",
            MarkerPhase::End,
            Some("old".to_string()),
        )
        .with_origin_option(None);

        assert!(marker.origin.is_none());
    }

    #[test]
    fn phases_are_distinct() {
        assert_ne!(MarkerPhase::Begin, MarkerPhase::End);
    }

    #[test]
    fn cloned_marker_events_compare_equal() {
        let marker = MarkerEvent::new(MarkerId::new(), "load", MarkerPhase::Begin, None);

        assert_eq!(marker.clone(), marker);
    }

    #[test]
    fn renders_a_uuid_shaped_string() {
        let id = MarkerId::new().to_string();

        assert_eq!(id.len(), 36);
        assert_eq!(&id[8..9], "-");
        assert_eq!(&id[13..14], "-");
        assert_eq!(&id[18..19], "-");
        assert_eq!(&id[23..24], "-");
    }

    #[test]
    fn sets_uuid_v4_version_and_variant_bits() {
        let id = MarkerId::new().to_string();

        // Version nibble at byte index 6 → string index 14.
        assert_eq!(&id[14..15], "4");
        // Variant nibble at byte index 8 → string index 19 ∈ {8, 9, a, b}.
        let variant = id.as_bytes()[19] as char;
        assert!(
            matches!(variant, '8' | '9' | 'a' | 'b'),
            "unexpected variant nibble: {variant}"
        );
    }

    #[test]
    fn as_str_and_display_agree() {
        let id = MarkerId::new();

        assert_eq!(id.as_str(), id.to_string());
    }

    #[test]
    fn default_produces_a_fresh_id() {
        let a = MarkerId::default();
        let b = MarkerId::default();

        assert_ne!(a, b);
    }

    #[test]
    fn successive_ids_are_unique() {
        let ids: HashSet<_> = (0..256).map(|_| MarkerId::new()).collect();

        assert_eq!(ids.len(), 256);
    }

    #[test]
    fn cloned_ids_compare_equal() {
        let id = MarkerId::new();

        assert_eq!(id.clone(), id);
    }

    #[test]
    fn stores_begin_and_end_events() {
        let (begin, end) = begin_end_pair("parse");
        let range = MarkerRange::new(begin.clone(), end.clone());

        assert_eq!(range.begin, begin);
        assert_eq!(range.end, end);
    }

    #[test]
    fn matching_pairs_share_marker_ids() {
        let (begin, end) = begin_end_pair("parse");
        let range = MarkerRange::new(begin, end);

        assert_eq!(range.begin.id, range.end.id);
        assert_eq!(range.begin.phase, MarkerPhase::Begin);
        assert_eq!(range.end.phase, MarkerPhase::End);
    }

    #[test]
    fn ranges_with_same_events_compare_equal() {
        let (begin, end) = begin_end_pair("scan");
        let a = MarkerRange::new(begin.clone(), end.clone());
        let b = MarkerRange::new(begin, end);

        assert_eq!(a, b);
    }

    #[test]
    fn ranges_with_different_ids_compare_unequal() {
        let (begin_a, end_a) = begin_end_pair("scan");
        let (begin_b, end_b) = begin_end_pair("scan");

        assert_ne!(
            MarkerRange::new(begin_a, end_a),
            MarkerRange::new(begin_b, end_b)
        );
    }
}

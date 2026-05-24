use super::{
    Audience, Event, EventLocation, Level, MarkerEvent, MarkerId, MarkerPhase, MarkerRange,
};
use std::path::PathBuf;

/// Listener that receives events as they are pushed into an `EventLog`.
pub trait EventLogListener {
    /// Handles one newly emitted event.
    fn on_event(&mut self, event: &Event);
}

/// Append-only event log with optional live listeners.
///
/// The log is the central diagnostic bus for CLI commands.  Callers can inspect
/// accumulated events after an operation, while listeners can render events as
/// they are emitted.
#[derive(Default)]
pub struct EventLog {
    /// Events emitted so far, in order.
    events: Vec<Event>,
    /// Live listeners notified whenever a new event is pushed.
    listeners: Vec<Box<dyn EventLogListener>>,
}

impl EventLog {
    /// Creates an empty event log.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a listener and replays already-emitted events to it.
    pub fn add_listener(&mut self, mut listener: impl EventLogListener + 'static) {
        for event in &self.events {
            listener.on_event(event);
        }

        self.listeners.push(Box::new(listener));
    }

    /// Adds a pre-boxed listener and replays already-emitted events to it.
    pub fn add_boxed_listener(&mut self, mut listener: Box<dyn EventLogListener>) {
        for event in &self.events {
            listener.on_event(event);
        }

        self.listeners.push(listener);
    }

    /// Appends an event and notifies all listeners.
    pub fn push(&mut self, event: Event) {
        self.events.push(event.clone());

        for listener in &mut self.listeners {
            listener.on_event(&event);
        }
    }

    /// Returns all events emitted so far.
    pub fn events(&self) -> &[Event] {
        &self.events
    }

    /// Returns true when any emitted message is an error.
    pub fn has_errors(&self) -> bool {
        self.events.iter().any(|event| {
            event
                .as_message()
                .is_some_and(|message| message.level == Level::Error)
        })
    }

    /// Counts user-facing non-log messages.
    ///
    /// This is the issue count shown by `mlg check`; system diagnostics and plain
    /// informational logs are intentionally excluded.
    pub fn issue_count(&self) -> usize {
        self.events
            .iter()
            .filter_map(Event::as_message)
            .filter(|message| message.audience == Audience::User && message.level != Level::Log)
            .count()
    }

    /// Emits a user-facing informational message.
    pub fn user_log(&mut self, origin: Option<&str>, message: impl Into<String>) {
        self.push(Event::user_log(message).with_origin_option(origin));
    }

    /// Emits a user-facing warning message.
    pub fn user_warning(&mut self, origin: Option<&str>, message: impl Into<String>) {
        self.push(Event::user_warning(message).with_origin_option(origin));
    }

    /// Emits a user-facing error message.
    pub fn user_error(&mut self, origin: Option<&str>, message: impl Into<String>) {
        self.push(Event::user_error(message).with_origin_option(origin));
    }

    /// Emits a user-facing debug message.
    pub fn user_debug(&mut self, origin: Option<&str>, message: impl Into<String>) {
        self.push(Event::user_debug(message).with_origin_option(origin));
    }

    /// Emits a user-facing warning at an in-memory row.
    pub fn user_warning_at_row(
        &mut self,
        origin: Option<&str>,
        row: usize,
        message: impl Into<String>,
    ) {
        self.push(Event::user_warning_at_row(row, message).with_origin_option(origin));
    }

    /// Emits a user-facing error at an in-memory row.
    pub fn user_error_at_row(
        &mut self,
        origin: Option<&str>,
        row: usize,
        message: impl Into<String>,
    ) {
        self.push(Event::user_error_at_row(row, message).with_origin_option(origin));
    }

    /// Emits a user-facing warning associated with a file path.
    pub fn user_warning_at_path(
        &mut self,
        origin: Option<&str>,
        path: impl Into<PathBuf>,
        message: impl Into<String>,
    ) {
        self.push(Event::user_path_warning(path, message).with_origin_option(origin));
    }

    /// Emits a user-facing error associated with a file path.
    pub fn user_error_at_path(
        &mut self,
        origin: Option<&str>,
        path: impl Into<PathBuf>,
        message: impl Into<String>,
    ) {
        self.push(Event::user_path_error(path, message).with_origin_option(origin));
    }

    /// Emits a user-facing warning at a file row.
    pub fn user_warning_at_file_row(
        &mut self,
        origin: Option<&str>,
        path: impl Into<PathBuf>,
        row: usize,
        message: impl Into<String>,
    ) {
        self.push(Event::user_file_warning(path, row, message).with_origin_option(origin));
    }

    /// Emits a user-facing error at a file row.
    pub fn user_error_at_file_row(
        &mut self,
        origin: Option<&str>,
        path: impl Into<PathBuf>,
        row: usize,
        message: impl Into<String>,
    ) {
        self.push(Event::user_file_error(path, row, message).with_origin_option(origin));
    }

    /// Emits a user-facing event with explicit level and optional location.
    pub fn user_event(
        &mut self,
        origin: Option<&str>,
        level: Level,
        location: Option<EventLocation>,
        message: impl Into<String>,
    ) {
        self.push(
            Event::message(message, level, Audience::User, location).with_origin_option(origin),
        );
    }

    /// Emits a system informational message.
    pub fn system_log(&mut self, origin: Option<&str>, message: impl Into<String>) {
        self.push(Event::system_log(message).with_origin_option(origin));
    }

    /// Emits a system warning message.
    pub fn system_warning(&mut self, origin: Option<&str>, message: impl Into<String>) {
        self.push(Event::system_warning(message).with_origin_option(origin));
    }

    /// Emits a system error message.
    pub fn system_error(&mut self, origin: Option<&str>, message: impl Into<String>) {
        self.push(Event::system_error(message).with_origin_option(origin));
    }

    /// Emits a system debug message.
    pub fn system_debug(&mut self, origin: Option<&str>, message: impl Into<String>) {
        self.push(Event::system_debug(message).with_origin_option(origin));
    }

    /// Emits and returns a begin marker.
    pub fn begin_marker(&mut self, label: impl Into<String>, origin: Option<&str>) -> MarkerEvent {
        let marker = MarkerEvent::new(
            MarkerId::new(),
            label,
            MarkerPhase::Begin,
            origin.map(str::to_owned),
        );
        self.push(Event::Marker(marker.clone()));
        marker
    }

    /// Emits and returns an end marker matching a begin marker.
    pub fn end_marker(&mut self, begin: &MarkerEvent, origin: Option<&str>) -> MarkerEvent {
        let marker = MarkerEvent::new(
            begin.id.clone(),
            begin.label.clone(),
            MarkerPhase::End,
            origin.map(str::to_owned),
        );
        self.push(Event::Marker(marker.clone()));
        marker
    }

    /// Emits an immediate begin/end marker pair and returns the range.
    pub fn range(&mut self, label: impl Into<String>, origin: Option<&str>) -> MarkerRange {
        let begin = self.begin_marker(label, origin);
        let end = self.end_marker(&begin, origin);
        MarkerRange::new(begin, end)
    }

    /// Returns the events between matching begin and end markers, inclusive.
    pub fn events_between<'a>(
        &'a self,
        begin: &MarkerEvent,
        end: &MarkerEvent,
    ) -> Option<Vec<&'a Event>> {
        if begin.phase != MarkerPhase::Begin || end.phase != MarkerPhase::End || begin.id != end.id
        {
            return None;
        }

        let start = self
            .events
            .iter()
            .position(|event| event.as_marker() == Some(begin))?;
        let finish = self
            .events
            .iter()
            .rposition(|event| event.as_marker() == Some(end))?;

        if start > finish {
            return None;
        }

        Some(self.events[start..=finish].iter().collect())
    }
}

// ===============================[ tests ]=====================================

#[cfg(test)]
mod tests {
    use super::{EventLog, EventLogListener};
    use crate::events::Event;
    use std::cell::RefCell;
    use std::rc::Rc;

    #[derive(Clone)]
    struct RecordingListener {
        events: Rc<RefCell<Vec<Event>>>,
    }

    impl EventLogListener for RecordingListener {
        fn on_event(&mut self, event: &Event) {
            self.events.borrow_mut().push(event.clone());
        }
    }

    #[test]
    fn replays_existing_events_to_new_listeners() {
        let mut log = EventLog::new();
        log.user_log(Some("mlg_check"), "Checked 1 file");

        let events = Rc::new(RefCell::new(Vec::new()));
        log.add_listener(RecordingListener {
            events: Rc::clone(&events),
        });

        assert_eq!(
            events.borrow().as_slice(),
            [Event::user_log("Checked 1 file").with_origin("mlg_check")]
        );
    }

    #[test]
    fn supports_markers_and_slicing_events_between_them() {
        let mut log = EventLog::new();
        let begin = log.begin_marker("check_in", Some("mlg_check"));
        log.system_debug(Some("mlg_check"), "Parsing file");
        log.user_error(Some("mlg_check"), "Unexpected token");
        let end = log.end_marker(&begin, Some("mlg_check"));

        let events = log
            .events_between(&begin, &end)
            .expect("expected marker range");

        assert_eq!(events.len(), 4);
        assert_eq!(
            events[1],
            &Event::system_debug("Parsing file").with_origin("mlg_check")
        );
    }

    #[test]
    fn counts_only_user_facing_non_log_messages_as_issues() {
        let mut log = EventLog::new();
        log.user_log(Some("mlg_check"), "Checked 1 file");
        log.user_warning(Some("mlg_check"), "Unused statement");
        log.system_error(Some("mlg_check"), "trace");
        log.user_debug(Some("mlg_check"), "Report this bug");

        assert!(log.has_errors());
        assert_eq!(log.issue_count(), 2);
    }
}

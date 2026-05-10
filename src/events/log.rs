use super::{
    Audience, Event, EventLocation, Level, MarkerEvent, MarkerId, MarkerPhase, MarkerRange,
};
use std::path::PathBuf;

pub trait EventLogListener {
    fn on_event(&mut self, event: &Event);
}

#[derive(Default)]
pub struct EventLog {
    events: Vec<Event>,
    listeners: Vec<Box<dyn EventLogListener>>,
}

impl EventLog {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_listener(&mut self, mut listener: impl EventLogListener + 'static) {
        for event in &self.events {
            listener.on_event(event);
        }

        self.listeners.push(Box::new(listener));
    }

    pub fn push(&mut self, event: Event) {
        self.events.push(event.clone());

        for listener in &mut self.listeners {
            listener.on_event(&event);
        }
    }

    pub fn events(&self) -> &[Event] {
        &self.events
    }

    pub fn has_errors(&self) -> bool {
        self.events.iter().any(|event| {
            event
                .as_message()
                .is_some_and(|message| message.level == Level::Error)
        })
    }

    pub fn issue_count(&self) -> usize {
        self.events
            .iter()
            .filter_map(Event::as_message)
            .filter(|message| message.audience == Audience::User && message.level != Level::Log)
            .count()
    }

    pub fn user_log(&mut self, origin: Option<&str>, message: impl Into<String>) {
        self.push(Event::user_log(message).with_origin_option(origin));
    }

    pub fn user_warning(&mut self, origin: Option<&str>, message: impl Into<String>) {
        self.push(Event::user_warning(message).with_origin_option(origin));
    }

    pub fn user_error(&mut self, origin: Option<&str>, message: impl Into<String>) {
        self.push(Event::user_error(message).with_origin_option(origin));
    }

    pub fn user_debug(&mut self, origin: Option<&str>, message: impl Into<String>) {
        self.push(Event::user_debug(message).with_origin_option(origin));
    }

    pub fn user_warning_at_row(
        &mut self,
        origin: Option<&str>,
        row: usize,
        message: impl Into<String>,
    ) {
        self.push(Event::user_warning_at_row(row, message).with_origin_option(origin));
    }

    pub fn user_error_at_row(
        &mut self,
        origin: Option<&str>,
        row: usize,
        message: impl Into<String>,
    ) {
        self.push(Event::user_error_at_row(row, message).with_origin_option(origin));
    }

    pub fn user_warning_at_path(
        &mut self,
        origin: Option<&str>,
        path: impl Into<PathBuf>,
        message: impl Into<String>,
    ) {
        self.push(Event::user_path_warning(path, message).with_origin_option(origin));
    }

    pub fn user_error_at_path(
        &mut self,
        origin: Option<&str>,
        path: impl Into<PathBuf>,
        message: impl Into<String>,
    ) {
        self.push(Event::user_path_error(path, message).with_origin_option(origin));
    }

    pub fn user_warning_at_file_row(
        &mut self,
        origin: Option<&str>,
        path: impl Into<PathBuf>,
        row: usize,
        message: impl Into<String>,
    ) {
        self.push(Event::user_file_warning(path, row, message).with_origin_option(origin));
    }

    pub fn user_error_at_file_row(
        &mut self,
        origin: Option<&str>,
        path: impl Into<PathBuf>,
        row: usize,
        message: impl Into<String>,
    ) {
        self.push(Event::user_file_error(path, row, message).with_origin_option(origin));
    }

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

    pub fn system_log(&mut self, origin: Option<&str>, message: impl Into<String>) {
        self.push(Event::system_log(message).with_origin_option(origin));
    }

    pub fn system_warning(&mut self, origin: Option<&str>, message: impl Into<String>) {
        self.push(Event::system_warning(message).with_origin_option(origin));
    }

    pub fn system_error(&mut self, origin: Option<&str>, message: impl Into<String>) {
        self.push(Event::system_error(message).with_origin_option(origin));
    }

    pub fn system_debug(&mut self, origin: Option<&str>, message: impl Into<String>) {
        self.push(Event::system_debug(message).with_origin_option(origin));
    }

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

    pub fn range(&mut self, label: impl Into<String>, origin: Option<&str>) -> MarkerRange {
        let begin = self.begin_marker(label, origin);
        let end = self.end_marker(&begin, origin);
        MarkerRange::new(begin, end)
    }

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

// =============================================================================

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

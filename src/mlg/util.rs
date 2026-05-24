use crate::events::{EventLog, Level};

/// Returns true when no error-level events were emitted after `starting_event_count`.
pub(super) fn no_errors_since(event_log: &EventLog, starting_event_count: usize) -> bool {
    !event_log.events()[starting_event_count..]
        .iter()
        .filter_map(|event| event.as_message())
        .any(|message| message.level == Level::Error)
}

use crate::events::{Audience, EventLog, Level};

/// Returns true when no error-level events were emitted after `starting_event_count`.
pub(super) fn no_errors_since(event_log: &EventLog, starting_event_count: usize) -> bool {
    !event_log.events()[starting_event_count..]
        .iter()
        .filter_map(|event| event.as_message())
        .any(|message| message.level == Level::Error)
}

/// Returns true when a user-facing blocking issue was emitted after `starting_event_count`.
pub(super) fn has_blocking_user_issues_since(
    event_log: &EventLog,
    starting_event_count: usize,
) -> bool {
    event_log.events()[starting_event_count..]
        .iter()
        .filter_map(|event| event.as_message())
        .any(|event| {
            event.audience == Audience::User && matches!(event.level, Level::Error | Level::Debug)
        })
}

/// Counts user-facing non-log issues emitted after `starting_event_count`.
pub(super) fn user_issue_count_since(event_log: &EventLog, starting_event_count: usize) -> usize {
    event_log.events()[starting_event_count..]
        .iter()
        .filter_map(|event| event.as_message())
        .filter(|event| event.audience == Audience::User && event.level != Level::Log)
        .count()
}

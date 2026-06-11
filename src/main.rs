use mlg::events::{EventConsoleWriter, EventLog};

fn main() {
    let writer = EventConsoleWriter::new();
    let mut event_log = EventLog::new();
    event_log.add_listener(writer);
    event_log.user_error(None, "Some user error");
    event_log.user_warning(Some("some source"), "Some user warning");
    event_log.user_log(Some("some other source"), "Some user log");
}

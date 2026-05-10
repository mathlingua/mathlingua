use crate::events::EventLog;

const ORIGIN: &str = "mlg_view";

pub fn view(event_log: &mut EventLog) {
    event_log.system_debug(Some(ORIGIN), "Rendering view output");
    event_log.user_log(Some(ORIGIN), "view");
}

#[cfg(test)]
mod tests {
    use super::view;
    use crate::events::{Event, EventLog};

    #[test]
    fn returns_placeholder_view_output() {
        let mut event_log = EventLog::new();

        view(&mut event_log);

        assert_eq!(
            event_log.events(),
            [
                Event::system_debug("Rendering view output").with_origin("mlg_view"),
                Event::user_log("view").with_origin("mlg_view")
            ]
        );
    }
}

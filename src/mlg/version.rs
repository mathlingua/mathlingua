use crate::events::EventLog;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const NAME: &str = env!("CARGO_PKG_NAME");
const ORIGIN: &str = "mlg_version";

pub fn version(event_log: &mut EventLog) {
    event_log.system_debug(Some(ORIGIN), "Rendering version information");
    event_log.user_log(Some(ORIGIN), format!("{NAME}: {VERSION}"));
}

#[cfg(test)]
mod tests {
    use super::version;
    use crate::events::{Event, EventLog};

    #[test]
    fn includes_package_name_and_version() {
        let mut event_log = EventLog::new();

        version(&mut event_log);

        assert_eq!(
            event_log.events(),
            [
                Event::system_debug("Rendering version information").with_origin("mlg_version"),
                Event::user_log(format!(
                    "{}: {}",
                    env!("CARGO_PKG_NAME"),
                    env!("CARGO_PKG_VERSION")
                ))
                .with_origin("mlg_version")
            ]
        );
    }
}

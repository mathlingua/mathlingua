use crate::events::{EventLog, EventLogListener, NoopEventLogListener};

const VERSION: &str = env!("CARGO_PKG_VERSION");
const NAME: &str = env!("CARGO_PKG_NAME");
const ORIGIN: &str = "mlg_version";

pub struct VersionResult {
    pub event_log: EventLog,
    pub successful: bool,
    pub name: &'static str,
    pub version: &'static str,
}

pub fn version(listener: Box<dyn EventLogListener>) -> VersionResult {
    let mut event_log = EventLog::new();
    event_log.add_listener(listener);

    emit_version(&mut event_log);

    let successful = !event_log.has_errors();
    VersionResult {
        event_log,
        successful,
        name: NAME,
        version: VERSION,
    }
}

pub(super) fn emit_version(event_log: &mut EventLog) {
    event_log.system_debug(Some(ORIGIN), "Rendering version information");
    event_log.user_log(Some(ORIGIN), format!("{NAME}: {VERSION}"));
}

// ===============================[ tests ]=====================================

#[cfg(test)]
mod tests {
    use super::{emit_version, version};
    use crate::events::{Event, EventLog, NoopEventLogListener};

    #[test]
    fn includes_package_name_and_version() {
        let mut event_log = EventLog::new();

        emit_version(&mut event_log);

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

    #[test]
    fn version_returns_a_successful_result() {
        let result = version(Box::new(NoopEventLogListener::new()));

        assert!(result.successful);
        assert_eq!(result.name, env!("CARGO_PKG_NAME"));
        assert_eq!(result.version, env!("CARGO_PKG_VERSION"));
        assert_eq!(result.event_log.events().len(), 2);
    }
}

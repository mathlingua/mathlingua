use crate::events::EventLog;

/// Package version compiled into the binary.
const VERSION: &str = env!("CARGO_PKG_VERSION");
/// Package name compiled into the binary.
const NAME: &str = env!("CARGO_PKG_NAME");
/// Event origin used by the version command.
const ORIGIN: &str = "mlg_version";

/// Emits the current package name and version.
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

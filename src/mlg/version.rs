use crate::events::{EventLog, EventLogListener};

/// Package version compiled into the binary.
const VERSION: &str = env!("CARGO_PKG_VERSION");
/// Package name compiled into the binary.
const NAME: &str = env!("CARGO_PKG_NAME");
/// Event origin used by the version command.
const ORIGIN: &str = "mlg_version";

/// Result of running [`version`].
pub struct VersionResult {
    /// Events emitted while rendering the version banner.
    pub event_log: EventLog,
    /// Whether the command produced no error-level events.
    pub successful: bool,
    /// Compiled package name.
    pub name: &'static str,
    /// Compiled package version.
    pub version: &'static str,
}

/// Emits the current package name and version.
pub fn version(listener: Option<Box<dyn EventLogListener>>) -> VersionResult {
    let mut event_log = EventLog::new();
    if let Some(listener) = listener {
        event_log.add_boxed_listener(listener);
    }

    emit_version(&mut event_log);

    let successful = !event_log.has_errors();
    VersionResult {
        event_log,
        successful,
        name: NAME,
        version: VERSION,
    }
}

/// Pushes the version banner events into an existing event log.
pub(super) fn emit_version(event_log: &mut EventLog) {
    event_log.system_debug(Some(ORIGIN), "Rendering version information");
    event_log.user_log(Some(ORIGIN), format!("{NAME}: {VERSION}"));
}

#[cfg(test)]
mod tests {
    use super::{emit_version, version};
    use crate::events::{Event, EventLog};

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
        let result = version(None);

        assert!(result.successful);
        assert_eq!(result.name, env!("CARGO_PKG_NAME"));
        assert_eq!(result.version, env!("CARGO_PKG_VERSION"));
        assert_eq!(result.event_log.events().len(), 2);
    }
}

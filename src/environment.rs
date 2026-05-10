use crate::events::EventLog;
use std::env;
use std::path::PathBuf;

const ORIGIN: &str = "environment";

pub fn current_working_directory(event_log: &mut EventLog) -> Option<PathBuf> {
    match env::current_dir() {
        Ok(cwd) => Some(cwd),
        Err(error) => {
            event_log.user_error(
                Some(ORIGIN),
                format!("Failed to determine the current working directory: {error}"),
            );
            None
        }
    }
}

// =============================================================================

#[cfg(test)]
mod tests {
    use super::current_working_directory;
    use crate::events::EventLog;

    #[test]
    fn returns_the_current_working_directory_without_emitting_events() {
        let mut event_log = EventLog::new();

        let cwd = current_working_directory(&mut event_log);

        assert!(cwd.is_some());
        assert!(event_log.events().is_empty());
    }
}

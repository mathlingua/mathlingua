use crate::events::{EventLog, EventLogListener};
use crate::mlg::collection::CONTENT_DIR;
use crate::mlg::config::{CONFIG_FILE, default_config_contents};
use crate::mlg::util::no_errors_since;
use std::fs;
use std::io;
use std::path::Path;

/// Event origin used by collection initialization.
const ORIGIN: &str = "mlg_init";

/// Result of running [`init`].
pub struct InitResult {
    /// Events emitted while initializing the collection.
    pub event_log: EventLog,
    /// Whether initialization completed without errors.
    pub successful: bool,
}

/// Initializes a MathLingua collection at the given working directory.
///
/// Existing `mlg.json` and `content/` entries are left untouched.  Missing
/// entries are created and reported through the returned event log.
pub fn init(cwd: &Path, listener: Option<Box<dyn EventLogListener>>) -> InitResult {
    let mut event_log = EventLog::new();
    if let Some(listener) = listener {
        event_log.add_boxed_listener(listener);
    }

    let starting_event_count = event_log.events().len();
    let io_ok = init_in(cwd, &mut event_log).is_ok();
    let successful = io_ok && no_errors_since(&event_log, starting_event_count);

    InitResult {
        event_log,
        successful,
    }
}

/// Pushes initialization events into an existing event log.
pub(super) fn init_in(root: &Path, event_log: &mut EventLog) -> io::Result<()> {
    event_log.system_debug(
        Some(ORIGIN),
        format!("Initializing collection at {}", root.display()),
    );

    let config_path = root.join(CONFIG_FILE);
    if config_path.exists() {
        event_log.user_log(
            Some(ORIGIN),
            format!("Skipping {CONFIG_FILE}; it already exists"),
        );
    } else {
        if let Err(error) = fs::write(&config_path, default_config_contents()) {
            event_log.user_error_at_path(
                Some(ORIGIN),
                config_path,
                format!("Failed to write {CONFIG_FILE}: {error}"),
            );
            return Err(error);
        }

        event_log.user_log(Some(ORIGIN), format!("Created {CONFIG_FILE}"));
    }

    let content_path = root.join(CONTENT_DIR);
    if content_path.exists() {
        event_log.user_log(
            Some(ORIGIN),
            format!("Skipping {CONTENT_DIR}/; it already exists"),
        );
    } else {
        if let Err(error) = fs::create_dir(&content_path) {
            event_log.user_error_at_path(
                Some(ORIGIN),
                content_path,
                format!("Failed to create {CONTENT_DIR}/: {error}"),
            );
            return Err(error);
        }

        event_log.user_log(Some(ORIGIN), format!("Created {CONTENT_DIR}/"));
    }

    Ok(())
}

// =============================================================================

#[cfg(test)]
mod tests {
    use super::{CONFIG_FILE, CONTENT_DIR, default_config_contents, init, init_in};
    use crate::events::{Event, EventLog};
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn init_creates_missing_config_and_content_directory() {
        let temp_dir = TestDir::new();
        let mut event_log = EventLog::new();

        init_in(temp_dir.path(), &mut event_log).expect("init should succeed");

        assert_eq!(
            event_log.events(),
            vec![
                Event::system_debug(format!(
                    "Initializing collection at {}",
                    temp_dir.path().display()
                ))
                .with_origin("mlg_init"),
                Event::user_log(format!("Created {CONFIG_FILE}")).with_origin("mlg_init"),
                Event::user_log(format!("Created {CONTENT_DIR}/")).with_origin("mlg_init"),
            ]
        );
        assert_eq!(
            fs::read_to_string(temp_dir.path().join(CONFIG_FILE)).unwrap(),
            default_config_contents()
        );
        assert!(temp_dir.path().join(CONTENT_DIR).is_dir());
    }

    #[test]
    fn init_returns_a_successful_result_for_a_fresh_directory() {
        let temp_dir = TestDir::new();

        let result = init(temp_dir.path(), None);

        assert!(result.successful);
        assert!(temp_dir.path().join(CONFIG_FILE).is_file());
        assert!(temp_dir.path().join(CONTENT_DIR).is_dir());
        assert!(!result.event_log.has_errors());
    }

    #[test]
    fn init_skips_existing_config_and_content_directory() {
        let temp_dir = TestDir::new();
        fs::write(temp_dir.path().join(CONFIG_FILE), default_config_contents()).unwrap();
        fs::create_dir(temp_dir.path().join(CONTENT_DIR)).unwrap();
        let mut event_log = EventLog::new();

        init_in(temp_dir.path(), &mut event_log).expect("init should succeed");

        assert_eq!(
            event_log.events(),
            vec![
                Event::system_debug(format!(
                    "Initializing collection at {}",
                    temp_dir.path().display()
                ))
                .with_origin("mlg_init"),
                Event::user_log(format!("Skipping {CONFIG_FILE}; it already exists"))
                    .with_origin("mlg_init"),
                Event::user_log(format!("Skipping {CONTENT_DIR}/; it already exists"))
                    .with_origin("mlg_init"),
            ]
        );
    }

    struct TestDir {
        path: PathBuf,
    }

    impl TestDir {
        fn new() -> Self {
            let unique = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let path = std::env::temp_dir().join(format!(
                "mlg-init-test-{}-{}",
                std::process::id(),
                unique
            ));
            fs::create_dir(&path).unwrap();
            Self { path }
        }

        fn path(&self) -> &Path {
            &self.path
        }
    }

    impl Drop for TestDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }
}

use crate::constants::{CONFIG_FILE, CONTENT_DIR};
use crate::events::EventLog;
use crate::mlg::config::default_config_contents;
use std::fs;
use std::io;
use std::path::Path;

/// Event origin used by collection initialization.
const ORIGIN: &str = "mlg_init";

/// Initializes a MathLingua collection at the given root directory.
///
/// Existing `mlg.json` and `content/` entries are left untouched.  Missing
/// entries are created and reported through the event log.
pub fn init(root: &Path, event_log: &mut EventLog) -> io::Result<()> {
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
    use super::{CONFIG_FILE, CONTENT_DIR, default_config_contents, init};
    use crate::events::{Event, EventLog};
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn init_creates_missing_config_and_content_directory() {
        let temp_dir = TestDir::new();
        let mut event_log = EventLog::new();

        init(temp_dir.path(), &mut event_log).expect("init should succeed");

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
    fn init_skips_existing_config_and_content_directory() {
        let temp_dir = TestDir::new();
        fs::write(temp_dir.path().join(CONFIG_FILE), default_config_contents()).unwrap();
        fs::create_dir(temp_dir.path().join(CONTENT_DIR)).unwrap();
        let mut event_log = EventLog::new();

        init(temp_dir.path(), &mut event_log).expect("init should succeed");

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

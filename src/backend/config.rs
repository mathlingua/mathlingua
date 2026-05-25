use crate::events::EventLog;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Default MathLingua collection configuration filename.
pub const CONFIG_FILE: &str = "mlg.json";

/// Parsed contents of `mlg.json`.
///
/// The configuration is intentionally small today, but it is validated explicitly
/// so user-facing errors can explain malformed or missing fields.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    /// Human-readable collection name.
    #[serde(default)]
    pub name: String,
    /// Collection schema/version string.
    #[serde(default = "default_version")]
    pub version: String,
}

/// Default version used when deserializing a config that omits `version`.
fn default_version() -> String {
    "0".to_string()
}

impl Default for Config {
    /// Returns the default config used by `mlg init`.
    fn default() -> Self {
        Self {
            name: String::new(),
            version: default_version(),
        }
    }
}

/// Returns pretty-printed default `mlg.json` contents with a trailing newline.
pub fn default_config_contents() -> String {
    let mut contents = serde_json::to_string_pretty(&Config::default())
        .expect("default Config should always serialize");
    contents.push('\n');
    contents
}

/// Validates a collection config file and emits user-facing diagnostics.
///
/// The validator accepts extra fields for forward compatibility but requires
/// `name` and `version` to be present and string-valued.
pub fn validate_config_file(path: &Path, event_log: &mut EventLog, origin: &str) {
    let contents = match fs::read_to_string(path) {
        Ok(contents) => contents,
        Err(error) => {
            event_log.user_error_at_path(
                Some(origin),
                path.to_path_buf(),
                format!("Failed to read {CONFIG_FILE}: {error}"),
            );
            return;
        }
    };

    let value: serde_json::Value = match serde_json::from_str(&contents) {
        Ok(value) => value,
        Err(error) => {
            event_log.user_error_at_path(
                Some(origin),
                path.to_path_buf(),
                format!("Invalid JSON in {CONFIG_FILE}: {error}"),
            );
            return;
        }
    };

    let serde_json::Value::Object(object) = value else {
        event_log.user_error_at_path(
            Some(origin),
            path.to_path_buf(),
            format!("{CONFIG_FILE} must be a JSON object"),
        );
        return;
    };

    validate_string_field(&object, "name", path, event_log, origin);
    validate_string_field(&object, "version", path, event_log, origin);
}

/// Validates one required string field in a JSON object.
fn validate_string_field(
    object: &serde_json::Map<String, serde_json::Value>,
    field: &str,
    path: &Path,
    event_log: &mut EventLog,
    origin: &str,
) {
    match object.get(field) {
        None => event_log.user_error_at_path(
            Some(origin),
            path.to_path_buf(),
            format!("{CONFIG_FILE} is missing required field \"{field}\""),
        ),
        Some(serde_json::Value::String(_)) => {}
        Some(_) => event_log.user_error_at_path(
            Some(origin),
            path.to_path_buf(),
            format!("{CONFIG_FILE} field \"{field}\" must be a string"),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::{Config, default_config_contents, default_version, validate_config_file};
    use crate::events::{Event, EventLog, Level};
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    const ORIGIN: &str = "test";

    static NEXT_TEST_DIR_ID: AtomicUsize = AtomicUsize::new(0);

    #[test]
    fn default_has_empty_name_and_version_zero() {
        let config = Config::default();
        assert_eq!(config.name, "");
        assert_eq!(config.version, "0");
    }

    #[test]
    fn default_contents_round_trip() {
        let contents = default_config_contents();
        let parsed: Config = serde_json::from_str(&contents).unwrap();
        assert_eq!(parsed, Config::default());
    }

    #[test]
    fn empty_object_uses_defaults() {
        let parsed: Config = serde_json::from_str("{}").unwrap();
        assert_eq!(parsed.name, "");
        assert_eq!(parsed.version, default_version());
    }

    #[test]
    fn validate_accepts_default_contents() {
        let dir = TestDir::new();
        let path = dir.path().join("mlg.json");
        fs::write(&path, default_config_contents()).unwrap();

        let mut event_log = EventLog::new();
        validate_config_file(&path, &mut event_log, ORIGIN);

        assert!(error_messages(&event_log).is_empty());
    }

    #[test]
    fn validate_accepts_extra_fields() {
        let dir = TestDir::new();
        let path = dir.path().join("mlg.json");
        fs::write(&path, r#"{"name": "thing", "version": "1", "extra": true}"#).unwrap();

        let mut event_log = EventLog::new();
        validate_config_file(&path, &mut event_log, ORIGIN);

        assert!(error_messages(&event_log).is_empty());
    }

    #[test]
    fn validate_reports_missing_fields() {
        let dir = TestDir::new();
        let path = dir.path().join("mlg.json");
        fs::write(&path, "{}\n").unwrap();

        let mut event_log = EventLog::new();
        validate_config_file(&path, &mut event_log, ORIGIN);

        assert_eq!(
            error_messages(&event_log),
            vec![
                "mlg.json is missing required field \"name\"".to_string(),
                "mlg.json is missing required field \"version\"".to_string(),
            ]
        );
    }

    #[test]
    fn validate_reports_wrong_type_fields() {
        let dir = TestDir::new();
        let path = dir.path().join("mlg.json");
        fs::write(&path, r#"{"name": 5, "version": true}"#).unwrap();

        let mut event_log = EventLog::new();
        validate_config_file(&path, &mut event_log, ORIGIN);

        assert_eq!(
            error_messages(&event_log),
            vec![
                "mlg.json field \"name\" must be a string".to_string(),
                "mlg.json field \"version\" must be a string".to_string(),
            ]
        );
    }

    #[test]
    fn validate_reports_invalid_json() {
        let dir = TestDir::new();
        let path = dir.path().join("mlg.json");
        fs::write(&path, "{not json").unwrap();

        let mut event_log = EventLog::new();
        validate_config_file(&path, &mut event_log, ORIGIN);

        let messages = error_messages(&event_log);
        assert_eq!(messages.len(), 1);
        assert!(
            messages[0].starts_with("Invalid JSON in mlg.json:"),
            "unexpected message: {}",
            messages[0]
        );
    }

    #[test]
    fn validate_reports_non_object_root() {
        let dir = TestDir::new();
        let path = dir.path().join("mlg.json");
        fs::write(&path, "[]").unwrap();

        let mut event_log = EventLog::new();
        validate_config_file(&path, &mut event_log, ORIGIN);

        assert_eq!(
            error_messages(&event_log),
            vec!["mlg.json must be a JSON object".to_string()]
        );
    }

    #[test]
    fn validate_reports_missing_file() {
        let dir = TestDir::new();
        let path = dir.path().join("mlg.json");

        let mut event_log = EventLog::new();
        validate_config_file(&path, &mut event_log, ORIGIN);

        let messages = error_messages(&event_log);
        assert_eq!(messages.len(), 1);
        assert!(
            messages[0].starts_with("Failed to read mlg.json:"),
            "unexpected message: {}",
            messages[0]
        );
    }

    fn error_messages(event_log: &EventLog) -> Vec<String> {
        event_log
            .events()
            .iter()
            .filter_map(Event::as_message)
            .filter(|message| message.level == Level::Error)
            .map(|message| message.message.clone())
            .collect()
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
            let id = NEXT_TEST_DIR_ID.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                "mlg-config-test-{}-{}-{}",
                std::process::id(),
                unique,
                id
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

use crate::events::EventLog;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

pub const CONFIG_FILE: &str = "mlg.json";

/// The default target width, in characters, for `mlg format`.
pub const DEFAULT_MARGIN: usize = 80;

/// Whether `mlg check` formats the collection before checking it, absent a
/// `format_on_check` setting. Formatting is normalization rather than a
/// judgement call, so a collection is kept formatted unless it opts out.
pub const DEFAULT_FORMAT_ON_CHECK: bool = true;

/// The former name of the `margin` field, still recognized so that a collection
/// carrying it is told to rename rather than silently losing its setting.
const LEGACY_MARGIN_FIELD: &str = "print_margin";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub name: String,
    #[serde(default = "default_version")]
    pub version: String,
    /// Target line width for `mlg format`; `None` uses `DEFAULT_MARGIN`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub margin: Option<usize>,
    /// Whether `mlg check` formats the collection before checking it; `None`
    /// uses `DEFAULT_FORMAT_ON_CHECK`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub format_on_check: Option<bool>,
}

impl Config {
    /// The configured margin, or the default when unset.
    pub fn margin(&self) -> usize {
        self.margin.unwrap_or(DEFAULT_MARGIN)
    }

    /// Whether `mlg check` should format first, or the default when unset.
    pub fn format_on_check(&self) -> bool {
        self.format_on_check.unwrap_or(DEFAULT_FORMAT_ON_CHECK)
    }
}

/// The config for the collection rooted at `root`, or the defaults when it has
/// no readable `mlg.json`.
///
/// Malformed contents fall back to the defaults rather than failing: the
/// callers that need a config also run [`validate_config_file`], which is what
/// reports the problem.
pub fn load_config(root: &Path) -> Config {
    fs::read_to_string(root.join(CONFIG_FILE))
        .ok()
        .and_then(|contents| serde_json::from_str::<Config>(&contents).ok())
        .unwrap_or_default()
}

fn default_version() -> String {
    "0".to_string()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            name: String::new(),
            version: default_version(),
            margin: None,
            format_on_check: None,
        }
    }
}

/// Whether `contents` still uses the pre-rename `print_margin` key.
///
/// `mlg format` reads the margin directly rather than going through
/// [`validate_config_file`], so it uses this to report the stale key instead of
/// silently formatting to the default width.
pub fn uses_legacy_margin_field(contents: &str) -> bool {
    serde_json::from_str::<serde_json::Value>(contents)
        .ok()
        .and_then(|value| value.as_object().cloned())
        .is_some_and(|object| object.contains_key(LEGACY_MARGIN_FIELD))
}

/// The error reported when a config still carries the pre-rename key.
pub fn legacy_margin_field_message() -> String {
    format!(
        "{CONFIG_FILE} field \"{LEGACY_MARGIN_FIELD}\" was renamed to \"margin\"; \
         rename it to keep the configured width"
    )
}

pub fn default_config_contents() -> String {
    let mut contents = serde_json::to_string_pretty(&Config::default())
        .expect("default Config should always serialize");
    contents.push('\n');
    contents
}

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

    // `margin` is optional, but if present it must be a positive integer.
    if let Some(value) = object.get("margin") {
        let valid = value.as_u64().is_some_and(|margin| margin > 0);
        if !valid {
            event_log.user_error_at_path(
                Some(origin),
                path.to_path_buf(),
                format!("{CONFIG_FILE} field \"margin\" must be a positive integer"),
            );
        }
    }

    // `format_on_check` is optional, but if present it must be a boolean —
    // otherwise it silently falls back to the default and formats a collection
    // that meant to opt out.
    if let Some(value) = object.get("format_on_check")
        && !value.is_boolean()
    {
        event_log.user_error_at_path(
            Some(origin),
            path.to_path_buf(),
            format!("{CONFIG_FILE} field \"format_on_check\" must be a boolean"),
        );
    }

    // Unknown fields are otherwise ignored, so a collection still carrying the
    // old `print_margin` would silently fall back to the default width. Name it
    // explicitly instead.
    if object.contains_key(LEGACY_MARGIN_FIELD) {
        event_log.user_error_at_path(
            Some(origin),
            path.to_path_buf(),
            legacy_margin_field_message(),
        );
    }
}

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

// ===============================[ tests ]=====================================

#[cfg(test)]
mod tests {
    use super::{
        Config, DEFAULT_MARGIN, default_config_contents, default_version, load_config,
        validate_config_file,
    };
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
    fn margin_defaults_when_unset() {
        let parsed: Config = serde_json::from_str(r#"{"name": "a", "version": "1"}"#).unwrap();

        assert_eq!(parsed.margin, None);
        assert_eq!(parsed.margin(), DEFAULT_MARGIN);
    }

    #[test]
    fn margin_is_read_from_the_config() {
        let parsed: Config =
            serde_json::from_str(r#"{"name": "a", "version": "1", "margin": 80}"#).unwrap();

        assert_eq!(parsed.margin(), 80);
    }

    #[test]
    fn validate_accepts_a_positive_margin() {
        let dir = TestDir::new();
        let path = dir.path().join("mlg.json");
        fs::write(&path, r#"{"name": "a", "version": "1", "margin": 80}"#).unwrap();

        let mut event_log = EventLog::new();
        validate_config_file(&path, &mut event_log, ORIGIN);

        assert!(error_messages(&event_log).is_empty());
    }

    #[test]
    fn validate_reports_a_non_positive_margin() {
        let dir = TestDir::new();
        let path = dir.path().join("mlg.json");
        fs::write(&path, r#"{"name": "a", "version": "1", "margin": 0}"#).unwrap();

        let mut event_log = EventLog::new();
        validate_config_file(&path, &mut event_log, ORIGIN);

        assert_eq!(
            error_messages(&event_log),
            vec!["mlg.json field \"margin\" must be a positive integer".to_string()]
        );
    }

    #[test]
    fn validate_reports_the_renamed_print_margin_field() {
        // Unknown fields are ignored, so without this the old key would silently
        // drop the configured width back to the default.
        let dir = TestDir::new();
        let path = dir.path().join("mlg.json");
        fs::write(
            &path,
            r#"{"name": "a", "version": "1", "print_margin": 80}"#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        validate_config_file(&path, &mut event_log, ORIGIN);

        assert_eq!(
            error_messages(&event_log),
            vec![
                "mlg.json field \"print_margin\" was renamed to \"margin\"; \
                 rename it to keep the configured width"
                    .to_string()
            ]
        );
    }

    #[test]
    fn format_on_check_defaults_to_true_when_unset() {
        let parsed: Config = serde_json::from_str(r#"{"name": "a", "version": "1"}"#).unwrap();

        assert_eq!(parsed.format_on_check, None);
        assert!(parsed.format_on_check());
    }

    #[test]
    fn format_on_check_can_be_turned_off() {
        let parsed: Config =
            serde_json::from_str(r#"{"name": "a", "version": "1", "format_on_check": false}"#)
                .unwrap();

        assert!(!parsed.format_on_check());
    }

    #[test]
    fn validate_accepts_a_boolean_format_on_check() {
        let dir = TestDir::new();
        let path = dir.path().join("mlg.json");
        fs::write(
            &path,
            r#"{"name": "a", "version": "1", "format_on_check": false}"#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        validate_config_file(&path, &mut event_log, ORIGIN);

        assert!(error_messages(&event_log).is_empty());
    }

    #[test]
    fn validate_reports_a_non_boolean_format_on_check() {
        // Without this the value is ignored and the collection is formatted
        // anyway — the opposite of what writing the field asked for.
        let dir = TestDir::new();
        let path = dir.path().join("mlg.json");
        fs::write(
            &path,
            r#"{"name": "a", "version": "1", "format_on_check": "no"}"#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        validate_config_file(&path, &mut event_log, ORIGIN);

        assert_eq!(
            error_messages(&event_log),
            vec!["mlg.json field \"format_on_check\" must be a boolean".to_string()]
        );
    }

    #[test]
    fn load_config_falls_back_to_defaults_without_a_config_file() {
        let dir = TestDir::new();

        assert_eq!(load_config(dir.path()), Config::default());
    }

    #[test]
    fn load_config_reads_the_collections_settings() {
        let dir = TestDir::new();
        fs::write(
            dir.path().join("mlg.json"),
            r#"{"name": "a", "version": "1", "format_on_check": false, "margin": 100}"#,
        )
        .unwrap();

        let config = load_config(dir.path());

        assert!(!config.format_on_check());
        assert_eq!(config.margin(), 100);
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

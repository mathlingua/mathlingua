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

/// Every field an `mlg.json` must carry, in the order `mlg init` writes them.
///
/// A collection must spell out every setting rather than lean on implicit
/// defaults, so that the whole configuration is visible and editable in one
/// place. `mlg check` reports any of these that is absent, and `mlg init` fills
/// them in with their defaults.
pub const CONFIG_FIELDS: [&str; 4] = ["name", "version", "margin", "format_on_check"];

/// The former name of the `margin` field, still recognized so that a collection
/// carrying it is told to rename rather than silently losing its setting.
const LEGACY_MARGIN_FIELD: &str = "print_margin";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub name: String,
    #[serde(default = "default_version")]
    pub version: String,
    /// Target line width for `mlg format`.
    ///
    /// A valid `mlg.json` always carries this, but parsing stays lenient — a
    /// missing value reads as `None` so tools keep running while `mlg check`
    /// reports the omission — and [`Self::margin`] falls back to `DEFAULT_MARGIN`.
    #[serde(default)]
    pub margin: Option<usize>,
    /// Whether `mlg check` formats the collection before checking it.
    ///
    /// Lenient in the same way as [`Self::margin`]: `None` when absent, with
    /// [`Self::format_on_check`] falling back to `DEFAULT_FORMAT_ON_CHECK`.
    #[serde(default)]
    pub format_on_check: Option<bool>,
}

impl Config {
    /// The configured margin, or the default when a partial config omitted it.
    pub fn margin(&self) -> usize {
        self.margin.unwrap_or(DEFAULT_MARGIN)
    }

    /// Whether `mlg check` should format first, or the default when a partial
    /// config omitted it.
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
            margin: Some(DEFAULT_MARGIN),
            format_on_check: Some(DEFAULT_FORMAT_ON_CHECK),
        }
    }
}

/// The default value of every config field, as a JSON object.
///
/// Derived from [`Config::default`] so the written defaults and the in-memory
/// ones can never drift apart.
fn default_field_values() -> serde_json::Map<String, serde_json::Value> {
    match serde_json::to_value(Config::default()) {
        Ok(serde_json::Value::Object(map)) => map,
        _ => unreachable!("Config serializes to a JSON object"),
    }
}

/// Parses `contents` as the JSON object an `mlg.json` should be, or `None` when
/// it is not valid JSON or not an object.
pub fn config_object(contents: &str) -> Option<serde_json::Map<String, serde_json::Value>> {
    match serde_json::from_str(contents) {
        Ok(serde_json::Value::Object(map)) => Some(map),
        _ => None,
    }
}

/// The required fields absent from `object`, in the canonical [`CONFIG_FIELDS`]
/// order.
pub fn missing_config_fields(object: &serde_json::Map<String, serde_json::Value>) -> Vec<String> {
    CONFIG_FIELDS
        .iter()
        .filter(|field| !object.contains_key(**field))
        .map(|field| (*field).to_string())
        .collect()
}

/// `object` re-serialized with every missing required field added at its default.
///
/// Existing fields keep their values and their position; the defaults for any
/// absent fields are appended. Unknown extra fields are preserved untouched.
pub fn merge_default_fields(object: &serde_json::Map<String, serde_json::Value>) -> String {
    let defaults = default_field_values();
    let mut merged = object.clone();
    for field in CONFIG_FIELDS {
        if !merged.contains_key(field)
            && let Some(value) = defaults.get(field)
        {
            merged.insert(field.to_string(), value.clone());
        }
    }

    let mut contents = serde_json::to_string_pretty(&merged)
        .expect("a config object always serializes to JSON");
    contents.push('\n');
    contents
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

    // Every field is required: a config must state its whole configuration
    // rather than lean on implicit defaults, so `mlg init` can show all of them.
    validate_string_field(&object, "name", path, event_log, origin);
    validate_string_field(&object, "version", path, event_log, origin);

    // `margin` must be present and a positive integer. When it is absent but the
    // old `print_margin` is present, the rename message below already tells the
    // author to add `margin`, so it is not also reported as missing here.
    match object.get("margin") {
        Some(value) if value.as_u64().is_some_and(|margin| margin > 0) => {}
        Some(_) => event_log.user_error_at_path(
            Some(origin),
            path.to_path_buf(),
            format!("{CONFIG_FILE} field \"margin\" must be a positive integer"),
        ),
        None if object.contains_key(LEGACY_MARGIN_FIELD) => {}
        None => report_missing_field("margin", path, event_log, origin),
    }

    // `format_on_check` must be present and a boolean — a non-boolean silently
    // falls back to the default and formats a collection that meant to opt out.
    match object.get("format_on_check") {
        Some(value) if value.is_boolean() => {}
        Some(_) => event_log.user_error_at_path(
            Some(origin),
            path.to_path_buf(),
            format!("{CONFIG_FILE} field \"format_on_check\" must be a boolean"),
        ),
        None => report_missing_field("format_on_check", path, event_log, origin),
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

fn report_missing_field(field: &str, path: &Path, event_log: &mut EventLog, origin: &str) {
    event_log.user_error_at_path(
        Some(origin),
        path.to_path_buf(),
        format!("{CONFIG_FILE} is missing required field \"{field}\""),
    );
}

fn validate_string_field(
    object: &serde_json::Map<String, serde_json::Value>,
    field: &str,
    path: &Path,
    event_log: &mut EventLog,
    origin: &str,
) {
    match object.get(field) {
        None => report_missing_field(field, path, event_log, origin),
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
        Config, DEFAULT_MARGIN, config_object, default_config_contents, default_version,
        load_config, merge_default_fields, missing_config_fields, validate_config_file,
    };
    use crate::events::{Event, EventLog, Level};
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    const ORIGIN: &str = "test";

    static NEXT_TEST_DIR_ID: AtomicUsize = AtomicUsize::new(0);

    #[test]
    fn default_fills_in_every_field() {
        let config = Config::default();
        assert_eq!(config.name, "");
        assert_eq!(config.version, "0");
        assert_eq!(config.margin, Some(80));
        assert_eq!(config.format_on_check, Some(true));
    }

    #[test]
    fn default_contents_spell_out_every_field() {
        // `mlg init` writes this, so the author sees every key and its default.
        let contents = default_config_contents();
        for field in ["name", "version", "margin", "format_on_check"] {
            assert!(
                contents.contains(&format!("\"{field}\"")),
                "default config should contain {field}: {contents}"
            );
        }
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
        fs::write(
            &path,
            r#"{"name": "thing", "version": "1", "margin": 80, "format_on_check": true, "extra": true}"#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        validate_config_file(&path, &mut event_log, ORIGIN);

        assert!(error_messages(&event_log).is_empty());
    }

    #[test]
    fn validate_reports_every_missing_field() {
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
                "mlg.json is missing required field \"margin\"".to_string(),
                "mlg.json is missing required field \"format_on_check\"".to_string(),
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
        fs::write(
            &path,
            r#"{"name": "a", "version": "1", "margin": 80, "format_on_check": true}"#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        validate_config_file(&path, &mut event_log, ORIGIN);

        assert!(error_messages(&event_log).is_empty());
    }

    #[test]
    fn validate_reports_a_non_positive_margin() {
        let dir = TestDir::new();
        let path = dir.path().join("mlg.json");
        fs::write(
            &path,
            r#"{"name": "a", "version": "1", "margin": 0, "format_on_check": true}"#,
        )
        .unwrap();

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
            r#"{"name": "a", "version": "1", "format_on_check": true, "print_margin": 80}"#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        validate_config_file(&path, &mut event_log, ORIGIN);

        // The rename message stands in for "missing margin": it already tells the
        // author to add the new key, so `margin` is not also flagged as missing.
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
            r#"{"name": "a", "version": "1", "margin": 80, "format_on_check": false}"#,
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
            r#"{"name": "a", "version": "1", "margin": 80, "format_on_check": "no"}"#,
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
        fs::write(
            &path,
            r#"{"name": 5, "version": true, "margin": 80, "format_on_check": true}"#,
        )
        .unwrap();

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

    #[test]
    fn missing_config_fields_lists_absent_fields_in_canonical_order() {
        let object = config_object(r#"{"version": "1", "extra": true}"#).expect("object");

        assert_eq!(
            missing_config_fields(&object),
            vec![
                "name".to_string(),
                "margin".to_string(),
                "format_on_check".to_string()
            ]
        );
    }

    #[test]
    fn missing_config_fields_is_empty_for_a_complete_config() {
        let object = config_object(&default_config_contents()).expect("object");

        assert!(missing_config_fields(&object).is_empty());
    }

    #[test]
    fn config_object_rejects_non_objects() {
        assert!(config_object("[]").is_none());
        assert!(config_object("not json").is_none());
    }

    #[test]
    fn merge_default_fields_fills_gaps_and_keeps_existing_values() {
        let object = config_object(r#"{"name": "mine", "extra": "kept"}"#).expect("object");

        let merged = merge_default_fields(&object);
        let reparsed = config_object(&merged).expect("merged object");

        // Existing values (including unknown extras) are preserved.
        assert_eq!(reparsed.get("name").and_then(|v| v.as_str()), Some("mine"));
        assert_eq!(
            reparsed.get("extra").and_then(|v| v.as_str()),
            Some("kept")
        );
        // Every previously missing field is now present at its default.
        assert!(missing_config_fields(&reparsed).is_empty());
        assert_eq!(reparsed.get("version").and_then(|v| v.as_str()), Some("0"));
        assert_eq!(reparsed.get("margin").and_then(|v| v.as_u64()), Some(80));
        assert_eq!(
            reparsed.get("format_on_check").and_then(|v| v.as_bool()),
            Some(true)
        );
    }

    #[test]
    fn merge_default_fields_preserves_a_complete_config() {
        let object = config_object(&default_config_contents()).expect("object");

        assert_eq!(merge_default_fields(&object), default_config_contents());
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

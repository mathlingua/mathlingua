use crate::backend::collection::CONTENT_DIR;
use crate::backend::config::{
    CONFIG_FILE, config_object, default_config_contents, merge_default_fields, missing_config_fields,
};
use crate::events::{EventLog, EventLogListener};
use crate::mlg::util::no_errors_since;
use std::fs;
use std::io::{self, IsTerminal, Write};
use std::path::Path;

const ORIGIN: &str = "mlg_init";

pub struct InitResult {
    pub event_log: EventLog,
    pub successful: bool,
}

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

pub(super) fn init_in(root: &Path, event_log: &mut EventLog) -> io::Result<()> {
    event_log.system_debug(
        Some(ORIGIN),
        format!("Initializing collection at {}", root.display()),
    );

    let config_path = root.join(CONFIG_FILE);
    if config_path.exists() {
        reconcile_existing_config(&config_path, event_log)?;
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

/// Reconciles an `mlg.json` that already exists.
///
/// Every field is required, so an older config may be missing some. When it is,
/// `mlg init` offers to fill the gaps with the defaults — the same values a fresh
/// `mlg.json` would carry — rather than leaving the author to add them by hand.
fn reconcile_existing_config(config_path: &Path, event_log: &mut EventLog) -> io::Result<()> {
    let contents = match fs::read_to_string(config_path) {
        Ok(contents) => contents,
        Err(error) => {
            event_log.user_error_at_path(
                Some(ORIGIN),
                config_path.to_path_buf(),
                format!("Failed to read {CONFIG_FILE}: {error}"),
            );
            return Err(error);
        }
    };

    // A malformed config cannot be safely merged; `mlg check` reports the details.
    let Some(object) = config_object(&contents) else {
        event_log.user_log(
            Some(ORIGIN),
            format!(
                "Skipping {CONFIG_FILE}; it already exists but is not a JSON object \
                 (run `mlg check` for details)"
            ),
        );
        return Ok(());
    };

    let missing = missing_config_fields(&object);
    if missing.is_empty() {
        event_log.user_log(
            Some(ORIGIN),
            format!("Skipping {CONFIG_FILE}; it already exists"),
        );
        return Ok(());
    }

    if !should_fill_missing_fields(&missing) {
        event_log.user_log(
            Some(ORIGIN),
            format!(
                "Left {CONFIG_FILE} unchanged; it is missing {}",
                describe_fields(&missing)
            ),
        );
        return Ok(());
    }

    let merged = merge_default_fields(&object);
    if let Err(error) = fs::write(config_path, merged) {
        event_log.user_error_at_path(
            Some(ORIGIN),
            config_path.to_path_buf(),
            format!("Failed to write {CONFIG_FILE}: {error}"),
        );
        return Err(error);
    }

    event_log.user_log(
        Some(ORIGIN),
        format!("Added the default {} to {CONFIG_FILE}", describe_fields(&missing)),
    );
    Ok(())
}

/// Whether the missing fields should be filled in.
///
/// The decision is the user's, so it is only made by asking. Without an
/// interactive terminal there is no one to ask, so the config is left as it is
/// (and `mlg check` still reports the missing fields).
fn should_fill_missing_fields(missing: &[String]) -> bool {
    io::stdin().is_terminal() && prompt_fill_missing_fields(missing)
}

/// Asks whether to fill the missing fields in, re-asking until the answer parses.
///
/// End-of-input is treated as "no": filling in fields rewrites the file, so an
/// unattended run should not do it.
fn prompt_fill_missing_fields(missing: &[String]) -> bool {
    loop {
        print!(
            "{CONFIG_FILE} is missing {}. Fill in the defaults? [Y/n]: ",
            describe_fields(missing)
        );
        if io::stdout().flush().is_err() {
            return false;
        }

        let mut answer = String::new();
        match io::stdin().read_line(&mut answer) {
            Ok(0) | Err(_) => return false,
            Ok(_) => {}
        }

        if let Some(choice) = parse_yes_no(&answer) {
            return choice;
        }
        println!("Please answer y or n.");
    }
}

/// Maps a typed answer to yes/no. A blank line is "yes": filling in the defaults
/// only adds the missing fields and is the recommended action, so it is the
/// default the capitalized `Y` in the prompt advertises.
fn parse_yes_no(input: &str) -> Option<bool> {
    match input.trim().to_ascii_lowercase().as_str() {
        "" | "y" | "yes" => Some(true),
        "n" | "no" => Some(false),
        _ => None,
    }
}

/// A human-readable phrase for a set of config field names, such as
/// `field "name"` or `fields "margin" and "format_on_check"`.
fn describe_fields(fields: &[String]) -> String {
    let quoted: Vec<String> = fields.iter().map(|field| format!("\"{field}\"")).collect();
    let noun = if quoted.len() == 1 { "field" } else { "fields" };
    let list = match quoted.split_last() {
        None => String::new(),
        Some((last, [])) => last.clone(),
        Some((last, rest)) => format!("{} and {last}", rest.join(", ")),
    };
    format!("{noun} {list}")
}

// ===============================[ tests ]=====================================

#[cfg(test)]
mod tests {
    use super::{
        CONFIG_FILE, CONTENT_DIR, default_config_contents, describe_fields, init, init_in,
        parse_yes_no,
    };
    use crate::backend::config::missing_config_fields;
    use crate::events::{Event, EventLog};
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    static TEST_DIR_COUNTER: AtomicUsize = AtomicUsize::new(0);

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

    #[test]
    fn init_leaves_an_incomplete_config_unchanged_without_a_terminal() {
        // `cargo test` runs without an interactive terminal, so init cannot ask
        // whether to fill in the gaps; it reports them and touches nothing.
        let temp_dir = TestDir::new();
        let partial = "{\n  \"name\": \"a\",\n  \"version\": \"1\"\n}\n";
        fs::write(temp_dir.path().join(CONFIG_FILE), partial).unwrap();
        let mut event_log = EventLog::new();

        init_in(temp_dir.path(), &mut event_log).expect("init should succeed");

        assert_eq!(
            fs::read_to_string(temp_dir.path().join(CONFIG_FILE)).unwrap(),
            partial,
            "the config must not be rewritten"
        );
        assert!(
            event_log
                .events()
                .iter()
                .filter_map(Event::as_message)
                .any(|message| message.message
                    == format!(
                        "Left {CONFIG_FILE} unchanged; it is missing fields \"margin\" and \"format_on_check\""
                    )),
            "the missing fields must be reported: {:#?}",
            event_log.events()
        );
    }

    #[test]
    fn init_reports_the_content_directory_normally_for_an_incomplete_config() {
        // A missing-field config is not an error, so init still finishes its other
        // work — here, creating the content directory.
        let temp_dir = TestDir::new();
        fs::write(
            temp_dir.path().join(CONFIG_FILE),
            "{\n  \"name\": \"a\",\n  \"version\": \"1\"\n}\n",
        )
        .unwrap();

        let result = init(temp_dir.path(), None);

        assert!(result.successful);
        assert!(!result.event_log.has_errors());
        assert!(temp_dir.path().join(CONTENT_DIR).is_dir());
    }

    #[test]
    fn parse_yes_no_reads_answers_and_defaults_blank_to_yes() {
        assert_eq!(parse_yes_no("y"), Some(true));
        assert_eq!(parse_yes_no("  YES \n"), Some(true));
        assert_eq!(parse_yes_no(""), Some(true));
        assert_eq!(parse_yes_no("n"), Some(false));
        assert_eq!(parse_yes_no("No"), Some(false));
        assert_eq!(parse_yes_no("maybe"), None);
    }

    #[test]
    fn describe_fields_reads_as_a_list() {
        assert_eq!(describe_fields(&["margin".to_string()]), "field \"margin\"");
        assert_eq!(
            describe_fields(&["margin".to_string(), "format_on_check".to_string()]),
            "fields \"margin\" and \"format_on_check\""
        );
        assert_eq!(
            describe_fields(&[
                "name".to_string(),
                "margin".to_string(),
                "format_on_check".to_string()
            ]),
            "fields \"name\", \"margin\" and \"format_on_check\""
        );
    }

    #[test]
    fn missing_config_fields_matches_the_reported_gaps() {
        // The message init prints is built from the same list `mlg check` uses.
        let object =
            crate::backend::config::config_object("{\"name\": \"a\", \"version\": \"1\"}").unwrap();
        assert_eq!(
            missing_config_fields(&object),
            vec!["margin".to_string(), "format_on_check".to_string()]
        );
    }

    struct TestDir {
        path: PathBuf,
    }

    impl TestDir {
        fn new() -> Self {
            loop {
                let unique = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_nanos();
                let sequence = TEST_DIR_COUNTER.fetch_add(1, Ordering::Relaxed);
                let path = std::env::temp_dir().join(format!(
                    "mlg-init-test-{}-{}-{}",
                    std::process::id(),
                    unique,
                    sequence
                ));
                match fs::create_dir(&path) {
                    Ok(()) => return Self { path },
                    Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
                    Err(error) => panic!("failed to create test directory {path:?}: {error}"),
                }
            }
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

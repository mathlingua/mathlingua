use crate::backend::collection::find_collection_root;
use crate::backend::config::load_config;
use crate::events::{EventLog, EventLogListener};
use crate::mlg::util::no_errors_since;
use std::fs;
use std::io;
use std::path::Path;

const ORIGIN: &str = "mlg_clean";

pub struct CleanResult {
    pub event_log: EventLog,
    pub successful: bool,
}

/// Remove the generated `docs/` directory from the collection rooted at (or
/// above) `cwd`. This is the inverse of `mlg export`, which builds into `docs/`.
pub fn clean(cwd: &Path, listener: Option<Box<dyn EventLogListener>>) -> CleanResult {
    let mut event_log = EventLog::new();
    if let Some(listener) = listener {
        event_log.add_boxed_listener(listener);
    }

    let starting_event_count = event_log.events().len();
    let io_ok = clean_in(cwd, &mut event_log).is_ok();
    let successful = io_ok && no_errors_since(&event_log, starting_event_count);

    CleanResult {
        event_log,
        successful,
    }
}

fn clean_in(cwd: &Path, event_log: &mut EventLog) -> io::Result<()> {
    let start = cwd.canonicalize().unwrap_or_else(|_| cwd.to_path_buf());
    let Some(root) = find_collection_root(&start) else {
        event_log.user_error(
            Some(ORIGIN),
            "Could not find an mlg.json; run `mlg clean` inside a Mathlingua collection",
        );
        return Err(io::Error::other("no collection root"));
    };

    // `output_dir()` sanitizes the configured value, so it always stays within the
    // collection root even if `mlg.json` was never checked.
    let output_dir = load_config(&root).output_dir();
    let output_path = root.join(&output_dir);
    if !output_path.exists() {
        event_log.user_log(
            Some(ORIGIN),
            format!("Nothing to remove; {output_dir}/ does not exist"),
        );
        return Ok(());
    }

    if let Err(error) = fs::remove_dir_all(&output_path) {
        event_log.user_error_at_path(
            Some(ORIGIN),
            output_path,
            format!("Failed to remove {output_dir}/: {error}"),
        );
        return Err(error);
    }

    event_log.user_log(Some(ORIGIN), format!("Removed {output_dir}/"));
    Ok(())
}

// ===============================[ tests ]=====================================

#[cfg(test)]
mod tests {
    use super::clean;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    const DOCS_DIR: &str = "docs";

    #[test]
    fn removes_the_docs_directory() {
        let dir = TestDir::new();
        let root = dir.path();
        fs::write(root.join("mlg.json"), "{\n  \"version\": \"0\"\n}\n").unwrap();
        fs::create_dir_all(root.join(DOCS_DIR).join("nested")).unwrap();
        fs::write(root.join(DOCS_DIR).join("index.html"), "<html>").unwrap();

        let result = clean(root, None);

        assert!(result.successful, "{:#?}", result.event_log.events());
        assert!(!root.join(DOCS_DIR).exists());
    }

    #[test]
    fn removes_a_custom_output_directory() {
        let dir = TestDir::new();
        let root = dir.path();
        fs::write(
            root.join("mlg.json"),
            "{\n  \"version\": \"0\",\n  \"outputDir\": \"site\"\n}\n",
        )
        .unwrap();
        fs::create_dir_all(root.join("site").join("nested")).unwrap();
        // The default `docs/` is left untouched when a custom directory is configured.
        fs::create_dir_all(root.join(DOCS_DIR)).unwrap();

        let result = clean(root, None);

        assert!(result.successful, "{:#?}", result.event_log.events());
        assert!(!root.join("site").exists());
        assert!(root.join(DOCS_DIR).exists(), "docs/ must be left alone");
    }

    #[test]
    fn ignores_an_unsafe_output_directory() {
        let dir = TestDir::new();
        let root = dir.path();
        // An escaping value must never be honored; clean falls back to `docs/`.
        fs::write(
            root.join("mlg.json"),
            "{\n  \"version\": \"0\",\n  \"outputDir\": \"../escape\"\n}\n",
        )
        .unwrap();
        let sibling = root.parent().unwrap().join("escape");
        fs::create_dir_all(&sibling).unwrap();

        let result = clean(root, None);

        assert!(result.successful);
        assert!(sibling.exists(), "must not delete outside the collection");
        let _ = fs::remove_dir_all(&sibling);
    }

    #[test]
    fn is_a_no_op_when_docs_is_absent() {
        let dir = TestDir::new();
        let root = dir.path();
        fs::write(root.join("mlg.json"), "{\n  \"version\": \"0\"\n}\n").unwrap();

        let result = clean(root, None);

        assert!(result.successful);
        assert!(!root.join(DOCS_DIR).exists());
    }

    #[test]
    fn fails_outside_a_collection() {
        let dir = TestDir::new();
        // No mlg.json anywhere: refuse to touch any docs/ directory.
        fs::create_dir_all(dir.path().join(DOCS_DIR)).unwrap();

        let result = clean(dir.path(), None);

        assert!(!result.successful);
        assert!(
            dir.path().join(DOCS_DIR).exists(),
            "docs must be left alone"
        );
    }

    struct TestDir {
        path: PathBuf,
    }

    impl TestDir {
        fn new() -> Self {
            static COUNTER: AtomicUsize = AtomicUsize::new(0);
            let unique = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let sequence = COUNTER.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                "mlg-clean-test-{}-{unique}-{sequence}",
                std::process::id()
            ));
            fs::create_dir_all(&path).unwrap();
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

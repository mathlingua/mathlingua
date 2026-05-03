use super::{CONFIG_FILE, CONTENT_DIR, DEFAULT_CONFIG, init_in};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn init_creates_missing_config_and_content_directory() {
    let temp_dir = TestDir::new();

    let messages = init_in(temp_dir.path()).expect("init should succeed");

    assert_eq!(
        messages,
        vec![
            format!("Created {CONFIG_FILE}"),
            format!("Created {CONTENT_DIR}/"),
        ]
    );
    assert_eq!(
        fs::read_to_string(temp_dir.path().join(CONFIG_FILE)).unwrap(),
        DEFAULT_CONFIG
    );
    assert!(temp_dir.path().join(CONTENT_DIR).is_dir());
}

#[test]
fn init_skips_existing_config_and_content_directory() {
    let temp_dir = TestDir::new();
    fs::write(temp_dir.path().join(CONFIG_FILE), DEFAULT_CONFIG).unwrap();
    fs::create_dir(temp_dir.path().join(CONTENT_DIR)).unwrap();

    let messages = init_in(temp_dir.path()).expect("init should succeed");

    assert_eq!(
        messages,
        vec![
            format!("Skipping {CONFIG_FILE}; it already exists"),
            format!("Skipping {CONTENT_DIR}/; it already exists"),
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
        let path =
            std::env::temp_dir().join(format!("mlg-init-test-{}-{}", std::process::id(), unique));
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

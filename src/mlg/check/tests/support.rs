
use super::check_in;
use crate::events::{Audience, Event, EventLog, Level};
use crate::mlg::collection::{find_collection_root, resolve_source_files};
use crate::mlg::config::default_config_contents;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static NEXT_TEST_DIR_ID: AtomicUsize = AtomicUsize::new(0);

fn user_events(event_log: &EventLog) -> Vec<Event> {
    event_log
        .events()
        .iter()
        .filter_map(|event| {
            event
                .as_message()
                .and_then(|message| (message.audience == Audience::User).then(|| event.clone()))
        })
        .collect()
}

fn has_user_error_at(
    event_log: &EventLog,
    path: &Path,
    row: usize,
    column: usize,
    message: &str,
) -> bool {
    event_log
        .events()
        .iter()
        .filter_map(Event::as_message)
        .any(|event| {
            event.message == message
                && event.location.as_ref().is_some_and(|location| {
                    matches!(
                        location,
                        crate::events::EventLocation::File {
                            path: event_path,
                            span: Some(span)
                        } if event_path == path
                            && span.start.row == Some(row)
                            && span.start.column == Some(column)
                    )
                })
        })
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
            "mlg-check-test-{}-{}-{}",
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


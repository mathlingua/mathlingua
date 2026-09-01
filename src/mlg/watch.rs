use crate::backend::collection::{CONTENT_DIR, find_collection_root};
use crate::backend::config::CONFIG_FILE;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const POLL_INTERVAL: Duration = Duration::from_millis(250);
const SETTLE_INTERVAL: Duration = Duration::from_millis(100);

pub(super) struct SourceWatcher {
    cwd: PathBuf,
    extra_paths: Vec<PathBuf>,
    fingerprint: SourceFingerprint,
}

impl SourceWatcher {
    pub(super) fn new(cwd: &Path) -> Self {
        Self::with_paths(cwd, &[])
    }

    pub(super) fn with_paths(cwd: &Path, paths: &[PathBuf]) -> Self {
        let extra_paths = paths
            .iter()
            .map(|path| {
                if path.is_absolute() {
                    path.clone()
                } else {
                    cwd.join(path)
                }
            })
            .collect::<Vec<_>>();
        Self {
            cwd: cwd.to_path_buf(),
            fingerprint: source_fingerprint(cwd, &extra_paths),
            extra_paths,
        }
    }

    pub(super) fn reset(&mut self) {
        self.fingerprint = source_fingerprint(&self.cwd, &self.extra_paths);
    }

    pub(super) fn wait_for_change(&mut self) {
        loop {
            thread::sleep(POLL_INTERVAL);
            let fingerprint = source_fingerprint(&self.cwd, &self.extra_paths);
            if fingerprint == self.fingerprint {
                continue;
            }

            self.fingerprint = fingerprint;
            self.wait_until_settled();
            return;
        }
    }

    pub(super) fn changed(&mut self) -> bool {
        let fingerprint = source_fingerprint(&self.cwd, &self.extra_paths);
        if fingerprint == self.fingerprint {
            return false;
        }

        self.fingerprint = fingerprint;
        true
    }

    fn wait_until_settled(&mut self) {
        loop {
            thread::sleep(SETTLE_INTERVAL);
            let fingerprint = source_fingerprint(&self.cwd, &self.extra_paths);
            if fingerprint == self.fingerprint {
                return;
            }
            self.fingerprint = fingerprint;
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct SourceFingerprint {
    root: PathBuf,
    files: Vec<FileFingerprint>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct FileFingerprint {
    path: PathBuf,
    len: u64,
    modified: SystemTime,
}

fn source_fingerprint(cwd: &Path, extra_paths: &[PathBuf]) -> SourceFingerprint {
    let start = cwd.canonicalize().unwrap_or_else(|_| cwd.to_path_buf());
    let root = find_collection_root(&start).unwrap_or(start);
    let source_root = if root.join(CONTENT_DIR).is_dir() {
        root.join(CONTENT_DIR)
    } else {
        root.clone()
    };
    let mut files = Vec::new();
    let mut visited_directories = BTreeSet::new();
    collect_fingerprint_entries(&source_root, &mut visited_directories, &mut files);

    let config = root.join(CONFIG_FILE);
    if config.is_file() {
        files.push(file_fingerprint(&config));
    }
    for path in extra_paths {
        collect_fingerprint_entries(path, &mut visited_directories, &mut files);
    }
    files.sort_by(|left, right| left.path.cmp(&right.path));
    files.dedup_by(|left, right| left.path == right.path);

    SourceFingerprint { root, files }
}

/// Records only filesystem metadata while watching. Rebuilding a
/// `SourceCollection` here would read and parse every source file several times
/// a second merely to learn that nothing changed.
fn collect_fingerprint_entries(
    path: &Path,
    visited_directories: &mut BTreeSet<PathBuf>,
    entries: &mut Vec<FileFingerprint>,
) {
    let Ok(metadata) = fs::metadata(path) else {
        return;
    };
    collect_fingerprint_entry(path, metadata, visited_directories, entries);
}

fn collect_fingerprint_entry(
    path: &Path,
    metadata: fs::Metadata,
    visited_directories: &mut BTreeSet<PathBuf>,
    entries: &mut Vec<FileFingerprint>,
) {
    if metadata.is_dir() {
        let normalized = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
        if !visited_directories.insert(normalized) {
            return;
        }
        entries.push(file_fingerprint_from_metadata(path, &metadata));

        let Ok(read_dir) = fs::read_dir(path) else {
            return;
        };
        let mut children = read_dir
            .flatten()
            .map(|entry| (entry.path(), entry.metadata()))
            .collect::<Vec<_>>();
        children.sort_by(|(left, _), (right, _)| left.cmp(right));
        for (child, metadata) in children {
            if let Ok(metadata) = metadata
                && (metadata.is_dir() || is_source_dependency(&child))
            {
                collect_fingerprint_entry(&child, metadata, visited_directories, entries);
            }
        }
    } else if metadata.is_file() && is_source_dependency(path) {
        entries.push(file_fingerprint_from_metadata(path, &metadata));
    }
}

fn is_source_dependency(path: &Path) -> bool {
    path.file_name().is_some_and(|name| name == "toc")
        || path
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| extension.eq_ignore_ascii_case("mlg"))
}

fn file_fingerprint(path: &Path) -> FileFingerprint {
    let metadata = fs::metadata(path).ok();
    FileFingerprint {
        path: path.to_path_buf(),
        len: metadata.as_ref().map_or(0, fs::Metadata::len),
        modified: metadata
            .and_then(|metadata| metadata.modified().ok())
            .unwrap_or(UNIX_EPOCH),
    }
}

fn file_fingerprint_from_metadata(path: &Path, metadata: &fs::Metadata) -> FileFingerprint {
    FileFingerprint {
        path: path.to_path_buf(),
        len: metadata.len(),
        modified: metadata.modified().unwrap_or(UNIX_EPOCH),
    }
}

#[cfg(test)]
mod tests {
    use super::SourceWatcher;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn test_dir(name: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        std::env::temp_dir().join(format!("mlg-watch-{name}-{}-{unique}", std::process::id()))
    }

    #[test]
    fn detects_collection_source_changes() {
        let root = test_dir("collection");
        let content = root.join("content");
        fs::create_dir_all(&content).unwrap();
        let mut watcher = SourceWatcher::new(&root);

        assert!(!watcher.changed());
        fs::write(content.join("example.mlg"), "Text: one\n").unwrap();
        assert!(watcher.changed());
        assert!(!watcher.changed());

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn detects_explicit_paths_outside_the_collection_content() {
        let root = test_dir("explicit");
        let content = root.join("content");
        let extra = root.join("notes.mlg");
        fs::create_dir_all(&content).unwrap();
        fs::write(&extra, "Text: one\n").unwrap();
        let mut watcher = SourceWatcher::with_paths(&root, &[extra.clone()]);

        fs::write(&extra, "Text: a longer value\n").unwrap();
        assert!(watcher.changed());

        let _ = fs::remove_dir_all(root);
    }
}

use crate::constants::{CONFIG_FILE, CONTENT_DIR};
use crate::events::EventLog;
use std::collections::BTreeSet;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Resolves the MathLingua source files requested by a command.
///
/// When no explicit paths are supplied, the nearest collection root is found by
/// searching ancestors for `mlg.json`, and all `.mlg` files under `content/` are
/// collected.  Explicit files and directories are resolved relative to `cwd`.
pub(crate) fn resolve_source_files(
    cwd: &Path,
    paths: &[PathBuf],
    event_log: &mut EventLog,
    origin: &str,
) -> Vec<PathBuf> {
    let mut files = BTreeSet::new();

    if paths.is_empty() {
        let Some(root) = find_collection_root(cwd) else {
            event_log.user_error_at_path(
                Some(origin),
                cwd.to_path_buf(),
                "Not inside a Mathlingua collection and no paths were provided",
            );
            return Vec::new();
        };

        event_log.system_debug(
            Some(origin),
            format!(
                "Using collection content directory {}",
                root.join(CONTENT_DIR).display()
            ),
        );
        collect_source_files(root.join(CONTENT_DIR), &mut files, event_log, origin);
    } else {
        for path in paths {
            if let Some(resolved_path) = resolve_input_path(cwd, path, event_log, origin) {
                collect_source_files(resolved_path, &mut files, event_log, origin);
            }
        }
    }

    files.into_iter().collect()
}

/// Resolves all source files in the current collection content directory.
pub(crate) fn resolve_collection_content_files(
    cwd: &Path,
    event_log: &mut EventLog,
    origin: &str,
) -> Vec<PathBuf> {
    resolve_source_files(cwd, &[], event_log, origin)
}

/// Finds the nearest ancestor directory containing the collection config file.
pub(crate) fn find_collection_root(start: &Path) -> Option<PathBuf> {
    start
        .ancestors()
        .find(|directory| directory.join(CONFIG_FILE).is_file())
        .map(Path::to_path_buf)
}

/// Resolves a user-supplied path against the command working directory.
fn resolve_input_path(
    cwd: &Path,
    path: &Path,
    event_log: &mut EventLog,
    origin: &str,
) -> Option<PathBuf> {
    let joined = if path.is_absolute() {
        path.to_path_buf()
    } else {
        cwd.join(path)
    };

    match joined.canonicalize() {
        Ok(path) => Some(path),
        Err(error) => {
            event_log.user_error_at_path(
                Some(origin),
                joined,
                format!("Failed to resolve path: {error}"),
            );
            None
        }
    }
}

/// Collects source files from a file or directory target.
///
/// Direct file targets must be `.mlg` files.  Directory targets are traversed
/// recursively and non-`.mlg` files inside them are ignored.
fn collect_source_files(
    target: PathBuf,
    files: &mut BTreeSet<PathBuf>,
    event_log: &mut EventLog,
    origin: &str,
) {
    match fs::metadata(&target) {
        Ok(metadata) if metadata.is_dir() => {
            collect_directory_source_files(&target, files, event_log, origin);
        }
        Ok(metadata) if metadata.is_file() => {
            if is_mathlingua_source_file(&target) {
                files.insert(target);
            } else {
                event_log.user_error_at_path(Some(origin), target, "Not a .mlg file");
            }
        }
        Ok(_) => event_log.user_error_at_path(Some(origin), target, "Unsupported filesystem entry"),
        Err(error) => event_log.user_error_at_path(
            Some(origin),
            target,
            format!("Failed to read path: {error}"),
        ),
    }
}

/// Recursively collects `.mlg` files from a directory in stable path order.
fn collect_directory_source_files(
    directory: &Path,
    files: &mut BTreeSet<PathBuf>,
    event_log: &mut EventLog,
    origin: &str,
) {
    let entries = match read_directory_entries(directory) {
        Ok(entries) => entries,
        Err(error) => {
            event_log.user_error_at_path(
                Some(origin),
                directory.to_path_buf(),
                format!("Failed to read directory: {error}"),
            );
            return;
        }
    };

    for entry in entries {
        let path = entry.path();

        if path.is_dir() {
            collect_directory_source_files(&path, files, event_log, origin);
        } else if path.is_file() && is_mathlingua_source_file(&path) {
            files.insert(path);
        }
    }
}

/// Reads directory entries and sorts them by path for deterministic traversal.
fn read_directory_entries(directory: &Path) -> io::Result<Vec<fs::DirEntry>> {
    let mut entries = fs::read_dir(directory)?.collect::<Result<Vec<_>, io::Error>>()?;
    entries.sort_by_key(fs::DirEntry::path);
    Ok(entries)
}

/// Returns true when a path has the `MathLingua` source extension.
fn is_mathlingua_source_file(path: &Path) -> bool {
    path.extension()
        .and_then(|value| value.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("mlg"))
}

use crate::backend::collection::{SourceCollection, find_collection_root};
use crate::backend::config::CONFIG_FILE;
use crate::backend::release::{ReleaseItem, build_release_items};
use crate::events::{EventLog, EventLogListener};
use crate::mlg::util::no_errors_since;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::Path;
use std::process::Command;

const ORIGIN: &str = "mlg_release";
const METADATA_DIR: &str = "metadata";
const ITEMS_DIR: &str = "items";
const COLLECTION_FILE: &str = "collection.json";

pub struct ReleaseResult {
    pub event_log: EventLog,
    pub successful: bool,
}

/// One entry in `metadata/collection.json`, appended per release.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct CollectionEntry {
    version: u64,
    version_control_sha256: String,
    summary: String,
}

/// One entry in `metadata/items/<id>.json`, appended when an item is (re)versioned.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct ItemEntry {
    version: u64,
    sha256: String,
    repo_version: u64,
}

pub fn release(
    cwd: &Path,
    summary: &str,
    listener: Option<Box<dyn EventLogListener>>,
) -> ReleaseResult {
    let mut event_log = EventLog::new();
    if let Some(listener) = listener {
        event_log.add_boxed_listener(listener);
    }

    let starting_event_count = event_log.events().len();
    let io_ok = release_in(cwd, summary, &mut event_log).is_ok();
    let successful = io_ok && no_errors_since(&event_log, starting_event_count);

    ReleaseResult {
        event_log,
        successful,
    }
}

fn release_in(cwd: &Path, summary: &str, event_log: &mut EventLog) -> io::Result<()> {
    let start = cwd.canonicalize().unwrap_or_else(|_| cwd.to_path_buf());
    let Some(root) = find_collection_root(&start) else {
        event_log.user_error(
            Some(ORIGIN),
            "Could not find an mlg.json; run `mlg release` inside a Mathlingua collection",
        );
        return Err(io::Error::other("no collection root"));
    };

    // 1. The repository must be a clean Git work tree before we snapshot it.
    ensure_git_repository(&root, event_log)?;
    ensure_clean_worktree(&root, event_log)?;
    let head_sha = head_commit_sha(&root, event_log)?;

    // 2. The collection must check cleanly. `run_check_passes` also generates any
    //    missing `Id:` sections, so every item is guaranteed an id afterward.
    let check_start = event_log.events().len();
    let mut collection = SourceCollection::load(&root, event_log, ORIGIN);
    collection.run_check_passes(event_log, ORIGIN);
    if !no_errors_since(event_log, check_start) {
        event_log.user_error(
            Some(ORIGIN),
            "Release stopped because `mlg check` reported errors",
        );
        return Err(io::Error::other("check failed"));
    }

    // 3. Build the item dependency graph and hash each item's current content.
    let items = build_release_items(collection.parsed_files());
    let shas = items
        .iter()
        .map(|item| sha256_hex(&item.source))
        .collect::<Vec<_>>();

    // 4. Read the existing metadata (missing files read as empty histories).
    let metadata_dir = root.join(METADATA_DIR);
    let items_dir = metadata_dir.join(ITEMS_DIR);
    let collection_path = metadata_dir.join(COLLECTION_FILE);

    let mut collection_entries = read_json_array::<CollectionEntry>(&collection_path, event_log)?;
    let existing_item_entries = items
        .iter()
        .map(|item| {
            read_json_array::<ItemEntry>(&items_dir.join(item_file_name(&item.id)), event_log)
        })
        .collect::<io::Result<Vec<_>>>()?;

    // 5. Compute the next repo version from mlg.json.
    let config_path = root.join(CONFIG_FILE);
    let (current_version, config_value) = read_repo_version(&config_path, event_log)?;
    let new_repo_version = current_version + 1;

    // 6. Decide which items to (re)version, entirely in memory before writing.
    let changed = items
        .iter()
        .zip(&existing_item_entries)
        .zip(&shas)
        .map(|((_, entries), sha)| latest_sha(entries) != Some(sha.as_str()))
        .collect::<Vec<_>>();
    let update = compute_update_closure(&items, &changed);

    // 7. Write the new metadata, then bump mlg.json last.
    let mut updated_count = 0;
    for (index, item) in items.iter().enumerate() {
        if !update[index] {
            continue;
        }
        let mut entries = existing_item_entries[index].clone();
        let next_version = entries.iter().map(|entry| entry.version).max().unwrap_or(0) + 1;
        entries.push(ItemEntry {
            version: next_version,
            sha256: shas[index].clone(),
            repo_version: new_repo_version,
        });
        write_json(&items_dir.join(item_file_name(&item.id)), &entries)?;
        updated_count += 1;
    }

    collection_entries.push(CollectionEntry {
        version: new_repo_version,
        version_control_sha256: head_sha.clone(),
        summary: summary.to_string(),
    });
    write_json(&collection_path, &collection_entries)?;
    write_repo_version(&config_path, config_value, new_repo_version)?;

    event_log.user_log(
        Some(ORIGIN),
        format!("Released repo version {new_repo_version} at commit {head_sha}"),
    );
    event_log.user_log(
        Some(ORIGIN),
        format!(
            "Recorded {updated_count} of {} item(s) in {}/",
            items.len(),
            METADATA_DIR
        ),
    );

    Ok(())
}

/// The transitive set of items to (re)version this release: every changed item,
/// plus every definition reachable from a changed definition through `uses`
/// edges. Each item is included at most once (the dedup the spec requires).
/// Non-definitions never propagate.
fn compute_update_closure(items: &[ReleaseItem], changed: &[bool]) -> Vec<bool> {
    let index_by_id = items
        .iter()
        .enumerate()
        .map(|(index, item)| (item.id.as_str(), index))
        .collect::<HashMap<_, _>>();

    let mut in_update = vec![false; items.len()];
    let mut worklist = changed
        .iter()
        .enumerate()
        .filter_map(|(index, &is_changed)| is_changed.then_some(index))
        .collect::<Vec<_>>();

    while let Some(index) = worklist.pop() {
        if in_update[index] {
            continue;
        }
        in_update[index] = true;

        if items[index].is_definition {
            for used_id in &items[index].uses {
                if let Some(&target) = index_by_id.get(used_id.as_str()) {
                    if !in_update[target] {
                        worklist.push(target);
                    }
                }
            }
        }
    }

    in_update
}

fn latest_sha(entries: &[ItemEntry]) -> Option<&str> {
    entries
        .iter()
        .max_by_key(|entry| entry.version)
        .map(|entry| entry.sha256.as_str())
}

fn ensure_git_repository(root: &Path, event_log: &mut EventLog) -> io::Result<()> {
    match git_output(root, &["rev-parse", "--is-inside-work-tree"]) {
        Ok(output)
            if output.status.success()
                && String::from_utf8_lossy(&output.stdout).trim() == "true" =>
        {
            Ok(())
        }
        Ok(_) => {
            event_log.user_error(
                Some(ORIGIN),
                "`mlg release` requires the collection to be inside a Git repository",
            );
            Err(io::Error::other("not a git repository"))
        }
        Err(error) => {
            event_log.user_error(Some(ORIGIN), format!("Failed to run git: {error}"));
            Err(error)
        }
    }
}

fn ensure_clean_worktree(root: &Path, event_log: &mut EventLog) -> io::Result<()> {
    let output = match git_output(root, &["status", "--porcelain"]) {
        Ok(output) => output,
        Err(error) => {
            event_log.user_error(Some(ORIGIN), format!("Failed to run git: {error}"));
            return Err(error);
        }
    };

    if !output.status.success() {
        event_log.user_error(Some(ORIGIN), "Failed to read Git status");
        return Err(io::Error::other("git status failed"));
    }

    if !String::from_utf8_lossy(&output.stdout).trim().is_empty() {
        event_log.user_error(
            Some(ORIGIN),
            "The repository has uncommitted changes; commit or stash them before releasing",
        );
        return Err(io::Error::other("uncommitted changes"));
    }

    Ok(())
}

fn head_commit_sha(root: &Path, event_log: &mut EventLog) -> io::Result<String> {
    let output = match git_output(root, &["rev-parse", "HEAD"]) {
        Ok(output) => output,
        Err(error) => {
            event_log.user_error(Some(ORIGIN), format!("Failed to run git: {error}"));
            return Err(error);
        }
    };

    if !output.status.success() {
        event_log.user_error(Some(ORIGIN), "The repository has no commits to release");
        return Err(io::Error::other("no HEAD commit"));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn git_output(root: &Path, args: &[&str]) -> io::Result<std::process::Output> {
    Command::new("git").arg("-C").arg(root).args(args).output()
}

fn read_repo_version(
    path: &Path,
    event_log: &mut EventLog,
) -> io::Result<(u64, serde_json::Value)> {
    let contents = fs::read_to_string(path)?;
    let value: serde_json::Value = match serde_json::from_str(&contents) {
        Ok(value) => value,
        Err(error) => {
            event_log.user_error(
                Some(ORIGIN),
                format!("Invalid JSON in {CONFIG_FILE}: {error}"),
            );
            return Err(io::Error::other("invalid mlg.json"));
        }
    };

    let version_text = value
        .get("version")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("0");
    match version_text.parse::<u64>() {
        Ok(version) => Ok((version, value)),
        Err(_) => {
            event_log.user_error(
                Some(ORIGIN),
                format!("{CONFIG_FILE} version \"{version_text}\" is not a non-negative integer"),
            );
            Err(io::Error::other("invalid version"))
        }
    }
}

fn write_repo_version(
    path: &Path,
    mut value: serde_json::Value,
    new_version: u64,
) -> io::Result<()> {
    let object = value
        .as_object_mut()
        .ok_or_else(|| io::Error::other(format!("{CONFIG_FILE} must be a JSON object")))?;
    object.insert(
        "version".to_string(),
        serde_json::Value::String(new_version.to_string()),
    );
    write_json(path, &value)
}

fn read_json_array<T: DeserializeOwned>(
    path: &Path,
    event_log: &mut EventLog,
) -> io::Result<Vec<T>> {
    if !path.exists() {
        return Ok(Vec::new());
    }

    let contents = fs::read_to_string(path)?;
    match serde_json::from_str(&contents) {
        Ok(entries) => Ok(entries),
        Err(error) => {
            event_log.user_error_at_path(
                Some(ORIGIN),
                path.to_path_buf(),
                format!("Failed to parse release metadata: {error}"),
            );
            Err(io::Error::other("invalid release metadata"))
        }
    }
}

fn write_json<T: Serialize>(path: &Path, value: &T) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut contents = serde_json::to_string_pretty(value).map_err(|error| {
        io::Error::other(format!("Failed to serialize release metadata: {error}"))
    })?;
    contents.push('\n');
    fs::write(path, contents)
}

fn item_file_name(id: &str) -> String {
    format!("{id}.json")
}

fn sha256_hex(content: &str) -> String {
    Sha256::digest(content.as_bytes())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

// ===============================[ tests ]=====================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn item(id: &str, is_definition: bool, uses: &[&str]) -> ReleaseItem {
        ReleaseItem {
            id: id.to_string(),
            path: PathBuf::from("content/x.mlg"),
            kind: if is_definition {
                "Describes"
            } else {
                "Theorem"
            }
            .to_string(),
            source: String::new(),
            is_definition,
            uses: uses.iter().map(|value| value.to_string()).collect(),
        }
    }

    fn updated_ids<'a>(items: &'a [ReleaseItem], update: &[bool]) -> Vec<&'a str> {
        items
            .iter()
            .zip(update)
            .filter_map(|(item, &included)| included.then_some(item.id.as_str()))
            .collect()
    }

    #[test]
    fn sha256_matches_known_vectors() {
        assert_eq!(
            sha256_hex(""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert_eq!(
            sha256_hex("abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn latest_sha_uses_the_highest_version() {
        let entries = vec![
            ItemEntry {
                version: 1,
                sha256: "aa".to_string(),
                repo_version: 1,
            },
            ItemEntry {
                version: 2,
                sha256: "bb".to_string(),
                repo_version: 3,
            },
        ];
        assert_eq!(latest_sha(&entries), Some("bb"));
        assert_eq!(latest_sha(&[]), None);
    }

    #[test]
    fn changed_definition_propagates_to_shared_dependencies_once() {
        // A uses {B, C}; D uses {B, C}. Changing A and B re-versions B and C once.
        let items = vec![
            item("A", true, &["B", "C"]),
            item("B", true, &[]),
            item("C", true, &[]),
            item("D", true, &["B", "C"]),
        ];
        let changed = vec![true, true, false, false];

        let update = compute_update_closure(&items, &changed);

        let mut ids = updated_ids(&items, &update);
        ids.sort_unstable();
        assert_eq!(ids, vec!["A", "B", "C"]);
    }

    #[test]
    fn propagation_cascades_transitively() {
        // A uses B, B uses C; only A changed. C must still be re-versioned.
        let items = vec![
            item("A", true, &["B"]),
            item("B", true, &["C"]),
            item("C", true, &[]),
        ];
        let changed = vec![true, false, false];

        let update = compute_update_closure(&items, &changed);

        let mut ids = updated_ids(&items, &update);
        ids.sort_unstable();
        assert_eq!(ids, vec!["A", "B", "C"]);
    }

    #[test]
    fn non_definitions_do_not_propagate() {
        // A changed Theorem uses \b, but a theorem has no dependencies to update.
        let items = vec![item("T", false, &["B"]), item("B", true, &[])];
        let changed = vec![true, false];

        let update = compute_update_closure(&items, &changed);

        assert_eq!(updated_ids(&items, &update), vec!["T"]);
    }

    #[test]
    fn unchanged_items_are_left_alone() {
        let items = vec![item("A", true, &["B"]), item("B", true, &[])];
        let changed = vec![false, false];

        let update = compute_update_closure(&items, &changed);

        assert!(updated_ids(&items, &update).is_empty());
    }

    const A_ID: &str = "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa";
    const B_ID: &str = "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb";

    /// `\a` (Describes A) uses `\b` (Describes B); `called` controls A's content.
    fn defs_source(a_called: &str) -> String {
        format!(
            "[\\b]\nDescribes: B\nDocumented:\n. called: \"b\"\nId: \"{B_ID}\"\n\n\n\
             [\\a]\nDescribes: A\nextends: A is \\b\nDocumented:\n. called: \"{a_called}\"\nId: \"{A_ID}\"\n"
        )
    }

    #[test]
    fn releases_metadata_and_propagates_over_a_git_repository() {
        if !git_available() {
            return; // git is required by the command; skip where it is unavailable
        }

        let dir = TempDir::new();
        let root = dir.path();
        init_git_collection(root, &defs_source("a"));

        // First release: every item is new, so both get version 1 / repo_version 1.
        let first = release(root, "first cut", None);
        assert!(first.successful, "{:#?}", first.event_log.events());

        let collection = read_entries::<CollectionEntry>(&root.join("metadata/collection.json"));
        assert_eq!(collection.len(), 1);
        assert_eq!(collection[0].version, 1);
        assert_eq!(collection[0].summary, "first cut");
        assert_eq!(
            collection[0].version_control_sha256,
            head_sha_of(root),
            "records the HEAD commit"
        );
        assert!(read_config_version(root).contains("\"1\""));

        let a_first = read_entries::<ItemEntry>(&item_json(root, A_ID));
        let b_first = read_entries::<ItemEntry>(&item_json(root, B_ID));
        assert_eq!(a_first.len(), 1);
        assert_eq!(b_first.len(), 1);

        // Commit the release output, then change only A's content and commit.
        commit_all(root, "release 1");
        fs::write(root.join("content/defs.mlg"), defs_source("a changed")).unwrap();
        commit_all(root, "edit A");

        // Second release: A changed, and A (a definition) uses \b, so B is
        // re-versioned by propagation even though its content did not change.
        let second = release(root, "second cut", None);
        assert!(second.successful, "{:#?}", second.event_log.events());

        let a_second = read_entries::<ItemEntry>(&item_json(root, A_ID));
        let b_second = read_entries::<ItemEntry>(&item_json(root, B_ID));
        assert_eq!(a_second.len(), 2);
        assert_eq!(b_second.len(), 2);
        assert_eq!(a_second[1].repo_version, 2);
        assert_eq!(b_second[1].repo_version, 2);
        assert_ne!(a_second[0].sha256, a_second[1].sha256, "A changed");
        assert_eq!(b_second[0].sha256, b_second[1].sha256, "B only propagated");
        assert!(read_config_version(root).contains("\"2\""));
    }

    fn init_git_collection(root: &Path, defs: &str) {
        fs::create_dir_all(root.join("content")).unwrap();
        fs::write(
            root.join("mlg.json"),
            "{\n  \"name\": \"t\",\n  \"version\": \"0\"\n}\n",
        )
        .unwrap();
        fs::write(root.join("content/defs.mlg"), defs).unwrap();
        assert!(run_git(root, &["init", "-q"]));
        assert!(run_git(root, &["config", "user.email", "test@example.com"]));
        assert!(run_git(root, &["config", "user.name", "Test"]));
        commit_all(root, "init");
    }

    fn commit_all(root: &Path, message: &str) {
        assert!(run_git(root, &["add", "-A"]));
        assert!(run_git(root, &["commit", "-q", "-m", message]));
    }

    fn head_sha_of(root: &Path) -> String {
        let output = git_output(root, &["rev-parse", "HEAD"]).unwrap();
        String::from_utf8_lossy(&output.stdout).trim().to_string()
    }

    fn read_config_version(root: &Path) -> String {
        fs::read_to_string(root.join(CONFIG_FILE)).unwrap()
    }

    fn item_json(root: &Path, id: &str) -> std::path::PathBuf {
        root.join("metadata")
            .join(ITEMS_DIR)
            .join(item_file_name(id))
    }

    fn read_entries<T: DeserializeOwned>(path: &Path) -> Vec<T> {
        serde_json::from_str(&fs::read_to_string(path).expect("metadata file exists"))
            .expect("metadata parses")
    }

    fn run_git(root: &Path, args: &[&str]) -> bool {
        git_output(root, args)
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    fn git_available() -> bool {
        Command::new("git")
            .arg("--version")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    struct TempDir {
        path: std::path::PathBuf,
    }

    impl TempDir {
        fn new() -> Self {
            use std::sync::atomic::{AtomicUsize, Ordering};
            use std::time::{SystemTime, UNIX_EPOCH};
            static COUNTER: AtomicUsize = AtomicUsize::new(0);
            let unique = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let sequence = COUNTER.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                "mlg-release-test-{}-{unique}-{sequence}",
                std::process::id()
            ));
            fs::create_dir_all(&path).unwrap();
            Self { path }
        }

        fn path(&self) -> &Path {
            &self.path
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }
}

use crate::backend::collection::{SourceCollection, find_collection_root};
use crate::backend::config::CONFIG_FILE;
use crate::backend::release::{ReleaseItem, build_release_items, item_source_by_id};
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
    dry_run: bool,
    diff: bool,
    listener: Option<Box<dyn EventLogListener>>,
) -> ReleaseResult {
    let mut event_log = EventLog::new();
    if let Some(listener) = listener {
        event_log.add_boxed_listener(listener);
    }

    let starting_event_count = event_log.events().len();
    let io_ok = release_in(cwd, summary, dry_run, diff, &mut event_log).is_ok();
    let successful = io_ok && no_errors_since(&event_log, starting_event_count);

    ReleaseResult {
        event_log,
        successful,
    }
}

fn release_in(
    cwd: &Path,
    summary: &str,
    dry_run: bool,
    diff: bool,
    event_log: &mut EventLog,
) -> io::Result<()> {
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
    let previous_release_sha = collection_entries
        .last()
        .map(|entry| entry.version_control_sha256.clone());
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

    // 7. Write the new metadata (unless this is a dry run), then bump mlg.json.
    let mut updated = Vec::new();
    for (index, item) in items.iter().enumerate() {
        if !update[index] {
            continue;
        }
        let previous_version = existing_item_entries[index]
            .iter()
            .map(|entry| entry.version)
            .max();
        let new_version = previous_version.unwrap_or(0) + 1;
        if !dry_run {
            let mut entries = existing_item_entries[index].clone();
            entries.push(ItemEntry {
                version: new_version,
                sha256: shas[index].clone(),
                repo_version: new_repo_version,
            });
            write_json(&items_dir.join(item_file_name(&item.id)), &entries)?;
        }
        updated.push(UpdatedItem {
            kind: item.kind.clone(),
            label: item_display_label(item),
            previous_version,
            new_version,
            content_changed: changed[index],
        });
    }

    if !dry_run {
        collection_entries.push(CollectionEntry {
            version: new_repo_version,
            version_control_sha256: head_sha.clone(),
            summary: summary.to_string(),
        });
        write_json(&collection_path, &collection_entries)?;
        write_repo_version(&config_path, config_value, new_repo_version)?;
    }

    // 8. Report what happened (or would happen), optionally with per-item diffs.
    let mut report = format_release_report(
        new_repo_version,
        &head_sha,
        summary,
        &updated,
        items.len(),
        dry_run,
    );
    if diff {
        let item_diffs =
            compute_item_diffs(&root, previous_release_sha.as_deref(), &items, &changed);
        report.push_str("\n\n");
        report.push_str(&format_diff_section(&item_diffs));
    }
    event_log.user_log(Some(ORIGIN), report);

    Ok(())
}

/// A per-item diff for `mlg release --diff`: the line-level changes from an
/// item's previous released contents to its current contents. Each item's source
/// slice already begins with its heading and ends with its `Id:`, so the diff is
/// self-identifying and needs no separate label.
type ItemDiff = Vec<(char, String)>;

/// Build a diff for every item whose contents changed this release, comparing
/// its current source against its source at the previous release's commit, in
/// collection order.
fn compute_item_diffs(
    root: &Path,
    previous_release_sha: Option<&str>,
    items: &[ReleaseItem],
    changed: &[bool],
) -> Vec<ItemDiff> {
    let mut old_files: HashMap<&Path, Option<String>> = HashMap::new();
    let mut diffs = Vec::new();

    for (index, item) in items.iter().enumerate() {
        if !changed[index] {
            continue;
        }
        let old_file = old_files
            .entry(item.path.as_path())
            .or_insert_with(|| show_file_at_commit(root, previous_release_sha, &item.path));
        let old_source = old_file
            .as_deref()
            .and_then(|contents| item_source_by_id(contents, &item.id));

        diffs.push(diff_lines(
            old_source.as_deref().unwrap_or(""),
            &item.source,
        ));
    }

    diffs
}

/// The full contents of `path` as of commit `sha`, via `git show`, or `None`
/// when there is no previous release or the file did not exist then.
fn show_file_at_commit(root: &Path, sha: Option<&str>, path: &Path) -> Option<String> {
    let sha = sha?;
    let relative = path.strip_prefix(root).ok()?;
    let spec = format!("{sha}:{}", relative.to_string_lossy().replace('\\', "/"));
    let output = git_output(root, &["show", &spec]).ok()?;
    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).into_owned())
}

/// A line-level diff of `old` against `new` as `(marker, line)` pairs, where the
/// marker is `' '` (unchanged), `'-'` (removed), or `'+'` (added). Items are
/// small, so the whole item is shown rather than collapsed hunks.
fn diff_lines(old: &str, new: &str) -> Vec<(char, String)> {
    let old_lines = old.lines().collect::<Vec<_>>();
    let new_lines = new.lines().collect::<Vec<_>>();
    let (rows, cols) = (old_lines.len(), new_lines.len());

    // Longest-common-subsequence lengths, filled from the bottom-right.
    let mut lcs = vec![vec![0usize; cols + 1]; rows + 1];
    for row in (0..rows).rev() {
        for col in (0..cols).rev() {
            lcs[row][col] = if old_lines[row] == new_lines[col] {
                lcs[row + 1][col + 1] + 1
            } else {
                lcs[row + 1][col].max(lcs[row][col + 1])
            };
        }
    }

    let mut result = Vec::new();
    let (mut row, mut col) = (0, 0);
    while row < rows && col < cols {
        if old_lines[row] == new_lines[col] {
            result.push((' ', old_lines[row].to_string()));
            row += 1;
            col += 1;
        } else if lcs[row + 1][col] >= lcs[row][col + 1] {
            result.push(('-', old_lines[row].to_string()));
            row += 1;
        } else {
            result.push(('+', new_lines[col].to_string()));
            col += 1;
        }
    }
    for line in &old_lines[row..] {
        result.push(('-', (*line).to_string()));
    }
    for line in &new_lines[col..] {
        result.push(('+', (*line).to_string()));
    }
    result
}

/// Render the optional diff section shown under the release summary as a
/// unified diff: one block per changed item (blank-line separated), each line
/// prefixed with `' '`, `'-'`, or `'+'`. A wholly new item shows every line as
/// added.
fn format_diff_section(diffs: &[ItemDiff]) -> String {
    if diffs.is_empty() {
        return "Diffs\n\n  (no item contents changed)".to_string();
    }

    let mut section = String::from("Diffs");
    for diff in diffs {
        section.push('\n'); // blank line before each item's diff block
        for (marker, content) in diff {
            section.push_str(&format!("\n{marker} {content}"));
        }
    }
    section
}

/// One updated item, as shown in the release report.
struct UpdatedItem {
    kind: String,
    label: String,
    previous_version: Option<u64>,
    new_version: u64,
    /// `true` when the item's own contents changed; `false` when it is only
    /// re-versioned because a definition that uses it changed (propagation).
    content_changed: bool,
}

/// Page-content kinds whose body text is previewed instead of a bracket heading.
const PAGE_KINDS: [&str; 4] = ["Title", "SectionTitle", "SubsectionTitle", "Text"];

/// The order top-level kinds are grouped in the release report. Kinds not listed
/// here sort after these, alphabetically.
const KIND_ORDER: [&str; 18] = [
    "Defines",
    "Describes",
    "States",
    "Refines",
    "Disambiguates",
    "Axiom",
    "Theorem",
    "Corollary",
    "Relation",
    "Equivalent",
    "Topic",
    "Title",
    "SectionTitle",
    "SubsectionTitle",
    "Text",
    "Person",
    "Resource",
    "Specify",
];

const PREVIEW_MAX: usize = 50;

/// How an item is labelled in the report: its bracket heading if it has one, a
/// truncated preview of its text for page content, else its id.
fn item_display_label(item: &ReleaseItem) -> String {
    if let Some(header) = item.header.as_deref() {
        return format!("[{header}]");
    }
    if PAGE_KINDS.contains(&item.kind.as_str()) {
        if let Some(preview) = item.preview.as_deref() {
            let preview = truncate_preview(preview, PREVIEW_MAX);
            if !preview.is_empty() {
                return format!("\u{201c}{preview}\u{201d}");
            }
        }
    }
    item.id.clone()
}

fn truncate_preview(text: &str, max: usize) -> String {
    let collapsed = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if collapsed.chars().count() <= max {
        collapsed
    } else {
        let head = collapsed.chars().take(max).collect::<String>();
        format!("{}\u{2026}", head.trim_end())
    }
}

fn kind_rank(kind: &str) -> usize {
    KIND_ORDER
        .iter()
        .position(|known| *known == kind)
        .unwrap_or(KIND_ORDER.len())
}

fn version_transition(item: &UpdatedItem) -> String {
    match item.previous_version {
        Some(version) => format!("v{version}"),
        None => "new".to_string(),
    }
}

/// The `previous → new` version cell, e.g. `v1 → v2` or `new → v1`, right-aligned
/// on its left edge so cells share a width.
fn version_cell(item: &UpdatedItem, from_width: usize) -> String {
    format!(
        "{:>from_width$} \u{2192} v{}",
        version_transition(item),
        item.new_version,
    )
}

fn item_noun(count: usize) -> &'static str {
    if count == 1 { "item" } else { "items" }
}

/// Render the clean, grouped release summary printed to the user.
fn format_release_report(
    new_version: u64,
    sha: &str,
    summary: &str,
    updated: &[UpdatedItem],
    total_items: usize,
    dry_run: bool,
) -> String {
    let mut report = if dry_run {
        format!("Would release repo version {new_version}  (dry run \u{2014} no files written)\n")
    } else {
        format!("Released repo version {new_version}\n")
    };
    report.push_str(&format!("Commit   {sha}\n"));

    // Split the update set into content changes and transitive (propagation)
    // bumps. The tags are only worth showing when both kinds are present.
    let changed_count = updated.iter().filter(|item| item.content_changed).count();
    let propagated_count = updated.len() - changed_count;
    let show_tags = changed_count > 0 && propagated_count > 0;

    report.push_str(&format!(
        "Updated  {} of {total_items} {}",
        updated.len(),
        item_noun(total_items),
    ));
    if show_tags {
        report.push_str(&format!(
            " ({changed_count} changed, {propagated_count} propagated)"
        ));
    }
    report.push('\n');
    report.push_str(&format!("Summary  {summary}"));

    if updated.is_empty() {
        return report;
    }

    // Align the label and version columns across the whole report so every
    // transition (and any trailing tag) lines up, regardless of kind group.
    let label_width = updated
        .iter()
        .map(|item| item.label.chars().count())
        .max()
        .unwrap_or(0);
    let from_width = updated
        .iter()
        .map(|item| version_transition(item).chars().count())
        .max()
        .unwrap_or(0);
    let cell_width = updated
        .iter()
        .map(|item| version_cell(item, from_width).chars().count())
        .max()
        .unwrap_or(0);

    // Group by kind, ordering the groups by `KIND_ORDER` then name while keeping
    // each item in its original collection order within the group.
    let mut kinds: Vec<&str> = Vec::new();
    let mut by_kind: HashMap<&str, Vec<&UpdatedItem>> = HashMap::new();
    for item in updated {
        if !by_kind.contains_key(item.kind.as_str()) {
            kinds.push(item.kind.as_str());
        }
        by_kind.entry(item.kind.as_str()).or_default().push(item);
    }
    kinds.sort_by(|left, right| kind_rank(left).cmp(&kind_rank(right)).then(left.cmp(right)));

    for kind in kinds {
        report.push_str(&format!("\n\n{kind}"));
        for item in &by_kind[kind] {
            let cell = version_cell(item, from_width);
            if show_tags {
                let tag = if item.content_changed {
                    "changed"
                } else {
                    "propagated"
                };
                report.push_str(&format!(
                    "\n  {label:<label_width$}   {cell:<cell_width$}   {tag}",
                    label = item.label,
                ));
            } else {
                report.push_str(&format!(
                    "\n  {label:<label_width$}   {cell}",
                    label = item.label
                ));
            }
        }
    }

    report
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
            header: None,
            preview: None,
            source: String::new(),
            is_definition,
            uses: uses.iter().map(|value| value.to_string()).collect(),
        }
    }

    fn labelled(kind: &str, header: Option<&str>, preview: Option<&str>) -> ReleaseItem {
        ReleaseItem {
            id: "1de3c455-b09b-4d80-8b44-7dd223da2083".to_string(),
            path: PathBuf::from("content/x.mlg"),
            kind: kind.to_string(),
            header: header.map(str::to_string),
            preview: preview.map(str::to_string),
            source: String::new(),
            is_definition: false,
            uses: Vec::new(),
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

    #[test]
    fn label_prefers_header_then_preview_then_id() {
        assert_eq!(
            item_display_label(&labelled("Describes", Some("\\set"), None)),
            "[\\set]"
        );
        assert_eq!(
            item_display_label(&labelled("Title", None, Some("Hello World"))),
            "\u{201c}Hello World\u{201d}"
        );
        // A non-page kind (or a page kind with no text) falls back to the id.
        let fallback = labelled("Theorem", None, None);
        assert_eq!(item_display_label(&fallback), fallback.id);
    }

    #[test]
    fn truncate_preview_collapses_whitespace_and_adds_ellipsis() {
        assert_eq!(truncate_preview("  a\n\n  b   c ", 50), "a b c");
        let long = "word ".repeat(40);
        let truncated = truncate_preview(&long, 10);
        assert!(truncated.ends_with('\u{2026}'));
        assert!(truncated.chars().count() <= 11);
    }

    #[test]
    fn report_groups_by_kind_in_order_with_version_transitions() {
        let updated = vec![
            UpdatedItem {
                kind: "Text".to_string(),
                label: "\u{201c}Intro\u{201d}".to_string(),
                previous_version: Some(1),
                new_version: 2,
                content_changed: true,
            },
            UpdatedItem {
                kind: "Describes".to_string(),
                label: "[\\set]".to_string(),
                previous_version: None,
                new_version: 1,
                content_changed: true,
            },
            UpdatedItem {
                kind: "Axiom".to_string(),
                label: "[\\axiom.of.extension]".to_string(),
                previous_version: Some(2),
                new_version: 3,
                content_changed: false,
            },
        ];

        let report = format_release_report(4, "abcdef0", "release notes", &updated, 10, false);

        assert!(report.starts_with("Released repo version 4\n"));
        assert!(
            !report.contains("(was"),
            "the previous version is not shown"
        );
        assert!(report.contains("Commit   abcdef0\n"));
        // Two contents changed, one is a propagation bump.
        assert!(report.contains("Updated  3 of 10 items (2 changed, 1 propagated)\n"));
        assert!(report.contains("Summary  release notes"));
        // `Updated` is listed before `Summary`.
        assert!(report.find("Updated  ").unwrap() < report.find("Summary  ").unwrap());
        // Groups appear in KIND_ORDER: Describes, then Axiom, then Text. Each
        // kind heading sits at column 0 (preceded by a blank line).
        let describes = report.find("\n\nDescribes\n").unwrap();
        let axiom = report.find("\n\nAxiom\n").unwrap();
        let text = report.find("\n\nText\n").unwrap();
        assert!(describes < axiom && axiom < text, "{report}");
        assert!(report.contains("new \u{2192} v1"));
        assert!(report.contains("v1 \u{2192} v2"));
        assert!(report.contains("v2 \u{2192} v3"));
        // The version column is aligned globally: every " → v" sits at the same
        // character column, across all groups (measured in chars, not bytes,
        // since labels contain multibyte characters).
        let arrow_columns = report
            .lines()
            .filter(|line| line.contains(" \u{2192} v"))
            .map(|line| {
                let byte = line.find(" \u{2192} v").unwrap();
                line[..byte].chars().count()
            })
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(arrow_columns.len(), 1, "{report}");
        // Content changes are tagged `changed`; propagation bumps `propagated`.
        let set_line = report
            .lines()
            .find(|line| line.contains("[\\set]"))
            .unwrap();
        assert!(set_line.ends_with("changed"), "{set_line}");
        let axiom_line = report
            .lines()
            .find(|line| line.contains("[\\axiom.of.extension]"))
            .unwrap();
        assert!(axiom_line.ends_with("propagated"), "{axiom_line}");
    }

    #[test]
    fn report_omits_tags_when_nothing_was_propagated() {
        // Every update is a real content change (e.g. a first release), so there
        // is no changed-vs-propagated distinction to draw.
        let updated = vec![
            UpdatedItem {
                kind: "Describes".to_string(),
                label: "[\\set]".to_string(),
                previous_version: None,
                new_version: 1,
                content_changed: true,
            },
            UpdatedItem {
                kind: "Axiom".to_string(),
                label: "[\\ax]".to_string(),
                previous_version: None,
                new_version: 1,
                content_changed: true,
            },
        ];

        let report = format_release_report(1, "sha", "s", &updated, 2, false);

        assert!(report.contains("Updated  2 of 2 items\n"), "{report}");
        assert!(!report.contains("propagated"), "{report}");
        assert!(!report.contains("changed"), "{report}");
    }

    #[test]
    fn report_with_no_updates_omits_the_item_list() {
        let report = format_release_report(2, "abc123", "empty", &[], 5, false);
        assert!(report.contains("Updated  0 of 5 items"));
        assert!(!report.contains('\u{2192}'));
    }

    #[test]
    fn report_uses_the_singular_noun_for_a_single_total_item() {
        let report = format_release_report(1, "sha", "note", &[], 1, false);
        assert!(report.contains("Updated  0 of 1 item\n"), "{report}");
        assert!(!report.contains("item(s)"));
    }

    #[test]
    fn dry_run_report_uses_the_would_release_title() {
        let report = format_release_report(3, "sha", "note", &[], 5, true);
        assert!(
            report.starts_with("Would release repo version 3"),
            "{report}"
        );
        assert!(report.contains("dry run"), "{report}");
    }

    #[test]
    fn diff_lines_marks_added_removed_and_unchanged_lines() {
        let lines = diff_lines("a\nb\nc\n", "a\nB\nc\n");
        assert_eq!(
            lines,
            vec![
                (' ', "a".to_string()),
                ('-', "b".to_string()),
                ('+', "B".to_string()),
                (' ', "c".to_string()),
            ]
        );
    }

    #[test]
    fn diff_section_shows_new_items_as_fully_added() {
        let diffs = vec![diff_lines("", "[\\set]\nDescribes: X")];
        let section = format_diff_section(&diffs);
        assert!(section.starts_with("Diffs"));
        assert!(section.contains("\n+ [\\set]"), "{section}");
        assert!(section.contains("\n+ Describes: X"), "{section}");
    }

    #[test]
    fn diff_section_reports_when_nothing_changed() {
        assert!(format_diff_section(&[]).contains("(no item contents changed)"));
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
        let first = release(root, "first cut", false, false, None);
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
        let second = release(root, "second cut", false, false, None);
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

    #[test]
    fn dry_run_reports_without_writing_anything() {
        if !git_available() {
            return;
        }
        let dir = TempDir::new();
        let root = dir.path();
        init_git_collection(root, &defs_source("a"));

        let result = release(root, "preview", true, false, None);
        assert!(result.successful, "{:#?}", result.event_log.events());

        // A dry run writes nothing: no metadata directory, version left at 0.
        assert!(
            !root.join("metadata").exists(),
            "dry run must not write metadata"
        );
        assert!(
            read_config_version(root).contains("\"0\""),
            "dry run must not bump the version"
        );

        let report = report_text(&result);
        assert!(report.contains("Would release repo version 1"), "{report}");
        assert!(report.contains("dry run"), "{report}");
    }

    #[test]
    fn diff_flag_shows_changed_item_contents() {
        if !git_available() {
            return;
        }
        let dir = TempDir::new();
        let root = dir.path();
        init_git_collection(root, &defs_source("a"));

        // A real first release gives us a previous release commit to diff against.
        assert!(release(root, "first", false, false, None).successful);
        commit_all(root, "release 1");
        fs::write(root.join("content/defs.mlg"), defs_source("a changed")).unwrap();
        commit_all(root, "edit A");

        // Dry run with --diff previews the change to A without writing.
        let result = release(root, "preview", true, true, None);
        assert!(result.successful, "{:#?}", result.event_log.events());

        let a = read_entries::<ItemEntry>(&item_json(root, A_ID));
        assert_eq!(a.len(), 1, "dry run must not append an entry");

        let report = report_text(&result);
        // A changed; B is only re-versioned by propagation.
        assert!(report.contains("(1 changed, 1 propagated)"), "{report}");
        assert!(report.contains("Diffs"), "{report}");
        // Only A's contents changed; the diff shows the old and new `called` lines.
        assert!(report.contains("- . called: \"a\""), "{report}");
        assert!(report.contains("+ . called: \"a changed\""), "{report}");
    }

    fn report_text(result: &ReleaseResult) -> String {
        result
            .event_log
            .events()
            .iter()
            .filter_map(|event| event.as_message())
            .filter(|message| {
                message.audience == crate::events::Audience::User
                    && message.level == crate::events::Level::Log
            })
            .map(|message| message.message.clone())
            .collect::<Vec<_>>()
            .join("\n")
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

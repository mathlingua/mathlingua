use crate::backend::collection::find_collection_root;
use crate::backend::semantic::{DefinitionSite, find_definition};
use crate::events::EventLog;
use crate::frontend::{
    ParsedSourceFile, SourceFileViewMetadata, parse_document, top_level_item_ids,
};
use std::fs;
use std::path::{Path, PathBuf};

/// Resolve the command at byte `offset` within `target_source` (the in-memory
/// buffer of `target`) to its defining top-level item, searching every `.mlg`
/// file in the collection rooted at (or above) `root`.
///
/// The lookup is read-only: unlike a full `check`, it never writes generated
/// `Id:` sections back to disk. Definitions in other files are read from disk;
/// the target file uses `target_source`, so the result reflects unsaved edits
/// to the file being navigated.
pub fn resolve_definition(
    root: &Path,
    target: &Path,
    target_source: &str,
    offset: usize,
) -> Option<DefinitionSite> {
    let root = find_collection_root(root).unwrap_or_else(|| root.to_path_buf());
    let target_key = canonical(target);

    let mut files = Vec::new();
    let mut included_target = false;
    for path in mlg_files(&root) {
        if canonical(&path) == target_key {
            files.push(parse_in_memory(&path, target_source));
            included_target = true;
        } else if let Ok(source) = fs::read_to_string(&path) {
            files.push(parse_in_memory(&path, &source));
        }
    }
    // The target may be outside `root` (or a not-yet-saved buffer); make sure
    // its own headings are still searchable.
    if !included_target {
        files.push(parse_in_memory(target, target_source));
    }

    find_definition(&files, target_source, offset)
}

fn parse_in_memory(path: &Path, source: &str) -> ParsedSourceFile {
    let mut event_log = EventLog::new();
    ParsedSourceFile {
        path: path.to_path_buf(),
        source: source.to_string(),
        document: parse_document(source, &mut event_log),
        item_ids: top_level_item_ids(source),
        view_metadata: SourceFileViewMetadata::default(),
    }
}

fn canonical(path: &Path) -> PathBuf {
    fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

/// Every `.mlg` file under `root`, skipping hidden directories (e.g. `.git`).
fn mlg_files(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_mlg_files(root, &mut files);
    files
}

fn collect_mlg_files(dir: &Path, files: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        let path = entry.path();
        if file_type.is_dir() {
            let hidden = path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with('.'));
            if !hidden {
                collect_mlg_files(&path, files);
            }
        } else if file_type.is_file() && is_mlg(&path) {
            files.push(path);
        }
    }
}

fn is_mlg(path: &Path) -> bool {
    path.extension()
        .and_then(|value| value.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("mlg"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static NEXT_ID: AtomicUsize = AtomicUsize::new(0);

    struct TempProject {
        root: PathBuf,
    }

    impl TempProject {
        fn new() -> Self {
            let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
            let root = std::env::temp_dir().join(format!(
                "mlg-definition-test-{}-{id}",
                std::process::id()
            ));
            fs::create_dir_all(root.join("content")).unwrap();
            fs::write(root.join("mlg.json"), "{}\n").unwrap();
            Self { root }
        }

        fn write(&self, name: &str, contents: &str) -> PathBuf {
            let path = self.root.join("content").join(name);
            fs::write(&path, contents).unwrap();
            path
        }
    }

    impl Drop for TempProject {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    #[test]
    fn resolves_across_files_on_disk() {
        let project = TempProject::new();
        project.write(
            "axioms.mlg",
            "[\\set]\nDescribes: S\nId: \"11111111-1111-4111-8111-111111111111\"\n",
        );
        let usage = "Theorem:\nthen: x is \\set\nId: \"22222222-2222-4222-8222-222222222222\"\n";
        let usage_path = project.write("thm.mlg", usage);

        let offset = usage.find("\\set").unwrap() + 2;
        let site = resolve_definition(&project.root, &usage_path, usage, offset)
            .expect("resolves across files on disk");

        assert_eq!(site.path.file_name().unwrap(), "axioms.mlg");
        assert_eq!(site.row, 0);
        assert_eq!(site.column, 1);
    }

    #[test]
    fn uses_buffer_text_for_the_target_over_disk() {
        // The definition exists only in the unsaved buffer, not on disk.
        let project = TempProject::new();
        let path = project.write("a.mlg", "Text: \"placeholder\"\nId: \"x\"\n");
        let buffer =
            "[\\gadget]\nDescribes: g\nId: \"a\"\n\nTheorem:\nthen: \\gadget\nId: \"b\"\n";

        let offset = buffer.rfind("\\gadget").unwrap() + 3;
        let site = resolve_definition(&project.root, &path, buffer, offset)
            .expect("resolves against the in-memory buffer");

        assert_eq!(site.path.file_name().unwrap(), "a.mlg");
        assert_eq!(site.row, 0);
    }
}

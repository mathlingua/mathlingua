use crate::backend::config::{CONFIG_FILE, validate_config_file};
use crate::backend::semantic::check_documents;
use crate::backend::view::{CollectionView, build_collection_view};
use crate::events::{Event, EventLocation, EventLog};
use crate::frontend::{ParsedSourceFile, parse_source_file};
use std::collections::BTreeSet;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Default directory containing MathLingua source files inside a collection.
pub(crate) const CONTENT_DIR: &str = "content";

/// A MathLingua source collection as it moves through backend passes.
///
/// The collection owns config validation, source-file discovery, structural
/// parsing, semantic checking, and optional viewer rendering.
pub(crate) struct SourceCollection {
    /// Root used for collection-relative output such as viewer file paths.
    root: PathBuf,
    /// Stable list of MathLingua source files in the collection.
    source_files: Vec<PathBuf>,
    /// Files successfully read and parsed by the structural parser pass.
    parsed_files: Vec<ParsedSourceFile>,
}

/// Selects which source-file diagnostics should be shown after a full pass run.
pub(crate) enum SourceFileFilter {
    /// Show diagnostics from every file in the collection.
    All,
    /// Show diagnostics only when they point at one of these source files.
    Only(BTreeSet<PathBuf>),
}

impl SourceFileFilter {
    /// Returns the number of collection files selected by this filter.
    pub(crate) fn selected_file_count(&self, collection: &SourceCollection) -> usize {
        match self {
            Self::All => collection.source_files.len(),
            Self::Only(files) => collection
                .source_files
                .iter()
                .filter(|file| files.contains(&normalize_path(file)))
                .count(),
        }
    }

    /// Returns true when an event should be replayed for this filter.
    fn allows(&self, event: &Event) -> bool {
        match self {
            Self::All => true,
            Self::Only(files) => event_file_path(event)
                .map(normalize_path)
                .is_some_and(|path| files.contains(&path)),
        }
    }
}

impl SourceCollection {
    /// Discovers every MathLingua source file in the collection rooted at `root`.
    pub(crate) fn load(root: &Path, event_log: &mut EventLog, origin: &str) -> Self {
        let root = normalize_path(root);
        let root = match find_collection_root(&root) {
            Some(collection_root) => {
                validate_config_file(&collection_root.join(CONFIG_FILE), event_log, origin);
                collection_root
            }
            None => root,
        };

        event_log.system_debug(
            Some(origin),
            format!("Using source collection root {}", root.display()),
        );

        let source_files = resolve_collection_source_files(&root, event_log, origin);
        Self::new(root, source_files)
    }

    /// Creates a collection from already-resolved source files.
    fn new(root: PathBuf, source_files: Vec<PathBuf>) -> Self {
        Self {
            root,
            source_files,
            parsed_files: Vec::new(),
        }
    }

    /// Returns the collection source files.
    pub(crate) fn source_files(&self) -> &[PathBuf] {
        &self.source_files
    }

    /// Returns the structurally parsed source files.
    pub(crate) fn parsed_files(&self) -> &[ParsedSourceFile] {
        &self.parsed_files
    }

    /// Runs the common checking passes in their intended order.
    pub(crate) fn run_check_passes(&mut self, event_log: &mut EventLog, origin: &str) {
        self.parse_structural(event_log, origin);
        self.check_semantics(event_log);
    }

    /// Runs the common checking passes, replaying only diagnostics accepted by a filter.
    pub(crate) fn run_check_passes_filtered(
        &mut self,
        event_log: &mut EventLog,
        origin: &str,
        filter: &SourceFileFilter,
    ) {
        match filter {
            SourceFileFilter::All => self.run_check_passes(event_log, origin),
            SourceFileFilter::Only(_) => {
                let mut pass_event_log = EventLog::new();
                self.run_check_passes(&mut pass_event_log, origin);

                for event in pass_event_log.events() {
                    if filter.allows(event) {
                        event_log.push(event.clone());
                    }
                }
            }
        }
    }

    /// Builds a filter from command paths without changing the collection.
    pub(crate) fn diagnostic_filter(
        &self,
        root: &Path,
        paths: &[PathBuf],
        event_log: &mut EventLog,
        origin: &str,
    ) -> SourceFileFilter {
        if paths.is_empty() {
            return SourceFileFilter::All;
        }

        let mut selected_files = resolve_filter_source_files(root, paths, event_log, origin)
            .into_iter()
            .map(|file| normalize_path(&file))
            .collect::<BTreeSet<_>>();
        let collection_files = self.normalized_source_files();

        selected_files.retain(|file| {
            let in_collection = collection_files.contains(file);
            if !in_collection {
                event_log.user_error_at_path(
                    Some(origin),
                    file.clone(),
                    "Path is not part of the source collection",
                );
            }
            in_collection
        });

        SourceFileFilter::Only(selected_files)
    }

    /// Reads each source file and parses it into the structural AST.
    fn parse_structural(&mut self, event_log: &mut EventLog, origin: &str) {
        self.parsed_files.clear();

        for file in &self.source_files {
            if let Some(parsed_file) = parse_source_file(file, event_log, origin) {
                self.parsed_files.push(parsed_file);
            }
        }
    }

    /// Runs backend semantic checks across all parsed files.
    fn check_semantics(&self, event_log: &mut EventLog) {
        check_documents(&self.parsed_files, event_log);
    }

    /// Runs the optional view-model generation pass.
    pub(crate) fn build_view(&self, event_log: &mut EventLog) -> Option<CollectionView> {
        build_collection_view(&self.root, &self.parsed_files, event_log)
    }

    /// Returns normalized source file paths for path-filter comparisons.
    fn normalized_source_files(&self) -> BTreeSet<PathBuf> {
        self.source_files
            .iter()
            .map(|file| normalize_path(file))
            .collect()
    }
}

/// Resolves every MathLingua source file under a collection root.
fn resolve_collection_source_files(
    root: &Path,
    event_log: &mut EventLog,
    origin: &str,
) -> Vec<PathBuf> {
    let mut files = BTreeSet::new();
    collect_source_files(root.to_path_buf(), &mut files, event_log, origin);
    files.into_iter().collect()
}

/// Resolves source files referenced by command paths for diagnostic filtering.
fn resolve_filter_source_files(
    root: &Path,
    paths: &[PathBuf],
    event_log: &mut EventLog,
    origin: &str,
) -> Vec<PathBuf> {
    let mut files = BTreeSet::new();

    for path in paths {
        if let Some(resolved_path) = resolve_input_path(root, path, event_log, origin) {
            collect_source_files(resolved_path, &mut files, event_log, origin);
        }
    }

    files.into_iter().collect()
}

/// Finds the nearest ancestor directory containing the collection config file.
pub(crate) fn find_collection_root(start: &Path) -> Option<PathBuf> {
    start
        .ancestors()
        .find(|directory| directory.join(CONFIG_FILE).is_file())
        .map(Path::to_path_buf)
}

/// Resolves a user-supplied path against the command root.
fn resolve_input_path(
    root: &Path,
    path: &Path,
    event_log: &mut EventLog,
    origin: &str,
) -> Option<PathBuf> {
    let joined = if path.is_absolute() {
        path.to_path_buf()
    } else {
        root.join(path)
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
/// Direct file targets must be `.mlg` files. Directory targets are traversed
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

/// Returns a stable absolute path when possible.
fn normalize_path(path: &Path) -> PathBuf {
    path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
}

/// Returns the file path attached to a message event.
fn event_file_path(event: &Event) -> Option<&Path> {
    event
        .as_message()
        .and_then(|message| match message.location.as_ref()? {
            EventLocation::File { path, .. } => Some(path.as_path()),
            EventLocation::InMemory { .. } => None,
        })
}

#[cfg(test)]
mod tests {
    use super::{SourceCollection, SourceFileFilter, find_collection_root};
    use crate::backend::config::default_config_contents;
    use crate::events::{Audience, Event, EventLog};
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    static NEXT_TEST_DIR_ID: AtomicUsize = AtomicUsize::new(0);

    #[test]
    fn source_collection_load_collects_all_mlg_files_under_root() {
        let temp_dir = TestDir::new();
        let docs = temp_dir.path().join("docs");
        let nested = docs.join("nested");
        let extra = temp_dir.path().join("extra.mlg");

        fs::create_dir_all(&nested).unwrap();
        fs::write(docs.join("a.mlg"), "Defines: A\n").unwrap();
        fs::write(nested.join("b.mlg"), "Defines: B\n").unwrap();
        fs::write(&extra, "Defines: C\n").unwrap();

        let mut event_log = EventLog::new();
        let collection =
            SourceCollection::load(temp_dir.path(), &mut event_log, "source_collection");

        assert!(user_events(&event_log).is_empty());
        assert_eq!(collection.source_files().len(), 3);
    }

    #[test]
    fn diagnostic_filter_collects_explicit_files_and_directories() {
        let temp_dir = TestDir::new();
        let docs = temp_dir.path().join("docs");
        let nested = docs.join("nested");
        let extra = temp_dir.path().join("extra.mlg");

        fs::create_dir_all(&nested).unwrap();
        fs::write(docs.join("a.mlg"), "Defines: A\n").unwrap();
        fs::write(nested.join("b.mlg"), "Defines: B\n").unwrap();
        fs::write(&extra, "Defines: C\n").unwrap();

        let mut event_log = EventLog::new();
        let collection =
            SourceCollection::load(temp_dir.path(), &mut event_log, "source_collection");
        let filter = collection.diagnostic_filter(
            temp_dir.path(),
            &[PathBuf::from("docs"), PathBuf::from("extra.mlg")],
            &mut event_log,
            "source_collection",
        );

        assert!(user_events(&event_log).is_empty());
        assert_eq!(filter.selected_file_count(&collection), 3);
        assert!(matches!(filter, SourceFileFilter::Only(_)));
    }

    #[test]
    fn finds_collection_root_in_ancestor_directories() {
        let temp_dir = TestDir::new();
        let root = temp_dir.path().join("repo");
        let nested = root.join("content/logic");

        fs::create_dir_all(&nested).unwrap();
        fs::write(root.join("mlg.json"), default_config_contents()).unwrap();

        let discovered = find_collection_root(&nested).expect("expected collection root");

        assert_eq!(discovered, root);
    }

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
                "source-collection-test-{}-{}-{}",
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

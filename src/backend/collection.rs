use crate::backend::config::{CONFIG_FILE, validate_config_file};
use crate::events::{Event, EventLocation, EventLog};
use std::collections::BTreeSet;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub(crate) const CONTENT_DIR: &str = "content";

#[derive(Clone, Debug)]
pub struct ParsedSourceFile {
    pub path: PathBuf,
    pub source: String,
    // TODO: Add the object representing the parsed Document
    //       after the parser has been implemented
}

pub(crate) struct SourceCollection {
    root: PathBuf,
    source_files: Vec<PathBuf>,
    parsed_files: Vec<ParsedSourceFile>,
}

pub(crate) enum SourceFileFilter {
    All,
    Only(BTreeSet<PathBuf>),
}

impl SourceFileFilter {
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

    fn new(root: PathBuf, source_files: Vec<PathBuf>) -> Self {
        Self {
            root,
            source_files,
            parsed_files: Vec::new(),
        }
    }

    pub(crate) fn source_files(&self) -> &[PathBuf] {
        &self.source_files
    }

    pub(crate) fn parsed_files(&self) -> &[ParsedSourceFile] {
        &self.parsed_files
    }

    pub(crate) fn run_check_passes(&mut self, _event_log: &mut EventLog, _origin: &str) {
        // TODO: Add the code for checking files here when the parser is implemented
    }

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

    fn normalized_source_files(&self) -> BTreeSet<PathBuf> {
        self.source_files
            .iter()
            .map(|file| normalize_path(file))
            .collect()
    }
}

fn resolve_collection_source_files(
    root: &Path,
    event_log: &mut EventLog,
    origin: &str,
) -> Vec<PathBuf> {
    let mut files = BTreeSet::new();
    collect_source_files(root.to_path_buf(), &mut files, event_log, origin);
    files.into_iter().collect()
}

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

pub(crate) fn find_collection_root(start: &Path) -> Option<PathBuf> {
    start
        .ancestors()
        .find(|directory| directory.join(CONFIG_FILE).is_file())
        .map(Path::to_path_buf)
}

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

fn read_directory_entries(directory: &Path) -> io::Result<Vec<fs::DirEntry>> {
    let mut entries = fs::read_dir(directory)?.collect::<Result<Vec<_>, io::Error>>()?;
    entries.sort_by_key(fs::DirEntry::path);
    Ok(entries)
}

fn is_mathlingua_source_file(path: &Path) -> bool {
    path.extension()
        .and_then(|value| value.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("mlg"))
}

fn normalize_path(path: &Path) -> PathBuf {
    path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
}

fn event_file_path(event: &Event) -> Option<&Path> {
    event
        .as_message()
        .and_then(|message| match message.location.as_ref()? {
            EventLocation::File { path, .. } => Some(path.as_path()),
            EventLocation::InMemory { .. } => None,
        })
}

// ===============================[ tests ]=====================================

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

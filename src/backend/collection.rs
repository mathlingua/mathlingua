use crate::backend::config::{CONFIG_FILE, validate_config_file};
use crate::backend::semantic::check_documents;
use crate::backend::view::{CollectionView, build_collection_view};
use crate::events::{Event, EventLocation, EventLog};
use crate::frontend::{
    ParsedSourceFile, ProtoGroup, ProtoParser, SourceFileViewMetadata, parse_source_file,
    top_level_group_id,
};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use uuid::Uuid;

pub(crate) const CONTENT_DIR: &str = "content";
pub(crate) const DOCS_DIR: &str = "docs";

pub(crate) struct SourceCollection {
    root: PathBuf,
    source_files: Vec<PathBuf>,
    source_file_view_metadata: BTreeMap<PathBuf, SourceFileViewMetadata>,
    source_directory_view_metadata: Vec<(PathBuf, SourceFileViewMetadata)>,
    toc_files: Vec<PathBuf>,
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
        ensure_source_file_ids(&source_files.source_files, event_log, origin);
        Self::new(root, source_files)
    }

    fn new(root: PathBuf, source_files: SourceFileDiscovery) -> Self {
        Self {
            root,
            source_files: source_files.source_files,
            source_file_view_metadata: source_files.view_metadata,
            source_directory_view_metadata: source_files.directory_metadata,
            toc_files: source_files.toc_files,
            parsed_files: Vec::new(),
        }
    }

    pub(crate) fn source_files(&self) -> &[PathBuf] {
        &self.source_files
    }

    pub(crate) fn toc_files(&self) -> &[PathBuf] {
        &self.toc_files
    }

    pub(crate) fn parsed_files(&self) -> &[ParsedSourceFile] {
        &self.parsed_files
    }

    pub(crate) fn root(&self) -> &Path {
        &self.root
    }

    pub(crate) fn run_check_passes(&mut self, event_log: &mut EventLog, origin: &str) {
        self.parse_structural(event_log, origin);
        self.check_text_fences(event_log, origin);
        self.check_semantics(event_log);
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

    fn parse_structural(&mut self, event_log: &mut EventLog, origin: &str) {
        self.parsed_files.clear();

        for file in &self.source_files {
            if let Some(mut parsed_file) = parse_source_file(file, event_log, origin) {
                parsed_file.view_metadata = self
                    .source_file_view_metadata
                    .get(&normalize_path(file))
                    .cloned()
                    .unwrap_or_default();
                self.parsed_files.push(parsed_file);
            }
        }
    }

    fn check_text_fences(&self, event_log: &mut EventLog, origin: &str) {
        for file in &self.parsed_files {
            crate::backend::text_fence::check_text_fence_syntax(
                &file.path,
                &file.source,
                event_log,
                origin,
            );
        }
    }

    fn check_semantics(&self, event_log: &mut EventLog) {
        check_documents(&self.parsed_files, event_log);
    }

    pub(crate) fn build_view(&self, event_log: &mut EventLog) -> Option<CollectionView> {
        build_collection_view(
            &self.root,
            &self.parsed_files,
            &self.source_directory_view_metadata,
            event_log,
        )
    }

    fn normalized_source_files(&self) -> BTreeSet<PathBuf> {
        self.source_files
            .iter()
            .map(|file| normalize_path(file))
            .collect()
    }
}

/// The `.mlg` source files of the collection rooted at `root`, in discovery order.
pub(crate) fn collection_source_files(
    root: &Path,
    event_log: &mut EventLog,
    origin: &str,
) -> Vec<PathBuf> {
    resolve_collection_source_files(root, event_log, origin).source_files
}

fn resolve_collection_source_files(
    root: &Path,
    event_log: &mut EventLog,
    origin: &str,
) -> SourceFileDiscovery {
    let mut discovery = SourceFileDiscovery::default();
    collect_source_files(
        collection_source_root(root),
        &mut discovery,
        event_log,
        origin,
    );
    discovery
}

fn collection_source_root(root: &Path) -> PathBuf {
    let content_dir = root.join(CONTENT_DIR);
    if content_dir.is_dir() {
        content_dir
    } else {
        root.to_path_buf()
    }
}

fn ensure_source_file_ids(files: &[PathBuf], event_log: &mut EventLog, origin: &str) {
    let mut records = Vec::new();
    let mut used_ids = BTreeSet::new();

    for file in files {
        let Ok(source) = fs::read_to_string(file) else {
            continue;
        };
        let groups = parse_proto_groups_for_ids(&source);
        for id in groups.iter().filter_map(top_level_group_id) {
            used_ids.insert(id);
        }
        records.push((file.clone(), source, groups));
    }

    for (file, source, groups) in records {
        let Some(updated) = source_with_generated_ids(&source, &groups, &mut used_ids) else {
            continue;
        };

        if let Err(error) = fs::write(&file, updated) {
            event_log.user_error_at_path(
                Some(origin),
                file,
                format!("Failed to write generated Id sections: {error}"),
            );
        }
    }
}

fn parse_proto_groups_for_ids(source: &str) -> Vec<ProtoGroup> {
    let mut event_log = EventLog::new();
    let mut parser = ProtoParser::new(source, &mut event_log);
    parser.parse()
}

fn source_with_generated_ids(
    source: &str,
    groups: &[ProtoGroup],
    used_ids: &mut BTreeSet<String>,
) -> Option<String> {
    let lines = source.split('\n').collect::<Vec<_>>();
    let mut output = Vec::new();
    let mut cursor = 0usize;
    let mut changed = false;

    for (index, group) in groups.iter().enumerate() {
        let start = group.metadata.row.min(lines.len());
        let next_start = groups
            .get(index + 1)
            .map(|next| next.metadata.row)
            .unwrap_or(lines.len())
            .min(lines.len());

        if start < cursor || next_start < cursor {
            continue;
        }

        let mut insertion = next_start;
        while insertion > start && is_trailing_id_gap(lines[insertion - 1]) {
            insertion -= 1;
        }

        output.extend(
            lines[cursor..insertion]
                .iter()
                .map(|line| (*line).to_string()),
        );

        if is_top_level_item_group(group) && !has_top_level_id_section(group) {
            output.push(format!("Id: \"{}\"", generate_unique_id(used_ids)));
            changed = true;
        }

        output.extend(
            lines[insertion..next_start]
                .iter()
                .map(|line| (*line).to_string()),
        );
        cursor = next_start;
    }

    output.extend(lines[cursor..].iter().map(|line| (*line).to_string()));

    changed.then(|| output.join("\n"))
}

fn is_trailing_id_gap(line: &str) -> bool {
    let trimmed = line.trim_start();
    trimmed.is_empty() || trimmed.starts_with("--")
}

fn is_top_level_item_group(group: &ProtoGroup) -> bool {
    group
        .sections
        .first()
        .is_some_and(|section| is_top_level_item_label(&section.label))
}

fn is_top_level_item_label(label: &str) -> bool {
    matches!(
        label,
        "Title"
            | "SectionTitle"
            | "SubsectionTitle"
            | "Text"
            | "Disambiguates"
            | "Describes"
            | "Defines"
            | "Refines"
            | "States"
            | "Axiom"
            | "Theorem"
            | "Corollary"
            | "Person"
            | "Resource"
            | "Specify"
            | "Relation"
            | "Equivalent"
            | "Topic"
    )
}

fn has_top_level_id_section(group: &ProtoGroup) -> bool {
    group.sections.iter().any(|section| section.label == "Id")
}

fn generate_unique_id(used_ids: &mut BTreeSet<String>) -> String {
    loop {
        let id = Uuid::new_v4().to_string();
        if used_ids.insert(id.clone()) {
            return id;
        }
    }
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
            collect_filter_source_files(resolved_path, &mut files, event_log, origin);
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
    discovery: &mut SourceFileDiscovery,
    event_log: &mut EventLog,
    origin: &str,
) {
    match fs::metadata(&target) {
        Ok(metadata) if metadata.is_dir() => {
            collect_directory_source_files(&target, discovery, event_log, origin, false);
        }
        Ok(metadata) if metadata.is_file() => {
            if is_mathlingua_source_file(&target) {
                discovery.add_source_file(target, SourceFileViewMetadata::default());
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
    discovery: &mut SourceFileDiscovery,
    event_log: &mut EventLog,
    origin: &str,
    inherited_hidden: bool,
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

    let mut children = directory_children(entries);
    if let Some(toc_path) = toc_file_path(&children) {
        discovery.add_toc_file(toc_path.clone());
        collect_directory_source_files_from_toc(
            &toc_path,
            &children,
            discovery,
            event_log,
            origin,
            inherited_hidden,
        );
        return;
    }

    children.sort_by(directory_child_order);
    for child in children {
        collect_directory_child(
            child,
            SourceFileViewMetadata::default(),
            discovery,
            event_log,
            origin,
            inherited_hidden,
        );
    }
}

fn collect_filter_source_files(
    target: PathBuf,
    files: &mut BTreeSet<PathBuf>,
    event_log: &mut EventLog,
    origin: &str,
) {
    match fs::metadata(&target) {
        Ok(metadata) if metadata.is_dir() => {
            collect_filter_directory_source_files(&target, files, event_log, origin);
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

fn collect_filter_directory_source_files(
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
            collect_filter_directory_source_files(&path, files, event_log, origin);
        } else if path.is_file() && is_mathlingua_source_file(&path) {
            files.insert(path);
        }
    }
}

#[derive(Default)]
struct SourceFileDiscovery {
    source_files: Vec<PathBuf>,
    view_metadata: BTreeMap<PathBuf, SourceFileViewMetadata>,
    directory_metadata: Vec<(PathBuf, SourceFileViewMetadata)>,
    toc_files: Vec<PathBuf>,
    seen_source_files: BTreeSet<PathBuf>,
    seen_directories: BTreeSet<PathBuf>,
    seen_toc_files: BTreeSet<PathBuf>,
}

impl SourceFileDiscovery {
    fn add_source_file(&mut self, path: PathBuf, metadata: SourceFileViewMetadata) {
        let normalized = normalize_path(&path);
        if self.seen_source_files.insert(normalized.clone()) {
            self.source_files.push(path);
        }
        self.view_metadata.insert(normalized, metadata);
    }

    fn add_toc_file(&mut self, path: PathBuf) {
        let normalized = normalize_path(&path);
        if self.seen_toc_files.insert(normalized) {
            self.toc_files.push(path);
        }
    }

    fn add_directory(&mut self, path: PathBuf, metadata: SourceFileViewMetadata) {
        let normalized = normalize_path(&path);
        if self.seen_directories.insert(normalized.clone()) {
            self.directory_metadata.push((path, metadata));
            return;
        }

        if let Some((_, existing_metadata)) = self
            .directory_metadata
            .iter_mut()
            .find(|(existing_path, _)| normalize_path(existing_path) == normalized)
        {
            *existing_metadata = metadata;
        }
    }
}

#[derive(Clone, Debug)]
enum DirectoryChild {
    Directory(PathBuf),
    SourceFile(PathBuf),
    TocFile(PathBuf),
    Other,
}

fn directory_children(entries: Vec<fs::DirEntry>) -> Vec<DirectoryChild> {
    entries
        .into_iter()
        .map(|entry| {
            let path = entry.path();
            if path.is_dir() {
                DirectoryChild::Directory(path)
            } else if path.is_file() && is_mathlingua_source_file(&path) {
                DirectoryChild::SourceFile(path)
            } else if path.is_file() && path.file_name().is_some_and(|name| name == "toc") {
                DirectoryChild::TocFile(path)
            } else {
                DirectoryChild::Other
            }
        })
        .collect()
}

fn toc_file_path(children: &[DirectoryChild]) -> Option<PathBuf> {
    children.iter().find_map(|child| match child {
        DirectoryChild::TocFile(path) => Some(path.clone()),
        _ => None,
    })
}

fn collect_directory_child(
    child: DirectoryChild,
    metadata: SourceFileViewMetadata,
    discovery: &mut SourceFileDiscovery,
    event_log: &mut EventLog,
    origin: &str,
    inherited_hidden: bool,
) {
    let metadata = merged_child_metadata(metadata, inherited_hidden);

    match child {
        DirectoryChild::Directory(path) => {
            discovery.add_directory(path.clone(), metadata.clone());
            collect_directory_source_files(&path, discovery, event_log, origin, metadata.hidden);
        }
        DirectoryChild::SourceFile(path) => {
            discovery.add_source_file(path, metadata);
        }
        DirectoryChild::TocFile(_) | DirectoryChild::Other => {}
    }
}

fn collect_directory_source_files_from_toc(
    toc_path: &Path,
    children: &[DirectoryChild],
    discovery: &mut SourceFileDiscovery,
    event_log: &mut EventLog,
    origin: &str,
    inherited_hidden: bool,
) {
    let direct_children = direct_toc_children_by_name(children);
    let toc_entries = parse_toc_entries(toc_path, event_log, origin);
    let mut listed_children = BTreeSet::new();

    for entry in toc_entries {
        if !listed_children.insert(entry.name.clone()) {
            event_log.user_error_at_file_row(
                Some(origin),
                toc_path.to_path_buf(),
                entry.row,
                format!("Duplicate toc entry `{}`", entry.name),
            );
            continue;
        }

        let Some(child) = direct_children.get(&entry.name) else {
            event_log.user_error_at_file_row(
                Some(origin),
                toc_path.to_path_buf(),
                entry.row,
                format!(
                    "toc entry `{}` does not match an existing .mlg file or directory",
                    entry.name
                ),
            );
            continue;
        };

        collect_directory_child(
            child.clone(),
            entry.view_metadata,
            discovery,
            event_log,
            origin,
            inherited_hidden,
        );
    }

    let mut unlisted_children = direct_children
        .iter()
        .filter(|(name, _)| !listed_children.contains(*name))
        .map(|(_, child)| child.clone())
        .collect::<Vec<_>>();
    unlisted_children.sort_by(directory_child_order);
    for child in unlisted_children {
        event_log.user_error_at_path(
            Some(origin),
            toc_path.to_path_buf(),
            format!("Directory toc is missing entry `{}`", child.name()),
        );
        collect_directory_child(
            child,
            SourceFileViewMetadata::default(),
            discovery,
            event_log,
            origin,
            inherited_hidden,
        );
    }
}

fn direct_toc_children_by_name(children: &[DirectoryChild]) -> BTreeMap<String, DirectoryChild> {
    children
        .iter()
        .filter_map(|child| match child {
            DirectoryChild::Directory(_) | DirectoryChild::SourceFile(_) => {
                Some((child.name(), child.clone()))
            }
            _ => None,
        })
        .collect()
}

#[derive(Clone, Debug)]
struct TocEntry {
    name: String,
    row: usize,
    view_metadata: SourceFileViewMetadata,
}

fn parse_toc_entries(toc_path: &Path, event_log: &mut EventLog, origin: &str) -> Vec<TocEntry> {
    let contents = match fs::read_to_string(toc_path) {
        Ok(contents) => contents,
        Err(error) => {
            event_log.user_error_at_path(
                Some(origin),
                toc_path.to_path_buf(),
                format!("Failed to read toc file: {error}"),
            );
            return Vec::new();
        }
    };

    contents
        .lines()
        .enumerate()
        .filter_map(|(index, line)| parse_toc_line(toc_path, index + 1, line, event_log, origin))
        .collect()
}

fn parse_toc_line(
    toc_path: &Path,
    row: usize,
    line: &str,
    event_log: &mut EventLog,
    origin: &str,
) -> Option<TocEntry> {
    let line = line.trim();
    if line.is_empty() {
        return None;
    }

    let (name, directive) = match line.split_once("->") {
        Some((name, directive)) => (name.trim(), Some(directive.trim())),
        None => (line, None),
    };

    if name.is_empty() {
        event_log.user_error_at_file_row(
            Some(origin),
            toc_path.to_path_buf(),
            row,
            "Missing toc file name",
        );
        return None;
    }

    if name.contains('/') || name.contains('\\') {
        event_log.user_error_at_file_row(
            Some(origin),
            toc_path.to_path_buf(),
            row,
            "toc entries must be direct .mlg file or directory names",
        );
        return None;
    }

    let view_metadata = match directive {
        Some("") => {
            event_log.user_error_at_file_row(
                Some(origin),
                toc_path.to_path_buf(),
                row,
                "toc entry title cannot be empty",
            );
            return None;
        }
        Some("HIDDEN") => SourceFileViewMetadata {
            hidden: true,
            title: None,
        },
        Some(title) => SourceFileViewMetadata {
            hidden: false,
            title: Some(title.to_string()),
        },
        None => SourceFileViewMetadata::default(),
    };

    Some(TocEntry {
        name: name.to_string(),
        row,
        view_metadata,
    })
}

fn merged_child_metadata(
    metadata: SourceFileViewMetadata,
    inherited_hidden: bool,
) -> SourceFileViewMetadata {
    SourceFileViewMetadata {
        hidden: inherited_hidden || metadata.hidden,
        title: metadata.title,
    }
}

impl DirectoryChild {
    fn name(&self) -> String {
        match self {
            DirectoryChild::Directory(path) | DirectoryChild::SourceFile(path) => path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("<unknown>")
                .to_string(),
            DirectoryChild::TocFile(_) => "toc".to_string(),
            DirectoryChild::Other => "<unknown>".to_string(),
        }
    }
}

fn directory_child_order(left: &DirectoryChild, right: &DirectoryChild) -> std::cmp::Ordering {
    directory_child_sort_key(left).cmp(&directory_child_sort_key(right))
}

fn directory_child_sort_key(child: &DirectoryChild) -> (String, String) {
    match child {
        DirectoryChild::Directory(path) => (
            display_sort_key(
                path.file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or(""),
            ),
            path.display().to_string(),
        ),
        DirectoryChild::SourceFile(path) => (
            display_sort_key(
                path.file_stem()
                    .and_then(|name| name.to_str())
                    .unwrap_or_default(),
            ),
            path.display().to_string(),
        ),
        DirectoryChild::TocFile(path) => ("".to_string(), path.display().to_string()),
        DirectoryChild::Other => ("".to_string(), String::new()),
    }
}

fn display_sort_key(segment: &str) -> String {
    segment.replace('_', " ").to_lowercase()
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
    fn source_collection_load_prefers_content_directory_when_present() {
        let temp_dir = TestDir::new();
        let content = temp_dir.path().join("content");
        let generated = temp_dir.path().join("dist");

        fs::create_dir_all(&content).unwrap();
        fs::create_dir_all(&generated).unwrap();
        fs::write(temp_dir.path().join("mlg.json"), default_config_contents()).unwrap();
        fs::write(temp_dir.path().join("root_file.mlg"), "Title: \"Root\"\n").unwrap();
        fs::write(
            generated.join("generated_file.mlg"),
            "Title: \"Generated\"\n",
        )
        .unwrap();
        fs::write(content.join("source_file.mlg"), "Title: \"Source\"\n").unwrap();

        let mut event_log = EventLog::new();
        let collection =
            SourceCollection::load(temp_dir.path(), &mut event_log, "source_collection");

        assert!(user_events(&event_log).is_empty());
        assert_eq!(collection.source_files().len(), 1);
        assert!(
            collection
                .source_files()
                .first()
                .is_some_and(|path| path.ends_with("content/source_file.mlg"))
        );
    }

    #[test]
    fn source_collection_load_uses_toc_order_and_tracks_toc_files() {
        let temp_dir = TestDir::new();
        let content = temp_dir.path().join("content");
        let nested = content.join("nested_folder");

        fs::create_dir_all(&nested).unwrap();
        fs::write(content.join("alpha_file.mlg"), "Title: \"Alpha\"\n").unwrap();
        fs::write(content.join("zeta_file.mlg"), "Title: \"Zeta\"\n").unwrap();
        fs::write(content.join("hidden_file.mlg"), "Title: \"Hidden\"\n").unwrap();
        fs::write(nested.join("inside.mlg"), "Title: \"Inside\"\n").unwrap();
        fs::write(
            content.join("toc"),
            "zeta_file.mlg -> Zeta Title\nhidden_file.mlg -> HIDDEN\nnested_folder -> Nested Folder\nalpha_file.mlg\n",
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let collection =
            SourceCollection::load(temp_dir.path(), &mut event_log, "source_collection");
        let collection_content = collection.root().join("content");

        assert!(user_events(&event_log).is_empty());
        assert_eq!(
            collection
                .source_files()
                .iter()
                .map(|path| path
                    .strip_prefix(&collection_content)
                    .unwrap_or(path)
                    .display()
                    .to_string())
                .collect::<Vec<_>>(),
            vec![
                "zeta_file.mlg",
                "hidden_file.mlg",
                "nested_folder/inside.mlg",
                "alpha_file.mlg",
            ]
        );
        assert_eq!(
            collection
                .source_directory_view_metadata
                .iter()
                .map(|(path, metadata)| (
                    path.strip_prefix(&collection_content)
                        .unwrap_or(path)
                        .display()
                        .to_string(),
                    metadata.title.clone()
                ))
                .filter(|(path, _)| !path.is_empty())
                .collect::<Vec<_>>(),
            vec![(
                "nested_folder".to_string(),
                Some("Nested Folder".to_string())
            )]
        );
        assert_eq!(collection.toc_files(), &[collection_content.join("toc")]);
    }

    #[test]
    fn source_collection_load_reports_toc_mismatches() {
        let temp_dir = TestDir::new();
        let content = temp_dir.path().join("content");
        let nested = content.join("extra_dir");

        fs::create_dir_all(&nested).unwrap();
        fs::write(content.join("extra_file.mlg"), "Title: \"Extra\"\n").unwrap();
        fs::write(nested.join("inside.mlg"), "Title: \"Inside\"\n").unwrap();
        fs::write(content.join("toc"), "missing_file.mlg\nmissing_dir\n").unwrap();

        let mut event_log = EventLog::new();
        let collection =
            SourceCollection::load(temp_dir.path(), &mut event_log, "source_collection");
        let messages = user_events(&event_log)
            .into_iter()
            .filter_map(|event| event.as_message().map(|message| message.message.clone()))
            .collect::<Vec<_>>();

        assert_eq!(collection.source_files().len(), 2);
        assert!(
            messages
                .iter()
                .any(|message| message.contains("missing_file.mlg")
                    && message.contains("does not match an existing .mlg file or directory"))
        );
        assert!(
            messages
                .iter()
                .any(|message| message.contains("missing_dir")
                    && message.contains("does not match an existing .mlg file or directory"))
        );
        assert!(
            messages
                .iter()
                .any(|message| message.contains("extra_file.mlg")
                    && message.contains("Directory toc is missing entry"))
        );
        assert!(messages.iter().any(|message| message.contains("extra_dir")
            && message.contains("Directory toc is missing entry")));
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

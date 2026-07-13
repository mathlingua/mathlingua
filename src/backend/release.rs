use crate::backend::semantic::{collect_definition_locations, command_occurrences};
use crate::events::EventLog;
use crate::frontend::{
    ParsedSourceFile, ProtoArgument, ProtoGroup, ProtoParser, top_level_group_id,
    unescape_quoted_text,
};
use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};

/// The top-level item kinds that participate in the release dependency graph as
/// *definitions*. A change to one of these re-versions the definitions it uses;
/// other kinds (page content, people, resources, and theorem-like items) are
/// versioned on their own content but never propagate.
const DEFINITION_KINDS: [&str; 6] = [
    "Defines",
    "Describes",
    "States",
    "Refines",
    "Disambiguates",
    "Equivalent",
];

/// A single top-level item as seen by `mlg release`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ReleaseItem {
    /// The item's `Id:` value.
    pub(crate) id: String,
    /// The source file the item lives in.
    pub(crate) path: PathBuf,
    /// The item's group kind (its first section label, e.g. `Describes`).
    pub(crate) kind: String,
    /// The bracketed heading of the item without its `[` `]`, when it has one
    /// (e.g. `\set` or `A \:subset:/ B`). `None` for page content and other
    /// heading-less items.
    pub(crate) header: Option<String>,
    /// The quoted text of the item's first section, when it has one (e.g. a
    /// `Title:` or `Text:` body). Used to preview page content.
    pub(crate) preview: Option<String>,
    /// The exact source slice of the item, used for content hashing. Matches the
    /// slice `mlg view`/`mlg export` present as the item's source.
    pub(crate) source: String,
    /// Whether this item is a definition for dependency-propagation purposes.
    pub(crate) is_definition: bool,
    /// Ids of the definitions this item uses, deduplicated and excluding itself.
    pub(crate) uses: Vec<String>,
}

struct RawItem {
    id: Option<String>,
    path: PathBuf,
    kind: String,
    header: Option<String>,
    preview: Option<String>,
    source: String,
    start_byte: usize,
    end_byte: usize,
    start_row: usize,
    end_row: usize,
}

/// Build the ordered list of every top-level item in the collection, together
/// with the ids of the definitions each item uses.
///
/// Item identity, kind, and source slicing come from the proto parser (the same
/// layer `mlg view` uses for display), while command-use resolution reuses the
/// semantic signature registry (the same resolution go-to-definition uses).
pub(crate) fn build_release_items(files: &[ParsedSourceFile]) -> Vec<ReleaseItem> {
    let locations = collect_definition_locations(files);

    let mut raw_items: Vec<RawItem> = Vec::new();
    let mut file_ranges: Vec<(usize, usize)> = Vec::with_capacity(files.len());
    for file in files {
        let start = raw_items.len();
        append_file_items(file, &mut raw_items);
        file_ranges.push((start, raw_items.len()));
    }

    let mut path_to_file: HashMap<&Path, usize> = HashMap::new();
    for (file_index, file) in files.iter().enumerate() {
        path_to_file.insert(file.path.as_path(), file_index);
    }

    // The id of the item whose heading declares each definition signature.
    let owner_by_location: Vec<Option<String>> = locations
        .iter()
        .map(|location| {
            let file_index = *path_to_file.get(location.path.as_path())?;
            let (start, end) = file_ranges[file_index];
            raw_items[start..end]
                .iter()
                .find(|item| item.start_row <= location.row && location.row < item.end_row)
                .and_then(|item| item.id.clone())
        })
        .collect();

    let mut uses: Vec<BTreeSet<String>> = raw_items.iter().map(|_| BTreeSet::new()).collect();
    for (file_index, file) in files.iter().enumerate() {
        let (start, end) = file_ranges[file_index];
        for (offset, location_index) in command_occurrences(&file.source, &locations) {
            let Some(owner_id) = owner_by_location[location_index].as_deref() else {
                continue;
            };
            let Some(item_index) = raw_items[start..end]
                .iter()
                .position(|item| item.start_byte <= offset && offset < item.end_byte)
                .map(|relative| start + relative)
            else {
                continue;
            };
            if raw_items[item_index].id.as_deref() == Some(owner_id) {
                continue; // a definition referencing its own signature is not a dependency
            }
            uses[item_index].insert(owner_id.to_string());
        }
    }

    raw_items
        .into_iter()
        .zip(uses)
        .filter_map(|(item, uses)| {
            let id = item.id?;
            Some(ReleaseItem {
                id,
                path: item.path,
                is_definition: DEFINITION_KINDS.contains(&item.kind.as_str()),
                kind: item.kind,
                header: item.header,
                preview: item.preview,
                source: item.source,
                uses: uses.into_iter().collect(),
            })
        })
        .collect()
}

fn append_file_items(file: &ParsedSourceFile, raw_items: &mut Vec<RawItem>) {
    let mut proto_log = EventLog::new();
    let groups = ProtoParser::new(&file.source, &mut proto_log).parse();
    let line_starts = line_start_offsets(&file.source);
    let line_count = line_starts.len();
    let lines = file.source.split('\n').collect::<Vec<_>>();

    for (index, group) in groups.iter().enumerate() {
        let start_row = group.metadata.row.min(line_count);
        let end_row = groups
            .get(index + 1)
            .map(|next| next.metadata.row)
            .unwrap_or(line_count)
            .min(line_count);
        let kind = group
            .sections
            .first()
            .map(|section| section.label.clone())
            .unwrap_or_default();

        raw_items.push(RawItem {
            id: top_level_group_id(group),
            path: file.path.clone(),
            kind,
            header: item_header(group),
            preview: item_preview(group),
            source: slice_item_source(&lines, start_row, end_row),
            start_byte: byte_offset_of_row(&line_starts, start_row, file.source.len()),
            end_byte: byte_offset_of_row(&line_starts, end_row, file.source.len()),
            start_row,
            end_row,
        });
    }
}

/// The item's bracketed heading text without the surrounding `[` `]`, if any.
fn item_header(group: &ProtoGroup) -> Option<String> {
    group
        .heading
        .as_deref()
        .map(str::trim)
        .filter(|heading| !heading.is_empty())
        .map(str::to_string)
}

/// The unescaped quoted text of the item's first section, if it has one. This is
/// the page text of a `Title:`/`SectionTitle:`/`SubsectionTitle:`/`Text:` item.
fn item_preview(group: &ProtoGroup) -> Option<String> {
    let section = group.sections.first()?;
    let raw = section.inline_argument.as_deref().or_else(|| {
        section
            .arguments
            .iter()
            .find_map(|argument| match argument {
                ProtoArgument::Text(text) => Some(text.text.as_str()),
                _ => None,
            })
    })?;
    let trimmed = raw.trim();
    let inner = trimmed.strip_prefix('"')?.strip_suffix('"')?;
    Some(unescape_quoted_text(inner))
}

/// The source slice of the top-level item whose `Id:` value is `id` within
/// `source`, using the same slicing as [`build_release_items`]. Returns `None`
/// when no top-level item in `source` has that id (for example a since-added
/// item that did not exist in an earlier revision). Used to recover an item's
/// previous contents for `mlg release --diff`.
pub(crate) fn item_source_by_id(source: &str, id: &str) -> Option<String> {
    let mut proto_log = EventLog::new();
    let groups = ProtoParser::new(source, &mut proto_log).parse();
    let lines = source.split('\n').collect::<Vec<_>>();
    let line_count = lines.len();

    for (index, group) in groups.iter().enumerate() {
        if top_level_group_id(group).as_deref() != Some(id) {
            continue;
        }
        let start_row = group.metadata.row.min(line_count);
        let end_row = groups
            .get(index + 1)
            .map(|next| next.metadata.row)
            .unwrap_or(line_count)
            .min(line_count);
        return Some(slice_item_source(&lines, start_row, end_row));
    }

    None
}

fn line_start_offsets(source: &str) -> Vec<usize> {
    let mut starts = vec![0usize];
    for (index, byte) in source.bytes().enumerate() {
        if byte == b'\n' {
            starts.push(index + 1);
        }
    }
    starts
}

fn byte_offset_of_row(line_starts: &[usize], row: usize, source_len: usize) -> usize {
    line_starts.get(row).copied().unwrap_or(source_len)
}

fn slice_item_source(lines: &[&str], start_row: usize, end_row: usize) -> String {
    let start = start_row.min(lines.len());
    let mut end = end_row.min(lines.len());
    while end > start && is_trailing_source_gap(lines[end - 1]) {
        end -= 1;
    }
    lines[start..end].join("\n")
}

fn is_trailing_source_gap(line: &str) -> bool {
    let trimmed = line.trim_start();
    trimmed.is_empty() || trimmed.starts_with("--")
}

// ===============================[ tests ]=====================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::{SourceFileViewMetadata, parse_document, top_level_item_ids};

    fn parsed(path: &str, source: &str) -> ParsedSourceFile {
        let mut event_log = EventLog::new();
        let document = parse_document(source, &mut event_log);
        ParsedSourceFile {
            path: PathBuf::from(path),
            source: source.to_string(),
            document,
            item_ids: top_level_item_ids(source),
            view_metadata: SourceFileViewMetadata::default(),
        }
    }

    fn item<'a>(items: &'a [ReleaseItem], id: &str) -> &'a ReleaseItem {
        items
            .iter()
            .find(|item| item.id == id)
            .unwrap_or_else(|| panic!("no item with id {id}"))
    }

    const A: &str = "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa";
    const B: &str = "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb";
    const C: &str = "cccccccc-cccc-4ccc-8ccc-cccccccccccc";
    const D: &str = "dddddddd-dddd-4ddd-8ddd-dddddddddddd";

    #[test]
    fn records_id_kind_and_definition_flag_and_source() {
        let source = format!(
            "[\\set]\nDescribes: A\nDocumented:\n. called: \"set\"\nId: \"{A}\"\n\n\n\
             Theorem:\nthen: A = A\nId: \"{B}\"\n"
        );
        let items = build_release_items(&[parsed("a.mlg", &source)]);

        assert_eq!(items.len(), 2);
        let describes = item(&items, A);
        assert_eq!(describes.kind, "Describes");
        assert!(describes.is_definition);
        assert_eq!(describes.header.as_deref(), Some("\\set"));
        assert_eq!(
            describes.source,
            format!("[\\set]\nDescribes: A\nDocumented:\n. called: \"set\"\nId: \"{A}\"")
        );

        let theorem = item(&items, B);
        assert_eq!(theorem.kind, "Theorem");
        assert!(
            theorem.header.is_none(),
            "the theorem has no bracket heading"
        );
        assert!(!theorem.is_definition, "theorems do not propagate");
    }

    #[test]
    fn captures_bracket_headers_and_page_previews() {
        let source = format!(
            "Title: \"Intro to Sets\"\nId: \"{A}\"\n\n\n\
             [A \\:subset:/ B]\nDescribes: A\nDocumented:\n. called: \"subset\"\nId: \"{B}\"\n"
        );
        let items = build_release_items(&[parsed("a.mlg", &source)]);

        let title = item(&items, A);
        assert_eq!(title.kind, "Title");
        assert!(title.header.is_none());
        assert_eq!(title.preview.as_deref(), Some("Intro to Sets"));

        let subset = item(&items, B);
        assert_eq!(subset.header.as_deref(), Some("A \\:subset:/ B"));
    }

    #[test]
    fn item_source_by_id_extracts_the_matching_item_slice() {
        let source = format!(
            "[\\b]\nDescribes: B\nDocumented:\n. called: \"b\"\nId: \"{B}\"\n\n\n\
             [\\a]\nDescribes: A\nDocumented:\n. called: \"a\"\nId: \"{A}\"\n"
        );
        let expected = format!("[\\a]\nDescribes: A\nDocumented:\n. called: \"a\"\nId: \"{A}\"");

        assert_eq!(
            item_source_by_id(&source, A).as_deref(),
            Some(expected.as_str())
        );
        assert_eq!(item_source_by_id(&source, "no-such-id"), None);
    }

    #[test]
    fn resolves_uses_and_excludes_self() {
        let defs = format!(
            "[\\b]\nDescribes: B\nDocumented:\n. called: \"b\"\nId: \"{B}\"\n\n\n\
             [\\c]\nDescribes: C\nDocumented:\n. called: \"c\"\nId: \"{C}\"\n"
        );
        let a = format!(
            "[\\a]\nDescribes: A\nextends: A is \\b\nDocumented:\n. called: \"uses \\c\"\nId: \"{A}\"\n"
        );
        let items = build_release_items(&[parsed("defs.mlg", &defs), parsed("a.mlg", &a)]);

        assert_eq!(item(&items, A).uses, vec![B.to_string(), C.to_string()]);
        // `\b` and `\c` are leaves; a definition never lists its own id in `uses`.
        assert!(item(&items, B).uses.is_empty());
        assert!(item(&items, C).uses.is_empty());
    }

    #[test]
    fn deduplicates_shared_dependencies_within_an_item() {
        let defs = format!(
            "[\\b]\nDescribes: B\nDocumented:\n. called: \"b\"\nId: \"{B}\"\n\n\n\
             [\\c]\nDescribes: C\nDocumented:\n. called: \"c\"\nId: \"{C}\"\n"
        );
        // `\b` referenced twice; it must appear once in `uses`.
        let a = format!(
            "[\\a]\nDescribes: A\nextends: A is \\b\nsatisfies: A is \\b\nDocumented:\n. called: \"\\c\"\nId: \"{A}\"\n"
        );
        let d = format!(
            "[\\d]\nDescribes: D\nextends: D is \\b\nDocumented:\n. called: \"\\c\"\nId: \"{D}\"\n"
        );
        let items = build_release_items(&[
            parsed("defs.mlg", &defs),
            parsed("a.mlg", &a),
            parsed("d.mlg", &d),
        ]);

        assert_eq!(item(&items, A).uses, vec![B.to_string(), C.to_string()]);
        assert_eq!(item(&items, D).uses, vec![B.to_string(), C.to_string()]);
    }
}

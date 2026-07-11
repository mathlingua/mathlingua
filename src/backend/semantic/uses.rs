use super::check::collect_document_definitions;
use super::*;

/// The source location of a defined command signature's heading — the file and
/// zero-based row of the top-level item (`Describes`, `Defines`, ...) whose
/// `[...]` heading declares that signature.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DefinitionLocation {
    pub(crate) signature: String,
    pub(crate) path: PathBuf,
    pub(crate) row: usize,
}

/// Collect every command signature defined across `files`, paired with the
/// heading location of the item that defines it.
///
/// This reuses the same registry that `check_documents` builds, so the set of
/// known signatures matches what the checker and go-to-definition see. Duplicate
/// signatures are already rejected by checking, so a single release runs this
/// only after a clean `mlg check`. Definitions whose heading position could not
/// be located are omitted, since they cannot be mapped back to an owning item.
pub(crate) fn collect_definition_locations(files: &[ParsedSourceFile]) -> Vec<DefinitionLocation> {
    let mut registry = SignatureRegistry::default();
    let mut sink = EventLog::new();
    for file in files {
        collect_document_definitions(file, &mut registry, &mut sink);
    }

    let mut locations = registry
        .definitions
        .into_iter()
        .filter_map(|(signature, entry)| {
            entry.position.map(|position| DefinitionLocation {
                signature,
                path: entry.path,
                row: position.row,
            })
        })
        .collect::<Vec<_>>();
    locations.sort_by(|left, right| left.signature.cmp(&right.signature));
    locations
}

/// Every command occurrence in `source` that resolves to one of `locations`,
/// returned as `(byte offset of the leading '\\', index into locations)`.
///
/// At each `\\` the longest — most specific — matching signature wins, mirroring
/// the tie-break used by go-to-definition's `signature_at_offset`. Backslash
/// tokens that match no known signature (ordinary LaTeX such as `\in`) are
/// skipped.
pub(crate) fn command_occurrences(
    source: &str,
    locations: &[DefinitionLocation],
) -> Vec<(usize, usize)> {
    let mut occurrences = Vec::new();

    for (offset, _) in source.match_indices('\\') {
        let mut best: Option<(usize, usize)> = None;
        for (index, location) in locations.iter().enumerate() {
            if !matches_signature_at(source, offset, &location.signature) {
                continue;
            }
            let length = location.signature.len();
            let better = match best {
                None => true,
                Some((best_length, _)) => length > best_length,
            };
            if better {
                best = Some((length, index));
            }
        }
        if let Some((_, index)) = best {
            occurrences.push((offset, index));
        }
    }

    occurrences
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn collects_definition_headings_with_locations() {
        let source = "[\\set]\nDescribes: S\nId: \"11111111-1111-4111-8111-111111111111\"\n";
        let files = vec![parsed("a.mlg", source)];

        let locations = collect_definition_locations(&files);

        let set = locations
            .iter()
            .find(|location| location.signature == "\\set")
            .expect("\\set is defined");
        assert_eq!(set.path, PathBuf::from("a.mlg"));
        assert_eq!(set.row, 0);
    }

    #[test]
    fn resolves_command_uses_to_their_definitions() {
        let def = "[\\set]\nDescribes: S\nId: \"11111111-1111-4111-8111-111111111111\"\n";
        let usage = "Theorem:\nthen: x is \\set\nId: \"22222222-2222-4222-8222-222222222222\"\n";
        let files = vec![parsed("def.mlg", def), parsed("thm.mlg", usage)];

        let locations = collect_definition_locations(&files);
        let occurrences = command_occurrences(usage, &locations);

        assert_eq!(occurrences.len(), 1);
        let (offset, index) = occurrences[0];
        assert_eq!(offset, usage.find("\\set").unwrap());
        assert_eq!(locations[index].signature, "\\set");
    }

    #[test]
    fn ignores_backslashes_that_match_no_signature() {
        let def = "[\\set]\nDescribes: S\nId: \"11111111-1111-4111-8111-111111111111\"\n";
        let usage = "[\\thing]\nDescribes: X\nDocumented:\n. written: \"x_? \\in X?\"\nId: \"22222222-2222-4222-8222-222222222222\"\n";
        let files = vec![parsed("def.mlg", def), parsed("thm.mlg", usage)];

        let locations = collect_definition_locations(&files);
        let occurrences = command_occurrences(usage, &locations);

        // The only resolvable command is the `\thing` heading; `\in` is LaTeX.
        assert!(
            occurrences
                .iter()
                .all(|(_, index)| locations[*index].signature == "\\thing")
        );
    }

    #[test]
    fn prefers_the_longer_signature_at_a_shared_start() {
        let def = "[\\set]\nDescribes: S\nId: \"11111111-1111-4111-8111-111111111111\"\n\n\
                   [\\set:complement{A}]\nDescribes: C\nwhen: A is \\set\nId: \"22222222-2222-4222-8222-222222222222\"\n";
        let usage = "Theorem:\nthen: x is \\set:complement{y}\nId: \"33333333-3333-4333-8333-333333333333\"\n";
        let files = vec![parsed("def.mlg", def), parsed("thm.mlg", usage)];

        let locations = collect_definition_locations(&files);
        let occurrences = command_occurrences(usage, &locations);

        let signatures = occurrences
            .iter()
            .map(|(_, index)| locations[*index].signature.as_str())
            .collect::<Vec<_>>();
        assert!(signatures.contains(&"\\set:complement"));
        assert!(!signatures.contains(&"\\set"));
    }
}

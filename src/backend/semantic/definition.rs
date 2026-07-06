use super::check::collect_document_definitions;
use super::*;

/// The resolved location of the top-level item that defines a command — the
/// position of the `\`-signature inside the item's `[...]` heading. Rows and
/// columns are zero-based.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DefinitionSite {
    pub path: PathBuf,
    pub row: usize,
    pub column: usize,
}

/// Resolve the command occurrence at byte `offset` within `target_source` to
/// the top-level item (`Describes`, `Defines`, `States`, ...) whose `[...]`
/// heading declares that command's signature.
///
/// `files` must supply every parsed document whose headings should be
/// searched, including the target itself so that a command defined and used in
/// the same file resolves. Returns `None` when the cursor is not inside a
/// command occurrence with a known definition.
pub fn find_definition(
    files: &[ParsedSourceFile],
    target_source: &str,
    offset: usize,
) -> Option<DefinitionSite> {
    let mut registry = SignatureRegistry::default();
    let mut sink = EventLog::new();
    for file in files {
        collect_document_definitions(file, &mut registry, &mut sink);
    }

    let signature = signature_at_offset(target_source, offset, &registry)?;
    let entry = registry.definitions.get(&signature)?;
    let position = entry.position?;
    Some(DefinitionSite {
        path: entry.path.clone(),
        row: position.row,
        column: position.column,
    })
}

/// The signature of the command occurrence covering `offset`. Every `\` in
/// `source` is a candidate command start; a candidate wins when its matched
/// extent `[start, end)` contains the cursor. Ties prefer the start closest to
/// the cursor (the innermost command, when one is nested inside another's
/// argument), then the longer — more specific — signature.
fn signature_at_offset(source: &str, offset: usize, registry: &SignatureRegistry) -> Option<String> {
    let mut best: Option<(usize, &str)> = None;
    for (start, _) in source.match_indices('\\') {
        if start > offset {
            break;
        }
        for signature in registry.definitions.keys() {
            let Some(end) = signature_match_end(source, start, signature) else {
                continue;
            };
            if offset >= end {
                continue;
            }
            let better = match best {
                None => true,
                Some((best_start, best_signature)) => {
                    start > best_start
                        || (start == best_start && signature.len() > best_signature.len())
                }
            };
            if better {
                best = Some((start, signature));
            }
        }
    }
    best.map(|(_, signature)| signature.to_string())
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

    /// Byte offset of the first occurrence of `needle` in `source`, offset by
    /// `within` bytes into that occurrence.
    fn offset_of(source: &str, needle: &str, within: usize) -> usize {
        source.find(needle).expect("needle present") + within
    }

    #[test]
    fn resolves_command_in_same_file_to_its_heading() {
        let source = "[\\function:on{A}:to{B}]\nDescribes: f\nId: \"x\"\n\n\
                      Theorem:\nthen: \\function:on{x + 1}:to{abc}\nId: \"y\"\n";
        let files = vec![parsed("a.mlg", source)];
        // cursor on the `to` label of the usage
        let cursor = offset_of(source, "\\function:on{x + 1}", 13);
        let site = find_definition(&files, source, cursor).expect("resolves");
        assert_eq!(site.path, PathBuf::from("a.mlg"));
        assert_eq!(site.row, 0); // the `[\function...]` heading line
        assert_eq!(site.column, 1); // the `\` just after `[`
    }

    #[test]
    fn resolves_from_inside_an_argument() {
        let source = "[\\function:on{A}:to{B}]\nDescribes: f\nId: \"x\"\n\n\
                      Theorem:\nthen: \\function:on{x + 1}:to{abc}\nId: \"y\"\n";
        let files = vec![parsed("a.mlg", source)];
        // cursor sitting inside `{x + 1}`
        let cursor = offset_of(source, "{x + 1}", 3);
        let site = find_definition(&files, source, cursor).expect("resolves");
        assert_eq!(site.row, 0);
        assert_eq!(site.column, 1);
    }

    #[test]
    fn resolves_across_files() {
        let axioms = "[\\set]\nStates:\nthat: \"x\"\nId: \"11111111-1111-4111-8111-111111111111\"\n";
        let usage = "Theorem:\nthen: x is \\set\nId: \"22222222-2222-4222-8222-222222222222\"\n";
        let files = vec![parsed("axioms.mlg", axioms), parsed("thm.mlg", usage)];
        let cursor = offset_of(usage, "\\set", 2);
        let site = find_definition(&files, usage, cursor).expect("resolves cross-file");
        assert_eq!(site.path, PathBuf::from("axioms.mlg"));
        assert_eq!(site.row, 0);
    }

    #[test]
    fn returns_none_off_a_command() {
        let source = "[\\set]\nStates:\nthat: \"x\"\nId: \"x\"\n";
        let files = vec![parsed("a.mlg", source)];
        // cursor on the `States` keyword, not a command
        let cursor = offset_of(source, "States", 2);
        assert_eq!(find_definition(&files, source, cursor), None);
    }

    #[test]
    fn returns_none_for_unknown_command() {
        let source = "Theorem:\nthen: \\undefined:thing{x}\nId: \"x\"\n";
        let files = vec![parsed("a.mlg", source)];
        let cursor = offset_of(source, "\\undefined", 3);
        assert_eq!(find_definition(&files, source, cursor), None);
    }

    #[test]
    fn resolves_innermost_nested_command() {
        // `\set` appears inside the argument of `\domain{...}`; clicking on the
        // inner `\set` resolves to `\set`, not the enclosing `\domain`.
        let source = "[\\set]\nStates:\nthat: \"x\"\nId: \"a\"\n\n\
                      [\\domain{R}]\nDefines: D\nId: \"b\"\n\n\
                      Theorem:\nthen: \\domain{\\set}\nId: \"c\"\n";
        let files = vec![parsed("a.mlg", source)];
        let inner = offset_of(source, "\\domain{\\set}", 8) + 2; // onto the inner `\set`
        let site = find_definition(&files, source, inner).expect("resolves inner");
        // `\set` heading is the first heading (row 0); `\domain` is at row 5.
        assert_eq!(site.row, 0);
    }
}

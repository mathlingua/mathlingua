use super::check::definition_item;
use super::*;

/// A zero-based `[start, end)` span in a document, with columns counted in
/// Unicode scalar values (matching go-to-definition and completion).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RenameSpan {
    pub start_row: usize,
    pub start_column: usize,
    pub end_row: usize,
    pub end_column: usize,
}

/// A single text replacement produced by a rename: replace the text of `span`
/// in `path` with `new_text`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenameEditPlan {
    pub path: PathBuf,
    pub span: RenameSpan,
    pub new_text: String,
}

/// The editable region a `prepareRename` reports: the written command signature
/// under the cursor, and its current text as the edit box's placeholder.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenamePreparation {
    pub span: RenameSpan,
    pub placeholder: String,
}

/// Why a rename could not be carried out. The message is meant to be surfaced
/// to the user by the editor.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RenameError {
    /// The cursor was not on a top-level item's command heading.
    NotOnHeading,
    /// The heading (or new name) is a command form rename does not yet handle.
    Unsupported(String),
    /// The new name is not a syntactically valid command heading.
    InvalidNewName(String),
    /// The new name changed the command's parameters, which would break the
    /// mapping from the heading to its uses.
    ParametersChanged(String),
}

/// If `offset` lies inside the command signature of a top-level item's heading
/// (`[\...]`), report that signature's span and text so the editor can offer a
/// rename seeded with the current signature.
pub fn prepare_rename(target: &ParsedSourceFile, offset: usize) -> Option<RenamePreparation> {
    let (heading, start, end) = locate_heading(target, offset)?;
    if !matches!(heading, CommandHeader::Command(_)) {
        return None;
    }
    Some(RenamePreparation {
        span: span_from_offsets(&target.source, start, end),
        placeholder: target.source.get(start..end)?.to_string(),
    })
}

/// Rename the command whose heading is under `offset` in `target` to `new_name`,
/// rewriting the heading and every use of the command across `files`.
///
/// `new_name` is the replacement written signature (e.g. `\map:to{A}`). The
/// rename is refused unless the new name keeps the command's parameters exactly:
/// same names, same count, same argument groups. Only the command-name tokens
/// (the parts between `:`) may change, which keeps the heading-to-use mapping
/// unambiguous.
pub fn plan_rename(
    files: &[ParsedSourceFile],
    target: &ParsedSourceFile,
    offset: usize,
    new_name: &str,
) -> Result<Vec<RenameEditPlan>, RenameError> {
    let (old_heading, heading_start, heading_end) =
        locate_heading(target, offset).ok_or(RenameError::NotOnHeading)?;
    if !matches!(old_heading, CommandHeader::Command(_)) {
        return Err(RenameError::Unsupported(
            "Renaming is only supported for plain command headings".to_string(),
        ));
    }

    let new_name = new_name.trim();
    if !new_name.starts_with('\\') {
        return Err(RenameError::InvalidNewName(
            "A command name must start with `\\`".to_string(),
        ));
    }

    // Splice the new signature into the buffer in place of the old one and
    // re-parse, so the new heading is validated by the real parser rather than
    // an ad-hoc one.
    let mut new_source = String::with_capacity(target.source.len() + new_name.len());
    new_source.push_str(&target.source[..heading_start]);
    new_source.push_str(new_name);
    new_source.push_str(&target.source[heading_end..]);

    let mut sink = EventLog::new();
    let new_document = parse_document(&new_source, &mut sink);
    let new_heading =
        heading_starting_at(&new_document, &new_source, heading_start).ok_or_else(|| {
            RenameError::InvalidNewName("The new name is not a valid command heading".to_string())
        })?;
    if !matches!(new_heading, CommandHeader::Command(_)) {
        return Err(RenameError::InvalidNewName(
            "The new name is not a valid command heading".to_string(),
        ));
    }

    let old_variants = shapes_for_header(old_heading);
    let new_variants = shapes_for_header(new_heading);
    if old_variants.len() != new_variants.len() {
        return Err(RenameError::ParametersChanged(
            "The new name must keep all parameters unchanged".to_string(),
        ));
    }

    let mut mapping: Vec<(String, String)> = Vec::new();
    let mut changed = false;
    for (old, new) in old_variants.iter().zip(new_variants.iter()) {
        if old.parameters != new.parameters
            || old.hidden_parameters != new.hidden_parameters
            || old.shape.arg_groups != new.shape.arg_groups
        {
            return Err(RenameError::ParametersChanged(
                "The new name must use the exact same parameters as the current name".to_string(),
            ));
        }
        if !is_plain_signature(&old.shape.signature) || !is_plain_signature(&new.shape.signature) {
            return Err(RenameError::Unsupported(
                "Renaming is only supported for plain command headings".to_string(),
            ));
        }
        if old.shape.signature != new.shape.signature {
            changed = true;
        }
        mapping.push((old.shape.signature.clone(), new.shape.signature.clone()));
    }

    if !changed {
        return Err(RenameError::InvalidNewName(
            "The new name is the same as the current name".to_string(),
        ));
    }

    Ok(collect_edits(files, &mapping))
}

/// Rewrite every occurrence of each `old` signature to its paired `new`
/// signature across `files`. A given occurrence matches at most one signature
/// in the mapping, but occurrences are still deduplicated per file by start
/// offset so overlapping edits are never emitted.
fn collect_edits(files: &[ParsedSourceFile], mapping: &[(String, String)]) -> Vec<RenameEditPlan> {
    let mut edits = Vec::new();
    for file in files {
        let mut seen_starts: Vec<usize> = Vec::new();
        for (old_sig, new_sig) in mapping {
            let mut search = 0;
            while let Some(relative) = file.source.get(search..).and_then(|rest| rest.find('\\')) {
                let start = search + relative;
                search = start + 1;
                let Some((end, replacement)) =
                    rewrite_plain_occurrence(&file.source, start, old_sig, new_sig)
                else {
                    continue;
                };
                if seen_starts.contains(&start) {
                    continue;
                }
                seen_starts.push(start);
                edits.push(RenameEditPlan {
                    path: file.path.clone(),
                    span: span_from_offsets(&file.source, start, end),
                    new_text: replacement,
                });
            }
        }
    }
    edits
}

/// The command heading of the top-level item whose heading signature covers
/// `offset`, along with the `[start, end)` byte range of that signature.
fn locate_heading(
    target: &ParsedSourceFile,
    offset: usize,
) -> Option<(&CommandHeader, usize, usize)> {
    let source = &target.source;
    for item in &target.document.items {
        let Some(definition) = definition_item(item) else {
            continue;
        };
        let heading = definition.heading();
        let shape = shape_for_header(heading);
        let mut search = 0;
        while let Some(start) =
            find_signature_occurrence(source, &shape, search, OccurrenceKind::Heading)
        {
            match signature_match_end(source, start, &shape.signature) {
                Some(end) if offset >= start && offset < end => {
                    return Some((heading, start, end));
                }
                _ => {}
            }
            search = start + 1;
        }
    }
    None
}

/// The command heading whose signature begins exactly at byte `start`.
fn heading_starting_at<'a>(
    document: &'a Document,
    source: &str,
    start: usize,
) -> Option<&'a CommandHeader> {
    for item in &document.items {
        let Some(definition) = definition_item(item) else {
            continue;
        };
        let heading = definition.heading();
        let shape = shape_for_header(heading);
        let mut search = 0;
        while let Some(offset) =
            find_signature_occurrence(source, &shape, search, OccurrenceKind::Heading)
        {
            if offset == start {
                return Some(heading);
            }
            if offset > start {
                break;
            }
            search = offset + 1;
        }
    }
    None
}

/// A plain (non-refined, non-infix, non-prefix) command signature: the only
/// form whose uses rename knows how to rewrite token-for-token.
fn is_plain_signature(signature: &str) -> bool {
    signature.starts_with('\\')
        && !signature.starts_with("\\:")
        && !signature.starts_with("\\.")
        && !signature.contains("::")
}

/// Match the plain signature `old_sig` at byte `start` in `source` exactly as
/// [`signature_match_end`] does, but build the rewritten occurrence: each
/// name token is swapped for `new_sig`'s corresponding token while colons,
/// optional `?` markers, and argument groups are copied verbatim. On a match,
/// returns the end offset and the replacement text.
fn rewrite_plain_occurrence(
    source: &str,
    start: usize,
    old_sig: &str,
    new_sig: &str,
) -> Option<(usize, String)> {
    let old_parts: Vec<&str> = old_sig.split(':').collect();
    let new_parts: Vec<&str> = new_sig.split(':').collect();
    if old_parts.len() != new_parts.len() {
        return None;
    }

    let mut out = String::new();
    let mut cursor = start;

    let head = *old_parts.first()?;
    if !source.get(cursor..)?.starts_with(head) {
        return None;
    }
    out.push_str(new_parts[0]);
    cursor += head.len();
    cursor = copy_argument_groups(source, cursor, &mut out)?;

    for index in 1..old_parts.len() {
        let after_colon = source.get(cursor..)?.strip_prefix(':')?;
        out.push(':');
        cursor += 1;
        let after_marker = match after_colon.strip_prefix('?') {
            Some(rest) => {
                out.push('?');
                cursor += 1;
                rest
            }
            None => after_colon,
        };
        let part = old_parts[index];
        if !after_marker.starts_with(part) {
            return None;
        }
        out.push_str(new_parts[index]);
        cursor += part.len();
        cursor = copy_argument_groups(source, cursor, &mut out)?;
    }

    let after = source.get(cursor..).unwrap_or("");
    let boundary_ok = !after
        .chars()
        .next()
        .is_some_and(|ch| ch == ':' || ch == '.' || ch == '_' || ch.is_ascii_alphanumeric());
    boundary_ok.then_some((cursor, out))
}

/// Copy the run of balanced `{...}` / `(...)` argument groups starting at
/// `cursor` into `out`, returning the offset just past them. Mirrors
/// [`skip_argument_groups`]: an unbalanced group stops the run without
/// consuming it.
fn copy_argument_groups(source: &str, mut cursor: usize, out: &mut String) -> Option<usize> {
    loop {
        let rest = source.get(cursor..)?;
        let (open, close) = match rest.chars().next() {
            Some('{') => ('{', '}'),
            Some('(') => ('(', ')'),
            _ => return Some(cursor),
        };
        let Some(end) = find_balanced_group_end(rest, open, close) else {
            return Some(cursor);
        };
        out.push_str(&rest[..end]);
        cursor += end;
    }
}

fn span_from_offsets(source: &str, start: usize, end: usize) -> RenameSpan {
    let start_position = position_at_offset(source, start);
    let end_position = position_at_offset(source, end);
    RenameSpan {
        start_row: start_position.row,
        start_column: start_position.column,
        end_row: end_position.row,
        end_column: end_position.column,
    }
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

    fn offset_of(source: &str, needle: &str, within: usize) -> usize {
        source.find(needle).expect("needle present") + within
    }

    /// Apply a set of edits to a single file's text (edits assumed
    /// non-overlapping) so tests can assert on the resulting source.
    fn apply(source: &str, edits: &[RenameEditPlan]) -> String {
        let mut ranges: Vec<(usize, usize, String)> = edits
            .iter()
            .map(|edit| {
                let start = byte_at(source, edit.span.start_row, edit.span.start_column);
                let end = byte_at(source, edit.span.end_row, edit.span.end_column);
                (start, end, edit.new_text.clone())
            })
            .collect();
        ranges.sort_by_key(|(start, _, _)| *start);
        let mut out = String::new();
        let mut cursor = 0;
        for (start, end, text) in ranges {
            out.push_str(&source[cursor..start]);
            out.push_str(&text);
            cursor = end;
        }
        out.push_str(&source[cursor..]);
        out
    }

    fn byte_at(source: &str, row: usize, column: usize) -> usize {
        let mut offset = 0;
        for _ in 0..row {
            offset += source[offset..].find('\n').expect("row present") + 1;
        }
        let line = &source[offset..];
        let column_byte = line
            .char_indices()
            .nth(column)
            .map(|(byte, _)| byte)
            .unwrap_or(line.len());
        offset + column_byte
    }

    #[test]
    fn renames_heading_and_uses_across_files() {
        let def = "[\\set]\nDescribes: S\nId: \"a\"\n";
        let usage = "Theorem:\nthen: x is \\set\nId: \"b\"\n";
        let def_file = parsed("def.mlg", def);
        let usage_file = parsed("thm.mlg", usage);
        let files = vec![def_file.clone(), usage_file.clone()];

        let cursor = offset_of(def, "\\set", 2);
        let edits = plan_rename(&files, &def_file, cursor, "\\collection").expect("plan");

        let def_edits: Vec<_> = edits
            .iter()
            .filter(|e| e.path == def_file.path)
            .cloned()
            .collect();
        let usage_edits: Vec<_> = edits
            .iter()
            .filter(|e| e.path == usage_file.path)
            .cloned()
            .collect();
        assert_eq!(
            apply(def, &def_edits),
            "[\\collection]\nDescribes: S\nId: \"a\"\n"
        );
        assert_eq!(
            apply(usage, &usage_edits),
            "Theorem:\nthen: x is \\collection\nId: \"b\"\n"
        );
    }

    #[test]
    fn preserves_arguments_when_renaming_command_name() {
        let source = "[\\function:on{A}:to{B}]\nDescribes: f\nId: \"x\"\n\n\
                      Theorem:\nthen: \\function:on{x + 1}:to{abc}\nId: \"y\"\n";
        let file = parsed("a.mlg", source);
        let files = vec![file.clone()];

        let cursor = offset_of(source, "[\\function", 2);
        let edits = plan_rename(&files, &file, cursor, "\\map:on{A}:to{B}").expect("plan");
        let renamed = apply(source, &edits);

        assert!(renamed.contains("[\\map:on{A}:to{B}]"));
        assert!(renamed.contains("\\map:on{x + 1}:to{abc}"));
        assert!(!renamed.contains("\\function"));
    }

    #[test]
    fn can_rename_a_middle_label_keeping_parameters() {
        let source = "[\\function:on{A}:to{B}]\nDescribes: f\nId: \"x\"\n\n\
                      Theorem:\nthen: \\function:on{p}:to{q}\nId: \"y\"\n";
        let file = parsed("a.mlg", source);
        let files = vec![file.clone()];

        let cursor = offset_of(source, "[\\function", 2);
        let edits = plan_rename(&files, &file, cursor, "\\function:at{A}:to{B}").expect("plan");
        let renamed = apply(source, &edits);

        assert!(renamed.contains("[\\function:at{A}:to{B}]"));
        assert!(renamed.contains("\\function:at{p}:to{q}"));
        assert!(!renamed.contains(":on{"));
    }

    #[test]
    fn blocks_renames_that_change_a_parameter_name() {
        let source = "[\\function:on{A}:to{B}]\nDescribes: f\nId: \"x\"\n";
        let file = parsed("a.mlg", source);
        let files = vec![file.clone()];

        let cursor = offset_of(source, "[\\function", 2);
        let error = plan_rename(&files, &file, cursor, "\\map:on{C}:to{B}").unwrap_err();
        assert!(matches!(error, RenameError::ParametersChanged(_)));
    }

    #[test]
    fn blocks_renames_that_drop_a_parameter() {
        let source = "[\\function:on{A}:to{B}]\nDescribes: f\nId: \"x\"\n";
        let file = parsed("a.mlg", source);
        let files = vec![file.clone()];

        let cursor = offset_of(source, "[\\function", 2);
        let error = plan_rename(&files, &file, cursor, "\\map:on{A}").unwrap_err();
        assert!(matches!(
            error,
            RenameError::ParametersChanged(_) | RenameError::InvalidNewName(_)
        ));
    }

    #[test]
    fn blocks_renames_off_a_heading() {
        let source = "[\\set]\nDescribes: S\nId: \"x\"\n";
        let file = parsed("a.mlg", source);
        let files = vec![file.clone()];

        let cursor = offset_of(source, "Describes", 2);
        assert_eq!(
            plan_rename(&files, &file, cursor, "\\collection").unwrap_err(),
            RenameError::NotOnHeading
        );
    }

    #[test]
    fn prepare_reports_the_signature_span() {
        let source = "[\\function:on{A}:to{B}]\nDescribes: f\nId: \"x\"\n";
        let file = parsed("a.mlg", source);

        let cursor = offset_of(source, "[\\function", 2);
        let prep = prepare_rename(&file, cursor).expect("prepare");
        assert_eq!(prep.placeholder, "\\function:on{A}:to{B}");
        assert_eq!(prep.span.start_row, 0);
        assert_eq!(prep.span.start_column, 1);
    }

    #[test]
    fn prepare_declines_off_a_heading() {
        let source = "[\\set]\nDescribes: S\nId: \"x\"\n";
        let file = parsed("a.mlg", source);
        let cursor = offset_of(source, "Describes", 2);
        assert!(prepare_rename(&file, cursor).is_none());
    }
}

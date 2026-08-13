use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct SourcePosition {
    pub(super) row: usize,
    pub(super) column: usize,
}

pub(super) struct SourceLocator<'a> {
    source: &'a str,
    /// A copy of `source` with all quoted text-value regions (prose, `Text:`
    /// Markdown, ` ```mlg ` fences, `called:`/`written:` values, and so on)
    /// blanked to spaces, keeping every byte offset and newline in place.
    ///
    /// All occurrence searches run against this masked copy so that a command,
    /// heading, or symbol that merely *appears* inside a quoted string (for
    /// example an ` ```mlg ` example embedded in a `Text:` value) is never
    /// mistaken for a real document position. Reported positions still come from
    /// `source`, which is byte-for-byte aligned with `search`.
    search: String,
    /// Sorted byte offsets where each top-level item begins, used to bound
    /// symbol and reference searches to the current item so a lookup can never
    /// spill into the following item.
    item_boundaries: Vec<usize>,
    /// Monotonic cursor into `item_boundaries` used by [`Self::begin_item`] to
    /// advance the window to the next item, including headingless items that
    /// [`Self::anchor_item_heading`] cannot anchor by signature.
    item_boundary_cursor: usize,
    item_cursor: usize,
    item_start: usize,
    /// Exclusive byte offset at which the current item ends (the start of the
    /// next item, or the end of the source). Symbol and reference searches stay
    /// within `[.., item_end)` before falling back to the whole source.
    item_end: usize,
    heading_cursor: usize,
    reference_cursor: usize,
    symbol_cursor: usize,
}

impl<'a> SourceLocator<'a> {
    pub(super) fn new(source: &'a str) -> Self {
        Self {
            source,
            search: mask_text_values(source),
            item_boundaries: item_boundaries(source),
            item_boundary_cursor: 0,
            item_cursor: 0,
            item_start: 0,
            item_end: source.len(),
            heading_cursor: 0,
            reference_cursor: 0,
            symbol_cursor: 0,
        }
    }

    /// The exclusive end of the item beginning at or after `from`.
    fn next_boundary_after(&self, from: usize) -> usize {
        self.item_boundaries
            .iter()
            .copied()
            .find(|&boundary| boundary > from)
            .unwrap_or(self.source.len())
    }

    /// Advance the window to the next top-level item. Called for every item —
    /// including headingless ones such as `Theorem:` and `Text:` — so a
    /// subsequent symbol or reference search is always bounded to the current
    /// item and never spills into the next.
    pub(super) fn begin_item(&mut self) {
        let start = self
            .item_boundaries
            .iter()
            .copied()
            .find(|&boundary| boundary >= self.item_boundary_cursor)
            .unwrap_or(self.source.len());
        self.item_start = start;
        self.item_end = self.next_boundary_after(start);
        self.item_boundary_cursor = self.item_end;
        self.symbol_cursor = start;
        self.reference_cursor = start;
    }

    /// The masked search source narrowed to the current item's window.
    fn item_window(&self) -> &str {
        &self.search[..self.item_end.min(self.search.len())]
    }

    pub(super) fn anchor_item_heading(&mut self, shape: &SignatureShape) {
        if let Some(offset) = find_signature_occurrence(
            &self.search,
            shape,
            self.item_cursor,
            OccurrenceKind::Heading,
        )
        .or_else(|| find_signature_occurrence(&self.search, shape, 0, OccurrenceKind::Heading))
        {
            self.item_cursor = offset.saturating_add(1);
            self.item_start = offset.saturating_add(1);
            self.item_end = self.next_boundary_after(offset);
            // Re-sync the boundary cursor to this item's end so the next
            // `begin_item` stays aligned even if the boundary heuristic and the
            // document's items drift apart.
            self.item_boundary_cursor = self.item_end;
            self.reference_cursor = offset.saturating_add(1);
            self.symbol_cursor = offset.saturating_add(1);
        }
    }

    pub(super) fn locate_heading(&mut self, shape: &SignatureShape) -> Option<SourcePosition> {
        let offset = find_signature_occurrence(
            &self.search,
            shape,
            self.heading_cursor,
            OccurrenceKind::Heading,
        )
        .or_else(|| find_signature_occurrence(&self.search, shape, 0, OccurrenceKind::Heading))?;
        self.heading_cursor = offset.saturating_add(1);
        Some(position_at_offset(self.source, offset))
    }

    pub(super) fn locate_reference(&mut self, shape: &SignatureShape) -> Option<SourcePosition> {
        let window = self.item_window();
        let offset = find_signature_occurrence(
            window,
            shape,
            self.reference_cursor,
            OccurrenceKind::Reference,
        )
        .or_else(|| {
            find_signature_occurrence(window, shape, self.item_start, OccurrenceKind::Reference)
        })
        .or_else(|| find_signature_occurrence(&self.search, shape, 0, OccurrenceKind::Reference))?;
        self.reference_cursor = offset.saturating_add(1);
        Some(position_at_offset(self.source, offset))
    }

    pub(super) fn locate_symbol(&mut self, name: &str) -> Option<SourcePosition> {
        let window = self.item_window();
        let offset = find_symbol_occurrence(window, name, self.symbol_cursor)
            .or_else(|| find_symbol_occurrence(window, name, self.item_start))
            .or_else(|| find_symbol_occurrence(&self.search, name, 0))?;
        self.symbol_cursor = offset.saturating_add(name.len());
        Some(position_at_offset(self.source, offset))
    }
}

/// Builds a search copy of `source` in which only ` ```mlg ` / ` ```mlg-fragment `
/// fenced code blocks are blanked, preserving byte length and newlines.
///
/// Unlike [`mask_text_values`], ordinary quoted text (`called:`/`written:` and
/// other prose) is left intact, because a command named there is a genuine
/// documentation reference that go-to-definition, find-uses, rename, and release
/// dependency resolution should still see. Only fenced *examples* are illustrative
/// and must be ignored.
pub(super) fn mask_mlg_fences(source: &str) -> String {
    let mut bytes = source.as_bytes().to_vec();
    let mut in_fence = false;
    for (start, end) in line_spans(source) {
        let trimmed = source[start..end].trim();
        if in_fence {
            if trimmed.starts_with("```") {
                in_fence = false;
            } else {
                for byte in &mut bytes[start..end] {
                    if *byte != b'\n' {
                        *byte = b' ';
                    }
                }
            }
        } else if trimmed == "```mlg" || trimmed == "```mlg-fragment" {
            in_fence = true;
        }
    }
    String::from_utf8(bytes).unwrap_or_else(|_| source.to_owned())
}

/// The sorted byte offsets at which each top-level item begins: a non-blank line
/// starting in column 0 that follows a blank line (or the start of the source).
/// Top-level items are always blank-line separated and their sections never
/// contain blank lines, so this yields exactly one offset per item.
fn item_boundaries(source: &str) -> Vec<usize> {
    let mut boundaries = Vec::new();
    let mut previous_blank = true;
    for (start, end) in line_spans(source) {
        let line = &source[start..end];
        let blank = line.trim().is_empty();
        let starts_in_column_zero = line.chars().next().is_some_and(|ch| !ch.is_whitespace());
        if !blank && starts_in_column_zero && previous_blank {
            boundaries.push(start);
        }
        previous_blank = blank;
    }
    boundaries
}

/// Builds a search copy of `source` in which every quoted text-value region is
/// replaced by spaces, preserving byte length and newline positions so offsets
/// stay aligned with `source`.
///
/// Occurrence searches that must ignore commands, headings, or symbols appearing
/// inside quoted prose (for example ` ```mlg ` examples embedded in a `Text:`
/// value) run against this masked copy — the semantic locator as well as
/// go-to-definition, find-uses, and rename.
pub(super) fn mask_text_values(source: &str) -> String {
    let mut bytes = source.as_bytes().to_vec();
    for (start, end) in text_value_byte_ranges(source) {
        for byte in &mut bytes[start..end] {
            if *byte != b'\n' {
                *byte = b' ';
            }
        }
    }
    // Text ranges are aligned to `"` and line boundaries, so whole characters
    // are always replaced and the result stays valid UTF-8.
    String::from_utf8(bytes).unwrap_or_else(|_| source.to_owned())
}

/// The byte ranges of quoted text values in `source`, mirroring the proto
/// parser's line-oriented text handling: a text value opens where a line's
/// argument begins with `"` and closes on the line whose trimmed text ends with
/// an unescaped `"`. This is line-oriented on purpose, so mid-line operator
/// quotes such as `"in"` in a formula never start a region.
fn text_value_byte_ranges(source: &str) -> Vec<(usize, usize)> {
    let spans = line_spans(source);
    let mut ranges = Vec::new();
    let mut index = 0;
    while index < spans.len() {
        let (line_start, line_end) = spans[index];
        let line = &source[line_start..line_end];
        let Some(quote_rel) = text_open_quote_index(line) else {
            index += 1;
            continue;
        };
        let open = line_start + quote_rel;
        if arg_is_complete_quoted_text(&line[quote_rel..]) {
            ranges.push((open, line_end));
            index += 1;
            continue;
        }
        let mut close = None;
        let mut scan = index + 1;
        while scan < spans.len() {
            let (scan_start, scan_end) = spans[scan];
            if line_closes_quoted_text(&source[scan_start..scan_end]) {
                close = Some(scan_end);
                break;
            }
            scan += 1;
        }
        match close {
            Some(end) => {
                ranges.push((open, end));
                index = scan + 1;
            }
            None => {
                ranges.push((open, source.len()));
                index = spans.len();
            }
        }
    }
    ranges
}

/// The `(start, end)` byte spans of each line in `source`, excluding the
/// terminating newline.
fn line_spans(source: &str) -> Vec<(usize, usize)> {
    let mut spans = Vec::new();
    let mut start = 0;
    for (offset, byte) in source.bytes().enumerate() {
        if byte == b'\n' {
            spans.push((start, offset));
            start = offset + 1;
        }
    }
    spans.push((start, source.len()));
    spans
}

/// If `line`'s argument begins a quoted text value, the byte index within `line`
/// of the opening `"`. Skips leading indentation, an optional `. ` dot marker,
/// and an optional `identifier:` section-label prefix.
fn text_open_quote_index(line: &str) -> Option<usize> {
    let bytes = line.as_bytes();
    let mut index = 0;
    while matches!(bytes.get(index), Some(b' ' | b'\t')) {
        index += 1;
    }
    if line[index..].starts_with(". ") {
        index += 2;
        while matches!(bytes.get(index), Some(b' ' | b'\t')) {
            index += 1;
        }
    }
    if let Some(after_label) = strip_leading_section_label(&line[index..]) {
        index = line.len() - after_label.len();
    }
    line[index..].starts_with('"').then_some(index)
}

/// Strips a leading `identifier:` section-label prefix (and any following
/// spaces) from `text`, returning the remainder, or `None` if `text` does not
/// begin with such a label.
fn strip_leading_section_label(text: &str) -> Option<&str> {
    let bytes = text.as_bytes();
    if !matches!(bytes.first(), Some(byte) if byte.is_ascii_alphabetic()) {
        return None;
    }
    let mut index = 1;
    while matches!(bytes.get(index), Some(byte) if byte.is_ascii_alphanumeric()) {
        index += 1;
    }
    if bytes.get(index) != Some(&b':') {
        return None;
    }
    index += 1;
    while matches!(bytes.get(index), Some(b' ' | b'\t')) {
        index += 1;
    }
    Some(&text[index..])
}

/// Whether `arg` (which begins with `"`) is a complete single-line quoted text.
fn arg_is_complete_quoted_text(arg: &str) -> bool {
    let trimmed = arg.trim_end();
    trimmed.len() >= 2
        && trimmed.starts_with('"')
        && trimmed.ends_with('"')
        && !trailing_quote_is_escaped(trimmed)
}

/// Whether `line`'s trimmed text ends with an unescaped `"`, closing a multiline
/// text value.
fn line_closes_quoted_text(line: &str) -> bool {
    let trimmed = line.trim_end();
    trimmed.ends_with('"') && !trailing_quote_is_escaped(trimmed)
}

/// Whether the trailing `"` of `text` is escaped by an odd run of backslashes.
fn trailing_quote_is_escaped(text: &str) -> bool {
    let mut backslashes = 0;
    for character in text[..text.len() - 1].chars().rev() {
        if character != '\\' {
            break;
        }
        backslashes += 1;
    }
    backslashes % 2 == 1
}

#[derive(Clone, Copy)]
pub(super) enum OccurrenceKind {
    Heading,
    Reference,
}

pub(super) fn find_signature_occurrence(
    source: &str,
    shape: &SignatureShape,
    start: usize,
    kind: OccurrenceKind,
) -> Option<usize> {
    for (relative, _) in source.get(start..)?.match_indices('\\') {
        let offset = start + relative;
        let is_heading = is_heading_line(source, offset);
        match kind {
            OccurrenceKind::Heading if !is_heading => continue,
            OccurrenceKind::Reference if is_heading => continue,
            _ => {}
        }
        if shape_signature_matches_at(source, offset, shape) {
            return Some(offset);
        }
    }
    None
}

fn shape_signature_matches_at(source: &str, offset: usize, shape: &SignatureShape) -> bool {
    matches_signature_at(source, offset, &shape.signature)
        || shape
            .fallback_shapes
            .iter()
            .any(|fallback| shape_signature_matches_at(source, offset, fallback))
}

pub(super) fn is_heading_line(source: &str, offset: usize) -> bool {
    let line_start = source[..offset].rfind('\n').map(|i| i + 1).unwrap_or(0);
    let line_end = source[offset..]
        .find('\n')
        .map(|i| offset + i)
        .unwrap_or(source.len());
    let line = source[line_start..line_end].trim();
    line.starts_with('[') && line.ends_with(']')
}

pub(super) fn matches_signature_at(source: &str, offset: usize, signature: &str) -> bool {
    signature_match_end(source, offset, signature).is_some()
}

/// Like [`matches_signature_at`], but on a match returns the byte offset in
/// `source` just past the matched command occurrence, including any argument
/// groups (`{...}` / `(...)`) that were skipped over. Go-to-definition uses
/// this to test whether a cursor falls within a command occurrence.
pub(super) fn signature_match_end(source: &str, offset: usize, signature: &str) -> Option<usize> {
    if signature.starts_with("\\:") {
        return infix_spec_match_end(source, offset, signature);
    }

    if signature.contains("::") {
        return refined_match_end(source, offset, signature);
    }

    if signature.starts_with("\\.") {
        let tail = source.get(offset..)?;
        return tail
            .starts_with(signature)
            .then(|| offset + signature.len());
    }

    let parts: Vec<&str> = signature.split(':').collect();
    let first = parts.first()?;
    let mut remaining = source.get(offset..)?;
    if !remaining.starts_with(first) {
        return None;
    }
    remaining = &remaining[first.len()..];
    remaining = skip_argument_groups(remaining);

    for part in parts.iter().skip(1) {
        let after_colon = remaining.strip_prefix(':')?;
        let after_marker = after_colon.strip_prefix('?').unwrap_or(after_colon);
        if !after_marker.starts_with(part) {
            return None;
        }
        remaining = &after_marker[part.len()..];
        remaining = skip_argument_groups(remaining);
    }

    let boundary_ok = !remaining
        .chars()
        .next()
        .is_some_and(|ch| ch == ':' || ch == '.' || ch == '_' || ch.is_ascii_alphanumeric());
    boundary_ok.then(|| source.len() - remaining.len())
}

fn infix_spec_match_end(source: &str, offset: usize, signature: &str) -> Option<usize> {
    let body = signature
        .strip_prefix("\\:")
        .and_then(|text| text.strip_suffix(":/"))?;
    let parts: Vec<&str> = body.split(':').collect();
    let first = parts.first()?;
    let mut remaining = source.get(offset..)?.strip_prefix("\\:")?;
    if !remaining.starts_with(first) {
        let wrapped = remaining.starts_with('(');
        if !wrapped {
            return None;
        }
        remaining = &remaining[1..];
        remaining = remaining.strip_prefix(first)?;
        remaining = remaining.strip_prefix(')')?;
    } else {
        remaining = &remaining[first.len()..];
    }
    remaining = skip_argument_groups(remaining);

    for part in parts.iter().skip(1) {
        let after_colon = remaining.strip_prefix(':')?;
        remaining = after_colon.strip_prefix('?').unwrap_or(after_colon);
        let wrapped = remaining.starts_with('(');
        if wrapped {
            remaining = &remaining[1..];
        }
        remaining = remaining.strip_prefix(part)?;
        if wrapped {
            remaining = remaining.strip_prefix(')')?;
        }
        remaining = skip_argument_groups(remaining);
    }

    let rest = remaining
        .strip_prefix(":/")
        .or_else(|| remaining.strip_prefix("?:/"))?;
    Some(source.len() - rest.len())
}

/// Match a refined-command signature — one containing `::`, such as
/// `\injective::function:from:to` — at `offset`. The written form wraps the
/// refinement adjective in parentheses that the canonical signature omits
/// (`\(injective)::function:from{A}:to{B}`), so each segment name may be
/// parenthesized in the source and may be followed by argument groups to skip.
fn refined_match_end(source: &str, offset: usize, signature: &str) -> Option<usize> {
    let mut remaining = source.get(offset..)?.strip_prefix('\\')?;
    let parts: Vec<&str> = signature.strip_prefix('\\')?.split(':').collect();

    for (index, part) in parts.iter().enumerate() {
        if index > 0 {
            let after_colon = remaining.strip_prefix(':')?;
            remaining = after_colon.strip_prefix('?').unwrap_or(after_colon);
        }
        // A segment name (the adjective) may be parenthesized in the source
        // where the canonical signature is bare.
        let wrapped = remaining.starts_with('(');
        if wrapped {
            remaining = &remaining[1..];
        }
        remaining = remaining.strip_prefix(part)?;
        if wrapped {
            remaining = remaining.strip_prefix(')')?;
        }
        remaining = skip_argument_groups(remaining);
    }

    let boundary_ok = !remaining
        .chars()
        .next()
        .is_some_and(|ch| ch == ':' || ch == '.' || ch == '_' || ch.is_ascii_alphanumeric());
    boundary_ok.then(|| source.len() - remaining.len())
}

pub(super) fn find_symbol_occurrence(source: &str, name: &str, start: usize) -> Option<usize> {
    if name.is_empty() {
        return None;
    }

    for (relative, _) in source.get(start..)?.match_indices(name) {
        let offset = start + relative;
        if is_heading_line(source, offset) {
            continue;
        }
        if matches_symbol_at(source, offset, name) {
            return Some(offset);
        }
    }

    None
}

pub(super) fn matches_symbol_at(source: &str, offset: usize, name: &str) -> bool {
    let Some(tail) = source.get(offset..) else {
        return false;
    };
    if !tail.starts_with(name) {
        return false;
    }

    let before = source[..offset].chars().next_back();
    let after = source[offset + name.len()..].chars().next();

    let invalid_before = before.is_some_and(|ch| {
        ch == '\\' || ch == ':' || ch == '.' || ch == '$' || ch.is_ascii_alphanumeric() || ch == '_'
    });
    let invalid_after = after.is_some_and(|ch| ch.is_ascii_alphanumeric() || ch == '_');

    !invalid_before && !invalid_after
}

pub(super) fn skip_argument_groups(mut input: &str) -> &str {
    loop {
        let Some(open) = input.chars().next() else {
            return input;
        };
        let close = match open {
            '{' => '}',
            '(' => ')',
            _ => return input,
        };
        let Some(end) = find_balanced_group_end(input, open, close) else {
            return input;
        };
        input = &input[end..];
    }
}

pub(super) fn find_balanced_group_end(input: &str, open: char, close: char) -> Option<usize> {
    let mut depth = 0usize;
    for (index, ch) in input.char_indices() {
        if ch == open {
            depth += 1;
        } else if ch == close {
            depth = depth.checked_sub(1)?;
            if depth == 0 {
                return Some(index + ch.len_utf8());
            }
        }
    }
    None
}

pub(super) fn position_at_offset(source: &str, offset: usize) -> SourcePosition {
    let mut row = 0usize;
    let mut line_start = 0usize;
    for (index, ch) in source.char_indices() {
        if index >= offset {
            break;
        }
        if ch == '\n' {
            row += 1;
            line_start = index + ch.len_utf8();
        }
    }

    SourcePosition {
        row,
        column: source[line_start..offset].chars().count(),
    }
}

pub(super) fn display_definition_location(entry: &DefinitionEntry) -> String {
    match entry.position {
        Some(position) => format!(
            "{}:{}:{}",
            entry.path.display(),
            position.row + 1,
            position.column + 1
        ),
        None => entry.path.display().to_string(),
    }
}

pub(super) fn emit_error(
    event_log: &mut EventLog,
    path: &Path,
    position: Option<SourcePosition>,
    message: impl Into<String>,
) {
    let location = position
        .map(|position| {
            EventLocation::file(
                path.to_path_buf(),
                Some(EventSpan::point(EventPosition::at_row_and_column(
                    position.row,
                    position.column,
                ))),
            )
        })
        .unwrap_or_else(|| EventLocation::file_path(path.to_path_buf()));
    event_log.user_event(Some(ORIGIN), Level::Error, Some(location), message);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn masking_preserves_length_and_newlines() {
        let source = "Text: \"a\n       b\"\nId: \"x\"\n";
        let masked = mask_text_values(source);
        assert_eq!(masked.len(), source.len());
        assert_eq!(
            masked.matches('\n').count(),
            source.matches('\n').count(),
            "newlines must be preserved so positions stay aligned"
        );
    }

    #[test]
    fn masks_headings_and_symbols_inside_text_values() {
        // A ```mlg fence embedded in a Text: value contains a heading-shaped
        // line and a bare `A`. Both must be blanked in the search copy, while an
        // identical real heading below stays intact.
        let source = concat!(
            "Text: \"see\n",
            "       ```mlg\n",
            "       [\\foo{A}]\n",
            "       Defines: A\n",
            "       ```\"\n",
            "\n",
            "[\\foo{A}]\n",
            "Defines: X\n",
        );
        let masked = mask_text_values(source);

        let first = source.find("[\\foo{A}]").unwrap();
        let last = source.rfind("[\\foo{A}]").unwrap();
        assert_ne!(first, last, "fixture needs two occurrences");
        assert!(
            masked.as_bytes()[first..first + "[\\foo{A}]".len()]
                .iter()
                .all(|byte| *byte == b' '),
            "the occurrence inside the Text value must be blanked"
        );
        assert_eq!(
            &masked[last..last + "[\\foo{A}]".len()],
            "[\\foo{A}]",
            "the real heading must be untouched"
        );
    }

    #[test]
    fn fence_mask_blanks_examples_but_keeps_doc_references() {
        // Fence-only masking (used by go-to-definition / find-uses / rename /
        // release) must blank commands inside ```mlg examples while keeping a
        // command named in ordinary `called:`/`written:` prose intact.
        let source = concat!(
            "Text: \"see\n",
            "       ```mlg\n",
            "       [\\foo]\n",
            "       ```\"\n",
            "\n",
            "[\\bar]\n",
            "Defines: X\n",
            "Documented:\n",
            ". written: \"uses \\foo\"\n",
        );
        let masked = mask_mlg_fences(source);
        assert_eq!(masked.len(), source.len());

        let fenced = source.find("[\\foo]").unwrap();
        assert!(
            masked.as_bytes()[fenced..fenced + "[\\foo]".len()]
                .iter()
                .all(|byte| *byte == b' '),
            "the fenced example must be blanked"
        );
        assert!(
            masked.contains("uses \\foo"),
            "a command named in `written:` prose must be preserved"
        );
    }

    #[test]
    fn does_not_mask_formula_operator_quotes() {
        // `"in"` operator quotes on a formula line are not a text value; nothing
        // on these lines should be masked.
        let source = "[\\set]\nDefines: X\nRequires:\n. capability: x_ \"in\" X :-> \\\\abstract\n";
        assert_eq!(
            mask_text_values(source),
            source,
            "formula operator quotes must not start a masked region"
        );
    }

    #[test]
    fn masks_single_line_text_values_but_keeps_the_label() {
        let source = "Documented:\n. written: \"\\set\"\n";
        let masked = mask_text_values(source);
        assert!(
            masked.contains(". written: "),
            "the section label must remain searchable"
        );
        assert!(
            !masked.contains("\\set"),
            "the quoted value must be blanked, got: {masked:?}"
        );
    }
}

use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct SourcePosition {
    pub(super) row: usize,
    pub(super) column: usize,
}

pub(super) struct SourceLocator<'a> {
    source: &'a str,
    item_cursor: usize,
    item_start: usize,
    heading_cursor: usize,
    reference_cursor: usize,
    symbol_cursor: usize,
}

impl<'a> SourceLocator<'a> {
    pub(super) fn new(source: &'a str) -> Self {
        Self {
            source,
            item_cursor: 0,
            item_start: 0,
            heading_cursor: 0,
            reference_cursor: 0,
            symbol_cursor: 0,
        }
    }

    pub(super) fn anchor_item_heading(&mut self, shape: &SignatureShape) {
        if let Some(offset) = find_signature_occurrence(
            self.source,
            shape,
            self.item_cursor,
            OccurrenceKind::Heading,
        )
        .or_else(|| find_signature_occurrence(self.source, shape, 0, OccurrenceKind::Heading))
        {
            self.item_cursor = offset.saturating_add(1);
            self.item_start = offset.saturating_add(1);
            self.reference_cursor = offset.saturating_add(1);
            self.symbol_cursor = offset.saturating_add(1);
        }
    }

    pub(super) fn locate_heading(&mut self, shape: &SignatureShape) -> Option<SourcePosition> {
        let offset = find_signature_occurrence(
            self.source,
            shape,
            self.heading_cursor,
            OccurrenceKind::Heading,
        )
        .or_else(|| find_signature_occurrence(self.source, shape, 0, OccurrenceKind::Heading))?;
        self.heading_cursor = offset.saturating_add(1);
        Some(position_at_offset(self.source, offset))
    }

    pub(super) fn locate_reference(&mut self, shape: &SignatureShape) -> Option<SourcePosition> {
        let offset = find_signature_occurrence(
            self.source,
            shape,
            self.reference_cursor,
            OccurrenceKind::Reference,
        )
        .or_else(|| {
            find_signature_occurrence(
                self.source,
                shape,
                self.item_start,
                OccurrenceKind::Reference,
            )
        })
        .or_else(|| find_signature_occurrence(self.source, shape, 0, OccurrenceKind::Reference))?;
        self.reference_cursor = offset.saturating_add(1);
        Some(position_at_offset(self.source, offset))
    }

    pub(super) fn locate_symbol(&mut self, name: &str) -> Option<SourcePosition> {
        let offset = find_symbol_occurrence(self.source, name, self.symbol_cursor)
            .or_else(|| find_symbol_occurrence(self.source, name, self.item_start))
            .or_else(|| find_symbol_occurrence(self.source, name, 0))?;
        self.symbol_cursor = offset.saturating_add(name.len());
        Some(position_at_offset(self.source, offset))
    }
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
        if matches_signature_at(source, offset, &shape.signature) {
            return Some(offset);
        }
    }
    None
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

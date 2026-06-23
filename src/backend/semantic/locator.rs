use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct SourcePosition {
    row: usize,
    column: usize,
}

pub(super) struct SourceLocator<'a> {
    source: &'a str,
    heading_cursor: usize,
    reference_cursor: usize,
    symbol_cursor: usize,
}

impl<'a> SourceLocator<'a> {
    pub(super) fn new(source: &'a str) -> Self {
        Self {
            source,
            heading_cursor: 0,
            reference_cursor: 0,
            symbol_cursor: 0,
        }
    }

    pub(super) fn locate_heading(&mut self, shape: &SignatureShape) -> Option<SourcePosition> {
        let offset = find_signature_occurrence(
            self.source,
            shape,
            self.heading_cursor,
            OccurrenceKind::Heading,
        )?;
        self.heading_cursor = offset.saturating_add(1);
        Some(position_at_offset(self.source, offset))
    }

    pub(super) fn locate_reference(&mut self, shape: &SignatureShape) -> Option<SourcePosition> {
        let offset = find_signature_occurrence(
            self.source,
            shape,
            self.reference_cursor,
            OccurrenceKind::Reference,
        )?;
        self.reference_cursor = offset.saturating_add(1);
        Some(position_at_offset(self.source, offset))
    }

    pub(super) fn locate_symbol(&mut self, name: &str) -> Option<SourcePosition> {
        let offset = find_symbol_occurrence(self.source, name, self.symbol_cursor)?;
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
    if signature.starts_with("\\:") || signature.contains("::") {
        return source
            .get(offset..)
            .is_some_and(|tail| tail.starts_with(signature));
    }

    let parts: Vec<&str> = signature.split(':').collect();
    let Some(first) = parts.first() else {
        return false;
    };
    let Some(mut remaining) = source.get(offset..) else {
        return false;
    };
    if !remaining.starts_with(first) {
        return false;
    }
    remaining = &remaining[first.len()..];
    remaining = skip_argument_groups(remaining);

    for part in parts.iter().skip(1) {
        let Some(after_colon) = remaining.strip_prefix(':') else {
            return false;
        };
        if !after_colon.starts_with(part) {
            return false;
        }
        remaining = &after_colon[part.len()..];
        remaining = skip_argument_groups(remaining);
    }

    !remaining
        .chars()
        .next()
        .is_some_and(|ch| ch == ':' || ch == '.' || ch == '_' || ch.is_ascii_alphanumeric())
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

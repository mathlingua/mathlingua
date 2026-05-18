#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// Zero-based source position used internally before converting to event spans.
struct SourcePosition {
    /// Zero-based row in the source file.
    row: usize,
    /// Zero-based Unicode scalar column in the row.
    column: usize,
}

/// Incremental source locator for matching parsed signatures back to raw text.
///
/// The frontend AST currently does not carry exact spans for every backend
/// diagnostic, so this locator scans the original source in order.  Separate
/// cursors are maintained for definitions and references to avoid repeatedly
/// reporting the first matching occurrence.
struct SourceLocator<'a> {
    /// Complete original source text for the file being checked.
    source: &'a str,
    /// Next byte offset to scan from when locating definition headings.
    heading_cursor: usize,
    /// Next byte offset to scan from when locating command references.
    reference_cursor: usize,
}

impl<'a> SourceLocator<'a> {
    /// Creates a locator for one source file.
    fn new(source: &'a str) -> Self {
        Self {
            source,
            heading_cursor: 0,
            reference_cursor: 0,
        }
    }

    /// Finds the next heading occurrence matching a signature shape.
    fn locate_heading(&mut self, shape: &SignatureShape) -> Option<SourcePosition> {
        let offset = find_signature_occurrence(
            self.source,
            shape,
            self.heading_cursor,
            OccurrenceKind::Heading,
        )?;
        self.heading_cursor = offset.saturating_add(1);
        Some(position_at_offset(self.source, offset))
    }

    /// Finds the next non-heading occurrence matching a signature shape.
    fn locate_reference(&mut self, shape: &SignatureShape) -> Option<SourcePosition> {
        let offset = find_signature_occurrence(
            self.source,
            shape,
            self.reference_cursor,
            OccurrenceKind::Reference,
        )?;
        self.reference_cursor = offset.saturating_add(1);
        Some(position_at_offset(self.source, offset))
    }
}

#[derive(Clone, Copy)]
/// Kind of source occurrence the locator should search for.
enum OccurrenceKind {
    /// Match command signatures inside bracketed headings.
    Heading,
    /// Match command signatures anywhere outside bracketed headings.
    Reference,
}

/// Finds the next byte offset where a signature shape appears in raw source text.
///
/// This scanner is deliberately conservative: it starts only at backslash tokens,
/// separates headings from references, and delegates argument skipping so
/// signatures such as `\function:on:to` can match text like
/// `\function:on{A}:to{B}`.
fn find_signature_occurrence(
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

/// Returns true when a byte offset belongs to a bracketed command heading line.
fn is_heading_line(source: &str, offset: usize) -> bool {
    let line_start = source[..offset].rfind('\n').map(|i| i + 1).unwrap_or(0);
    let line_end = source[offset..]
        .find('\n')
        .map(|i| offset + i)
        .unwrap_or(source.len());
    let line = source[line_start..line_end].trim();
    line.starts_with('[') && line.ends_with(']')
}

/// Checks whether a canonical signature matches source text at a byte offset.
///
/// Normal command signatures skip over concrete argument groups between tail
/// labels.  Refined and infix signatures are currently matched as direct text
/// because their canonical text contains enough punctuation to avoid prefix
/// ambiguity.
fn matches_signature_at(source: &str, offset: usize, signature: &str) -> bool {
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

/// Skips any immediately adjacent balanced `{...}` or `(...)` groups.
fn skip_argument_groups(mut input: &str) -> &str {
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

/// Returns the byte index just after a balanced delimiter group.
///
/// Nested groups of the same delimiter are handled so a command argument can
/// contain structured expressions without breaking signature matching.
fn find_balanced_group_end(input: &str, open: char, close: char) -> Option<usize> {
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

/// Converts a byte offset into a zero-based row and column position.
fn position_at_offset(source: &str, offset: usize) -> SourcePosition {
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

/// Formats a definition location for duplicate-signature diagnostics.
fn display_definition_location(entry: &DefinitionEntry) -> String {
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

/// Emits a user-facing semantic error at an optional source position.
///
/// When a position is unavailable, the diagnostic still points at the owning
/// file so command-line output remains actionable.
fn emit_error(
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


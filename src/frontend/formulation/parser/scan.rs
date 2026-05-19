use super::*;

/// Splits a quoted-operator specification into subject, operator, and target.
///
/// The scan ignores quotes inside backticks and nested delimiters.  It returns
/// the first top-level quoted segment because specification syntax contains a
/// single operator between the subject and target name.
pub(super) fn split_subject_operator_name(input: &str) -> Option<(&str, &str, &str)> {
    let input = input.trim();
    let mut state = ScanState::default();
    let mut start = None;

    for (index, ch) in input.char_indices() {
        if state.in_quote {
            if ch == '"' {
                let open = start?;
                let subject = input[..open].trim();
                let operator = input[open + 1..index].trim();
                let name = input[index + 1..].trim();
                return Some((subject, operator, name));
            }
            continue;
        }

        if state.in_backtick {
            if ch == '`' {
                state.in_backtick = false;
            }
            continue;
        }

        match ch {
            '"' if state.is_top_level() => {
                start = Some(index);
                state.in_quote = true;
            }
            '`' => state.in_backtick = true,
            '(' => state.paren_depth += 1,
            ')' => state.paren_depth = state.paren_depth.saturating_sub(1),
            '{' => state.brace_depth += 1,
            '}' => state.brace_depth = state.brace_depth.saturating_sub(1),
            '[' => state.bracket_depth += 1,
            ']' => state.bracket_depth = state.bracket_depth.saturating_sub(1),
            _ => {}
        }
    }

    None
}

/// Splits an input at the first top-level delimiter from `delimiters`.
///
/// The delimiter remains in the returned suffix, which lets callers dispatch on
/// the next syntactic marker without losing it.
pub(super) fn split_prefix_by_delimiters<'a>(
    input: &'a str,
    delimiters: &[char],
) -> (&'a str, &'a str) {
    if let Some(index) = find_first_top_level_delimiter(input, delimiters) {
        (&input[..index], &input[index..])
    } else {
        (input, "")
    }
}

/// Finds the first top-level delimiter character from a set.
///
/// Delimiters nested inside parentheses, braces, brackets, quoted strings, or
/// backticks are ignored so command and alias splitting respects nested syntax.
pub(super) fn find_first_top_level_delimiter(input: &str, delimiters: &[char]) -> Option<usize> {
    let mut state = ScanState::default();
    for (index, ch) in input.char_indices() {
        if state.is_top_level() && delimiters.contains(&ch) {
            return Some(index);
        }
        state.advance(ch);
    }

    None
}

/// Finds the first occurrence of one top-level character.
///
/// This is a convenience wrapper around the same scanner used for delimiter
/// sets when callers need to locate a specific syntactic marker.
pub(super) fn find_first_top_level_char(input: &str, target: char) -> Option<usize> {
    let mut state = ScanState::default();
    for (index, ch) in input.char_indices() {
        if state.is_top_level() && ch == target {
            return Some(index);
        }
        state.advance(ch);
    }

    None
}

/// Returns whether `needle` appears outside nested syntax.
///
/// This is used for syntactic dispatch before selecting a more specific parser,
/// such as distinguishing `is` statements from specifications.
pub(super) fn contains_top_level(input: &str, needle: &str) -> bool {
    find_top_level_substring(input, needle).is_some()
}

/// Finds a top-level substring while respecting delimiter and quote state.
///
/// The scan advances by characters but checks substring matches at each byte
/// index yielded by `char_indices`, which keeps returned indices valid UTF-8
/// byte offsets into the original input.
pub(super) fn find_top_level_substring(input: &str, needle: &str) -> Option<usize> {
    let mut state = ScanState::default();
    for (index, ch) in input.char_indices() {
        if state.is_top_level() && input[index..].starts_with(needle) {
            return Some(index);
        }
        state.advance(ch);
    }

    None
}

/// Splits input on a top-level delimiter and rejects empty items.
///
/// Empty entries are almost always author mistakes in MathLingua lists, so this
/// helper reports them instead of silently dropping them.
pub(super) fn split_top_level(input: &str, delimiter: char) -> Result<Vec<&str>, ParseError> {
    let mut parts = Vec::new();
    let mut start = 0;
    let mut state = ScanState::default();

    for (index, ch) in input.char_indices() {
        if state.is_top_level() && ch == delimiter {
            let part = input[start..index].trim();
            if part.is_empty() {
                return Err(ParseError::custom("empty item in comma-separated list"));
            }
            parts.push(part);
            start = index + ch.len_utf8();
        }
        state.advance(ch);
    }

    let tail = input[start..].trim();
    if !tail.is_empty() {
        parts.push(tail);
    }

    Ok(parts)
}

/// Consumes one balanced delimiter block from the start of `input`.
///
/// The returned tuple contains the block contents without outer delimiters and
/// the remaining suffix after the closing delimiter.  Quoted and backticked
/// content is skipped while counting nested delimiter depth.
pub(super) fn consume_balanced_prefix(
    input: &str,
    open: char,
    close: char,
) -> Result<(&str, &str), ParseError> {
    let input = input.trim_start();
    if !input.starts_with(open) {
        return Err(ParseError::custom(format!("expected `{open}`")));
    }

    let mut depth = 0usize;
    let mut in_quote = false;
    let mut in_backtick = false;

    for (index, ch) in input.char_indices() {
        if index == 0 {
            depth = 1;
            continue;
        }

        if in_quote {
            if ch == '"' {
                in_quote = false;
            }
            continue;
        }

        if in_backtick {
            if ch == '`' {
                in_backtick = false;
            }
            continue;
        }

        match ch {
            '"' => in_quote = true,
            '`' => in_backtick = true,
            c if c == open => depth += 1,
            c if c == close => {
                depth -= 1;
                if depth == 0 {
                    return Ok((&input[1..index], &input[index + close.len_utf8()..]));
                }
            }
            _ => {}
        }
    }

    Err(ParseError::custom(format!(
        "unterminated `{open}` ... `{close}` block"
    )))
}

/// Returns whether text is valid in a name position.
///
/// Plain names must begin and end with alphanumeric characters and may contain
/// underscores internally.  Backtick-wrapped operator text is also accepted so
/// operator names can be referenced by command/name infrastructure.
pub(super) fn is_name_text(input: &str) -> bool {
    if input.is_empty() {
        return false;
    }

    if input.starts_with('`') && input.ends_with('`') && input.len() >= 2 {
        return is_operator_text(&input[1..input.len() - 1]);
    }

    let mut chars = input.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    let Some(last) = input.chars().last() else {
        return false;
    };

    if !first.is_ascii_alphanumeric() || !last.is_ascii_alphanumeric() {
        return false;
    }

    input
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
}

/// Returns whether text consists solely of operator characters.
///
/// This deliberately uses a narrow ASCII operator alphabet until the language
/// has a richer token classification for symbolic operators.
pub(super) fn is_operator_text(input: &str) -> bool {
    if input.is_empty() {
        return false;
    }

    input.chars().all(|ch| "-~!#%^&*\\+=|<>/".contains(ch))
}

/// Builds a span covering the trimmed input.
///
/// Hand-written parser helpers use this because they currently parse complete
/// substrings instead of preserving exact source offsets through each split.
pub(super) fn span_all(input: &str) -> Span {
    Span::new(0, input.trim().len())
}

/// Scanner state for top-level string searches.
///
/// The state tracks bracket nesting plus quoted/backticked regions so helpers
/// can safely split formulation text without building a full token stream for
/// every small dispatch decision.
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct ScanState {
    /// Current nesting depth inside `(...)`.
    paren_depth: usize,
    /// Current nesting depth inside `{...}`.
    brace_depth: usize,
    /// Current nesting depth inside `[...]`.
    bracket_depth: usize,
    /// Whether the scan is currently inside double-quoted text.
    in_quote: bool,
    /// Whether the scan is currently inside backtick-delimited operator text.
    in_backtick: bool,
}

impl ScanState {
    /// Returns whether the scanner is outside all nested syntax.
    ///
    /// Top-level splitters only consider separators when this returns true.
    fn is_top_level(&self) -> bool {
        !self.in_quote
            && !self.in_backtick
            && self.paren_depth == 0
            && self.brace_depth == 0
            && self.bracket_depth == 0
    }

    /// Advances the scanner state by one character.
    ///
    /// Closing delimiters use saturating subtraction so malformed intermediate
    /// text does not panic while a later parser emits the actual syntax error.
    fn advance(&mut self, ch: char) {
        if self.in_quote {
            if ch == '"' {
                self.in_quote = false;
            }
            return;
        }

        if self.in_backtick {
            if ch == '`' {
                self.in_backtick = false;
            }
            return;
        }

        match ch {
            '"' => self.in_quote = true,
            '`' => self.in_backtick = true,
            '(' => self.paren_depth += 1,
            ')' => self.paren_depth = self.paren_depth.saturating_sub(1),
            '{' => self.brace_depth += 1,
            '}' => self.brace_depth = self.brace_depth.saturating_sub(1),
            '[' => self.bracket_depth += 1,
            ']' => self.bracket_depth = self.bracket_depth.saturating_sub(1),
            _ => {}
        }
    }
}

use super::ast::{Line, Metadata};
use crate::events::EventLog;

/// Diagnostic origin attached to errors raised by the proto lexer.
///
/// Keeping a distinct origin for this earliest frontend pass makes it easier to
/// tell whether a malformed document failed during raw line handling or during
/// later structural/formulation parsing.
const ORIGIN: &str = "proto_lexer";

/// Line-oriented lexer for the proto MathLingua syntax.
///
/// The proto lexer does not tokenize the language deeply.  It normalizes each
/// physical source line into text plus indentation metadata, while preserving
/// enough information for parser recovery and row-based diagnostics.
pub struct Lexer<'a> {
    /// Preprocessed source lines in original order.
    lines: Vec<Line>,
    /// Index of the next line that will be returned by iteration.
    cursor: usize,
    /// Shared diagnostic sink used when higher parser layers need row errors.
    event_log: &'a mut EventLog,
}

impl<'a> Lexer<'a> {
    /// Builds a lexer over `input` and records diagnostics in `event_log`.
    ///
    /// The whole input is normalized up front because the proto parser needs
    /// cheap peeking and cloning of lines while deciding whether a block is a
    /// formulation, text literal, or nested group.
    pub fn new(input: &str, event_log: &'a mut EventLog) -> Self {
        Self {
            lines: text_to_lines(input),
            cursor: 0,
            event_log,
        }
    }

    /// Returns the next line without consuming it.
    ///
    /// The proto parser relies on this to decide whether indentation still
    /// belongs to the current section or whether control should return to the
    /// enclosing group.
    pub fn peek(&self) -> Option<&Line> {
        self.lines.get(self.cursor)
    }

    /// Emits a lexer-originated error at the given zero-based source row.
    ///
    /// Higher-level proto parsing code uses this helper so all line-shape
    /// diagnostics share a consistent origin and row-only location policy.
    pub fn error(&mut self, row: usize, message: impl Into<String>) {
        self.event_log.user_error_at_row(Some(ORIGIN), row, message);
    }
}

impl Iterator for Lexer<'_> {
    /// Iterator item yielded for each normalized source line.
    type Item = Line;

    /// Consumes and returns the next normalized line.
    ///
    /// Lines are cloned out of the internal buffer so the parser can keep
    /// previously consumed metadata while still using the lexer as a simple
    /// forward-only cursor.
    fn next(&mut self) -> Option<Self::Item> {
        let line = self.lines.get(self.cursor)?.clone();
        self.cursor += 1;
        Some(line)
    }
}

/// Converts raw source text into proto lines with indentation metadata.
///
/// Leading whitespace is counted as indentation and stripped from the line
/// text.  A leading `. ` marker is treated as two additional indentation
/// columns and recorded separately so rendering and diagnostics can reconstruct
/// the author's proto structure.
fn text_to_lines(input: &str) -> Vec<Line> {
    input
        .split('\n')
        .enumerate()
        .map(|(row, raw_line)| {
            let trimmed = raw_line.trim_start();
            let mut indent = raw_line.len() - trimmed.len();
            let mut text = trimmed.to_owned();
            let mut has_dot = false;

            if let Some(stripped) = text.strip_prefix(". ") {
                has_dot = true;
                indent += 2;
                text = stripped.to_owned();
            }

            Line {
                text,
                metadata: Metadata {
                    row,
                    indent,
                    has_dot,
                },
            }
        })
        .collect()
}

// =============================================================================

#[cfg(test)]
mod tests {
    use super::Lexer;
    use crate::events::EventLog;

    #[test]
    fn lexes_indent_and_dot_prefixes() {
        let mut event_log = EventLog::new();
        let lexer = Lexer::new("  . x", &mut event_log);
        let line = lexer.peek().expect("expected one lexed line");

        assert_eq!(line.text, "x");
        assert_eq!(line.metadata.row, 0);
        assert_eq!(line.metadata.indent, 4);
        assert!(line.metadata.has_dot);
        assert_eq!(line.to_string(), "  . x");
    }

    #[test]
    fn identifies_blank_lines_after_trimming_leading_whitespace() {
        let mut event_log = EventLog::new();
        let lexer = Lexer::new("   \n-- comment", &mut event_log);
        let blank = lexer.peek().expect("expected first line");

        assert!(blank.is_blank());
    }

    #[test]
    fn peeks_and_iterates_through_multiple_lines_in_order() {
        let mut event_log = EventLog::new();
        let mut lexer = Lexer::new("alpha\n  . beta\n", &mut event_log);

        let first = lexer.peek().cloned().expect("expected first line");
        assert_eq!(first.text, "alpha");
        assert_eq!(first.metadata.row, 0);
        assert_eq!(first.metadata.indent, 0);
        assert!(!first.metadata.has_dot);
        assert_eq!(lexer.next(), Some(first));

        let second = lexer.peek().cloned().expect("expected second line");
        assert_eq!(second.text, "beta");
        assert_eq!(second.metadata.row, 1);
        assert_eq!(second.metadata.indent, 4);
        assert!(second.metadata.has_dot);
        assert_eq!(lexer.next(), Some(second));

        let trailing_blank = lexer.next().expect("expected trailing blank line");
        assert!(trailing_blank.is_blank());
        assert_eq!(trailing_blank.metadata.row, 2);

        assert!(lexer.peek().is_none());
        assert!(lexer.next().is_none());
    }

    #[test]
    fn only_treats_dot_followed_by_space_as_a_dot_prefix() {
        let mut event_log = EventLog::new();
        let mut lexer = Lexer::new("  .x", &mut event_log);
        let line = lexer.next().expect("expected one line");

        assert_eq!(line.text, ".x");
        assert_eq!(line.metadata.indent, 2);
        assert!(!line.metadata.has_dot);
        assert_eq!(line.to_string(), "  .x");
    }

    #[test]
    fn preserves_trailing_whitespace_after_trimming_leading_whitespace() {
        let mut event_log = EventLog::new();
        let mut lexer = Lexer::new("  value  ", &mut event_log);
        let line = lexer.next().expect("expected one line");

        assert_eq!(line.text, "value  ");
        assert_eq!(line.metadata.indent, 2);
        assert_eq!(line.to_string(), "  value  ");
    }
}

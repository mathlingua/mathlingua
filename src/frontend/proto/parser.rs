use super::ast::{Argument, Formulation, Group, Section, TextLiteral};
use super::lexer::Lexer;
use crate::events::EventLog;

/// Opening and closing delimiters that create multiline formulation blocks.
///
/// A line containing exactly one opening delimiter starts a block.  The block
/// closes only when the matching delimiter appears at the same indentation,
/// which lets inner formulation text contain delimiter-shaped lines safely.
const MULTILINE_FORMULATION_DELIMITERS: [(&str, &str); 4] =
    [("(", ")"), ("[", "]"), ("{", "}"), ("(.", ".)")];

/// Parser for the proto MathLingua document shape.
///
/// This pass recognizes groups, sections, text literals, and raw formulation
/// strings without interpreting mathematical grammar.  It is deliberately
/// recovery-oriented: malformed lines are reported and skipped where possible
/// so later diagnostics can still be produced from the rest of the file.
pub struct Parser<'a> {
    /// Line lexer that provides normalized source lines and row diagnostics.
    lexer: Lexer<'a>,
}

impl<'a> Parser<'a> {
    /// Creates a parser for a raw MathLingua source string.
    ///
    /// Diagnostics are emitted into the supplied event log by both this parser
    /// and the underlying proto lexer.
    pub fn new(input: &str, event_log: &'a mut EventLog) -> Self {
        Self {
            lexer: Lexer::new(input, event_log),
        }
    }

    /// Parses the whole source into top-level proto groups.
    ///
    /// Blank lines and comments may separate groups.  Unexpected top-level
    /// lines are reported and consumed one at a time so the parser can continue
    /// looking for subsequent well-formed groups.
    pub fn parse(&mut self) -> Vec<Group> {
        let mut groups = Vec::new();

        loop {
            self.skip_blank_lines_and_comments();

            let Some(next_line) = self.lexer.peek().cloned() else {
                break;
            };

            match self.parse_group(0) {
                Some(group) => groups.push(group),
                None => {
                    self.lexer.error(
                        next_line.metadata.row,
                        format!("Unexpected line: {}", next_line.text),
                    );
                    let _ = self.lexer.next();
                }
            }
        }

        groups
    }

    /// Consumes all blank lines and comments at the current cursor.
    ///
    /// This is used between top-level groups where both blank space and comment
    /// lines are structural separators.
    fn skip_blank_lines_and_comments(&mut self) {
        while matches!(self.lexer.peek(), Some(line) if line.is_blank_or_comment()) {
            let _ = self.lexer.next();
        }
    }

    /// Consumes only comments at the current cursor.
    ///
    /// Inside groups and sections, blank lines are meaningful block terminators
    /// while comments are ignored, so this helper intentionally leaves blanks in
    /// place for the caller to observe.
    fn skip_comments(&mut self) {
        while matches!(self.lexer.peek(), Some(line) if line.is_comment()) {
            let _ = self.lexer.next();
        }
    }

    /// Parses one group at the requested indentation level.
    ///
    /// A group may start with a bracketed heading, such as `[\set]`, or with a
    /// section line at the current indentation.  The returned group keeps the
    /// metadata from its first physical line so structural diagnostics can point
    /// back to the group start.
    fn parse_group(&mut self, indent: usize) -> Option<Group> {
        self.skip_comments();

        let first_line = self.lexer.peek()?.clone();
        if first_line.is_blank() {
            return None;
        }

        let metadata = first_line.metadata.clone();
        let heading = if first_line.is_header() {
            let _ = self.lexer.next();
            Some(first_line.text[1..first_line.text.len() - 1].to_owned())
        } else {
            if first_line.metadata.indent != indent {
                return None;
            }
            None
        };

        while let Some(line) = self.lexer.peek().cloned() {
            if !line.is_header() {
                break;
            }

            self.lexer.error(
                line.metadata.row,
                format!("Unexpected header: {}", line.text),
            );
            let _ = self.lexer.next();
        }

        let mut sections = Vec::new();
        loop {
            self.skip_comments();

            let Some(line) = self.lexer.peek().cloned() else {
                break;
            };

            if line.is_blank() || line.metadata.indent != indent {
                break;
            }

            match self.parse_section(indent) {
                Some(section) => sections.push(section),
                None => {
                    self.lexer.error(line.metadata.row, "Expected a section");
                    let _ = self.lexer.next();
                }
            }
        }

        Some(Group {
            heading,
            sections,
            metadata,
        })
    }

    /// Parses a single section and all of its arguments.
    ///
    /// Section bodies are expected two spaces deeper than the section line.
    /// Over-indented arguments are diagnosed but still parsed so a single
    /// spacing issue does not discard the rest of the section.
    fn parse_section(&mut self, indent: usize) -> Option<Section> {
        self.skip_comments();

        let first_line = self.lexer.next()?;
        if first_line.is_blank_or_comment() {
            return None;
        }

        if first_line.metadata.indent != indent {
            self.lexer.error(
                first_line.metadata.row,
                format!(
                    "Expected section indent {indent}, found {}",
                    first_line.metadata.indent
                ),
            );
        }

        let (label, inline_argument) = match structural_colon_index(&first_line.text) {
            Some(index) => {
                let (label, rest) = first_line.text.split_at(index);
                let argument = rest[1..].trim();
                (
                    label.to_owned(),
                    self.parse_inline_argument(
                        argument,
                        first_line.metadata.row,
                        first_line.metadata.indent,
                    ),
                )
            }
            None => {
                self.lexer.error(
                    first_line.metadata.row,
                    format!("Expected ':' in section line: {}", first_line.text),
                );
                (first_line.text.clone(), None)
            }
        };

        let mut arguments = Vec::new();
        let expected_indent = indent + 2;

        loop {
            self.skip_comments();

            let Some(line) = self.lexer.peek().cloned() else {
                break;
            };

            if line.is_blank() || line.metadata.indent < expected_indent {
                break;
            }

            if line.metadata.indent > expected_indent {
                self.lexer.error(line.metadata.row, "Unexpected indent");
            }

            match self.parse_argument(expected_indent) {
                Some(argument) => arguments.push(argument),
                None => {
                    self.lexer.error(line.metadata.row, "Expected an argument");
                    let _ = self.lexer.next();
                }
            }
        }

        Some(Section {
            label,
            inline_argument,
            arguments,
            metadata: first_line.metadata,
        })
    }

    /// Parses an inline section argument that appears after `:`.
    ///
    /// Inline text may be the opening line of a multiline formulation block.  A
    /// fully single-quoted inline formulation is accepted as text for recovery
    /// purposes but reported because the current syntax only allows double
    /// quoted text literals.
    fn parse_inline_argument(
        &mut self,
        argument: &str,
        row: usize,
        indent: usize,
    ) -> Option<String> {
        if argument.is_empty() {
            return None;
        }

        let mut text = argument.to_owned();
        if let Some(close_delimiter) = multiline_formulation_close(argument) {
            text = self.consume_multiline_formulation(text, close_delimiter, row, indent);
        } else if is_single_quoted_formulation(argument) {
            self.lexer
                .error(row, "Single-quoted formulations are not allowed");
        }

        Some(text)
    }

    /// Parses one raw formulation argument at the current cursor.
    ///
    /// Formulations are any nonblank, noncomment, nonheader, nontext argument
    /// lines.  This function preserves the formulation text verbatim, including
    /// multiline delimiter blocks, so the formulation parser can later handle
    /// the mathematical syntax.
    fn parse_formulation(&mut self) -> Option<Formulation> {
        if !matches!(
            self.lexer.peek(),
            Some(line)
                if !line.is_blank_or_comment() && !line.is_header() && !line.is_text()
        ) {
            return None;
        }

        let line = self.lexer.next()?;
        let text = if let Some(close_delimiter) = multiline_formulation_close(&line.text) {
            self.consume_multiline_formulation(
                line.text,
                close_delimiter,
                line.metadata.row,
                line.metadata.indent,
            )
        } else {
            if is_single_quoted_formulation(&line.text) {
                self.lexer.error(
                    line.metadata.row,
                    "Single-quoted formulations are not allowed",
                );
            }
            line.text
        };

        Some(Formulation {
            text,
            metadata: line.metadata,
        })
    }

    /// Parses one double-quoted text literal argument.
    ///
    /// Text literals stay quoted at the proto layer.  Structural parsing strips
    /// the quotes later once it knows which section-specific text wrapper is
    /// being populated.
    fn parse_text(&mut self) -> Option<TextLiteral> {
        if !matches!(self.lexer.peek(), Some(line) if line.is_text()) {
            return None;
        }

        let line = self.lexer.next()?;
        Some(TextLiteral {
            text: line.text,
            metadata: line.metadata,
        })
    }

    /// Parses the next section argument using text, group, then formulation order.
    ///
    /// Text lines are recognized first because quoted strings may contain
    /// characters that look like formulations.  Group detection happens before
    /// formulation parsing so nested section blocks remain structured.
    fn parse_argument(&mut self, indent: usize) -> Option<Argument> {
        if let Some(text) = self.parse_text() {
            return Some(Argument::Text(text));
        }

        if self.peek_starts_group() {
            return self.parse_group(indent).map(Argument::Group);
        }

        self.parse_formulation().map(Argument::Formulation)
    }

    /// Returns whether the next line can start a nested group.
    ///
    /// Bracketed headings always start groups.  Otherwise, a nontext argument
    /// starts a group only when it has a section-shaped label before `:`.
    fn peek_starts_group(&self) -> bool {
        let Some(line) = self.lexer.peek() else {
            return false;
        };

        if line.is_blank_or_comment() || line.is_text() {
            return false;
        }

        line.is_header() || structural_colon_index(&line.text).is_some()
    }

    /// Consumes a multiline formulation until its matching closing delimiter.
    ///
    /// Closing must occur on a line with the same indentation as the opener.
    /// If the source ends first, the partial block is returned so downstream
    /// parsing can still inspect it after the unterminated-block diagnostic.
    fn consume_multiline_formulation(
        &mut self,
        opening_line: String,
        close_delimiter: &str,
        row: usize,
        indent: usize,
    ) -> String {
        let mut lines = vec![opening_line.clone()];

        loop {
            let Some(line) = self.lexer.next() else {
                self.lexer.error(
                    row,
                    format!("Unterminated formulation block starting with {opening_line}"),
                );
                return lines.join("\n");
            };

            let rendered = line.to_string();
            let is_closing_line = line.metadata.indent == indent && line.text == close_delimiter;
            lines.push(rendered);

            if is_closing_line {
                return lines.join("\n");
            }
        }
    }
}

/// Finds the colon that separates a section label from its inline argument.
///
/// Formulation operators such as `:=`, `:->`, `:=>`, `:~>`, and command tails
/// such as `\foo{A}:with{B}` can appear in section bodies.  A structural colon
/// therefore requires a section-label-shaped prefix.
fn structural_colon_index(text: &str) -> Option<usize> {
    let index = text.find(':')?;
    let prefix = text[..index].trim();
    let rest = &text[index..];

    if matches!(rest, ":=" | ":->" | ":=>" | ":~>" | ":/")
        || rest.starts_with(":=")
        || rest.starts_with(":->")
        || rest.starts_with(":=>")
        || rest.starts_with(":~>")
        || rest.starts_with(":/")
    {
        return None;
    }

    if prefix.is_empty()
        || !prefix
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
    {
        return None;
    }

    Some(index)
}

/// Returns the required closing delimiter when `text` opens a multiline block.
///
/// The match is exact by design: delimiters only have block meaning when they
/// occupy the whole proto line after indentation stripping.
fn multiline_formulation_close(text: &str) -> Option<&'static str> {
    MULTILINE_FORMULATION_DELIMITERS
        .iter()
        .find_map(|(open, close)| (*open == text).then_some(*close))
}

/// Detects legacy single-quoted formulation text.
///
/// Single-quoted formulations are no longer valid syntax, but detecting them
/// separately lets the parser report a targeted diagnostic instead of a generic
/// malformed argument error.
fn is_single_quoted_formulation(text: &str) -> bool {
    text.len() >= 2 && text.starts_with('\'') && text.ends_with('\'')
}

// =============================================================================

#[cfg(test)]
mod tests;

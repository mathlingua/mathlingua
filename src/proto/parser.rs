use super::ast::{Argument, Formulation, Group, Section, TextLiteral};
use super::lexer::Lexer;
use crate::diagnostics::DiagnosticTracker;

const MULTILINE_FORMULATION_DELIMITERS: [(&str, &str); 4] =
    [("(", ")"), ("[", "]"), ("{", "}"), ("(.", ".)")];

#[derive(Debug)]
pub struct Parser {
    lexer: Lexer,
}

impl Parser {
    pub fn new(input: &str) -> Self {
        Self {
            lexer: Lexer::new(input),
        }
    }

    pub fn parse(&mut self, diagnostics: &mut DiagnosticTracker) -> Vec<Group> {
        let mut groups = Vec::new();

        loop {
            self.skip_blank_lines_and_comments();

            let Some(next_line) = self.lexer.peek().cloned() else {
                break;
            };

            match self.parse_group(0, diagnostics) {
                Some(group) => groups.push(group),
                None => {
                    diagnostics.error(
                        next_line.metadata.row,
                        format!("Unexpected line: {}", next_line.text),
                    );
                    let _ = self.lexer.next();
                }
            }
        }

        groups
    }

    fn skip_blank_lines_and_comments(&mut self) {
        while matches!(self.lexer.peek(), Some(line) if line.is_blank_or_comment()) {
            let _ = self.lexer.next();
        }
    }

    fn skip_comments(&mut self) {
        while matches!(self.lexer.peek(), Some(line) if line.is_comment()) {
            let _ = self.lexer.next();
        }
    }

    fn parse_group(&mut self, indent: usize, diagnostics: &mut DiagnosticTracker) -> Option<Group> {
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

            diagnostics.error(
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

            match self.parse_section(indent, diagnostics) {
                Some(section) => sections.push(section),
                None => {
                    diagnostics.error(line.metadata.row, "Expected a section");
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

    fn parse_section(
        &mut self,
        indent: usize,
        diagnostics: &mut DiagnosticTracker,
    ) -> Option<Section> {
        self.skip_comments();

        let first_line = self.lexer.next()?;
        if first_line.is_blank_or_comment() {
            return None;
        }

        if first_line.metadata.indent != indent {
            diagnostics.error(
                first_line.metadata.row,
                format!(
                    "Expected section indent {indent}, found {}",
                    first_line.metadata.indent
                ),
            );
        }

        let (label, inline_argument) = match first_line.text.split_once(':') {
            Some((label, rest)) => {
                let argument = rest.trim();
                (
                    label.to_owned(),
                    self.parse_inline_argument(
                        argument,
                        first_line.metadata.row,
                        first_line.metadata.indent,
                        diagnostics,
                    ),
                )
            }
            None => {
                diagnostics.error(
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
                diagnostics.error(line.metadata.row, "Unexpected indent");
            }

            match self.parse_argument(expected_indent, diagnostics) {
                Some(argument) => arguments.push(argument),
                None => {
                    diagnostics.error(line.metadata.row, "Expected an argument");
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

    fn parse_inline_argument(
        &mut self,
        argument: &str,
        row: usize,
        indent: usize,
        diagnostics: &mut DiagnosticTracker,
    ) -> Option<String> {
        if argument.is_empty() {
            return None;
        }

        let mut text = argument.to_owned();
        if let Some(close_delimiter) = multiline_formulation_close(argument) {
            text =
                self.consume_multiline_formulation(text, close_delimiter, row, indent, diagnostics);
        } else if is_single_quoted_formulation(argument) {
            diagnostics.error(row, "Single-quoted formulations are not allowed");
        }

        Some(text)
    }

    fn parse_formulation(&mut self, diagnostics: &mut DiagnosticTracker) -> Option<Formulation> {
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
                diagnostics,
            )
        } else {
            if is_single_quoted_formulation(&line.text) {
                diagnostics.error(
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

    fn parse_argument(
        &mut self,
        indent: usize,
        diagnostics: &mut DiagnosticTracker,
    ) -> Option<Argument> {
        if let Some(text) = self.parse_text() {
            return Some(Argument::Text(text));
        }

        if self.peek_starts_group() {
            return self.parse_group(indent, diagnostics).map(Argument::Group);
        }

        self.parse_formulation(diagnostics)
            .map(Argument::Formulation)
    }

    fn peek_starts_group(&self) -> bool {
        let Some(line) = self.lexer.peek() else {
            return false;
        };

        if line.is_blank_or_comment() || line.is_text() {
            return false;
        }

        line.is_header() || line.text.contains(':')
    }

    fn consume_multiline_formulation(
        &mut self,
        opening_line: String,
        close_delimiter: &str,
        row: usize,
        indent: usize,
        diagnostics: &mut DiagnosticTracker,
    ) -> String {
        let mut lines = vec![opening_line.clone()];

        loop {
            let Some(line) = self.lexer.next() else {
                diagnostics.error(
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

fn multiline_formulation_close(text: &str) -> Option<&'static str> {
    MULTILINE_FORMULATION_DELIMITERS
        .iter()
        .find_map(|(open, close)| (*open == text).then_some(*close))
}

fn is_single_quoted_formulation(text: &str) -> bool {
    text.len() >= 2 && text.starts_with('\'') && text.ends_with('\'')
}

#[cfg(test)]
mod tests;

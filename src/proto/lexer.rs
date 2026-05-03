use super::ast::{Line, Metadata};
use crate::diagnostics::DiagnosticTracker;

#[derive(Debug)]
pub struct Lexer<'a> {
    lines: Vec<Line>,
    cursor: usize,
    diagnostics: &'a mut DiagnosticTracker,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &str, diagnostics: &'a mut DiagnosticTracker) -> Self {
        Self {
            lines: text_to_lines(input),
            cursor: 0,
            diagnostics,
        }
    }

    pub fn peek(&self) -> Option<&Line> {
        self.lines.get(self.cursor)
    }

    pub fn error(&mut self, row: usize, message: impl Into<String>) {
        self.diagnostics.error(row, message);
    }
}

impl Iterator for Lexer<'_> {
    type Item = Line;

    fn next(&mut self) -> Option<Self::Item> {
        let line = self.lines.get(self.cursor)?.clone();
        self.cursor += 1;
        Some(line)
    }
}

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

#[cfg(test)]
mod tests;

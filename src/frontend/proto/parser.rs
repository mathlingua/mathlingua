use super::ast::{Argument, Formulation, Group, Section, TextLiteral};
use super::lexer::Lexer;
use crate::events::EventLog;

const MULTILINE_FORMULATION_DELIMITERS: [(&str, &str); 4] =
    [("(", ")"), ("[", "]"), ("{", "}"), ("(.", ".)")];

pub struct Parser<'a> {
    lexer: Lexer<'a>,
}

impl<'a> Parser<'a> {
    pub fn new(input: &str, event_log: &'a mut EventLog) -> Self {
        Self {
            lexer: Lexer::new(input, event_log),
        }
    }

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

    fn parse_argument(&mut self, indent: usize) -> Option<Argument> {
        if let Some(text) = self.parse_text() {
            return Some(Argument::Text(text));
        }

        if self.peek_starts_group() {
            return self.parse_group(indent).map(Argument::Group);
        }

        self.parse_formulation().map(Argument::Formulation)
    }

    fn peek_starts_group(&self) -> bool {
        let Some(line) = self.lexer.peek() else {
            return false;
        };

        if line.is_blank_or_comment() || line.is_text() {
            return false;
        }

        line.is_header() || structural_colon_index(&line.text).is_some()
    }

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
/// Formulation operators such as `::=`, `:=`, `:?`, `:->`, `:=>`, `:~>`, and command tails
/// such as `\foo{A}:with{B}` can appear in section bodies.  A structural colon
/// therefore requires a section-label-shaped prefix.
fn structural_colon_index(text: &str) -> Option<usize> {
    let index = text.find(':')?;
    let prefix = text[..index].trim();
    let rest = &text[index..];

    if matches!(rest, "::=" | ":=" | ":?" | ":->" | ":=>" | ":~>")
        || rest.starts_with("::=")
        || rest.starts_with(":=")
        || rest.starts_with(":?")
        || rest.starts_with(":->")
        || rest.starts_with(":=>")
        || rest.starts_with(":~>")
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

// ===============================[ tests ]=====================================

#[cfg(test)]
mod tests {
    use super::{Parser, is_single_quoted_formulation, multiline_formulation_close};
    use crate::events::{Event, EventLog};
    use crate::frontend::proto::ast::{Argument, Group};

    fn parse_input(input: &str) -> (Vec<Group>, Vec<Event>) {
        let mut event_log = EventLog::new();
        let groups = {
            let mut parser = Parser::new(input, &mut event_log);
            parser.parse()
        };

        (groups, event_log.events().to_vec())
    }

    #[test]
    fn parses_sections_and_nested_groups() {
        let input = r#"
-- comment

[heading]
Defines: f(x_)
when:
. x "in" A
. y "in" B
then:
. exists: z
  suchThat:
  . "abc"
"#;
        let mut event_log = EventLog::new();
        let groups = {
            let mut parser = Parser::new(input, &mut event_log);
            parser.parse()
        };

        assert!(event_log.events().is_empty());
        assert_eq!(groups.len(), 1);

        let group = &groups[0];
        assert_eq!(group.heading.as_deref(), Some("heading"));
        assert_eq!(group.sections.len(), 3);
        assert_eq!(group.sections[0].label, "Defines");
        assert_eq!(group.sections[0].inline_argument.as_deref(), Some("f(x_)"));

        let when_arguments = &group.sections[1].arguments;
        assert_eq!(when_arguments.len(), 2);
        assert!(matches!(
            &when_arguments[0],
            Argument::Formulation(item) if item.text == "x \"in\" A"
        ));
        assert!(matches!(
            &when_arguments[1],
            Argument::Formulation(item) if item.text == "y \"in\" B"
        ));

        let then_arguments = &group.sections[2].arguments;
        assert_eq!(then_arguments.len(), 1);
        match &then_arguments[0] {
            Argument::Group(group) => {
                assert_eq!(group.sections.len(), 2);
                assert_eq!(group.sections[0].label, "exists");
                assert_eq!(group.sections[0].inline_argument.as_deref(), Some("z"));
                assert_eq!(group.sections[1].label, "suchThat");
                assert_eq!(group.sections[1].arguments.len(), 1);
                assert!(matches!(
                    &group.sections[1].arguments[0],
                    Argument::Text(item) if item.text == "\"abc\""
                ));
            }
            other => panic!("expected nested group, got {other:?}"),
        }
    }

    #[test]
    fn reports_unexpected_headers_and_continues() {
        let input = r#"[heading]
[duplicate]
Defines: f(x_)
"#;
        let mut event_log = EventLog::new();
        let groups = {
            let mut parser = Parser::new(input, &mut event_log);
            parser.parse()
        };
        let events = event_log.events();

        assert_eq!(groups.len(), 1);
        assert_eq!(events.len(), 1);
        assert_eq!(
            events[0].as_message().unwrap().message,
            "Unexpected header: [duplicate]"
        );
    }

    #[test]
    fn skips_comments_inside_groups_and_arguments() {
        let input = r#"when:
-- comment
. x
-- another comment
. y
"#;
        let mut event_log = EventLog::new();
        let groups = {
            let mut parser = Parser::new(input, &mut event_log);
            parser.parse()
        };

        assert!(event_log.events().is_empty());
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].sections[0].arguments.len(), 2);
    }

    #[test]
    fn reports_malformed_sections_without_panicking() {
        let input = r#"broken section
Defines: f(x_)
"#;
        let mut event_log = EventLog::new();
        let groups = {
            let mut parser = Parser::new(input, &mut event_log);
            parser.parse()
        };
        let events = event_log.events();

        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].sections.len(), 2);
        assert_eq!(groups[0].sections[0].label, "broken section");
        assert_eq!(events.len(), 1);
        assert_eq!(
            events[0].as_message().unwrap().message,
            "Expected ':' in section line: broken section"
        );
    }

    #[test]
    fn reports_unexpected_indentation_in_arguments() {
        let input = "when:\n    x\n";
        let mut event_log = EventLog::new();
        let groups = {
            let mut parser = Parser::new(input, &mut event_log);
            parser.parse()
        };
        let events = event_log.events();

        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].sections[0].arguments.len(), 1);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].as_message().unwrap().message, "Unexpected indent");
    }

    #[test]
    fn parses_multiline_formulations() {
        let input = r#"[heading]
Defines: (
  f(
    x_
)
when:
. (
    x "in" A
  )
"#;
        let mut event_log = EventLog::new();
        let groups = {
            let mut parser = Parser::new(input, &mut event_log);
            parser.parse()
        };

        assert!(event_log.events().is_empty());
        assert_eq!(groups.len(), 1);
        assert_eq!(
            groups[0].sections[0].inline_argument.as_deref(),
            Some("(\n  f(\n    x_\n)")
        );
        assert!(matches!(
            &groups[0].sections[1].arguments[0],
            Argument::Formulation(item) if item.text == "(\n    x \"in\" A\n  )"
        ));
        assert_eq!(
            groups[0].to_string(),
            "[heading]\nDefines: (\n  f(\n    x_\n)\nwhen:\n. (\n    x \"in\" A\n  )"
        );
    }

    #[test]
    fn parses_dot_delimited_multiline_formulations() {
        let input = "when:\n. (.\n    x + y\n  .)\n";
        let mut event_log = EventLog::new();
        let groups = {
            let mut parser = Parser::new(input, &mut event_log);
            parser.parse()
        };

        assert!(event_log.events().is_empty());
        assert!(matches!(
            &groups[0].sections[0].arguments[0],
            Argument::Formulation(item) if item.text == "(.\n    x + y\n  .)"
        ));
    }

    #[test]
    fn reports_single_quoted_formulations_as_invalid() {
        let input = "Defines: 'f(x_)'\nwhen:\n. 'x in A'\n";
        let mut event_log = EventLog::new();
        let groups = {
            let mut parser = Parser::new(input, &mut event_log);
            parser.parse()
        };
        let events = event_log.events();

        assert_eq!(groups.len(), 1);
        assert_eq!(events.len(), 2);
        assert_eq!(
            events[0].as_message().unwrap().message,
            "Single-quoted formulations are not allowed"
        );
        assert_eq!(
            events[1].as_message().unwrap().message,
            "Single-quoted formulations are not allowed"
        );
    }

    #[test]
    fn reports_unterminated_multiline_formulations() {
        let input = "when:\n. (\n    x in A\n";
        let mut event_log = EventLog::new();
        let groups = {
            let mut parser = Parser::new(input, &mut event_log);
            parser.parse()
        };
        let events = event_log.events();

        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].sections[0].arguments.len(), 1);
        assert_eq!(events.len(), 1);
        assert_eq!(
            events[0].as_message().unwrap().message,
            "Unterminated formulation block starting with ("
        );
    }

    #[test]
    fn parses_empty_input_as_no_groups() {
        let (groups, diagnostics) = parse_input("\n-- comment\n  \n");

        assert!(groups.is_empty());
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn parses_multiple_top_level_groups_separated_by_blank_lines() {
        let input = r#"Defines: f(x_)
when:
. x in A

States:
. "Claim"
that:
. y in B
"#;

        let (groups, diagnostics) = parse_input(input);

        assert!(diagnostics.is_empty());
        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].sections[0].label, "Defines");
        assert_eq!(groups[1].sections[0].label, "States");
        assert!(matches!(
            &groups[1].sections[0].arguments[0],
            Argument::Text(item) if item.text == "\"Claim\""
        ));
    }

    #[test]
    fn reports_unexpected_top_level_lines_and_recovers() {
        let input = r#"  orphan
Defines: f(x_)
"#;

        let (groups, diagnostics) = parse_input(input);

        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].sections[0].label, "Defines");
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(
            diagnostics[0].as_message().unwrap().message,
            "Unexpected line: orphan"
        );
    }

    #[test]
    fn treats_text_with_colons_as_text_arguments_instead_of_groups() {
        let input = r#"when:
. "label: value"
. A ::= B := B
"#;

        let (groups, diagnostics) = parse_input(input);

        assert!(diagnostics.is_empty());
        assert!(matches!(
            &groups[0].sections[0].arguments[0],
            Argument::Text(item) if item.text == "\"label: value\""
        ));
        assert!(matches!(
            &groups[0].sections[0].arguments[1],
            Argument::Formulation(item) if item.text == "A ::= B := B"
        ));
    }

    #[test]
    fn reports_group_like_arguments_with_too_much_indent_and_continues() {
        let input = r#"when:
    child: value
. x"#;

        let (groups, diagnostics) = parse_input(input);

        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].sections[0].arguments.len(), 1);
        assert!(matches!(
            &groups[0].sections[0].arguments[0],
            Argument::Formulation(item) if item.text == "x"
        ));
        assert_eq!(diagnostics.len(), 2);
        assert_eq!(
            diagnostics[0].as_message().unwrap().message,
            "Unexpected indent"
        );
        assert_eq!(
            diagnostics[1].as_message().unwrap().message,
            "Expected an argument"
        );
    }

    #[test]
    fn parses_bracket_and_brace_multiline_formulations() {
        let input = r#"Defines: [
  f(x_)
]
when:
. {
    x in A
  }
"#;

        let (groups, diagnostics) = parse_input(input);

        assert!(diagnostics.is_empty());
        assert_eq!(
            groups[0].sections[0].inline_argument.as_deref(),
            Some("[\n  f(x_)\n]")
        );
        assert!(matches!(
            &groups[0].sections[1].arguments[0],
            Argument::Formulation(item) if item.text == "{\n    x in A\n  }"
        ));
    }

    #[test]
    fn requires_matching_indent_to_close_multiline_formulations() {
        let input = r#"when:
. (
    x in A
    )
  )
"#;

        let (groups, diagnostics) = parse_input(input);

        assert!(diagnostics.is_empty());
        assert!(matches!(
            &groups[0].sections[0].arguments[0],
            Argument::Formulation(item) if item.text == "(\n    x in A\n    )\n  )"
        ));
    }

    #[test]
    fn recognizes_all_multiline_formulation_delimiters() {
        assert_eq!(multiline_formulation_close("("), Some(")"));
        assert_eq!(multiline_formulation_close("["), Some("]"));
        assert_eq!(multiline_formulation_close("{"), Some("}"));
        assert_eq!(multiline_formulation_close("(."), Some(".)"));
        assert_eq!(multiline_formulation_close("x"), None);
    }

    #[test]
    fn only_recognizes_fully_wrapped_single_quoted_formulations() {
        assert!(is_single_quoted_formulation("'x'"));
        assert!(!is_single_quoted_formulation("'x"));
        assert!(!is_single_quoted_formulation("x'"));
        assert!(!is_single_quoted_formulation("\"x\""));
    }
}

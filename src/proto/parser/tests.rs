use super::{Parser, is_single_quoted_formulation, multiline_formulation_close};
use crate::diagnostics::{Diagnostic, DiagnosticTracker};
use crate::proto::ast::{Argument, Group};

fn parse_input(input: &str) -> (Vec<Group>, Vec<Diagnostic>) {
    let mut tracker = DiagnosticTracker::new();
    let mut parser = Parser::new(input);
    let groups = parser.parse(&mut tracker);

    (groups, tracker.diagnostics().to_vec())
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
    let mut tracker = DiagnosticTracker::new();
    let mut parser = Parser::new(input);

    let groups = parser.parse(&mut tracker);

    assert!(tracker.diagnostics().is_empty());
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
    let mut tracker = DiagnosticTracker::new();
    let mut parser = Parser::new(input);

    let groups = parser.parse(&mut tracker);
    let diagnostics = tracker.diagnostics();

    assert_eq!(groups.len(), 1);
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].message, "Unexpected header: [duplicate]");
}

#[test]
fn skips_comments_inside_groups_and_arguments() {
    let input = r#"when:
-- comment
. x
-- another comment
. y
"#;
    let mut tracker = DiagnosticTracker::new();
    let mut parser = Parser::new(input);

    let groups = parser.parse(&mut tracker);

    assert!(tracker.diagnostics().is_empty());
    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0].sections[0].arguments.len(), 2);
}

#[test]
fn reports_malformed_sections_without_panicking() {
    let input = r#"broken section
Defines: f(x_)
"#;
    let mut tracker = DiagnosticTracker::new();
    let mut parser = Parser::new(input);

    let groups = parser.parse(&mut tracker);
    let diagnostics = tracker.diagnostics();

    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0].sections.len(), 2);
    assert_eq!(groups[0].sections[0].label, "broken section");
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(
        diagnostics[0].message,
        "Expected ':' in section line: broken section"
    );
}

#[test]
fn reports_unexpected_indentation_in_arguments() {
    let input = "when:\n    x\n";
    let mut tracker = DiagnosticTracker::new();
    let mut parser = Parser::new(input);

    let groups = parser.parse(&mut tracker);
    let diagnostics = tracker.diagnostics();

    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0].sections[0].arguments.len(), 1);
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].message, "Unexpected indent");
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
    let mut tracker = DiagnosticTracker::new();
    let mut parser = Parser::new(input);

    let groups = parser.parse(&mut tracker);

    assert!(tracker.diagnostics().is_empty());
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
    let mut tracker = DiagnosticTracker::new();
    let mut parser = Parser::new(input);

    let groups = parser.parse(&mut tracker);

    assert!(tracker.diagnostics().is_empty());
    assert!(matches!(
        &groups[0].sections[0].arguments[0],
        Argument::Formulation(item) if item.text == "(.\n    x + y\n  .)"
    ));
}

#[test]
fn reports_single_quoted_formulations_as_invalid() {
    let input = "Defines: 'f(x_)'\nwhen:\n. 'x in A'\n";
    let mut tracker = DiagnosticTracker::new();
    let mut parser = Parser::new(input);

    let groups = parser.parse(&mut tracker);
    let diagnostics = tracker.diagnostics();

    assert_eq!(groups.len(), 1);
    assert_eq!(diagnostics.len(), 2);
    assert_eq!(
        diagnostics[0].message,
        "Single-quoted formulations are not allowed"
    );
    assert_eq!(
        diagnostics[1].message,
        "Single-quoted formulations are not allowed"
    );
}

#[test]
fn reports_unterminated_multiline_formulations() {
    let input = "when:\n. (\n    x in A\n";
    let mut tracker = DiagnosticTracker::new();
    let mut parser = Parser::new(input);

    let groups = parser.parse(&mut tracker);
    let diagnostics = tracker.diagnostics();

    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0].sections[0].arguments.len(), 1);
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(
        diagnostics[0].message,
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
    let input = "  orphan\nDefines: f(x_)\n";

    let (groups, diagnostics) = parse_input(input);

    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0].sections[0].label, "Defines");
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].message, "Unexpected line: orphan");
}

#[test]
fn treats_text_with_colons_as_text_arguments_instead_of_groups() {
    let input = "when:\n. \"label: value\"\n";

    let (groups, diagnostics) = parse_input(input);

    assert!(diagnostics.is_empty());
    assert!(matches!(
        &groups[0].sections[0].arguments[0],
        Argument::Text(item) if item.text == "\"label: value\""
    ));
}

#[test]
fn reports_group_like_arguments_with_too_much_indent_and_continues() {
    let input = "when:\n    child: value\n. x\n";

    let (groups, diagnostics) = parse_input(input);

    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0].sections[0].arguments.len(), 1);
    assert!(matches!(
        &groups[0].sections[0].arguments[0],
        Argument::Formulation(item) if item.text == "x"
    ));
    assert_eq!(diagnostics.len(), 2);
    assert_eq!(diagnostics[0].message, "Unexpected indent");
    assert_eq!(diagnostics[1].message, "Expected an argument");
}

#[test]
fn parses_bracket_and_brace_multiline_formulations() {
    let input = "Defines: [\n  f(x_)\n]\nwhen:\n. {\n    x in A\n  }\n";

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
    let input = "when:\n. (\n    x in A\n    )\n  )\n";

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

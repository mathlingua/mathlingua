use super::Lexer;
use crate::diagnostics::DiagnosticTracker;

#[test]
fn lexes_indent_and_dot_prefixes() {
    let mut tracker = DiagnosticTracker::new();
    let lexer = Lexer::new("  . x", &mut tracker);
    let line = lexer.peek().expect("expected one lexed line");

    assert_eq!(line.text, "x");
    assert_eq!(line.metadata.row, 0);
    assert_eq!(line.metadata.indent, 4);
    assert!(line.metadata.has_dot);
    assert_eq!(line.to_string(), "  . x");
}

#[test]
fn identifies_blank_lines_after_trimming_leading_whitespace() {
    let mut tracker = DiagnosticTracker::new();
    let lexer = Lexer::new("   \n-- comment", &mut tracker);
    let blank = lexer.peek().expect("expected first line");

    assert!(blank.is_blank());
}

#[test]
fn peeks_and_iterates_through_multiple_lines_in_order() {
    let mut tracker = DiagnosticTracker::new();
    let mut lexer = Lexer::new("alpha\n  . beta\n", &mut tracker);

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
    let mut tracker = DiagnosticTracker::new();
    let mut lexer = Lexer::new("  .x", &mut tracker);
    let line = lexer.next().expect("expected one line");

    assert_eq!(line.text, ".x");
    assert_eq!(line.metadata.indent, 2);
    assert!(!line.metadata.has_dot);
    assert_eq!(line.to_string(), "  .x");
}

#[test]
fn preserves_trailing_whitespace_after_trimming_leading_whitespace() {
    let mut tracker = DiagnosticTracker::new();
    let mut lexer = Lexer::new("  value  ", &mut tracker);
    let line = lexer.next().expect("expected one line");

    assert_eq!(line.text, "value  ");
    assert_eq!(line.metadata.indent, 2);
    assert_eq!(line.to_string(), "  value  ");
}

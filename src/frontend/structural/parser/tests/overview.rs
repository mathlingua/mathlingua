#[test]
fn parses_mixed_structural_document() {
    let text = r#"
[\function]
Describes: f(x_)
using:
. x is \type{A}
when:
. x = x
Provides:
. [symbol]
  symbol: f(x_) :=> x
Aliases:
. [alias]
  alias: f(x_) :=> x
References:
. $elements
Metadata:
. id: "desc-1"

[\statement]
States:
that:
. if:
  . x = x
  then:
  . x = x

[@euclid]
Person:
name:
. "Euclid"
biography: "Greek mathematician"

[$elements]
Resource:
. title: "Elements"
"#;

    let mut tracker = EventLog::new();
    let document = parse_document(text, &mut tracker);

    assert!(!tracker.has_errors(), "{:#?}", tracker.events());
    assert_eq!(document.items.len(), 4);
    assert!(matches!(document.items[0], TopLevelItem::Describes(_)));
    assert!(matches!(document.items[1], TopLevelItem::States(_)));
    assert!(matches!(document.items[2], TopLevelItem::Person(_)));
    assert!(matches!(document.items[3], TopLevelItem::Resource(_)));
}

#[test]
fn recovers_after_invalid_group() {
    let text = r#"
[\function]
Describes: f(x_)
that:
. x = x

Title: "Valid Title"
"#;

    let mut tracker = EventLog::new();
    let document = parse_document(text, &mut tracker);

    assert!(tracker.has_errors());
    assert_eq!(document.items.len(), 1);
    assert!(matches!(document.items[0], TopLevelItem::Title(_)));
}

#[test]
fn parses_clause_groups_as_clauses() {
    let text = r#"
[\property]
States:
that:
. exists: x is \type{A}
  suchThat:
  . x = x
"#;

    let mut tracker = EventLog::new();
    let document = parse_document(text, &mut tracker);

    assert!(!tracker.has_errors(), "{:#?}", tracker.events());
    match &document.items[0] {
        TopLevelItem::States(states) => {
            assert!(matches!(states.that.arguments[0], Clause::Exists(_)));
        }
        other => panic!("expected states item, got {other:?}"),
    }
}

#[test]
fn parses_is_statements_as_inline_clauses() {
    let text = r#"
[\function:on{A}:to{B}]
Describes: f(x__)
when:
. A, B is \set
satisfies:
. forAll: x "in" A
  then:
  . existsUnique: y "in" B
    suchThat:
    . f(x) = y
"#;

    let mut tracker = EventLog::new();
    let document = parse_document(text, &mut tracker);

    assert!(!tracker.has_errors(), "{:#?}", tracker.events());
    match &document.items[0] {
        TopLevelItem::Describes(group) => {
            assert!(matches!(
                group.when.as_ref().expect("expected when").arguments[0],
                Clause::IsOrSpec(_)
            ));
            assert!(matches!(
                group
                    .satisfies
                    .as_ref()
                    .expect("expected satisfies")
                    .arguments[0],
                Clause::ForAll(_)
            ));
        }
        other => panic!("expected describes item, got {other:?}"),
    }
}

#[test]
fn parses_outline_groups() {
    let document = parse_ok(
        r#"
Title: "Foundations"

Section: "Sets"

Subsection: "Membership"

Subsubsection: "Examples"
"#,
    );

    assert_eq!(document.items.len(), 4);

    match &document.items[0] {
        TopLevelItem::Title(group) => assert_eq!(group.title.argument.0, "Foundations"),
        other => panic!("expected title group, got {other:?}"),
    }
    match &document.items[1] {
        TopLevelItem::Section(group) => assert_eq!(group.section.argument.0, "Sets"),
        other => panic!("expected section group, got {other:?}"),
    }
    match &document.items[2] {
        TopLevelItem::Subsection(group) => {
            assert_eq!(group.subsection.argument.0, "Membership")
        }
        other => panic!("expected subsection group, got {other:?}"),
    }
    match &document.items[3] {
        TopLevelItem::Subsubsection(group) => {
            assert_eq!(group.subsubsection.argument.0, "Examples")
        }
        other => panic!("expected subsubsection group, got {other:?}"),
    }
}


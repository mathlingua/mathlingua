//! Integration tests for the structural parser.
//!
//! These tests exercise [`parse_document`] end-to-end and assert on the
//! resulting [`Document`] or on diagnostic events emitted via the event log.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use super::parse_document;
use crate::events::{Event, EventLog};
use crate::frontend::structural::ast::{
    AliasItem, AliasKind, Clause, Document, DocumentedItem, IsOrViaItem, JustifiedItem,
    MetadataItem, ProvidesItem, ResourceItem, SpecifyItem, TopLevelItem,
};

fn split_test_chunks(text: &str) -> Vec<String> {
    text.replace("\r\n", "\n")
        .split("\n\n")
        .filter_map(|entry| {
            let entry = entry.trim();
            (!entry.is_empty()).then(|| entry.to_owned())
        })
        .collect()
}

fn read_test_chunks(path: &Path) -> Vec<String> {
    let text = fs::read_to_string(path).unwrap_or_else(|error| {
        panic!(
            "expected structural golden file {}: {error}",
            path.display()
        )
    });
    split_test_chunks(&text)
}

fn read_test_files(directory: &Path, extension: &str) -> Vec<PathBuf> {
    let mut files = fs::read_dir(directory)
        .unwrap_or_else(|error| panic!("expected directory {}: {error}", directory.display()))
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| path.extension().and_then(|value| value.to_str()) == Some(extension))
        .collect::<Vec<_>>();
    files.sort();
    files
}

fn file_name(path: &Path) -> String {
    path.file_name()
        .and_then(|value| value.to_str())
        .expect("expected valid utf-8 file name")
        .to_owned()
}

fn parse_ok(text: &str) -> Document {
    let mut tracker = EventLog::new();
    let document = parse_document(text, &mut tracker);

    assert!(!tracker.has_errors(), "{:#?}", tracker.events());

    document
}

fn parse_with_diagnostics(text: &str) -> (Document, Vec<Event>) {
    let mut tracker = EventLog::new();
    let document = parse_document(text, &mut tracker);
    let messages = tracker.events().to_vec();

    (document, messages)
}

// ===============================[ definitions ]=====================================

#[test]
fn parses_definition_like_groups_with_nested_sections_and_items() {
    let document = parse_ok(
        r#"
[\structure]
Describes: S := (X, *)
using:
. X is \set
. X "contains" Element
when:
. [logic.when]
  allOf:
  . x = x
  . y = y
extends: X is \set via (X, Y)
specifies:
. Y is \set via (X, Y)
. y "contains" Y
satisfies:
. [logic.satisfies]
  not:
  . x = y
Provides:
. [symbol.plus]
  symbol: plus(x_, y_) :=> x + y
  written:
  . "+"
Justified:
. [proof.label]
  label:
  . "Closure"
  by:
  . "Definition"
  comment: "standard"
Documented:
. [docs.written]
  written:
  . "plus"
Aliases:
. [alias.expr]
  alias: plus(x_, y_) :=> x + y
  written:
  . "+"
References:
. $book.plus
Metadata:
. id: "desc-1"

[\structure.connection]
Describes: T
Provides:
. [conn.plus]
  connection:
  . "addition"
  to:
  . "binary operation"
  using:
  . X is \set
  means:
  . "adds elements"
  signifies:
  . "closure"
  viewable:
  . "as a table"
  through:
  . "worked examples"
Justified:
. [proof.by]
  by:
  . "Convention"
  comment: "accepted"
Documented:
. [docs.called]
  called:
  . "addition"
Aliases:
. [alias.spec]
  alias: x_ "in" X :-> x is \element
Metadata:
. version: "1.0"

[\structure.writing]
Describes: W
Documented:
. [docs.writing]
  writing: plus(x_, y_) :~> x + y
  as:
  . "inline notation"

[\structure.overview]
Describes: O
Documented:
. [docs.overview]
  overview: "Binary operation on X"

[\structure.related]
Describes: R
Documented:
. [docs.related]
  related:
  . "group"
  . "ring"

[\structure.discoverer]
Describes: D
Documented:
. [docs.discoverer]
  discoverer:
  . "Gauss"

[\constant]
Defines: zero is \element
using:
. X is \set
expresses:
. [logic.expr]
  piecewise:
  . "choose a representative"
  if:
  . x = x
  then:
  . x = x
  else:
  . y = y

[\transform]
Refines: x is \(f)::[[g]]
using:
. X is \set
when:
. [logic.exists]
  existsUnique: x is \element
  suchThat:
  . x = x
specifies: y is \(f)::[[g]]
satisfies:
. [logic.given]
  given: x is \element
  where:
  . x = x
  then:
  . y = y

[\statement]
States:
. "Closure law"
. "Associativity"
using:
. X is \set
that:
. [logic.exists]
  exists: y is \element
  suchThat:
  . y = y

[\statement.expr]
States:
that:
. y = y
"#,
    );

    assert_eq!(document.items.len(), 10);

    match &document.items[0] {
        TopLevelItem::Describes(group) => {
            assert_eq!(
                group
                    .using
                    .as_ref()
                    .expect("expected using")
                    .arguments
                    .len(),
                2
            );
            assert!(matches!(
                group.when.as_ref().expect("expected when").arguments[0],
                Clause::AllOf(_)
            ));
            assert!(matches!(
                group.extends.as_ref().expect("expected extends").argument,
                IsOrViaItem::IsVia(_)
            ));
            assert_eq!(
                group
                    .specifies
                    .as_ref()
                    .expect("expected specifies")
                    .arguments
                    .len(),
                2
            );
            assert!(matches!(
                group
                    .satisfies
                    .as_ref()
                    .expect("expected satisfies")
                    .arguments[0],
                Clause::Not(_)
            ));
            assert!(matches!(
                group
                    .provides
                    .as_ref()
                    .expect("expected provides")
                    .arguments[0],
                ProvidesItem::Symbol(_)
            ));
            assert!(matches!(
                group
                    .justified
                    .as_ref()
                    .expect("expected justified")
                    .arguments[0],
                JustifiedItem::Label(_)
            ));
            assert!(matches!(
                group
                    .documented
                    .as_ref()
                    .expect("expected documented")
                    .arguments[0],
                DocumentedItem::Written(_)
            ));
            match &group.aliases.as_ref().expect("expected aliases").arguments[0] {
                AliasItem::Alias(alias) => {
                    assert!(matches!(alias.alias.argument, AliasKind::Expression(_)))
                }
            }
            assert!(matches!(
                group
                    .metadata
                    .as_ref()
                    .expect("expected metadata")
                    .arguments[0],
                MetadataItem::Id(_)
            ));
        }
        other => panic!("expected describes group, got {other:?}"),
    }

    match &document.items[1] {
        TopLevelItem::Describes(group) => {
            assert!(matches!(
                group
                    .provides
                    .as_ref()
                    .expect("expected provides")
                    .arguments[0],
                ProvidesItem::Connection(_)
            ));
            assert!(matches!(
                group
                    .justified
                    .as_ref()
                    .expect("expected justified")
                    .arguments[0],
                JustifiedItem::By(_)
            ));
            assert!(matches!(
                group
                    .documented
                    .as_ref()
                    .expect("expected documented")
                    .arguments[0],
                DocumentedItem::Called(_)
            ));
            match &group.aliases.as_ref().expect("expected aliases").arguments[0] {
                AliasItem::Alias(alias) => {
                    assert!(matches!(alias.alias.argument, AliasKind::SpecOperator(_)))
                }
            }
            assert!(matches!(
                group
                    .metadata
                    .as_ref()
                    .expect("expected metadata")
                    .arguments[0],
                MetadataItem::Version(_)
            ));
        }
        other => panic!("expected describes group, got {other:?}"),
    }

    match &document.items[2] {
        TopLevelItem::Describes(group) => {
            assert!(matches!(
                group
                    .documented
                    .as_ref()
                    .expect("expected documented")
                    .arguments[0],
                DocumentedItem::Writing(_)
            ));
        }
        other => panic!("expected describes group, got {other:?}"),
    }

    match &document.items[3] {
        TopLevelItem::Describes(group) => {
            assert!(matches!(
                group
                    .documented
                    .as_ref()
                    .expect("expected documented")
                    .arguments[0],
                DocumentedItem::Overview(_)
            ));
        }
        other => panic!("expected describes group, got {other:?}"),
    }

    match &document.items[4] {
        TopLevelItem::Describes(group) => {
            assert!(matches!(
                group
                    .documented
                    .as_ref()
                    .expect("expected documented")
                    .arguments[0],
                DocumentedItem::Related(_)
            ));
        }
        other => panic!("expected describes group, got {other:?}"),
    }

    match &document.items[5] {
        TopLevelItem::Describes(group) => {
            assert!(matches!(
                group
                    .documented
                    .as_ref()
                    .expect("expected documented")
                    .arguments[0],
                DocumentedItem::Discoverer(_)
            ));
        }
        other => panic!("expected describes group, got {other:?}"),
    }

    match &document.items[6] {
        TopLevelItem::Defines(group) => {
            assert!(matches!(
                group
                    .expresses
                    .as_ref()
                    .expect("expected expresses")
                    .argument,
                Clause::Piecewise(_)
            ));
        }
        other => panic!("expected defines group, got {other:?}"),
    }

    match &document.items[7] {
        TopLevelItem::Refines(group) => {
            assert!(matches!(
                group.refines.argument,
                crate::frontend::formulation::ast::IsOrRefinedStatementSpec::Is(_)
            ));
            assert!(group.specifies.is_some());
            assert!(matches!(
                group.when.as_ref().expect("expected when").arguments[0],
                Clause::ExistsUnique(_)
            ));
            assert!(matches!(
                group
                    .satisfies
                    .as_ref()
                    .expect("expected satisfies")
                    .arguments[0],
                Clause::Given(_)
            ));
        }
        other => panic!("expected refines group, got {other:?}"),
    }

    match &document.items[8] {
        TopLevelItem::States(group) => {
            assert_eq!(group.states.arguments.len(), 2);
            assert_eq!(group.states.arguments[0].0, "Closure law");
            assert_eq!(group.states.arguments[1].0, "Associativity");
            assert!(matches!(group.that.arguments[0], Clause::Exists(_)));
        }
        other => panic!("expected states group, got {other:?}"),
    }

    match &document.items[9] {
        TopLevelItem::States(group) => {
            assert!(matches!(group.that.arguments[0], Clause::Expression(_)));
        }
        other => panic!("expected states group, got {other:?}"),
    }
}

#[test]
fn parses_provided_symbol_with_builtin_spec_operator_target() {
    let document = parse_ok(
        r#"
[\set]
Describes: X
Provides:
. symbol: x_ "in" X :-> \\abstract
Documented:
. called: "set"
"#,
    );

    let TopLevelItem::Describes(group) = &document.items[0] else {
        panic!("expected describes group");
    };

    assert!(matches!(
        group
            .provides
            .as_ref()
            .expect("expected provides")
            .arguments[0],
        ProvidesItem::Symbol(_)
    ));
}

// ===============================[ diagnostics ]=====================================

#[test]
fn reports_section_order_errors_and_recovers() {
    let (document, diagnostics) = parse_with_diagnostics(
        r#"
[\statement]
States:
References:
. $bad.ref
that:
. x = x

Section: "Recovered"
"#,
    );

    assert_eq!(document.items.len(), 1);
    assert!(matches!(document.items[0], TopLevelItem::Section(_)));
    assert_eq!(diagnostics.len(), 1);
    assert!(
        diagnostics[0]
            .as_message()
            .expect("expected message event")
            .message
            .contains("Expected `that` but found `References`")
    );
}

#[test]
fn parses_structural_golden_directory() {
    let directory = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/goldens/structural"));
    let files = read_test_files(directory, "text");
    let expected_names = BTreeSet::from([
        "axioms.text".to_owned(),
        "conjectures.text".to_owned(),
        "corollaries.text".to_owned(),
        "defines.text".to_owned(),
        "describes.text".to_owned(),
        "lemmas.text".to_owned(),
        "outline.text".to_owned(),
        "persons.text".to_owned(),
        "refines.text".to_owned(),
        "resources.text".to_owned(),
        "specify.text".to_owned(),
        "states.text".to_owned(),
        "theorems.text".to_owned(),
    ]);

    assert!(!files.is_empty(), "expected structural golden files");

    let actual_names = files
        .iter()
        .map(|path| file_name(path))
        .collect::<BTreeSet<_>>();
    assert_eq!(
        actual_names, expected_names,
        "unexpected structural golden files"
    );

    for path in files {
        let name = file_name(&path);
        let entries = read_test_chunks(&path);

        assert!(!entries.is_empty(), "expected cases in {}", path.display());

        for (index, entry) in entries.iter().enumerate() {
            let mut tracker = EventLog::new();
            let parse_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                parse_document(entry, &mut tracker)
            }));

            if let Err(payload) = parse_result {
                let message = if let Some(message) = payload.downcast_ref::<&str>() {
                    *message
                } else if let Some(message) = payload.downcast_ref::<String>() {
                    message.as_str()
                } else {
                    "unknown panic"
                };
                panic!(
                    "structural golden case {} chunk {} panicked: {}\n\n{}",
                    name,
                    index + 1,
                    message,
                    entry
                );
            }

            assert!(
                !tracker.has_errors(),
                "failed to parse structural golden case {} chunk {}:\n{}\n\n{:#?}",
                name,
                index + 1,
                entry,
                tracker.events()
            );
        }
    }
}

// ===============================[ metadata ]=====================================

#[test]
fn parses_person_resource_and_specify_variants() {
    let document = parse_ok(
        r#"
[@euclid]
Person:
. "Ancient mathematician"
name:
. "Euclid"
. "Euclides"
biography: "Greek mathematician"

[$book.title]
Resource:
. title: "Elements"

[$book.author]
Resource:
. author:
  . "Euclid"
  . "Translator"

[$book.offset]
Resource:
. offset: "Book I"

[$book.url]
Resource:
. url: "https://example.com/elements"

[$book.homepage]
Resource:
. homepage: "https://example.com"

[$book.type]
Resource:
. type: "book"

[$book.edition]
Resource:
. edition: "second"

[$book.editor]
Resource:
. editor: "Editor Name"

[$book.institution]
Resource:
. institution: "Library"

[$book.journal]
Resource:
. journal: "Journal Name"

[$book.publisher]
Resource:
. publisher: "Publisher Name"

[$book.volume]
Resource:
. volume: "I"

[$book.month]
Resource:
. month: "January"

[$book.year]
Resource:
. year: "300BC"

[$book.description]
Resource:
. description: "Classic text"

Specify:
. [numbers.positive.int]
  positive:
  . "positive"
  int:
  . "integer"
  is:
  . "greater than zero"

Specify:
. [numbers.negative.int]
  negative:
  . "negative"
  int:
  . "integer"
  is:
  . "less than zero"

Specify:
. [numbers.zero]
  zero:
  . "zero"
  is:
  . "additive identity"

Specify:
. [numbers.positive.decimal]
  positive:
  . "positive"
  decimal:
  . "decimal"
  is:
  . "greater than zero"

Specify:
. [numbers.negative.decimal]
  negative:
  . "negative"
  decimal:
  . "decimal"
  is:
  . "less than zero"
"#,
    );

    assert_eq!(document.items.len(), 21);

    match &document.items[0] {
        TopLevelItem::Person(group) => {
            assert_eq!(group.person.arguments.len(), 1);
            assert_eq!(group.name.arguments.len(), 2);
            assert_eq!(group.biography.argument.0, "Greek mathematician");
        }
        other => panic!("expected person group, got {other:?}"),
    }

    match &document.items[1] {
        TopLevelItem::Resource(group) => {
            assert!(matches!(
                group.resource.arguments[0],
                ResourceItem::Title(_)
            ));
        }
        other => panic!("expected resource group, got {other:?}"),
    }

    match &document.items[2] {
        TopLevelItem::Resource(group) => {
            assert!(matches!(
                group.resource.arguments[0],
                ResourceItem::Author(_)
            ));
        }
        other => panic!("expected resource group, got {other:?}"),
    }

    match &document.items[3] {
        TopLevelItem::Resource(group) => {
            assert!(matches!(
                group.resource.arguments[0],
                ResourceItem::Offset(_)
            ));
        }
        other => panic!("expected resource group, got {other:?}"),
    }

    match &document.items[4] {
        TopLevelItem::Resource(group) => {
            assert!(matches!(group.resource.arguments[0], ResourceItem::Url(_)));
        }
        other => panic!("expected resource group, got {other:?}"),
    }

    match &document.items[5] {
        TopLevelItem::Resource(group) => {
            assert!(matches!(
                group.resource.arguments[0],
                ResourceItem::Homepage(_)
            ));
        }
        other => panic!("expected resource group, got {other:?}"),
    }

    match &document.items[6] {
        TopLevelItem::Resource(group) => {
            assert!(matches!(group.resource.arguments[0], ResourceItem::Type(_)));
        }
        other => panic!("expected resource group, got {other:?}"),
    }

    match &document.items[7] {
        TopLevelItem::Resource(group) => {
            assert!(matches!(
                group.resource.arguments[0],
                ResourceItem::Edition(_)
            ));
        }
        other => panic!("expected resource group, got {other:?}"),
    }

    match &document.items[8] {
        TopLevelItem::Resource(group) => {
            assert!(matches!(
                group.resource.arguments[0],
                ResourceItem::Editor(_)
            ));
        }
        other => panic!("expected resource group, got {other:?}"),
    }

    match &document.items[9] {
        TopLevelItem::Resource(group) => {
            assert!(matches!(
                group.resource.arguments[0],
                ResourceItem::Institution(_)
            ));
        }
        other => panic!("expected resource group, got {other:?}"),
    }

    match &document.items[10] {
        TopLevelItem::Resource(group) => {
            assert!(matches!(
                group.resource.arguments[0],
                ResourceItem::Journal(_)
            ));
        }
        other => panic!("expected resource group, got {other:?}"),
    }

    match &document.items[11] {
        TopLevelItem::Resource(group) => {
            assert!(matches!(
                group.resource.arguments[0],
                ResourceItem::Publisher(_)
            ));
        }
        other => panic!("expected resource group, got {other:?}"),
    }

    match &document.items[12] {
        TopLevelItem::Resource(group) => {
            assert!(matches!(
                group.resource.arguments[0],
                ResourceItem::Volume(_)
            ));
        }
        other => panic!("expected resource group, got {other:?}"),
    }

    match &document.items[13] {
        TopLevelItem::Resource(group) => {
            assert!(matches!(
                group.resource.arguments[0],
                ResourceItem::Month(_)
            ));
        }
        other => panic!("expected resource group, got {other:?}"),
    }

    match &document.items[14] {
        TopLevelItem::Resource(group) => {
            assert!(matches!(group.resource.arguments[0], ResourceItem::Year(_)));
        }
        other => panic!("expected resource group, got {other:?}"),
    }

    match &document.items[15] {
        TopLevelItem::Resource(group) => {
            assert!(matches!(
                group.resource.arguments[0],
                ResourceItem::Description(_)
            ));
        }
        other => panic!("expected resource group, got {other:?}"),
    }

    match &document.items[16] {
        TopLevelItem::Specify(group) => {
            assert!(matches!(
                group.specify.arguments[0],
                SpecifyItem::PositiveInt(_)
            ));
        }
        other => panic!("expected specify group, got {other:?}"),
    }

    match &document.items[17] {
        TopLevelItem::Specify(group) => {
            assert!(matches!(
                group.specify.arguments[0],
                SpecifyItem::NegativeInt(_)
            ));
        }
        other => panic!("expected specify group, got {other:?}"),
    }

    match &document.items[18] {
        TopLevelItem::Specify(group) => {
            assert!(matches!(group.specify.arguments[0], SpecifyItem::Zero(_)));
        }
        other => panic!("expected specify group, got {other:?}"),
    }

    match &document.items[19] {
        TopLevelItem::Specify(group) => {
            assert!(matches!(
                group.specify.arguments[0],
                SpecifyItem::PositiveDecimal(_)
            ));
        }
        other => panic!("expected specify group, got {other:?}"),
    }

    match &document.items[20] {
        TopLevelItem::Specify(group) => {
            assert!(matches!(
                group.specify.arguments[0],
                SpecifyItem::NegativeDecimal(_)
            ));
        }
        other => panic!("expected specify group, got {other:?}"),
    }
}

// ===============================[ overview ]=====================================

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

// ===============================[ theorems ]=====================================

#[test]
fn parses_theorem_like_groups_and_clause_variants() {
    let document = parse_ok(
        r#"
[\axiom]
Axiom:
. "Every element equals itself"
given:
. X is \set
where:
. [logic.not]
  not:
  . x = y
then:
. [logic.if]
  if:
  . x = x
  then:
  . y = y
iff:
. [logic.iff]
  iff:
  . x = x
  then:
  . y = y
Justified:
. [axiom.justified]
  by:
  . "Definition"
  comment: "classical"
Documented:
. [axiom.written]
  written:
  . "axiom"
Aliases:
. [axiom.alias]
  alias: axiom(x_) :=> x
References:
. $axiom.ref
Metadata:
. id: "ax-1"

Theorem:
then:
. [logic.any]
  anyOf:
  . x = x
  . y = y

[\corollary]
Corollary:
. "Immediate consequence"
of:
. "Previous theorem"
then:
. [logic.one]
  oneOf:
  . x = x
  . y = y

Lemma:
then:
. [logic.forall]
  forAll: x is \element
  where:
  . x = x
  then:
  . x = x

[\conjecture]
Conjecture:
. "Open question"
then:
. [logic.exists]
  exists: x is \element
  suchThat:
  . x = x
"#,
    );

    assert_eq!(document.items.len(), 5);

    match &document.items[0] {
        TopLevelItem::Axiom(group) => {
            assert!(group.heading.is_some());
            assert_eq!(group.axiom.arguments.len(), 1);
            assert!(group.given.is_some());
            assert!(matches!(
                group.where_.as_ref().expect("expected where").arguments[0],
                Clause::Not(_)
            ));
            assert!(matches!(group.then.arguments[0], Clause::If(_)));
            assert!(matches!(
                group.iff.as_ref().expect("expected iff").arguments[0],
                Clause::Iff(_)
            ));
            assert!(group.justified.is_some());
            assert!(group.documented.is_some());
            assert!(group.aliases.is_some());
            assert!(group.references.is_some());
            assert!(group.metadata.is_some());
        }
        other => panic!("expected axiom group, got {other:?}"),
    }

    match &document.items[1] {
        TopLevelItem::Theorem(group) => {
            assert!(group.heading.is_none());
            assert!(matches!(group.then.arguments[0], Clause::AnyOf(_)));
        }
        other => panic!("expected theorem group, got {other:?}"),
    }

    match &document.items[2] {
        TopLevelItem::Corollary(group) => {
            assert!(group.heading.is_some());
            assert_eq!(group.of.arguments[0].0, "Previous theorem");
            assert!(matches!(group.then.arguments[0], Clause::OneOf(_)));
        }
        other => panic!("expected corollary group, got {other:?}"),
    }

    match &document.items[3] {
        TopLevelItem::Lemma(group) => {
            assert!(group.heading.is_none());
            assert!(matches!(group.then.arguments[0], Clause::ForAll(_)));
        }
        other => panic!("expected lemma group, got {other:?}"),
    }

    match &document.items[4] {
        TopLevelItem::Conjecture(group) => {
            assert!(group.heading.is_some());
            assert!(matches!(group.then.arguments[0], Clause::Exists(_)));
        }
        other => panic!("expected conjecture group, got {other:?}"),
    }
}

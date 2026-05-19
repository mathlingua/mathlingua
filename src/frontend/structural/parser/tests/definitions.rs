use super::*;

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

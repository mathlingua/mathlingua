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


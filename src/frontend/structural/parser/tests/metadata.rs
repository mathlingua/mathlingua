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


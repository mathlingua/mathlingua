use super::*;

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

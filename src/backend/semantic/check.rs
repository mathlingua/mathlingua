use super::*;

pub fn check_documents(files: &[ParsedSourceFile], event_log: &mut EventLog) {
    let mut registry = SignatureRegistry::default();
    for file in files {
        collect_document_definitions(file, &mut registry, event_log);
    }

    for file in files {
        validate_document_references(file, &registry, event_log);
    }

    for file in files {
        validate_document_types(file, &registry, event_log);
    }
}

pub(super) fn collect_document_definitions(
    file: &ParsedSourceFile,
    registry: &mut SignatureRegistry,
    event_log: &mut EventLog,
) {
    let mut locator = SourceLocator::new(&file.source);
    for item in &file.document.items {
        let Some(definition) = definition_item(item) else {
            continue;
        };
        let kind = definition.kind;
        let full_shape = shape_for_header(definition.heading);
        let position = locator.locate_heading(&full_shape);
        check_documented_rendering(file, kind, definition.documented, position, event_log);
        for header_shape in shapes_for_header(definition.heading) {
            if let Some(previous) = registry.definitions.get(&header_shape.shape.signature) {
                emit_error(
                    event_log,
                    &file.path,
                    position,
                    format!(
                        "Duplicate command signature `{}` in {}; previously defined as {} in {}",
                        header_shape.shape.signature,
                        kind.label(),
                        previous.kind.label(),
                        display_definition_location(previous)
                    ),
                );
                continue;
            }

            let type_shape = header_shape.clone();
            registry.definitions.insert(
                header_shape.shape.signature.clone(),
                DefinitionEntry {
                    kind,
                    shape: header_shape.shape,
                    path: file.path.clone(),
                    position,
                },
            );
            collect_definition_type_metadata(item, &type_shape, registry);
        }
    }
}

pub(super) struct DefinitionItem<'a> {
    kind: DefinitionKind,
    heading: &'a CommandHeader,
    documented: Option<&'a DocumentedSection>,
}

pub(super) fn definition_item(item: &TopLevelItem) -> Option<DefinitionItem<'_>> {
    match item {
        TopLevelItem::Describes(group) => Some(DefinitionItem {
            kind: DefinitionKind::Describes,
            heading: &group.heading,
            documented: group.documented.as_ref(),
        }),
        TopLevelItem::Defines(group) => Some(DefinitionItem {
            kind: DefinitionKind::Defines,
            heading: &group.heading,
            documented: group.documented.as_ref(),
        }),
        TopLevelItem::Refines(group) => Some(DefinitionItem {
            kind: DefinitionKind::Refines,
            heading: &group.heading,
            documented: group.documented.as_ref(),
        }),
        TopLevelItem::States(group) => Some(DefinitionItem {
            kind: DefinitionKind::States,
            heading: &group.heading,
            documented: group.documented.as_ref(),
        }),
        TopLevelItem::Axiom(group) => group.heading.as_ref().map(|heading| DefinitionItem {
            kind: DefinitionKind::Axiom,
            heading,
            documented: group.documented.as_ref(),
        }),
        TopLevelItem::Theorem(group) => group.heading.as_ref().map(|heading| DefinitionItem {
            kind: DefinitionKind::Theorem,
            heading,
            documented: group.documented.as_ref(),
        }),
        TopLevelItem::Corollary(group) => group.heading.as_ref().map(|heading| DefinitionItem {
            kind: DefinitionKind::Corollary,
            heading,
            documented: group.documented.as_ref(),
        }),
        TopLevelItem::Lemma(group) => group.heading.as_ref().map(|heading| DefinitionItem {
            kind: DefinitionKind::Lemma,
            heading,
            documented: group.documented.as_ref(),
        }),
        TopLevelItem::Conjecture(group) => group.heading.as_ref().map(|heading| DefinitionItem {
            kind: DefinitionKind::Conjecture,
            heading,
            documented: group.documented.as_ref(),
        }),
        _ => None,
    }
}

pub(super) fn check_documented_rendering(
    file: &ParsedSourceFile,
    kind: DefinitionKind,
    documented: Option<&DocumentedSection>,
    position: Option<SourcePosition>,
    event_log: &mut EventLog,
) {
    if !matches!(
        kind,
        DefinitionKind::Describes | DefinitionKind::Defines | DefinitionKind::Refines
    ) {
        return;
    }

    let has_called = documented.is_some_and(|section| {
        section
            .arguments
            .iter()
            .any(|item| matches!(item, DocumentedItem::Called(_)))
    });

    if !has_called {
        emit_error(
            event_log,
            &file.path,
            position,
            format!(
                "{} entries must include a `called:` item in `Documented:`",
                kind.label()
            ),
        );
    }
}

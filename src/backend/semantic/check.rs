use super::*;

/// Runs semantic checks across a set of parsed MathLingua source files.
///
/// This is intentionally a two-pass check.  The first pass collects all command
/// definitions so cross-file references can be resolved independent of file
/// order.  The second pass walks every expression/specification and validates
/// command existence plus argument shape.
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

/// Collects every signature-defining top-level item from a single document.
///
/// During collection this also performs checks that are naturally tied to the
/// definition itself: duplicate signatures and required documented rendering
/// metadata for `Defines`, `Describes`, and `Refines`.
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
        let shape = shape_for_header(definition.heading);
        let position = locator.locate_heading(&shape);
        check_documented_rendering(file, kind, definition.documented, position, event_log);
        if let Some(previous) = registry.definitions.get(&shape.signature) {
            emit_error(
                event_log,
                &file.path,
                position,
                format!(
                    "Duplicate command signature `{}` in {}; previously defined as {} in {}",
                    shape.signature,
                    kind.label(),
                    previous.kind.label(),
                    display_definition_location(previous)
                ),
            );
            continue;
        }

        let type_shape = shape.clone();
        registry.definitions.insert(
            shape.signature.clone(),
            DefinitionEntry {
                kind,
                shape,
                path: file.path.clone(),
                position,
            },
        );
        collect_definition_type_metadata(item, &type_shape, registry);
    }
}

/// Borrowed view of the definition-relevant pieces of a top-level item.
///
/// Different structural group types store their heading and documented section
/// under different field names.  This adapter lets the collection pass treat
/// them uniformly.
pub(super) struct DefinitionItem<'a> {
    /// Kind of top-level group that owns this definition.
    kind: DefinitionKind,
    /// Parsed command header that defines the signature.
    heading: &'a CommandHeader,
    /// Optional documented section for rendering metadata checks.
    documented: Option<&'a DocumentedSection>,
}

/// Extracts definition metadata from top-level items that introduce signatures.
///
/// Anonymous theorem-like groups do not define a command signature, so they are
/// ignored here unless they have an explicit heading.
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

/// Verifies that renderable definition groups provide a documented `called:` item.
///
/// Only `Defines`, `Describes`, and `Refines` currently participate in command
/// rendering.  Theorem-like groups may have headings for reference purposes but
/// do not need `Documented:` rendering metadata.
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

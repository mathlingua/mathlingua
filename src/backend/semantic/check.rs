use super::*;

pub fn check_documents(files: &[ParsedSourceFile], event_log: &mut EventLog) {
    validate_top_level_item_ids(files, event_log);

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

fn validate_top_level_item_ids(files: &[ParsedSourceFile], event_log: &mut EventLog) {
    let mut seen: HashMap<String, (PathBuf, usize)> = HashMap::new();

    for file in files {
        for id in &file.item_ids {
            let row = id.id_row.unwrap_or(id.group_row);
            let Some(value) = id.value.as_ref() else {
                let message = if id.id_row.is_some() {
                    "`Id:` section must contain a quoted UUID"
                } else {
                    "Top-level item must include an `Id:` section"
                };
                event_log.user_error_at_file_row(Some(ORIGIN), file.path.clone(), row, message);
                continue;
            };

            if !is_uuid(value) {
                event_log.user_error_at_file_row(
                    Some(ORIGIN),
                    file.path.clone(),
                    row,
                    format!("`Id:` value `{value}` must be a UUID"),
                );
                continue;
            }

            if let Some((first_path, first_row)) = seen.get(value) {
                event_log.user_error_at_file_row(
                    Some(ORIGIN),
                    file.path.clone(),
                    row,
                    format!(
                        "Duplicate Id `{value}`; first used at {}:{}",
                        first_path.display(),
                        first_row + 1
                    ),
                );
            } else {
                seen.insert(value.clone(), (file.path.clone(), row));
            }
        }
    }
}

fn is_uuid(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() == 36
        && bytes[8] == b'-'
        && bytes[13] == b'-'
        && bytes[18] == b'-'
        && bytes[23] == b'-'
        && bytes
            .iter()
            .enumerate()
            .all(|(index, byte)| matches!(index, 8 | 13 | 18 | 23) || byte.is_ascii_hexdigit())
}

pub(super) fn collect_document_definitions(
    file: &ParsedSourceFile,
    registry: &mut SignatureRegistry,
    event_log: &mut EventLog,
) {
    let mut locator = SourceLocator::new(&file.source);
    for item in &file.document.items {
        if let Some(rule) = disambiguation_rule_from_item(item) {
            let position = None;
            if registry
                .disambiguations
                .iter()
                .any(|existing| existing.key == rule.key)
            {
                emit_error(
                    event_log,
                    &file.path,
                    position,
                    format!(
                        "Duplicate disambiguation for `{}`",
                        format_disambiguation_key(&rule.key)
                    ),
                );
            } else {
                registry.disambiguations.push(rule);
            }
            continue;
        }

        let Some(definition) = definition_item(item) else {
            continue;
        };
        let kind = definition.kind;
        let full_shape = shape_for_header(definition.heading);
        let position = locator.locate_heading(&full_shape);
        if matches!(definition.heading, CommandHeader::InfixSpec(_))
            && kind != DefinitionKind::Describes
        {
            emit_error(
                event_log,
                &file.path,
                position,
                "Spec-infix headings may only be used with Describes entries",
            );
            continue;
        }
        if matches!(definition.heading, CommandHeader::Refined(_))
            && kind != DefinitionKind::Refines
        {
            emit_error(
                event_log,
                &file.path,
                position,
                "Refined command headings may only be used with Refines entries",
            );
            continue;
        }
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

fn format_disambiguation_key(key: &DisambiguationKey) -> String {
    match key {
        DisambiguationKey::BinaryOperator(operator)
        | DisambiguationKey::PrefixOperator(operator)
        | DisambiguationKey::PostfixOperator(operator) => operator.clone(),
        DisambiguationKey::Function { name, arity } => format!("{name}/{arity}"),
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

    if kind == DefinitionKind::Refines {
        let has_adjective = documented.is_some_and(|section| {
            section
                .arguments
                .iter()
                .any(|item| matches!(item, DocumentedItem::Adjective(_)))
        });

        if !has_adjective {
            emit_error(
                event_log,
                &file.path,
                position,
                "Refines entries must include an `adjective:` item in `Documented:`",
            );
        }
        return;
    }

    let has_rendering = documented.is_some_and(|section| {
        section.arguments.iter().any(|item| match item {
            DocumentedItem::Written(_) => true,
            DocumentedItem::Called(_) => true,
            _ => false,
        })
    });

    if !has_rendering {
        emit_error(
            event_log,
            &file.path,
            position,
            format!(
                "{} entries must include either a `called:` or `written:` item in `Documented:`",
                kind.label()
            ),
        );
    }
}

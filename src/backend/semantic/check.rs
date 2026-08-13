use super::*;

pub fn check_documents(files: &[ParsedSourceFile], event_log: &mut EventLog) {
    validate_top_level_item_ids(files, event_log);
    validate_top_level_writing_count(files, event_log);
    validate_documented_mapping_writing(files, event_log);

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

/// Restricts documented `writing:` rules to mapping-shaped `Defines:` items and
/// requires their target to mirror that mapping exactly.
fn validate_documented_mapping_writing(files: &[ParsedSourceFile], event_log: &mut EventLog) {
    for file in files {
        for (index, item) in file.document.items.iter().enumerate() {
            let row = file
                .item_ids
                .get(index)
                .map(|id| id.group_row)
                .unwrap_or_default();
            let documented = documented_section(item);
            let Some(documented) = documented else {
                continue;
            };
            let writing = documented.arguments.iter().filter_map(|item| match item {
                DocumentedItem::Writing(group) => Some(group),
                _ => None,
            });

            match item {
                TopLevelItem::Defines(group) => {
                    let mapping = defines_mapping_parts(&group.defines.argument);
                    for writing in writing {
                        let Some(mapping) = &mapping else {
                            event_log.user_error_at_file_row(
                                Some(ORIGIN),
                                file.path.clone(),
                                row,
                                "Documented `writing:` is only allowed when `Defines:` targets a mapping",
                            );
                            continue;
                        };
                        if !mapping_writing_target_matches(&writing.writing.argument, mapping) {
                            event_log.user_error_at_file_row(
                                Some(ORIGIN),
                                file.path.clone(),
                                row,
                                format!(
                                    "Documented `writing:` must be exactly `{}` or `{}`",
                                    mapping_form_label(mapping, true),
                                    mapping_form_label(mapping, false),
                                ),
                            );
                        }
                    }
                }
                _ => {
                    for _ in writing {
                        event_log.user_error_at_file_row(
                            Some(ORIGIN),
                            file.path.clone(),
                            row,
                            "Documented `writing:` is only allowed inside a mapping-shaped `Defines:` item",
                        );
                    }
                }
            }
        }
    }
}

fn documented_section(item: &TopLevelItem) -> Option<&DocumentedSection> {
    match item {
        TopLevelItem::Disambiguates(group) => group.documented.as_ref(),
        TopLevelItem::Defines(group) => group.documented.as_ref(),
        TopLevelItem::Declares(group) => group.documented.as_ref(),
        TopLevelItem::Refines(group) => group.documented.as_ref(),
        TopLevelItem::States(group) => group.documented.as_ref(),
        TopLevelItem::Axiom(group) => group.documented.as_ref(),
        TopLevelItem::Theorem(group) => group.documented.as_ref(),
        TopLevelItem::Conjecture(group) => group.documented.as_ref(),
        TopLevelItem::Equivalent(group) => group.documented.as_ref(),
        TopLevelItem::TextItem(group) => group.documented.as_ref(),
        TopLevelItem::Relation(group) => group.documented.as_ref(),
        TopLevelItem::Topic(group) => group.documented.as_ref(),
        _ => None,
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct MappingFormParts {
    name: String,
    parameters: Vec<String>,
    magnetic: bool,
}

fn defines_mapping_parts(target: &DefinesTarget) -> Option<MappingFormParts> {
    match target {
        DefinesTarget::Form(form) => mapping_form_parts(form),
        DefinesTarget::Declaration(statement) => match &statement.subject.kind {
            IsSubjectKind::Forms(forms) => match forms.as_slice() {
                [IsSubjectForm::Form(form)] => mapping_form_parts(form),
                _ => None,
            },
            IsSubjectKind::Operator(_) => None,
        },
    }
}

fn mapping_form_parts(form: &FormOrDeclaration) -> Option<MappingFormParts> {
    let FormOrDeclarationKind::FunctionDeclaration { form, .. } = &form.kind else {
        return None;
    };
    Some(MappingFormParts {
        name: form.name.clone(),
        parameters: form
            .magnetic_placeholder
            .iter()
            .map(|placeholder| placeholder.name.clone())
            .chain(
                form.placeholders
                    .iter()
                    .map(|placeholder| placeholder.name.clone()),
            )
            .collect(),
        magnetic: form.magnetic_placeholder.is_some(),
    })
}

fn mapping_writing_target_matches(
    target: &MappingWritingTarget,
    expected: &MappingFormParts,
) -> bool {
    match target {
        MappingWritingTarget::Mapping(form) => {
            mapping_form_parts(form).is_some_and(|parts| parts == *expected)
        }
        MappingWritingTarget::Invocation(expression) => {
            let ExpressionKind::FunctionCall { name, arguments } = &expression.kind else {
                return false;
            };
            name == &expected.name
                && arguments.len() == expected.parameters.len()
                && arguments
                    .iter()
                    .zip(&expected.parameters)
                    .all(|(argument, parameter)| {
                        matches!(
                            &argument.kind,
                            ExpressionKind::Name(name)
                                if name == parameter
                                    && argument.span.end.saturating_sub(argument.span.start)
                                        == name.len()
                        )
                    })
        }
    }
}

fn mapping_form_label(mapping: &MappingFormParts, placeholders: bool) -> String {
    let arguments = mapping
        .parameters
        .iter()
        .map(|parameter| {
            if placeholders {
                let suffix = if mapping.magnetic { "__" } else { "_" };
                format!("{parameter}{suffix}")
            } else {
                parameter.clone()
            }
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!("{}({arguments})", mapping.name)
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

fn validate_top_level_writing_count(files: &[ParsedSourceFile], event_log: &mut EventLog) {
    let mut first: Option<(PathBuf, usize)> = None;

    for file in files {
        for (index, item) in file.document.items.iter().enumerate() {
            if !matches!(item, TopLevelItem::Writing(_)) {
                continue;
            }

            let row = file
                .item_ids
                .get(index)
                .map(|id| id.group_row)
                .unwrap_or_default();
            if let Some((first_path, first_row)) = &first {
                event_log.user_error_at_file_row(
                    Some(ORIGIN),
                    file.path.clone(),
                    row,
                    format!(
                        "Only one top-level `Writing:` item is allowed per collection; first used at {}:{}",
                        first_path.display(),
                        first_row + 1
                    ),
                );
            } else {
                first = Some((file.path.clone(), row));
            }
        }
    }
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
        if matches!(definition.heading, CommandHeader::InfixSpec(spec) if spec.refinement.is_none())
            && kind != DefinitionKind::Defines
        {
            emit_error(
                event_log,
                &file.path,
                position,
                "Spec-infix headings may only be used with Defines entries",
            );
            continue;
        }
        if (matches!(definition.heading, CommandHeader::Refined(_))
            || matches!(definition.heading, CommandHeader::InfixSpec(spec) if spec.refinement.is_some()))
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
        let placeholder_pattern = match placeholder_signature_for_header(definition.heading) {
            Ok(pattern) => pattern.map(|(_, pattern)| pattern),
            Err(message) => {
                emit_error(event_log, &file.path, position, message);
                continue;
            }
        };
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
            let registered_signature = header_shape.shape.signature.clone();
            registry.definitions.insert(
                registered_signature.clone(),
                DefinitionEntry {
                    kind,
                    shape: header_shape.shape,
                    path: file.path.clone(),
                    position,
                    placeholder_pattern: placeholder_pattern.clone(),
                },
            );
            if let Some(pattern) = &placeholder_pattern {
                registry
                    .placeholder_definitions
                    .entry(pattern.general_signature.clone())
                    .or_default()
                    .push(registered_signature);
            }
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

impl<'a> DefinitionItem<'a> {
    pub(super) fn heading(&self) -> &'a CommandHeader {
        self.heading
    }
}

pub(super) fn definition_item(item: &TopLevelItem) -> Option<DefinitionItem<'_>> {
    match item {
        TopLevelItem::Defines(group) => Some(DefinitionItem {
            kind: DefinitionKind::Defines,
            heading: &group.heading,
            documented: group.documented.as_ref(),
        }),
        TopLevelItem::Declares(group) => Some(DefinitionItem {
            kind: DefinitionKind::Declares,
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
        TopLevelItem::Conjecture(group) => group.heading.as_ref().map(|heading| DefinitionItem {
            kind: DefinitionKind::Conjecture,
            heading,
            documented: group.documented.as_ref(),
        }),
        TopLevelItem::Equivalent(group) => Some(DefinitionItem {
            kind: DefinitionKind::Equivalent,
            heading: &group.heading,
            documented: group.documented.as_ref(),
        }),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parsed_file(path: &str, source: &str) -> ParsedSourceFile {
        let mut event_log = EventLog::new();
        let document = parse_document(source, &mut event_log);
        assert!(!event_log.has_errors(), "{:#?}", event_log.events());

        ParsedSourceFile {
            path: PathBuf::from(path),
            source: source.to_string(),
            document,
            item_ids: top_level_item_ids(source),
            view_metadata: SourceFileViewMetadata::default(),
        }
    }

    #[test]
    fn reports_more_than_one_top_level_writing_group() {
        let files = vec![
            parsed_file(
                "a.mlg",
                r#"Writing:
. "alpha :~> \alpha"
Id: "11111111-1111-4111-8111-111111111111"
"#,
            ),
            parsed_file(
                "b.mlg",
                r#"Writing:
. "beta :~> \beta"
Id: "22222222-2222-4222-8222-222222222222"
"#,
            ),
        ];

        let mut event_log = EventLog::new();
        check_documents(&files, &mut event_log);

        assert!(
            event_log
                .events()
                .iter()
                .any(|event| event.as_message().is_some_and(|message| message
                    .message
                    .contains("Only one top-level `Writing:` item")))
        );
    }

    #[test]
    fn accepts_mapping_writing_for_exact_definition_and_invocation_forms() {
        let files = vec![parsed_file(
            "mapping.mlg",
            r#"[\real.sequence]
Defines: X ::= x(i_)
Documented:
. called: "real sequence"
. writing: x(i)
  as: "x?_{i?}"
. writing: x(i_)
  as: "\left\{x?\right\}_{i_?=1}^{\infty}"
Id: "11111111-1111-4111-8111-111111111111"
"#,
        )];
        let mut event_log = EventLog::new();

        validate_documented_mapping_writing(&files, &mut event_log);

        assert!(!event_log.has_errors(), "{:#?}", event_log.events());
    }

    #[test]
    fn rejects_mapping_writing_outside_matching_mapping_defines() {
        let files = vec![
            parsed_file(
                "not-mapping.mlg",
                r#"[\thing]
Defines: X
Documented:
. called: "thing"
. writing: x(i)
  as: "x?_{i?}"
Id: "11111111-1111-4111-8111-111111111111"
"#,
            ),
            parsed_file(
                "wrong-form.mlg",
                r#"[\real.sequence]
Defines: x(i_)
Documented:
. called: "real sequence"
. writing: x(j)
  as: "x?_{j?}"
Id: "22222222-2222-4222-8222-222222222222"
"#,
            ),
            parsed_file(
                "declares.mlg",
                r#"[\sequence]
Declares: X is \type
Documented:
. called: "sequence"
. writing: x(i)
  as: "x?_{i?}"
Id: "33333333-3333-4333-8333-333333333333"
"#,
            ),
            parsed_file(
                "wrong-placeholder-kind.mlg",
                r#"[\ordinary.mapping]
Defines: x(i_)
Documented:
. called: "ordinary mapping"
. writing: x(i__)
  as: "x?_{i_?}"
Id: "44444444-4444-4444-8444-444444444444"
"#,
            ),
            parsed_file(
                "partly-replaced-placeholders.mlg",
                r#"[\binary.mapping]
Defines: x(i_, j_)
Documented:
. called: "binary mapping"
. writing: x(i_, j)
  as: "x?_{i_?,j?}"
Id: "55555555-5555-4555-8555-555555555555"
"#,
            ),
        ];
        let mut event_log = EventLog::new();

        validate_documented_mapping_writing(&files, &mut event_log);

        let messages = event_log
            .events()
            .iter()
            .filter_map(|event| event.as_message())
            .map(|message| message.message.as_str())
            .collect::<Vec<_>>();
        assert_eq!(messages.len(), 5);
        assert!(messages[0].contains("only allowed when `Defines:` targets a mapping"));
        assert!(messages[1].contains("must be exactly `x(i_)` or `x(i)`"));
        assert!(messages[2].contains("only allowed inside a mapping-shaped `Defines:`"));
        assert!(messages[3].contains("must be exactly `x(i_)` or `x(i)`"));
        assert!(messages[4].contains("must be exactly `x(i_, j_)` or `x(i, j)`"));
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
        DefinitionKind::Defines
            | DefinitionKind::Declares
            | DefinitionKind::Refines
            | DefinitionKind::States
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

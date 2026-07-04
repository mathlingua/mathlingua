use super::*;
use std::collections::{BTreeSet, HashMap, HashSet};

pub(super) fn collect_definition_type_metadata(
    item: &TopLevelItem,
    header_shape: &HeaderShape,
    registry: &mut SignatureRegistry,
) {
    let Some(info) = definition_type_info(item, header_shape) else {
        return;
    };

    collect_type_extension_rules(item, &info, registry);
    collect_refinement_extension_rules(item, &info, registry);
    collect_spec_operator_rules(item, &info, registry);
    collect_provided_symbol_rules(item, &info, registry);
    collect_cast_as_rules(item, &info, registry);
    collect_viewable_rules(item, &info, registry);
    collect_collection_type_signature(item, &info, registry);
    registry
        .type_infos
        .insert(header_shape.shape.signature.clone(), info);
}

fn collect_collection_type_signature(
    item: &TopLevelItem,
    info: &DefinitionTypeInfo,
    registry: &mut SignatureRegistry,
) {
    let TopLevelItem::Describes(group) = item else {
        return;
    };
    if !describes_target_is_collection(&group.describes.argument) {
        return;
    }
    if !registry
        .collection_type_signatures
        .iter()
        .any(|signature| signature == &info.signature)
    {
        registry
            .collection_type_signatures
            .push(info.signature.clone());
    }
}

fn describes_target_is_collection(target: &DescribesTarget) -> bool {
    match target {
        DescribesTarget::Form(FormOrDeclaration {
            kind: FormOrDeclarationKind::SetDeclaration { .. },
            ..
        }) => true,
        DescribesTarget::Declaration(statement) => declaration_has_collection_literal(statement),
        _ => false,
    }
}

fn declaration_has_collection_literal(statement: &DeclarationStatement) -> bool {
    matches!(
        &statement.definition,
        Some(Expression {
            kind: ExpressionKind::Set(_),
            ..
        })
    ) || matches!(
        &statement.relation,
        Some(DeclarationRelation::Is(TypeExpression::Coercion { .. }))
    )
}

pub(super) fn disambiguation_rule_from_item(item: &TopLevelItem) -> Option<DisambiguationRule> {
    let TopLevelItem::Disambiguates(group) = item else {
        return None;
    };

    let (key, parameters) = disambiguation_key_and_parameters(&group.heading)?;
    let branches = group
        .branches
        .iter()
        .map(|branch| {
            let mut context = TypeContext::default();
            declare_form_or_declaration(&group.heading, &mut context);
            for clause in &branch.when.arguments {
                collect_clause_assumptions(clause, &mut context);
            }
            let requirements = context
                .facts
                .iter()
                .map(|fact| context.normalize_fact(fact))
                .collect();
            DisambiguationBranch {
                requirements,
                substitutions: context.substitutions,
                to: branch.to.argument.clone(),
            }
        })
        .collect();

    Some(DisambiguationRule {
        key,
        parameters,
        branches,
        else_expression: group.else_.as_ref().map(|section| section.argument.clone()),
    })
}

pub(super) fn validate_document_types(
    file: &ParsedSourceFile,
    registry: &SignatureRegistry,
    event_log: &mut EventLog,
) {
    let mut locator = SourceLocator::new(&file.source);
    for item in &file.document.items {
        validate_top_level_item_types(item, file.path.as_path(), &mut locator, registry, event_log);
    }
}

fn definition_type_info(
    item: &TopLevelItem,
    header_shape: &HeaderShape,
) -> Option<DefinitionTypeInfo> {
    match item {
        TopLevelItem::Describes(group) => Some(type_info_from_parts(
            header_shape,
            &group.heading,
            group.using.as_ref(),
            group.when.as_ref(),
            None,
            Some(&group.describes.argument),
            group.specifies.as_ref(),
        )),
        TopLevelItem::Defines(group) => Some(type_info_from_parts(
            header_shape,
            &group.heading,
            group.using.as_ref(),
            group.when.as_ref(),
            Some(&group.defines.argument),
            None,
            None,
        )),
        TopLevelItem::Refines(group) => Some(type_info_from_parts(
            header_shape,
            &group.heading,
            group.using.as_ref(),
            group.when.as_ref(),
            None,
            None,
            None,
        )),
        TopLevelItem::States(group) => Some(type_info_from_parts(
            header_shape,
            &group.heading,
            group.using.as_ref(),
            group.when.as_ref(),
            None,
            None,
            None,
        )),
        TopLevelItem::Axiom(group) => group.heading.as_ref().map(|heading| {
            type_info_from_parts(header_shape, heading, None, None, None, None, None)
        }),
        TopLevelItem::Theorem(group) => group.heading.as_ref().map(|heading| {
            type_info_from_parts(header_shape, heading, None, None, None, None, None)
        }),
        TopLevelItem::Corollary(group) => group.heading.as_ref().map(|heading| {
            type_info_from_parts(header_shape, heading, None, None, None, None, None)
        }),
        TopLevelItem::Lemma(group) => group.heading.as_ref().map(|heading| {
            type_info_from_parts(header_shape, heading, None, None, None, None, None)
        }),
        TopLevelItem::Conjecture(group) => group.heading.as_ref().map(|heading| {
            type_info_from_parts(header_shape, heading, None, None, None, None, None)
        }),
        _ => None,
    }
}

fn type_info_from_parts(
    header_shape: &HeaderShape,
    heading: &CommandHeader,
    using: Option<&UsingSection>,
    when: Option<&WhenSection>,
    defines: Option<&DeclarationStatement>,
    described: Option<&DescribesTarget>,
    describes_specifies: Option<&DescribesSpecifiesSection>,
) -> DefinitionTypeInfo {
    let mut context = TypeContext::default();
    declare_header_symbols(heading, &mut context);

    if let Some(using) = using {
        for statement in &using.arguments {
            declare_is_subject(&statement.subject, &mut context);
            if let Some(expansion) = &statement.expansion {
                declare_is_subject(expansion, &mut context);
            }
            for fact in facts_from_declaration_statement_in_context(statement, &context) {
                context.add_fact(fact);
            }
        }
    }

    if let Some(when) = when {
        for clause in &when.arguments {
            collect_clause_assumptions(clause, &mut context);
        }
    }

    let requirements = context
        .facts
        .iter()
        .map(|fact| context.normalize_fact(fact))
        .collect();
    let mut outputs: Vec<TypeFact> = defines
        .map(|statement| {
            facts_from_declaration_statement_in_context(statement, &context)
                .into_iter()
                .map(|fact| context.normalize_fact(&fact))
                .collect()
        })
        .unwrap_or_default();
    if let Some(fact) = described
        .zip(describes_specifies)
        .and_then(|(target, specifies)| {
            function_type_fact_from_describes_specifies(target, specifies, &context)
        })
    {
        outputs.push(context.normalize_fact(&fact));
    }

    DefinitionTypeInfo {
        signature: header_shape.shape.signature.clone(),
        parameters: header_shape.parameters.clone(),
        hidden_parameters: header_shape.hidden_parameters.clone(),
        requirements,
        outputs,
        substitutions: context.substitutions,
        described: described.map(described_target_subject_key),
    }
}

fn collect_spec_operator_rules(
    item: &TopLevelItem,
    info: &DefinitionTypeInfo,
    registry: &mut SignatureRegistry,
) {
    let Some(_described) = info.described.as_ref() else {
        return;
    };

    for capability in capability_aliases(item) {
        let AliasKind::SpecOperator(alias) = capability.alias else {
            continue;
        };
        let mut source_subject = capability.source_subject;
        let mut source_requires_literal = capability.source_requires_literal;
        if source_subject.is_none()
            && item_describes_collection(item)
            && let Some(described) = &info.described
        {
            source_subject = Some(described.clone());
            source_requires_literal = false;
        }
        if let Some(rule) =
            spec_operator_rule_from_alias(alias, info, source_subject, source_requires_literal)
        {
            registry.spec_rules.push(rule);
        }
    }
}

fn item_describes_collection(item: &TopLevelItem) -> bool {
    matches!(
        item,
        TopLevelItem::Describes(group) if describes_target_is_collection(&group.describes.argument)
    )
}

fn collect_provided_symbol_rules(
    item: &TopLevelItem,
    info: &DefinitionTypeInfo,
    registry: &mut SignatureRegistry,
) {
    let Some(described) = info.described.as_ref() else {
        return;
    };

    for capability in capability_aliases(item) {
        let AliasKind::Expression(alias) = capability.alias else {
            continue;
        };
        let Some((key, parameters)) = provided_symbol_key_and_parameters(&alias.lhs) else {
            continue;
        };
        registry.provided_symbols.push(ProvidedSymbolRule {
            owner_signature: info.signature.clone(),
            owner_subject: described.clone(),
            source_subject: capability.source_subject,
            key,
            parameters,
            target: alias.expression.clone(),
        });
    }
}

fn collect_cast_as_rules(
    item: &TopLevelItem,
    info: &DefinitionTypeInfo,
    registry: &mut SignatureRegistry,
) {
    let Some(described) = info.described.as_ref() else {
        return;
    };
    let Some(enables) = enables_section(item) else {
        return;
    };

    for item in &enables.arguments {
        let EnablesItem::FromAs(group) = item else {
            continue;
        };
        registry.cast_as_rules.push(CastAsRule {
            owner_signature: info.signature.clone(),
            owner_subject: described.clone(),
            source_subject: primary_subject_key(&group.from.argument.subject),
            left: group.as_.argument.left.clone(),
            right: group.as_.argument.right.clone(),
        });
    }
}

fn collect_viewable_rules(
    item: &TopLevelItem,
    info: &DefinitionTypeInfo,
    registry: &mut SignatureRegistry,
) {
    let Some(source_subject) = info.described.as_ref() else {
        return;
    };
    let Some(enables) = enables_section(item) else {
        return;
    };

    for item in &enables.arguments {
        let EnablesItem::Viewable(group) = item else {
            continue;
        };
        let Some(target @ TypeFact::Is { .. }) =
            facts_from_declaration_statement(&group.as_.argument)
                .into_iter()
                .next()
        else {
            continue;
        };
        registry.viewable_rules.push(ViewableRule {
            source_signature: info.signature.clone(),
            source_subject: source_subject.clone(),
            parameters: info.parameters.clone(),
            target_subject: fact_subject(&target).to_owned(),
            target,
        });
    }
}

fn collect_type_extension_rules(
    item: &TopLevelItem,
    info: &DefinitionTypeInfo,
    registry: &mut SignatureRegistry,
) {
    let Some(extends) = extends_item(item) else {
        return;
    };

    for fact in facts_from_is_or_via_item(extends) {
        let subject = match &fact {
            TypeFact::Is { subject, .. }
            | TypeFact::Spec { subject, .. }
            | TypeFact::InfixSpec { subject, .. }
            | TypeFact::RefinedIs { subject, .. }
            | TypeFact::MemberOf { subject, .. }
            | TypeFact::FunctionType { subject, .. } => subject.clone(),
        };
        registry.extension_rules.push(TypeExtensionRule {
            subtype_signature: info.signature.clone(),
            subject,
            parameters: info.parameters.clone(),
            target: fact,
        });
    }
}

fn collect_refinement_extension_rules(
    item: &TopLevelItem,
    info: &DefinitionTypeInfo,
    registry: &mut SignatureRegistry,
) {
    let TopLevelItem::Refines(group) = item else {
        return;
    };
    let Some(extends) = &group.extends else {
        return;
    };

    for target in refinement_extension_targets_from_declaration(&extends.argument) {
        registry
            .refinement_extension_rules
            .push(RefinementExtensionRule {
                subtype_signature: info.signature.clone(),
                subject: primary_subject_key(&extends.argument.subject),
                parameters: info.parameters.clone(),
                target,
            });
    }
}

fn refinement_extension_targets_from_declaration(
    statement: &DeclarationStatement,
) -> Vec<RefinementExtensionTarget> {
    match &statement.relation {
        Some(DeclarationRelation::Is(TypeExpression::RefinedCommand(command)))
            if matches!(command.refined_tail, RefinedTail::Name { .. }) =>
        {
            vec![RefinementExtensionTarget::DynamicRefinedIs {
                subject: primary_subject_key(&statement.subject),
                command: command.clone(),
            }]
        }
        _ => facts_from_declaration_statement(statement)
            .into_iter()
            .map(RefinementExtensionTarget::Fact)
            .collect(),
    }
}

fn extends_item(item: &TopLevelItem) -> Option<&IsOrViaItem> {
    match item {
        TopLevelItem::Describes(group) => group.extends.as_ref().map(|section| &section.argument),
        _ => None,
    }
}

fn enables_section(item: &TopLevelItem) -> Option<&EnablesSection> {
    match item {
        TopLevelItem::Describes(group) => group.enables.as_ref(),
        TopLevelItem::Defines(group) => group.enables.as_ref(),
        TopLevelItem::Refines(group) => group.enables.as_ref(),
        TopLevelItem::States(group) => group.enables.as_ref(),
        _ => None,
    }
}

fn requires_section(item: &TopLevelItem) -> Option<&RequiresSection> {
    match item {
        TopLevelItem::Describes(group) => group.requires.as_ref(),
        TopLevelItem::Defines(group) => group.requires.as_ref(),
        TopLevelItem::Refines(group) => group.requires.as_ref(),
        TopLevelItem::States(group) => group.requires.as_ref(),
        _ => None,
    }
}

struct CapabilityAliasRef<'a> {
    alias: &'a AliasKind,
    source_subject: Option<String>,
    source_requires_literal: bool,
}

fn capability_aliases(item: &TopLevelItem) -> Vec<CapabilityAliasRef<'_>> {
    let mut result = Vec::new();
    if let Some(requires) = requires_section(item) {
        result.extend(requires.arguments.iter().filter_map(|item| match item {
            RequiresItem::Capability(group) => Some(CapabilityAliasRef {
                alias: &group.capability.argument,
                source_subject: None,
                source_requires_literal: false,
            }),
            RequiresItem::Definition(_) => None,
        }));
    }
    if let Some(enables) = enables_section(item) {
        result.extend(enables.arguments.iter().filter_map(|item| match item {
            EnablesItem::Capability(group) => Some(CapabilityAliasRef {
                alias: &group.capability.argument,
                source_subject: None,
                source_requires_literal: false,
            }),
            EnablesItem::FromCapability(group) => Some(CapabilityAliasRef {
                alias: &group.capability.argument,
                source_subject: Some(primary_subject_key(&group.from.argument.subject)),
                source_requires_literal: true,
            }),
            EnablesItem::FromAs(_) | EnablesItem::Viewable(_) => None,
            EnablesItem::Connection(_) => None,
        }));
    }
    result
}

fn spec_operator_rule_from_alias(
    alias: &SpecOperatorAlias,
    info: &DefinitionTypeInfo,
    source_subject: Option<String>,
    source_requires_literal: bool,
) -> Option<SpecOperatorRule> {
    let placeholder = placeholder_pattern_name(&alias.placeholder_spec.placeholder_form)?;
    Some(SpecOperatorRule {
        owner_signature: info.signature.clone(),
        source_subject,
        source_requires_literal,
        placeholder,
        operator: alias.placeholder_spec.operator.clone(),
        target: alias.placeholder_spec.name.clone(),
        target_alias: alias.target.clone(),
    })
}

fn validate_top_level_item_types(
    item: &TopLevelItem,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    registry: &SignatureRegistry,
    event_log: &mut EventLog,
) {
    anchor_top_level_item(item, locator);

    match item {
        TopLevelItem::Disambiguates(group) => {
            validate_disambiguates(group, path, locator, registry, event_log);
        }
        TopLevelItem::Describes(group) => {
            let mut context = TypeContext::default();
            validate_spec_infix_describes_header(
                &group.heading,
                &group.describes.argument,
                path,
                locator,
                event_log,
            );
            declare_header_symbols(&group.heading, &mut context);
            declare_describes_target(&group.describes.argument, &mut context);
            assume_described_type(&group.heading, &group.describes.argument, &mut context);
            check_describes_target(
                &group.describes.argument,
                &context,
                path,
                locator,
                registry,
                event_log,
            );
            assume_optional_using(
                &group.using,
                &mut context,
                path,
                locator,
                registry,
                event_log,
            );
            let when_parameters = describes_when_parameters_from_usage(group);
            validate_when_section(&group.when, &when_parameters, path, locator, event_log);
            assume_optional_clauses(
                &group.when,
                &mut context,
                path,
                locator,
                registry,
                event_log,
            );
            assume_optional_specifies(
                &group.specifies,
                &mut context,
                path,
                locator,
                registry,
                event_log,
            );
            assume_describes_function_type(
                &group.describes.argument,
                &group.specifies,
                &mut context,
            );
            if let Some(extends) = &group.extends {
                check_is_or_via_item(
                    &extends.argument,
                    &context,
                    path,
                    locator,
                    registry,
                    event_log,
                );
            }
            validate_describes_target_symbol_specifications(group, path, locator, event_log);
            validate_optional_requires(
                &group.requires,
                &context,
                Some(&shapes_for_header(&group.heading)),
                Some(&described_target_subject_key(&group.describes.argument)),
                path,
                locator,
                registry,
                event_log,
            );
            validate_optional_enables(
                &group.enables,
                &context,
                &shapes_for_header(&group.heading),
                &described_target_subject_key(&group.describes.argument),
                path,
                locator,
                registry,
                event_log,
            );
            check_optional_clauses(
                &group.satisfies,
                &context,
                path,
                locator,
                registry,
                event_log,
            );
        }
        TopLevelItem::Defines(group) => {
            let mut context = TypeContext::default();
            declare_header_symbols(&group.heading, &mut context);
            declare_declaration_statement_subjects(&group.defines.argument, &mut context);
            assume_optional_using(
                &group.using,
                &mut context,
                path,
                locator,
                registry,
                event_log,
            );
            validate_when_section(
                &group.when,
                &header_when_parameters(&group.heading),
                path,
                locator,
                event_log,
            );
            assume_optional_clauses(
                &group.when,
                &mut context,
                path,
                locator,
                registry,
                event_log,
            );
            complete_introduced_declaration_statement(
                &group.defines.argument,
                &mut context,
                path,
                locator,
                registry,
                event_log,
            );
            validate_defines_target_symbol_specifications(group, path, locator, event_log);
            validate_optional_requires(
                &group.requires,
                &context,
                None,
                None,
                path,
                locator,
                registry,
                event_log,
            );
            if let Some(expresses) = &group.expresses {
                check_clause(
                    &expresses.argument,
                    &context,
                    path,
                    locator,
                    registry,
                    event_log,
                );
            }
        }
        TopLevelItem::Refines(group) => {
            let mut context = TypeContext::default();
            declare_header_symbols(&group.heading, &mut context);
            declare_declaration_statement_subjects(&group.refines.argument, &mut context);
            validate_refines_form_only(group, path, locator, event_log);
            assume_optional_using(
                &group.using,
                &mut context,
                path,
                locator,
                registry,
                event_log,
            );
            validate_when_section(
                &group.when,
                &header_when_parameters(&group.heading),
                path,
                locator,
                event_log,
            );
            assume_optional_clauses(
                &group.when,
                &mut context,
                path,
                locator,
                registry,
                event_log,
            );
            assume_refines_base_type(&group.heading, &group.refines.argument, &mut context);
            complete_introduced_declaration_statement(
                &group.refines.argument,
                &mut context,
                path,
                locator,
                registry,
                event_log,
            );
            if group.extends.is_some() {
                check_refines_extends(group, &context, path, locator, registry, event_log);
            }
            validate_refines_target_symbol_specifications(group, path, locator, event_log);
            validate_optional_requires(
                &group.requires,
                &context,
                None,
                None,
                path,
                locator,
                registry,
                event_log,
            );
            check_optional_clauses(
                &group.satisfies,
                &context,
                path,
                locator,
                registry,
                event_log,
            );
        }
        TopLevelItem::States(group) => {
            let mut context = TypeContext::default();
            declare_header_symbols(&group.heading, &mut context);
            assume_optional_using(
                &group.using,
                &mut context,
                path,
                locator,
                registry,
                event_log,
            );
            validate_when_section(
                &group.when,
                &header_when_parameters(&group.heading),
                path,
                locator,
                event_log,
            );
            assume_optional_clauses(
                &group.when,
                &mut context,
                path,
                locator,
                registry,
                event_log,
            );
            for clause in &group.that.arguments {
                check_clause(clause, &context, path, locator, registry, event_log);
            }
            validate_optional_requires(
                &group.requires,
                &context,
                None,
                None,
                path,
                locator,
                registry,
                event_log,
            );
        }
        TopLevelItem::Axiom(group) => validate_theorem_like(
            TheoremLikeSections::new(
                group.heading.as_ref(),
                group.given.as_ref(),
                group.where_.as_ref(),
                &group.then,
                group.iff.as_ref(),
            ),
            path,
            locator,
            registry,
            event_log,
        ),
        TopLevelItem::Theorem(group) => validate_theorem_like(
            TheoremLikeSections::new(
                group.heading.as_ref(),
                group.given.as_ref(),
                group.where_.as_ref(),
                &group.then,
                group.iff.as_ref(),
            ),
            path,
            locator,
            registry,
            event_log,
        ),
        TopLevelItem::Corollary(group) => validate_theorem_like(
            TheoremLikeSections::new(
                group.heading.as_ref(),
                group.given.as_ref(),
                group.where_.as_ref(),
                &group.then,
                group.iff.as_ref(),
            ),
            path,
            locator,
            registry,
            event_log,
        ),
        TopLevelItem::Lemma(group) => validate_theorem_like(
            TheoremLikeSections::new(
                group.heading.as_ref(),
                group.given.as_ref(),
                group.where_.as_ref(),
                &group.then,
                group.iff.as_ref(),
            ),
            path,
            locator,
            registry,
            event_log,
        ),
        TopLevelItem::Conjecture(group) => validate_theorem_like(
            TheoremLikeSections::new(
                group.heading.as_ref(),
                group.given.as_ref(),
                group.where_.as_ref(),
                &group.then,
                group.iff.as_ref(),
            ),
            path,
            locator,
            registry,
            event_log,
        ),
        TopLevelItem::Specify(_)
        | TopLevelItem::Title(_)
        | TopLevelItem::SectionTitle(_)
        | TopLevelItem::SubsectionTitle(_)
        | TopLevelItem::Text(_)
        | TopLevelItem::Writing(_)
        | TopLevelItem::Person(_)
        | TopLevelItem::Resource(_) => {}
    }
}

fn anchor_top_level_item(item: &TopLevelItem, locator: &mut SourceLocator<'_>) {
    let heading = match item {
        TopLevelItem::Describes(group) => Some(&group.heading),
        TopLevelItem::Defines(group) => Some(&group.heading),
        TopLevelItem::Refines(group) => Some(&group.heading),
        TopLevelItem::States(group) => Some(&group.heading),
        TopLevelItem::Axiom(group) => group.heading.as_ref(),
        TopLevelItem::Theorem(group) => group.heading.as_ref(),
        TopLevelItem::Corollary(group) => group.heading.as_ref(),
        TopLevelItem::Lemma(group) => group.heading.as_ref(),
        TopLevelItem::Conjecture(group) => group.heading.as_ref(),
        _ => None,
    };

    if let Some(heading) = heading {
        locator.anchor_item_heading(&shape_for_header(heading));
    }
}

fn check_refines_extends(
    group: &RefinesGroup,
    context: &TypeContext,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    registry: &SignatureRegistry,
    event_log: &mut EventLog,
) {
    let Some(extends) = &group.extends else {
        return;
    };
    let refines_subject = primary_subject_key(&group.refines.argument.subject);
    let extends_subject = primary_subject_key(&extends.argument.subject);
    if extends_subject != refines_subject {
        emit_error(
            event_log,
            path,
            locator.locate_heading(&shape_for_header(&group.heading)),
            "`Refines` extends subject must match the `Refines:` subject",
        );
    }

    let Some(DeclarationRelation::Is(TypeExpression::RefinedCommand(command))) =
        &extends.argument.relation
    else {
        check_declaration_statement(
            &extends.argument,
            context,
            path,
            locator,
            registry,
            event_log,
        );
        return;
    };

    if let RefinedTail::Name { name, .. } = &command.refined_tail {
        if name != &refines_subject {
            emit_error(
                event_log,
                path,
                locator.locate_heading(&shape_for_header(&group.heading)),
                "`[[...]]` in a `Refines` extends clause must name the `Refines:` subject",
            );
        }

        check_is_subject(&extends.argument.subject, context, path, locator, event_log);
        let active_command = active_refined_command_expression(command, context);
        for expression in refined_command_expression_arguments(&active_command) {
            check_expression(expression, context, path, locator, registry, event_log);
        }
        return;
    }

    check_declaration_statement(
        &extends.argument,
        context,
        path,
        locator,
        registry,
        event_log,
    );
}

fn validate_refines_form_only(
    group: &RefinesGroup,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    event_log: &mut EventLog,
) {
    if group.refines.argument.relation.is_none()
        && group.refines.argument.expansion.is_none()
        && group.refines.argument.definition.is_none()
    {
        return;
    }

    emit_error(
        event_log,
        path,
        locator.locate_heading(&shape_for_header(&group.heading)),
        "Refines entries must have the form `Refines: <form>`; the refined target is inferred from the heading",
    );
}

fn validate_disambiguates(
    group: &DisambiguatesGroup,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    registry: &SignatureRegistry,
    event_log: &mut EventLog,
) {
    for branch in &group.branches {
        let mut context = TypeContext::default();
        context.defer_unresolved_provided_symbols = true;
        declare_form_or_declaration(&group.heading, &mut context);
        for clause in &branch.when.arguments {
            assume_clause(clause, &mut context, path, locator, registry, event_log);
        }
        check_expression(
            &branch.to.argument,
            &context,
            path,
            locator,
            registry,
            event_log,
        );
    }

    if let Some(else_) = &group.else_ {
        let mut context = TypeContext::default();
        context.defer_unresolved_provided_symbols = true;
        declare_form_or_declaration(&group.heading, &mut context);
        check_expression(
            &else_.argument,
            &context,
            path,
            locator,
            registry,
            event_log,
        );
    }
}

fn assume_described_type(
    heading: &CommandHeader,
    described: &DescribesTarget,
    context: &mut TypeContext,
) {
    if matches!(heading, CommandHeader::InfixSpec(_)) {
        for header_shape in shapes_for_header(heading) {
            let Some((subject, target)) = header_shape
                .parameters
                .first()
                .cloned()
                .zip(header_shape.parameters.last().cloned())
            else {
                continue;
            };
            let args = if header_shape.parameters.len() > 2 {
                header_shape.parameters[1..header_shape.parameters.len() - 1].to_vec()
            } else {
                Vec::new()
            };
            context.add_fact(TypeFact::InfixSpec {
                subject,
                signature: header_shape.shape.signature,
                args,
                target,
            });
        }
        return;
    }

    let subject = described_target_subject_key(described);

    for header_shape in shapes_for_header(heading) {
        context.add_fact(TypeFact::Is {
            subject: subject.clone(),
            ty: header_shape.type_key,
            signature: header_shape.shape.signature,
        });
    }
}

fn assume_refines_base_type(
    heading: &CommandHeader,
    refined: &DeclarationStatement,
    context: &mut TypeContext,
) {
    let CommandHeader::Refined(_) = heading else {
        return;
    };
    let subject = primary_subject_key(&refined.subject);

    for header_shape in shapes_for_header(heading) {
        let Some((ty, signature)) = refined_header_base_type_fact_parts(&header_shape) else {
            continue;
        };
        context.add_fact(TypeFact::Is {
            subject: subject.clone(),
            ty,
            signature,
        });
    }
}

fn refined_header_base_type_fact_parts(header_shape: &HeaderShape) -> Option<(String, String)> {
    let signature_segments = split_refined_key(&header_shape.shape.signature)?;
    let type_key_segments = split_refined_key(&header_shape.type_key)?;
    let signature = format!("\\{}", signature_segments.last()?);
    let ty = format!("\\{}", type_key_segments.last()?);
    Some((ty, signature))
}

fn validate_spec_infix_describes_header(
    heading: &CommandHeader,
    described: &DescribesTarget,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    event_log: &mut EventLog,
) {
    let CommandHeader::InfixSpec(header) = heading else {
        return;
    };

    if key_for_form_or_declaration(&header.left) == described_target_subject_key(described) {
        return;
    }

    emit_error(
        event_log,
        path,
        locator.locate_heading(&shape_for_infix_spec_header(header)),
        "Spec-infix Describes heading left operand must match the Describes argument",
    );
}

struct TheoremLikeSections<'a> {
    heading: Option<&'a CommandHeader>,
    given: Option<&'a GivenSection>,
    where_: Option<&'a WhereSection>,
    then: &'a ThenSection,
    iff: Option<&'a IffSection>,
}

impl<'a> TheoremLikeSections<'a> {
    fn new(
        heading: Option<&'a CommandHeader>,
        given: Option<&'a GivenSection>,
        where_: Option<&'a WhereSection>,
        then: &'a ThenSection,
        iff: Option<&'a IffSection>,
    ) -> Self {
        Self {
            heading,
            given,
            where_,
            then,
            iff,
        }
    }
}

fn validate_theorem_like(
    sections: TheoremLikeSections<'_>,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    registry: &SignatureRegistry,
    event_log: &mut EventLog,
) {
    let mut context = TypeContext::default();
    if let Some(heading) = sections.heading {
        declare_header_symbols(heading, &mut context);
    }

    if let Some(given) = sections.given {
        for statement in &given.arguments {
            assume_declaration_statement(
                statement,
                &mut context,
                path,
                locator,
                registry,
                event_log,
            );
        }
    }

    if let Some(where_) = sections.where_ {
        for clause in &where_.arguments {
            assume_clause(clause, &mut context, path, locator, registry, event_log);
        }
    }

    for clause in &sections.then.arguments {
        check_clause(clause, &context, path, locator, registry, event_log);
    }

    if let Some(iff) = sections.iff {
        for clause in &iff.arguments {
            check_clause(clause, &context, path, locator, registry, event_log);
        }
    }
}

fn assume_optional_using(
    using: &Option<UsingSection>,
    context: &mut TypeContext,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    registry: &SignatureRegistry,
    event_log: &mut EventLog,
) {
    if let Some(using) = using {
        for statement in &using.arguments {
            assume_declaration_statement(statement, context, path, locator, registry, event_log);
        }
    }
}

fn assume_optional_specifies(
    specifies: &Option<DescribesSpecifiesSection>,
    context: &mut TypeContext,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    registry: &SignatureRegistry,
    event_log: &mut EventLog,
) {
    if let Some(specifies) = specifies {
        for item in &specifies.arguments {
            assume_is_or_via_item(item, context, path, locator, registry, event_log);
        }
    }
}

fn assume_is_or_via_item(
    item: &IsOrViaItem,
    context: &mut TypeContext,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    registry: &SignatureRegistry,
    event_log: &mut EventLog,
) {
    match item {
        IsOrViaItem::IsVia(statement) => {
            declare_is_subject(&statement.is_statement.subject, context);
            check_is_statement(
                &statement.is_statement,
                context,
                path,
                locator,
                registry,
                event_log,
            );
            check_form_or_declaration(&statement.via, context, path, locator, event_log);
            for fact in facts_from_is_statement(&statement.is_statement) {
                context.add_fact(fact);
            }
        }
        IsOrViaItem::Declaration(statement) => {
            assume_declaration_statement(statement, context, path, locator, registry, event_log);
        }
    }
}

fn declare_describes_target(target: &DescribesTarget, context: &mut TypeContext) {
    match target {
        DescribesTarget::Form(form) => declare_form_or_declaration(form, context),
        DescribesTarget::Declaration(statement) => {
            declare_declaration_statement_subjects(statement, context)
        }
    }
}

fn check_describes_target(
    target: &DescribesTarget,
    context: &TypeContext,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    registry: &SignatureRegistry,
    event_log: &mut EventLog,
) {
    match target {
        DescribesTarget::Form(form) => {
            check_form_or_declaration(form, context, path, locator, event_log);
        }
        DescribesTarget::Declaration(statement) => {
            check_declaration_statement(statement, context, path, locator, registry, event_log);
        }
    }
}

fn assume_describes_function_type(
    target: &DescribesTarget,
    specifies: &Option<DescribesSpecifiesSection>,
    context: &mut TypeContext,
) {
    let Some(specifies) = specifies else {
        return;
    };
    if let Some(fact) = function_type_fact_from_describes_specifies(target, specifies, context) {
        context.add_fact(fact);
    }
}

fn assume_optional_clauses<T>(
    section: &Option<T>,
    context: &mut TypeContext,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    registry: &SignatureRegistry,
    event_log: &mut EventLog,
) where
    T: ClauseSection,
{
    if let Some(section) = section {
        for clause in section.clauses() {
            assume_clause(clause, context, path, locator, registry, event_log);
        }
    }
}

fn validate_when_section<T>(
    section: &Option<T>,
    parameters: &WhenParameters,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    event_log: &mut EventLog,
) where
    T: ClauseSection,
{
    let mut covered_parameters = HashSet::new();
    if let Some(section) = section {
        for clause in section.clauses() {
            validate_when_clause(
                clause,
                parameters,
                &mut covered_parameters,
                path,
                locator,
                event_log,
            );
        }
    }

    let mut missing_parameters = parameters
        .required
        .iter()
        .filter(|parameter| !covered_parameters.contains(*parameter))
        .cloned()
        .collect::<Vec<_>>();
    missing_parameters.sort();
    for parameter in missing_parameters {
        emit_error(
            event_log,
            path,
            locator.locate_symbol(&parameter),
            format!("Missing `when:` requirement for parameter `{parameter}`"),
        );
    }
}

fn describes_when_parameters_from_usage(group: &DescribesGroup) -> WhenParameters {
    let mut parameters = header_when_parameters(&group.heading);
    let described_spec_infix_subject =
        described_spec_infix_subject(&group.heading, &group.describes.argument);
    if let Some(subject) = &described_spec_infix_subject {
        parameters.required.remove(subject);
    }
    for name in describes_used_names(group) {
        if described_spec_infix_subject.as_ref() == Some(&name) {
            continue;
        }
        if parameters.allowed.contains(&name) {
            parameters.require(name);
        }
    }
    parameters
}

fn described_spec_infix_subject(
    heading: &CommandHeader,
    described: &DescribesTarget,
) -> Option<String> {
    let CommandHeader::InfixSpec(header) = heading else {
        return None;
    };
    let subject = key_for_form_or_declaration(&header.left);
    if subject == described_target_subject_key(described) {
        Some(subject)
    } else {
        None
    }
}

fn describes_used_names(group: &DescribesGroup) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    if let Some(extends) = &group.extends {
        collect_is_or_via_names(&extends.argument, &mut names);
    }
    if let Some(specifies) = &group.specifies {
        for item in &specifies.arguments {
            collect_is_or_via_names(item, &mut names);
        }
    }
    if let Some(satisfies) = &group.satisfies {
        for clause in &satisfies.arguments {
            collect_clause_names(clause, &mut names);
        }
    }
    names
}

fn validate_describes_target_symbol_specifications(
    group: &DescribesGroup,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    event_log: &mut EventLog,
) {
    let mut covered = BTreeSet::new();
    covered.insert(described_target_subject_key(&group.describes.argument));
    collect_using_covered_symbols(&group.using, &mut covered);
    collect_valid_when_covered_symbols(
        &group.when,
        &header_when_parameters(&group.heading),
        &mut covered,
    );
    collect_specifies_covered_symbols(&group.specifies, &mut covered);
    collect_extends_covered_symbols(&group.extends, &mut covered);
    let symbols = describes_target_symbols(&group.describes.argument);
    validate_target_symbol_specifications(&symbols, &covered, path, locator, event_log);
}

fn validate_defines_target_symbol_specifications(
    group: &DefinesGroup,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    event_log: &mut EventLog,
) {
    let mut covered = BTreeSet::new();
    collect_using_covered_symbols(&group.using, &mut covered);
    collect_valid_when_covered_symbols(
        &group.when,
        &header_when_parameters(&group.heading),
        &mut covered,
    );
    collect_declaration_statement_covered_symbols(&group.defines.argument, &mut covered);
    validate_declaration_target_symbol_specifications(
        &group.defines.argument,
        &covered,
        path,
        locator,
        event_log,
    );
}

fn validate_refines_target_symbol_specifications(
    group: &RefinesGroup,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    event_log: &mut EventLog,
) {
    let mut covered = BTreeSet::new();
    covered.insert(primary_subject_key(&group.refines.argument.subject));
    collect_using_covered_symbols(&group.using, &mut covered);
    collect_valid_when_covered_symbols(
        &group.when,
        &header_when_parameters(&group.heading),
        &mut covered,
    );
    if let Some(extends) = &group.extends {
        collect_declaration_statement_covered_symbols(&extends.argument, &mut covered);
    }
    validate_declaration_target_symbol_specifications(
        &group.refines.argument,
        &covered,
        path,
        locator,
        event_log,
    );
}

fn validate_declaration_target_symbol_specifications(
    statement: &DeclarationStatement,
    covered: &BTreeSet<String>,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    event_log: &mut EventLog,
) {
    let symbols = declaration_target_symbols(statement);
    validate_target_symbol_specifications(&symbols, covered, path, locator, event_log);
}

fn validate_target_symbol_specifications(
    symbols: &BTreeSet<String>,
    covered: &BTreeSet<String>,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    event_log: &mut EventLog,
) {
    for symbol in symbols {
        if covered.contains(symbol) {
            continue;
        }
        emit_error(
            event_log,
            path,
            locator.locate_symbol(symbol),
            format!(
                "Missing specification for target symbol `{symbol}`; specify it directly or through `extends:`"
            ),
        );
    }
}

fn collect_using_covered_symbols(using: &Option<UsingSection>, covered: &mut BTreeSet<String>) {
    if let Some(using) = using {
        for statement in &using.arguments {
            collect_declaration_statement_covered_symbols(statement, covered);
        }
    }
}

fn collect_specifies_covered_symbols(
    specifies: &Option<DescribesSpecifiesSection>,
    covered: &mut BTreeSet<String>,
) {
    if let Some(specifies) = specifies {
        for item in &specifies.arguments {
            collect_is_or_via_covered_symbols(item, covered);
        }
    }
}

fn collect_extends_covered_symbols(
    extends: &Option<ExtendsSection>,
    covered: &mut BTreeSet<String>,
) {
    if let Some(extends) = extends {
        collect_is_or_via_covered_symbols(&extends.argument, covered);
    }
}

fn collect_valid_when_covered_symbols<T>(
    section: &Option<T>,
    parameters: &WhenParameters,
    covered: &mut BTreeSet<String>,
) where
    T: ClauseSection,
{
    if let Some(section) = section {
        for clause in section.clauses() {
            collect_valid_when_clause_covered_symbols(clause, parameters, covered);
        }
    }
}

fn collect_valid_when_clause_covered_symbols(
    clause: &Clause,
    parameters: &WhenParameters,
    covered: &mut BTreeSet<String>,
) {
    match clause {
        Clause::Declaration(statement)
            if statement.expansion.is_none() && statement.definition.is_none() =>
        {
            match &statement.relation {
                Some(DeclarationRelation::Is(_))
                | Some(DeclarationRelation::Spec { .. })
                | Some(DeclarationRelation::InfixSpec { .. }) => {
                    for subject in declaration_subject_keys(statement) {
                        if parameters.allowed.contains(&subject) {
                            covered.insert(subject);
                        }
                    }
                }
                None => {}
            }
        }
        Clause::Expression(expression) => {
            for subject in when_expression_subjects(expression) {
                if parameters.allowed.contains(&subject) {
                    covered.insert(subject);
                }
            }
        }
        _ => {}
    }
}

fn when_expression_subjects(expression: &Expression) -> Vec<String> {
    match &expression.kind {
        ExpressionKind::IsType { subject, .. } => vec![key_for_expression(subject)],
        ExpressionKind::SpecStatement(statement) => {
            vec![key_for_expression(&statement.subject)]
        }
        ExpressionKind::InfixSpecStatement { left, spec, .. } if !spec.predicate => {
            vec![key_for_expression(left)]
        }
        _ => Vec::new(),
    }
}

fn collect_is_or_via_covered_symbols(item: &IsOrViaItem, covered: &mut BTreeSet<String>) {
    match item {
        IsOrViaItem::IsVia(statement) => {
            collect_is_subject_covered_symbols(&statement.is_statement.subject, covered);
            collect_form_or_declaration_target_symbols(&statement.via, covered);
        }
        IsOrViaItem::Declaration(statement) => {
            collect_declaration_statement_covered_symbols(statement, covered);
        }
    }
}

fn collect_declaration_statement_covered_symbols(
    statement: &DeclarationStatement,
    covered: &mut BTreeSet<String>,
) {
    if statement.relation.is_none() {
        return;
    }
    collect_is_subject_covered_symbols(&statement.subject, covered);
}

fn collect_is_subject_covered_symbols(subject: &IsSubject, covered: &mut BTreeSet<String>) {
    match &subject.kind {
        IsSubjectKind::Forms(forms) => {
            for form in forms {
                if let IsSubjectForm::Form(form) = form {
                    collect_form_or_declaration_target_symbols(form, covered);
                }
            }
        }
        IsSubjectKind::Operator(operator) => {
            covered.insert(operator.text.clone());
        }
    }
}

fn declaration_target_symbols(statement: &DeclarationStatement) -> BTreeSet<String> {
    let mut symbols = BTreeSet::new();
    collect_is_subject_target_symbols(&statement.subject, &mut symbols);
    if let Some(expansion) = &statement.expansion {
        collect_is_subject_target_symbols(expansion, &mut symbols);
    }
    symbols
}

fn describes_target_symbols(target: &DescribesTarget) -> BTreeSet<String> {
    let mut symbols = BTreeSet::new();
    match target {
        DescribesTarget::Form(form) => {
            collect_form_or_declaration_target_symbols(form, &mut symbols);
        }
        DescribesTarget::Declaration(statement) => {
            symbols.extend(declaration_target_symbols(statement));
        }
    }
    symbols
}

fn collect_is_subject_target_symbols(subject: &IsSubject, symbols: &mut BTreeSet<String>) {
    match &subject.kind {
        IsSubjectKind::Forms(forms) => {
            for form in forms {
                if let IsSubjectForm::Form(form) = form {
                    collect_form_or_declaration_target_symbols(form, symbols);
                }
            }
        }
        IsSubjectKind::Operator(operator) => {
            symbols.insert(operator.text.clone());
        }
    }
}

fn collect_form_or_declaration_target_symbols(
    form: &FormOrDeclaration,
    symbols: &mut BTreeSet<String>,
) {
    match &form.kind {
        FormOrDeclarationKind::Name(name) => {
            symbols.insert(name.clone());
        }
        FormOrDeclarationKind::FunctionDeclaration { name, form } => {
            symbols.insert(name.as_ref().unwrap_or(&form.name).clone());
        }
        FormOrDeclarationKind::TupleDeclaration { name, form } => {
            if let Some(name) = name {
                symbols.insert(name.clone());
            }
            for element in &form.elements {
                match element {
                    TupleFormElement::Form(form) => {
                        collect_form_or_declaration_target_symbols(form, symbols);
                    }
                    TupleFormElement::Operator(operator) => {
                        symbols.insert(operator.text.clone());
                    }
                }
            }
        }
        FormOrDeclarationKind::SetDeclaration { name, .. } => {
            if let Some(name) = name {
                symbols.insert(name.clone());
            }
        }
        FormOrDeclarationKind::InfixOperator { operator, .. }
        | FormOrDeclarationKind::PrefixOperator { operator, .. }
        | FormOrDeclarationKind::PostfixOperator { operator, .. } => {
            symbols.insert(operator.text.clone());
        }
    }
}

fn collect_clause_names(clause: &Clause, names: &mut BTreeSet<String>) {
    match clause {
        Clause::Not(group) => collect_clause_names(&group.not.argument, names),
        Clause::AllOf(group) => {
            for clause in &group.all_of.arguments {
                collect_clause_names(clause, names);
            }
        }
        Clause::AnyOf(group) => {
            for clause in &group.any_of.arguments {
                collect_clause_names(clause, names);
            }
        }
        Clause::OneOf(group) => {
            for clause in &group.one_of.arguments {
                collect_clause_names(clause, names);
            }
        }
        Clause::Exists(group) => {
            for item in &group.exists.arguments {
                collect_binding_or_spec_names(item, names);
            }
            if let Some(such_that) = &group.such_that {
                for clause in &such_that.arguments {
                    collect_clause_names(clause, names);
                }
            }
        }
        Clause::ExistsUnique(group) => {
            for item in &group.exists_unique.arguments {
                collect_binding_or_spec_names(item, names);
            }
            if let Some(such_that) = &group.such_that {
                for clause in &such_that.arguments {
                    collect_clause_names(clause, names);
                }
            }
        }
        Clause::ForAll(group) => {
            for item in &group.for_all.arguments {
                collect_binding_or_spec_names(item, names);
            }
            if let Some(where_) = &group.where_ {
                for clause in &where_.arguments {
                    collect_clause_names(clause, names);
                }
            }
            for clause in &group.then.arguments {
                collect_clause_names(clause, names);
            }
        }
        Clause::If(group) => {
            for clause in &group.if_.arguments {
                collect_clause_names(clause, names);
            }
            for clause in &group.then.arguments {
                collect_clause_names(clause, names);
            }
        }
        Clause::Iff(group) => {
            for clause in &group.iff.arguments {
                collect_clause_names(clause, names);
            }
            for clause in &group.then.arguments {
                collect_clause_names(clause, names);
            }
        }
        Clause::Piecewise(group) => {
            for clause in &group.if_.arguments {
                collect_clause_names(clause, names);
            }
            for clause in &group.then.arguments {
                collect_clause_names(clause, names);
            }
            if let Some(else_) = &group.else_ {
                for clause in &else_.arguments {
                    collect_clause_names(clause, names);
                }
            }
        }
        Clause::Given(group) => {
            for statement in &group.given.arguments {
                collect_declaration_statement_names(statement, names);
            }
            if let Some(where_) = &group.where_ {
                for clause in &where_.arguments {
                    collect_clause_names(clause, names);
                }
            }
            for clause in &group.then.arguments {
                collect_clause_names(clause, names);
            }
        }
        Clause::Declaration(statement) => collect_declaration_statement_names(statement, names),
        Clause::Expression(expression) => collect_expression_names(expression, names),
    }
}

fn collect_binding_or_spec_names(item: &BindingOrSpec, names: &mut BTreeSet<String>) {
    match item {
        BindingOrSpec::Declaration(statement) => {
            collect_declaration_statement_names(statement, names)
        }
    }
}

fn collect_is_or_via_names(item: &IsOrViaItem, names: &mut BTreeSet<String>) {
    match item {
        IsOrViaItem::IsVia(statement) => {
            collect_is_subject_names(&statement.is_statement.subject, names);
            collect_type_expression_names(&statement.is_statement.ty, names);
            collect_form_or_declaration_names(&statement.via, names);
        }
        IsOrViaItem::Declaration(statement) => {
            collect_declaration_statement_names(statement, names)
        }
    }
}

fn collect_declaration_statement_names(
    statement: &DeclarationStatement,
    names: &mut BTreeSet<String>,
) {
    collect_is_subject_names(&statement.subject, names);
    if let Some(expansion) = &statement.expansion {
        collect_is_subject_names(expansion, names);
    }
    if let Some(definition) = &statement.definition {
        collect_expression_names(definition, names);
    }
    match &statement.relation {
        Some(DeclarationRelation::Is(ty)) => collect_type_expression_names(ty, names),
        Some(DeclarationRelation::Spec { target, .. }) => {
            collect_expression_names(target, names);
        }
        Some(DeclarationRelation::InfixSpec { spec, target }) => {
            collect_infix_spec_names(spec, names);
            collect_expression_names(target, names);
        }
        None => {}
    }
}

fn collect_is_subject_names(subject: &IsSubject, names: &mut BTreeSet<String>) {
    match &subject.kind {
        IsSubjectKind::Forms(forms) => {
            for form in forms {
                match form {
                    IsSubjectForm::Form(form) => collect_form_or_declaration_names(form, names),
                    IsSubjectForm::PlaceholderForm(form) => {
                        collect_placeholder_form_names(form, names)
                    }
                }
            }
        }
        IsSubjectKind::Operator(operator) => {
            names.insert(operator.text.clone());
        }
    }
}

fn collect_form_or_declaration_names(form: &FormOrDeclaration, names: &mut BTreeSet<String>) {
    match &form.kind {
        FormOrDeclarationKind::Name(name) => {
            names.insert(name.clone());
        }
        FormOrDeclarationKind::FunctionDeclaration { name, form } => {
            names.insert(name.as_ref().unwrap_or(&form.name).clone());
            if let Some(placeholder) = &form.magnetic_placeholder {
                names.insert(placeholder.name.clone());
            }
            for placeholder in &form.placeholders {
                names.insert(placeholder.name.clone());
            }
        }
        FormOrDeclarationKind::TupleDeclaration { name, form } => {
            if let Some(name) = name {
                names.insert(name.clone());
            }
            for element in &form.elements {
                match element {
                    TupleFormElement::Form(form) => collect_form_or_declaration_names(form, names),
                    TupleFormElement::Operator(operator) => {
                        names.insert(operator.text.clone());
                    }
                }
            }
        }
        FormOrDeclarationKind::SetDeclaration { name, form } => {
            if let Some(name) = name {
                names.insert(name.clone());
            }
            collect_placeholder_form_names(&form.placeholder_form, names);
        }
        FormOrDeclarationKind::InfixOperator {
            left,
            operator,
            right,
        } => {
            names.insert(left.name.clone());
            names.insert(operator.text.clone());
            names.insert(right.name.clone());
        }
        FormOrDeclarationKind::PrefixOperator {
            operator,
            placeholder,
        } => {
            names.insert(operator.text.clone());
            names.insert(placeholder.name.clone());
        }
        FormOrDeclarationKind::PostfixOperator {
            placeholder,
            operator,
        } => {
            names.insert(placeholder.name.clone());
            names.insert(operator.text.clone());
        }
    }
}

fn collect_placeholder_form_names(form: &PlaceholderForm, names: &mut BTreeSet<String>) {
    match &form.kind {
        PlaceholderFormKind::Placeholder(placeholder) => {
            names.insert(placeholder.name.clone());
        }
        PlaceholderFormKind::Function {
            placeholder,
            arguments,
        } => {
            names.insert(placeholder.name.clone());
            for argument in arguments {
                names.insert(argument.name.clone());
            }
        }
    }
}

fn collect_expression_names(expression: &Expression, names: &mut BTreeSet<String>) {
    match &expression.kind {
        ExpressionKind::Name(name) => {
            names.insert(name.clone());
        }
        ExpressionKind::FunctionCall { name, arguments } => {
            names.insert(name.clone());
            for argument in arguments {
                collect_expression_names(argument, names);
            }
        }
        ExpressionKind::FunctionNamedCall { name, elements } => {
            names.insert(name.clone());
            for element in elements {
                match &element.lhs {
                    FunctionNamedExpressionElementLhs::Name(name) => {
                        names.insert(name.clone());
                    }
                    FunctionNamedExpressionElementLhs::SubsetCall(subset) => {
                        collect_subset_call_names(subset, names);
                    }
                }
                collect_expression_names(&element.expression, names);
            }
        }
        ExpressionKind::MemberCall {
            owner,
            name,
            arguments,
        } => {
            collect_expression_names(owner, names);
            names.insert(name.clone());
            for argument in arguments {
                collect_expression_names(argument, names);
            }
        }
        ExpressionKind::MemberAccess { owner, name } => {
            collect_expression_names(owner, names);
            names.insert(name.clone());
        }
        ExpressionKind::Tuple(elements) => {
            for element in elements {
                if let TupleExpressionElement::Expression(expression) = element {
                    collect_expression_names(expression, names);
                }
            }
        }
        ExpressionKind::Set(set) => collect_set_expression_names(set, names),
        ExpressionKind::Grouped { expression, .. } | ExpressionKind::Labeled { expression, .. } => {
            collect_expression_names(expression, names);
        }
        ExpressionKind::SubsetCall(subset) => collect_subset_call_names(subset, names),
        ExpressionKind::Command(command) => collect_command_expression_names(command, names),
        ExpressionKind::InfixCommand {
            left,
            command,
            right,
        } => {
            collect_expression_names(left, names);
            collect_infix_command_names(command, names);
            collect_expression_names(right, names);
        }
        ExpressionKind::InfixSpecStatement { left, spec, right } => {
            collect_expression_names(left, names);
            collect_infix_spec_names(spec, names);
            collect_expression_names(right, names);
        }
        ExpressionKind::Prefix { expression, .. } | ExpressionKind::Postfix { expression, .. } => {
            collect_expression_names(expression, names);
        }
        ExpressionKind::Binary { left, right, .. } => {
            collect_expression_names(left, names);
            collect_expression_names(right, names);
        }
        ExpressionKind::SpecStatement(statement) | ExpressionKind::SpecPredicate(statement) => {
            collect_expression_names(&statement.subject, names);
            names.insert(statement.name.clone());
        }
        ExpressionKind::IsPredicate { subject, command }
        | ExpressionKind::IsNotPredicate { subject, command } => {
            collect_expression_names(subject, names);
            collect_command_expression_names(command, names);
        }
        ExpressionKind::IsBuiltinPredicate { subject, ty }
        | ExpressionKind::IsNotBuiltinPredicate { subject, ty }
        | ExpressionKind::IsType { subject, ty } => {
            collect_expression_names(subject, names);
            collect_type_expression_names(ty, names);
        }
        ExpressionKind::IsRefinedPredicate { subject, command }
        | ExpressionKind::IsNotRefinedPredicate { subject, command } => {
            collect_expression_names(subject, names);
            collect_refined_command_expression_names(command, names);
        }
        ExpressionKind::MemberOf {
            subject,
            collection,
        } => {
            collect_expression_names(subject, names);
            collect_expression_names(collection, names);
        }
    }
}

fn collect_type_expression_names(ty: &TypeExpression, names: &mut BTreeSet<String>) {
    match ty {
        TypeExpression::Builtin { .. } => {}
        TypeExpression::Command(command) => collect_command_expression_names(command, names),
        TypeExpression::RefinedCommand(command) => {
            collect_refined_command_expression_names(command, names);
        }
        TypeExpression::Function(function_type) => {
            for spec in function_type
                .inputs
                .iter()
                .chain(std::iter::once(&function_type.output))
            {
                names.insert(spec.subject.clone());
                match &spec.kind {
                    FunctionTypeSpecKind::Is(ty) => collect_type_expression_names(ty, names),
                    FunctionTypeSpecKind::Spec { target, .. } => {
                        names.insert(target.clone());
                    }
                }
            }
        }
        TypeExpression::Coercion { ty, literal, .. } => {
            collect_type_expression_names(ty, names);
            collect_set_expression_names(literal, names);
        }
    }
}

fn collect_set_expression_names(set: &SetExpression, names: &mut BTreeSet<String>) {
    collect_set_target_names(&set.target, names);
    for spec in &set.specs {
        collect_expression_names(spec, names);
    }
    if let Some(predicate) = &set.predicate {
        collect_expression_names(predicate, names);
    }
}

fn collect_set_target_names(target: &SetTarget, names: &mut BTreeSet<String>) {
    match &target.kind {
        SetTargetKind::Name(name) => {
            names.insert(name.clone());
        }
        SetTargetKind::PlaceholderForm(form) => collect_placeholder_form_names(form, names),
        SetTargetKind::Alias { name, target } => {
            names.insert(name.clone());
            collect_set_target_names(target, names);
        }
        SetTargetKind::Function { name, arguments } => {
            names.insert(name.clone());
            for argument in arguments {
                collect_set_target_names(argument, names);
            }
        }
        SetTargetKind::Tuple(elements) => {
            for element in elements {
                if let SetTargetElement::Target(target) = element {
                    collect_set_target_names(target, names);
                }
            }
        }
    }
}

fn collect_command_expression_names(command: &CommandExpression, names: &mut BTreeSet<String>) {
    for expression in command_expression_arguments(command) {
        collect_expression_names(expression, names);
    }
}

fn collect_infix_command_names(command: &InfixCommand, names: &mut BTreeSet<String>) {
    for expression in infix_command_arguments(command) {
        collect_expression_names(expression, names);
    }
}

fn collect_infix_spec_names(spec: &InfixSpec, names: &mut BTreeSet<String>) {
    for expression in infix_spec_arguments(spec) {
        collect_expression_names(expression, names);
    }
}

fn collect_refined_command_expression_names(
    command: &RefinedCommandExpression,
    names: &mut BTreeSet<String>,
) {
    for expression in refined_command_expression_arguments(command) {
        collect_expression_names(expression, names);
    }
}

fn collect_subset_call_names(subset: &SubsetCall, names: &mut BTreeSet<String>) {
    match subset {
        SubsetCall::One { target, first, .. } => {
            names.insert(target.clone());
            names.insert(first.clone());
        }
        SubsetCall::Two {
            target,
            first,
            second,
            ..
        } => {
            names.insert(target.clone());
            names.insert(first.clone());
            names.insert(second.clone());
        }
        SubsetCall::Nested {
            target,
            outer,
            inner_target,
            ..
        } => {
            names.insert(target.clone());
            names.insert(outer.clone());
            names.insert(inner_target.clone());
        }
    }
}

fn validate_when_clause(
    clause: &Clause,
    parameters: &WhenParameters,
    covered_parameters: &mut HashSet<String>,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    event_log: &mut EventLog,
) {
    match clause {
        Clause::Declaration(statement) => validate_when_declaration(
            statement,
            parameters,
            covered_parameters,
            path,
            locator,
            event_log,
        ),
        Clause::Expression(expression) => validate_when_expression(
            expression,
            parameters,
            covered_parameters,
            path,
            locator,
            event_log,
        ),
        _ => emit_invalid_when_clause_error(path, locator, event_log),
    }
}

fn validate_when_declaration(
    statement: &DeclarationStatement,
    parameters: &WhenParameters,
    covered_parameters: &mut HashSet<String>,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    event_log: &mut EventLog,
) {
    if statement.expansion.is_some() || statement.definition.is_some() {
        emit_invalid_when_clause_error(path, locator, event_log);
        return;
    }

    match &statement.relation {
        Some(DeclarationRelation::Is(_))
        | Some(DeclarationRelation::Spec { .. })
        | Some(DeclarationRelation::InfixSpec { .. }) => {
            for subject in declaration_subject_keys(statement) {
                validate_when_subject(
                    &subject,
                    parameters,
                    covered_parameters,
                    path,
                    locator,
                    event_log,
                );
            }
        }
        None => emit_invalid_when_clause_error(path, locator, event_log),
    }
}

fn validate_when_expression(
    expression: &Expression,
    parameters: &WhenParameters,
    covered_parameters: &mut HashSet<String>,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    event_log: &mut EventLog,
) {
    match &expression.kind {
        ExpressionKind::IsType { subject, .. } => validate_when_subject(
            &key_for_expression(subject),
            parameters,
            covered_parameters,
            path,
            locator,
            event_log,
        ),
        ExpressionKind::SpecStatement(statement) => validate_when_subject(
            &key_for_expression(&statement.subject),
            parameters,
            covered_parameters,
            path,
            locator,
            event_log,
        ),
        ExpressionKind::InfixSpecStatement { left, spec, .. } if !spec.predicate => {
            validate_when_subject(
                &key_for_expression(left),
                parameters,
                covered_parameters,
                path,
                locator,
                event_log,
            );
        }
        _ => emit_invalid_when_clause_error(path, locator, event_log),
    }
}

fn validate_when_subject(
    subject: &str,
    parameters: &WhenParameters,
    covered_parameters: &mut HashSet<String>,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    event_log: &mut EventLog,
) {
    if parameters.allowed.contains(subject) {
        covered_parameters.insert(subject.to_string());
        return;
    }

    emit_error(
        event_log,
        path,
        locator.locate_symbol(subject),
        format!(
            "`when:` requirement for `{subject}` is not allowed because `{subject}` is not a parameter of this definition"
        ),
    );
}

fn emit_invalid_when_clause_error(
    path: &Path,
    locator: &mut SourceLocator<'_>,
    event_log: &mut EventLog,
) {
    emit_error(
        event_log,
        path,
        locator.locate_symbol("when"),
        "`when:` clauses only support `<subject> is <type>` or `<subject> \"op\" <target>` requirements",
    );
}

fn check_optional_clauses<T>(
    section: &Option<T>,
    context: &TypeContext,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    registry: &SignatureRegistry,
    event_log: &mut EventLog,
) where
    T: ClauseSection,
{
    if let Some(section) = section {
        for clause in section.clauses() {
            check_clause(clause, context, path, locator, registry, event_log);
        }
    }
}

fn assume_clause(
    clause: &Clause,
    context: &mut TypeContext,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    registry: &SignatureRegistry,
    event_log: &mut EventLog,
) {
    match clause {
        Clause::Declaration(statement) => {
            assume_declaration_statement(statement, context, path, locator, registry, event_log);
        }
        Clause::Expression(expression)
            if fact_from_expression_in_context(expression, context).is_some() =>
        {
            assume_fact_expression(expression, context, path, locator, registry, event_log);
            if let Some(fact) = fact_from_expression_in_context(expression, context) {
                context.add_fact(fact);
            }
        }
        Clause::AllOf(group) => {
            for clause in &group.all_of.arguments {
                assume_clause(clause, context, path, locator, registry, event_log);
            }
        }
        _ => {
            check_clause(clause, context, path, locator, registry, event_log);
            collect_clause_assumptions(clause, context);
        }
    }
}

fn check_clause(
    clause: &Clause,
    context: &TypeContext,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    registry: &SignatureRegistry,
    event_log: &mut EventLog,
) {
    match clause {
        Clause::Not(group) => check_clause(
            &group.not.argument,
            context,
            path,
            locator,
            registry,
            event_log,
        ),
        Clause::AllOf(group) => {
            for clause in &group.all_of.arguments {
                check_clause(clause, context, path, locator, registry, event_log);
            }
        }
        Clause::AnyOf(group) => {
            for clause in &group.any_of.arguments {
                check_clause(clause, context, path, locator, registry, event_log);
            }
        }
        Clause::OneOf(group) => {
            for clause in &group.one_of.arguments {
                check_clause(clause, context, path, locator, registry, event_log);
            }
        }
        Clause::Exists(group) => {
            let mut child = context.clone();
            for item in &group.exists.arguments {
                assume_binding_or_spec(item, &mut child, path, locator, registry, event_log);
            }
            if let Some(section) = &group.such_that {
                for clause in &section.arguments {
                    assume_clause(clause, &mut child, path, locator, registry, event_log);
                }
            }
        }
        Clause::ExistsUnique(group) => {
            let mut child = context.clone();
            for item in &group.exists_unique.arguments {
                assume_binding_or_spec(item, &mut child, path, locator, registry, event_log);
            }
            if let Some(section) = &group.such_that {
                for clause in &section.arguments {
                    assume_clause(clause, &mut child, path, locator, registry, event_log);
                }
            }
        }
        Clause::ForAll(group) => {
            let mut child = context.clone();
            for item in &group.for_all.arguments {
                assume_binding_or_spec(item, &mut child, path, locator, registry, event_log);
            }
            if let Some(where_) = &group.where_ {
                for clause in &where_.arguments {
                    assume_clause(clause, &mut child, path, locator, registry, event_log);
                }
            }
            for clause in &group.then.arguments {
                check_clause(clause, &child, path, locator, registry, event_log);
            }
        }
        Clause::If(group) => {
            let mut child = context.clone();
            for clause in &group.if_.arguments {
                assume_clause(clause, &mut child, path, locator, registry, event_log);
            }
            for clause in &group.then.arguments {
                check_clause(clause, &child, path, locator, registry, event_log);
            }
        }
        Clause::Iff(group) => {
            let mut child = context.clone();
            for clause in &group.iff.arguments {
                assume_clause(clause, &mut child, path, locator, registry, event_log);
            }
            for clause in &group.then.arguments {
                check_clause(clause, &child, path, locator, registry, event_log);
            }
        }
        Clause::Piecewise(group) => {
            let mut child = context.clone();
            for clause in &group.if_.arguments {
                assume_clause(clause, &mut child, path, locator, registry, event_log);
            }
            for clause in &group.then.arguments {
                check_clause(clause, &child, path, locator, registry, event_log);
            }
            if let Some(else_) = &group.else_ {
                for clause in &else_.arguments {
                    check_clause(clause, context, path, locator, registry, event_log);
                }
            }
        }
        Clause::Given(group) => {
            let mut child = context.clone();
            for statement in &group.given.arguments {
                assume_declaration_statement(
                    statement, &mut child, path, locator, registry, event_log,
                );
            }
            if let Some(where_) = &group.where_ {
                for clause in &where_.arguments {
                    assume_clause(clause, &mut child, path, locator, registry, event_log);
                }
            }
            for clause in &group.then.arguments {
                check_clause(clause, &child, path, locator, registry, event_log);
            }
        }
        Clause::Declaration(statement) => {
            check_declaration_statement(statement, context, path, locator, registry, event_log);
        }
        Clause::Expression(expression) => {
            check_expression(expression, context, path, locator, registry, event_log)
        }
    }
}

fn assume_binding_or_spec(
    item: &BindingOrSpec,
    context: &mut TypeContext,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    registry: &SignatureRegistry,
    event_log: &mut EventLog,
) {
    match item {
        BindingOrSpec::Declaration(statement) => {
            assume_declaration_statement(statement, context, path, locator, registry, event_log);
        }
    }
}

fn collect_clause_assumptions(clause: &Clause, context: &mut TypeContext) {
    match clause {
        Clause::Declaration(statement) => {
            declare_is_subject(&statement.subject, context);
            if let Some(expansion) = &statement.expansion {
                declare_is_subject(expansion, context);
            }
            for fact in facts_from_declaration_statement_in_context(statement, context) {
                context.add_fact(fact);
            }
            if let Some((left, right)) = declaration_substitution(statement) {
                context.add_substitution(left, right);
            }
        }
        Clause::Expression(expression) => {
            if let Some(fact) = fact_from_expression_in_context(expression, context) {
                context.add_fact(fact);
            }
        }
        Clause::AllOf(group) => {
            for clause in &group.all_of.arguments {
                collect_clause_assumptions(clause, context);
            }
        }
        _ => {}
    }
}

fn check_declaration_statement(
    statement: &DeclarationStatement,
    context: &TypeContext,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    registry: &SignatureRegistry,
    event_log: &mut EventLog,
) {
    check_is_subject(&statement.subject, context, path, locator, event_log);
    if let Some(expansion) = &statement.expansion {
        check_is_subject(expansion, context, path, locator, event_log);
    }
    if let Some(definition) = &statement.definition {
        check_expression(definition, context, path, locator, registry, event_log);
    }
    if let Some(relation) = &statement.relation {
        check_declaration_relation(relation, context, path, locator, registry, event_log);
    }
    check_declaration_spec_facts_supported(statement, context, path, locator, registry, event_log);
}

fn assume_declaration_statement(
    statement: &DeclarationStatement,
    context: &mut TypeContext,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    registry: &SignatureRegistry,
    event_log: &mut EventLog,
) {
    declare_is_subject(&statement.subject, context);
    if let Some(expansion) = &statement.expansion {
        declare_is_subject(expansion, context);
    }
    if let Some(relation) = &statement.relation {
        check_declaration_relation(relation, context, path, locator, registry, event_log);
    }
    check_declaration_spec_facts_supported(statement, context, path, locator, registry, event_log);
    if let Some(definition) = &statement.definition {
        check_expression(definition, context, path, locator, registry, event_log);
    }
    register_declaration_collection_literal(statement, context);
    if let Some((left, right)) = declaration_substitution(statement) {
        context.add_substitution(left, right);
    }
    for fact in facts_from_declaration_statement_in_context(statement, context) {
        context.add_fact(fact);
    }
}

fn declare_declaration_statement_subjects(
    statement: &DeclarationStatement,
    context: &mut TypeContext,
) {
    declare_is_subject(&statement.subject, context);
    if let Some(expansion) = &statement.expansion {
        declare_is_subject(expansion, context);
    }
}

fn complete_introduced_declaration_statement(
    statement: &DeclarationStatement,
    context: &mut TypeContext,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    registry: &SignatureRegistry,
    event_log: &mut EventLog,
) {
    if let Some(relation) = &statement.relation {
        check_declaration_relation(relation, context, path, locator, registry, event_log);
    }
    if let Some(definition) = &statement.definition {
        check_expression(definition, context, path, locator, registry, event_log);
    }
    register_declaration_collection_literal(statement, context);
    if let Some((left, right)) = declaration_substitution(statement) {
        context.add_substitution(left, right);
    }
    for fact in facts_from_declaration_statement_in_context(statement, context) {
        context.add_fact(fact);
    }
}

fn register_declaration_collection_literal(
    statement: &DeclarationStatement,
    context: &mut TypeContext,
) {
    if let Some(Expression {
        kind: ExpressionKind::Set(set),
        ..
    }) = &statement.definition
    {
        for subject in declaration_subject_keys(statement) {
            context.add_collection_literal(subject, set.clone());
        }
    }

    if let Some(DeclarationRelation::Is(TypeExpression::Coercion { literal, .. })) =
        &statement.relation
    {
        for subject in declaration_subject_keys(statement) {
            context.add_collection_literal(subject, literal.clone());
        }
    }
}

fn register_coerced_collection_literal(
    subject: &Expression,
    ty: &TypeExpression,
    context: &mut TypeContext,
) {
    let TypeExpression::Coercion { literal, .. } = ty else {
        return;
    };
    context.add_collection_literal(key_for_expression(subject), literal.clone());
}

fn register_expression_collection_literal(expression: &Expression, context: &mut TypeContext) {
    let ExpressionKind::Set(set) = &expression.kind else {
        return;
    };
    context.add_collection_literal(key_for_expression(expression), set.clone());
}

fn check_declaration_relation(
    relation: &DeclarationRelation,
    context: &TypeContext,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    registry: &SignatureRegistry,
    event_log: &mut EventLog,
) {
    match relation {
        DeclarationRelation::Is(ty) => {
            check_type_expression(ty, context, path, locator, registry, event_log);
        }
        DeclarationRelation::Spec { target, .. } => {
            check_expression(target, context, path, locator, registry, event_log);
        }
        DeclarationRelation::InfixSpec { spec, target } => {
            check_inactive_expression_tail(&spec.tail, context, path, locator, registry, event_log);
            let active_spec = active_infix_spec(spec, context);
            for expression in infix_spec_arguments(&active_spec) {
                check_expression(expression, context, path, locator, registry, event_log);
            }
            check_expression(target, context, path, locator, registry, event_log);
        }
    }
}

fn check_declaration_spec_facts_supported(
    statement: &DeclarationStatement,
    context: &TypeContext,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    registry: &SignatureRegistry,
    event_log: &mut EventLog,
) {
    let position = match &statement.relation {
        Some(DeclarationRelation::Spec { target, .. }) => spec_target_position(target, locator),
        Some(DeclarationRelation::InfixSpec { target, .. }) => {
            spec_target_position(target, locator)
        }
        _ => None,
    };
    for fact in facts_from_declaration_statement_in_context(statement, context) {
        check_spec_fact_supported(&fact, context, path, position, registry, event_log);
    }
}

fn check_spec_fact_supported(
    fact: &TypeFact,
    context: &TypeContext,
    path: &Path,
    position: Option<SourcePosition>,
    registry: &SignatureRegistry,
    event_log: &mut EventLog,
) {
    match fact {
        TypeFact::Spec {
            operator, target, ..
        } => {
            let target = context.normalize_key(target);
            if registry.spec_rules.iter().any(|rule| {
                rule.operator == *operator
                    && has_type_signature(&target, &rule.owner_signature, context, registry)
            }) {
                return;
            }

            emit_error(
                event_log,
                path,
                position,
                format!(
                    "Could not validate spec fact `{}`: no provided spec operator `\"{}\"` is available for `{}`",
                    format_fact(&context.normalize_fact(fact)),
                    operator,
                    target
                ),
            );
        }
        TypeFact::InfixSpec {
            signature,
            subject,
            args,
            target,
        } => {
            let Some(definition) = registry.definitions.get(signature) else {
                emit_error(
                    event_log,
                    path,
                    position,
                    format!(
                        "Could not validate spec fact `{}`: undefined spec-infix signature `{}`",
                        format_fact(&context.normalize_fact(fact)),
                        signature
                    ),
                );
                return;
            };

            if definition.kind != DefinitionKind::Describes {
                emit_error(
                    event_log,
                    path,
                    position,
                    format!(
                        "Could not validate spec fact `{}`: spec-infix signature `{}` must be defined by Describes",
                        format_fact(&context.normalize_fact(fact)),
                        signature
                    ),
                );
                return;
            }

            let mut actuals = Vec::with_capacity(args.len() + 2);
            actuals.push(subject.clone());
            actuals.extend(args.iter().cloned());
            actuals.push(target.clone());
            check_command_requirements(
                signature, &actuals, context, path, position, registry, event_log,
            );
        }
        _ => {}
    }
}

fn spec_target_position(
    target: &Expression,
    locator: &mut SourceLocator<'_>,
) -> Option<SourcePosition> {
    match &target.kind {
        ExpressionKind::Name(name) => locator.locate_symbol(name),
        _ => None,
    }
}

fn check_is_or_via_item(
    item: &IsOrViaItem,
    context: &TypeContext,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    registry: &SignatureRegistry,
    event_log: &mut EventLog,
) {
    match item {
        IsOrViaItem::IsVia(statement) => {
            check_is_statement(
                &statement.is_statement,
                context,
                path,
                locator,
                registry,
                event_log,
            );
            check_form_or_declaration(&statement.via, context, path, locator, event_log);
        }
        IsOrViaItem::Declaration(statement) => {
            check_declaration_statement(statement, context, path, locator, registry, event_log);
        }
    }
}

fn check_is_statement(
    statement: &IsStatement,
    context: &TypeContext,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    registry: &SignatureRegistry,
    event_log: &mut EventLog,
) {
    check_is_subject(&statement.subject, context, path, locator, event_log);
    check_type_expression(&statement.ty, context, path, locator, registry, event_log);
}

fn check_expression(
    expression: &Expression,
    context: &TypeContext,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    registry: &SignatureRegistry,
    event_log: &mut EventLog,
) {
    match &expression.kind {
        ExpressionKind::Name(name) => {
            check_name(name, context, path, locator, event_log);
        }
        ExpressionKind::FunctionCall { name, arguments } => {
            let function_types = function_type_facts_for_subject(name, context, registry);
            let has_disambiguation =
                has_function_call_disambiguation(name, arguments.len(), registry);
            if !has_disambiguation || !function_types.is_empty() {
                check_name(name, context, path, locator, event_log);
            }
            for argument in arguments {
                check_expression(argument, context, path, locator, registry, event_log);
            }
            if function_types.is_empty() {
                check_disambiguated_function_call(
                    name, arguments, context, path, locator, registry, event_log,
                );
            } else {
                check_function_call_inputs(
                    name, arguments, context, path, locator, registry, event_log,
                );
            }
        }
        ExpressionKind::FunctionNamedCall { name, elements } => {
            check_name(name, context, path, locator, event_log);
            for element in elements {
                check_expression(
                    &element.expression,
                    context,
                    path,
                    locator,
                    registry,
                    event_log,
                );
            }
        }
        ExpressionKind::MemberCall {
            owner,
            name,
            arguments,
        } => {
            check_expression(owner, context, path, locator, registry, event_log);
            for argument in arguments {
                check_expression(argument, context, path, locator, registry, event_log);
            }
            check_provided_member(
                owner, name, arguments, context, path, locator, registry, event_log,
            );
        }
        ExpressionKind::MemberAccess { owner, name } => {
            check_expression(owner, context, path, locator, registry, event_log);
            check_provided_member(
                owner,
                name,
                &[],
                context,
                path,
                locator,
                registry,
                event_log,
            );
        }
        ExpressionKind::Tuple(elements) => {
            for element in elements {
                if let TupleExpressionElement::Expression(expression) = element {
                    check_expression(expression, context, path, locator, registry, event_log);
                }
            }
        }
        ExpressionKind::Set(set) => {
            let mut child = context.clone();
            declare_set_target(&set.target, &mut child);
            for spec in &set.specs {
                assume_fact_expression(spec, &mut child, path, locator, registry, event_log);
                if let Some(fact) = fact_from_expression_in_context(spec, &child) {
                    child.add_fact(fact);
                }
            }
            if let Some(predicate) = &set.predicate {
                check_expression(predicate, &child, path, locator, registry, event_log);
            }
        }
        ExpressionKind::Grouped { expression, .. } | ExpressionKind::Labeled { expression, .. } => {
            check_expression(expression, context, path, locator, registry, event_log);
        }
        ExpressionKind::Prefix {
            operator,
            expression,
        } => {
            check_expression(expression, context, path, locator, registry, event_log);
            check_disambiguated_prefix(
                operator, expression, context, path, locator, registry, event_log,
            );
            check_provided_prefix_operator(
                operator, expression, context, path, locator, registry, event_log,
            );
        }
        ExpressionKind::Postfix {
            expression,
            operator,
        } => {
            check_expression(expression, context, path, locator, registry, event_log);
            check_disambiguated_postfix(
                expression, operator, context, path, locator, registry, event_log,
            );
            check_provided_postfix_operator(
                expression, operator, context, path, locator, registry, event_log,
            );
        }
        ExpressionKind::SubsetCall(subset) => {
            check_subset_call(subset, context, path, locator, event_log);
        }
        ExpressionKind::Command(command) => {
            check_command_expression(command, context, path, locator, registry, event_log);
            let active_command = active_command_expression(command, context);
            for expression in command_expression_arguments(&active_command) {
                check_expression(expression, context, path, locator, registry, event_log);
            }
        }
        ExpressionKind::InfixCommand {
            left,
            command,
            right,
        } => {
            check_expression(left, context, path, locator, registry, event_log);
            check_infix_command(
                left, command, right, context, path, locator, registry, event_log,
            );
            let active_command = active_infix_command(command, context);
            for expression in infix_command_arguments(&active_command) {
                check_expression(expression, context, path, locator, registry, event_log);
            }
            check_expression(right, context, path, locator, registry, event_log);
        }
        ExpressionKind::InfixSpecStatement { left, spec, right } => {
            check_expression(left, context, path, locator, registry, event_log);
            check_inactive_expression_tail(&spec.tail, context, path, locator, registry, event_log);
            let active_spec = active_infix_spec(spec, context);
            for expression in infix_spec_arguments(&active_spec) {
                check_expression(expression, context, path, locator, registry, event_log);
            }
            check_expression(right, context, path, locator, registry, event_log);
            if let Some(fact) =
                fact_from_infix_spec_statement_in_context(left, spec, right, context)
            {
                check_spec_fact_supported(
                    &fact,
                    context,
                    path,
                    locator.locate_reference(&shape_for_infix_spec(&active_spec)),
                    registry,
                    event_log,
                );
            }
        }
        ExpressionKind::Binary {
            left,
            operator,
            right,
        } => {
            check_expression(left, context, path, locator, registry, event_log);
            check_expression(right, context, path, locator, registry, event_log);
            let resolved_from_provided = check_provided_binary_operator(
                left, operator, right, context, path, locator, registry, event_log,
            );
            if !resolved_from_provided && !binary_operator_uses_provided_by_default(operator) {
                check_disambiguated_binary(
                    left, operator, right, context, path, locator, registry, event_log,
                );
            }
        }
        ExpressionKind::SpecStatement(statement) => {
            check_expression(
                &statement.subject,
                context,
                path,
                locator,
                registry,
                event_log,
            );
            check_name(&statement.name, context, path, locator, event_log);
            check_function_call_spec_result(statement, context, path, locator, registry, event_log);
            if let Some(fact) = fact_from_expression(expression) {
                check_spec_fact_supported(
                    &fact,
                    context,
                    path,
                    locator.locate_symbol(&statement.name),
                    registry,
                    event_log,
                );
            }
        }
        ExpressionKind::SpecPredicate(statement) => {
            check_expression(
                &statement.subject,
                context,
                path,
                locator,
                registry,
                event_log,
            );
            check_name(&statement.name, context, path, locator, event_log);
            let fact = TypeFact::Spec {
                subject: key_for_expression(&statement.subject),
                operator: statement.operator.clone(),
                target: statement.name.clone(),
            };
            check_spec_fact_supported(
                &fact,
                context,
                path,
                locator.locate_symbol(&statement.name),
                registry,
                event_log,
            );
        }
        ExpressionKind::MemberOf {
            subject,
            collection,
        } => {
            check_expression(subject, context, path, locator, registry, event_log);
            check_expression(collection, context, path, locator, registry, event_log);
        }
        ExpressionKind::IsPredicate { subject, command }
        | ExpressionKind::IsNotPredicate { subject, command } => {
            check_expression(subject, context, path, locator, registry, event_log);
            check_command_predicate(command, context, path, locator, registry, event_log);
            let active_command = active_command_expression(command, context);
            for expression in command_expression_arguments(&active_command) {
                check_expression(expression, context, path, locator, registry, event_log);
            }
        }
        ExpressionKind::IsBuiltinPredicate { subject, ty } => {
            check_builtin_type_predicate(
                subject, ty, false, context, path, locator, registry, event_log,
            );
        }
        ExpressionKind::IsNotBuiltinPredicate { subject, ty } => {
            check_builtin_type_predicate(
                subject, ty, true, context, path, locator, registry, event_log,
            );
        }
        ExpressionKind::IsRefinedPredicate { subject, command }
        | ExpressionKind::IsNotRefinedPredicate { subject, command } => {
            check_expression(subject, context, path, locator, registry, event_log);
            check_refined_command_expression(command, context, path, locator, registry, event_log);
            let active_command = active_refined_command_expression(command, context);
            for expression in refined_command_expression_arguments(&active_command) {
                check_expression(expression, context, path, locator, registry, event_log);
            }
        }
        ExpressionKind::IsType { subject, ty } => {
            check_expression(subject, context, path, locator, registry, event_log);
            check_type_expression(ty, context, path, locator, registry, event_log);
            check_function_call_result(subject, ty, context, path, locator, registry, event_log);
        }
    }
}

fn check_type_expression(
    ty: &TypeExpression,
    context: &TypeContext,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    registry: &SignatureRegistry,
    event_log: &mut EventLog,
) {
    match ty {
        TypeExpression::Builtin { .. } => {}
        TypeExpression::Command(command) => {
            check_command_type_expression(command, context, path, locator, registry, event_log);
            let active_command = active_command_expression(command, context);
            for expression in command_expression_arguments(&active_command) {
                check_expression(expression, context, path, locator, registry, event_log);
            }
        }
        TypeExpression::RefinedCommand(command) => {
            check_refined_command_type_expression(
                command, context, path, locator, registry, event_log,
            );
            let active_command = active_refined_command_expression(command, context);
            for expression in refined_command_expression_arguments(&active_command) {
                check_expression(expression, context, path, locator, registry, event_log);
            }
        }
        TypeExpression::Function(function_type) => {
            for spec in function_type
                .inputs
                .iter()
                .chain(std::iter::once(&function_type.output))
            {
                check_function_type_spec(spec, context, path, locator, registry, event_log);
            }
        }
        TypeExpression::Coercion { ty, literal, .. } => {
            check_type_expression(ty, context, path, locator, registry, event_log);
            check_set_literal(literal, context, path, locator, registry, event_log);
        }
    }
}

fn check_set_literal(
    set: &SetExpression,
    context: &TypeContext,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    registry: &SignatureRegistry,
    event_log: &mut EventLog,
) {
    let mut child = context.clone();
    declare_set_target(&set.target, &mut child);
    for spec in &set.specs {
        assume_fact_expression(spec, &mut child, path, locator, registry, event_log);
        if let Some(fact) = fact_from_expression_in_context(spec, &child) {
            child.add_fact(fact);
        }
    }
    if let Some(predicate) = &set.predicate {
        check_expression(predicate, &child, path, locator, registry, event_log);
    }
}

fn check_builtin_type_predicate(
    subject: &Expression,
    ty: &TypeExpression,
    negated: bool,
    context: &TypeContext,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    registry: &SignatureRegistry,
    event_log: &mut EventLog,
) {
    let position = builtin_predicate_position(subject, context, locator);
    check_expression(subject, context, path, locator, registry, event_log);
    check_type_expression(ty, context, path, locator, registry, event_log);
    let Some(required) = fact_from_type_assertion_in_context(subject, ty, context) else {
        return;
    };
    let holds = prove_fact(&required, context, registry);
    if (!negated && holds) || (negated && !holds) {
        return;
    }

    let predicate = if negated { "is_not?" } else { "is?" };
    emit_error(
        event_log,
        path,
        position,
        format!(
            "Could not establish predicate `{} {predicate} {}`",
            key_for_expression(subject),
            key_for_type_expression_in_context(ty, context)
                .map(|(key, _)| key)
                .unwrap_or_else(|| key_for_non_command_type_expression(ty))
        ),
    );
}

fn builtin_predicate_position(
    subject: &Expression,
    context: &TypeContext,
    locator: &mut SourceLocator<'_>,
) -> Option<SourcePosition> {
    match &subject.kind {
        ExpressionKind::Command(command) => {
            let active_command = active_command_expression(command, context);
            locator.locate_reference(&shape_for_command_expression(&active_command))
        }
        ExpressionKind::Name(name) => locator.locate_symbol(name),
        _ => None,
    }
}

fn check_function_type_spec(
    spec: &FunctionTypeSpec,
    context: &TypeContext,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    registry: &SignatureRegistry,
    event_log: &mut EventLog,
) {
    if spec.subject != "_" {
        emit_error(
            event_log,
            path,
            locator.locate_symbol(&spec.subject),
            "Function type parameters must be `_`",
        );
    }

    match &spec.kind {
        FunctionTypeSpecKind::Is(ty) => {
            check_type_expression(ty, context, path, locator, registry, event_log);
        }
        FunctionTypeSpecKind::Spec { target, .. } => {
            check_name(target, context, path, locator, event_log);
        }
    }
}

fn check_function_call_inputs(
    name: &str,
    arguments: &[Expression],
    context: &TypeContext,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    registry: &SignatureRegistry,
    event_log: &mut EventLog,
) {
    let function_types = function_type_facts_for_subject(name, context, registry);
    let mut matched_arity = false;
    for function_type in function_types {
        let TypeFact::FunctionType {
            inputs,
            output: _,
            variadic_tuple_input,
            ..
        } = function_type
        else {
            continue;
        };
        let argument_subjects = function_type_argument_subjects_from_expressions(
            inputs.len(),
            variadic_tuple_input,
            arguments,
            context,
            registry,
        );
        let Some(argument_subjects) = argument_subjects else {
            continue;
        };
        matched_arity = true;

        for (input, argument_subject) in inputs.iter().zip(argument_subjects) {
            let required = instantiate_function_type_spec(input, &argument_subject);
            if !prove_fact(&required, context, registry) {
                emit_error(
                    event_log,
                    path,
                    locator.locate_symbol(name),
                    format!(
                        "Could not establish requirement `{}` for function `{name}`",
                        format_fact(&required)
                    ),
                );
            }
        }
    }

    if !matched_arity {
        emit_error(
            event_log,
            path,
            locator.locate_symbol(name),
            format!(
                "Could not match function `{name}` with {} argument(s)",
                arguments.len()
            ),
        );
    }
}

fn has_function_call_disambiguation(
    name: &str,
    arity: usize,
    registry: &SignatureRegistry,
) -> bool {
    let key = DisambiguationKey::Function {
        name: name.to_owned(),
        arity,
    };
    has_disambiguation_for_key(&key, registry)
}

fn check_disambiguated_function_call(
    name: &str,
    arguments: &[Expression],
    context: &TypeContext,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    registry: &SignatureRegistry,
    event_log: &mut EventLog,
) {
    let key = DisambiguationKey::Function {
        name: name.to_owned(),
        arity: arguments.len(),
    };
    if !registry
        .disambiguations
        .iter()
        .any(|rule| disambiguation_keys_match(&key, &rule.key))
    {
        return;
    }
    let actuals = arguments
        .iter()
        .map(|argument| effective_key_for_expression(argument, context, registry))
        .collect::<Vec<_>>();
    let position = locator.locate_symbol(name);
    check_disambiguated_expression(
        &key,
        &actuals,
        &format!("function `{name}`"),
        position,
        context,
        path,
        locator,
        registry,
        event_log,
    );
}

fn check_disambiguated_binary(
    left: &Expression,
    operator: &BinaryOperator,
    right: &Expression,
    context: &TypeContext,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    registry: &SignatureRegistry,
    event_log: &mut EventLog,
) {
    let Some((key, label, symbol)) = disambiguation_key_for_binary_operator(operator) else {
        return;
    };
    if !has_disambiguation_for_key(&key, registry) {
        if context
            .active_disambiguations
            .iter()
            .any(|active| disambiguation_keys_match(active, &key))
        {
            return;
        }

        emit_error(
            event_log,
            path,
            locator.locate_symbol(&symbol),
            format!("Could not resolve {label}: no matching `Disambiguates` entry was found"),
        );
        return;
    }
    let actuals = vec![
        effective_key_for_expression(left, context, registry),
        effective_key_for_expression(right, context, registry),
    ];
    let position = locator.locate_symbol(&symbol);
    check_disambiguated_expression(
        &key, &actuals, &label, position, context, path, locator, registry, event_log,
    );
}

fn check_disambiguated_prefix(
    operator: &UnaryOperator,
    expression: &Expression,
    context: &TypeContext,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    registry: &SignatureRegistry,
    event_log: &mut EventLog,
) {
    let (key, label, symbol) = disambiguation_key_for_prefix_operator(operator);
    if !registry
        .disambiguations
        .iter()
        .any(|rule| disambiguation_keys_match(&key, &rule.key))
    {
        return;
    }
    let actuals = vec![effective_key_for_expression(expression, context, registry)];
    let position = locator.locate_symbol(&symbol);
    check_disambiguated_expression(
        &key, &actuals, &label, position, context, path, locator, registry, event_log,
    );
}

fn check_disambiguated_postfix(
    expression: &Expression,
    operator: &Operator,
    context: &TypeContext,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    registry: &SignatureRegistry,
    event_log: &mut EventLog,
) {
    let key = DisambiguationKey::PostfixOperator(operator.text.clone());
    if !has_disambiguation_for_key(&key, registry) {
        return;
    }
    let actuals = vec![effective_key_for_expression(expression, context, registry)];
    let position = locator.locate_symbol(&operator.text);
    check_disambiguated_expression(
        &key,
        &actuals,
        &format!("operator `|{}`", operator.text),
        position,
        context,
        path,
        locator,
        registry,
        event_log,
    );
}

fn check_provided_binary_operator(
    left: &Expression,
    operator: &BinaryOperator,
    right: &Expression,
    context: &TypeContext,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    registry: &SignatureRegistry,
    event_log: &mut EventLog,
) -> bool {
    let (symbol, kind) = binary_operator_symbol_and_kind(operator);
    let Some(kind) = provided_binary_operator_kind(&symbol, kind) else {
        return false;
    };

    let key = DisambiguationKey::BinaryOperator(symbol.clone());
    let actuals = vec![key_for_expression(left), key_for_expression(right)];
    let Some(rule) = find_provided_symbol_rule(&key, kind, &actuals, context, registry) else {
        if context.defer_unresolved_provided_symbols
            || binary_operator_uses_provided_by_default(operator)
        {
            return false;
        }

        emit_error(
            event_log,
            path,
            locator.locate_symbol(&symbol),
            format!(
                "Could not resolve operator `{symbol}` from {}",
                resolution_kind_label(kind)
            ),
        );
        return false;
    };

    let owner_actual = provided_symbol_owner_actual(kind, &actuals);
    check_provided_symbol_target(
        rule,
        &actuals,
        owner_actual.as_deref(),
        context,
        path,
        locator,
        registry,
        event_log,
    );
    true
}

fn check_provided_prefix_operator(
    operator: &UnaryOperator,
    expression: &Expression,
    context: &TypeContext,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    registry: &SignatureRegistry,
    event_log: &mut EventLog,
) {
    let (key, _, _) = disambiguation_key_for_prefix_operator(operator);
    let actuals = vec![key_for_expression(expression)];
    let Some(rule) = find_provided_symbol_rule(
        &key,
        NamedOperatorKind::BothColon,
        &actuals,
        context,
        registry,
    ) else {
        return;
    };

    let owner_actual = provided_symbol_owner_actual(NamedOperatorKind::LeftColon, &actuals);
    check_provided_symbol_target(
        rule,
        &actuals,
        owner_actual.as_deref(),
        context,
        path,
        locator,
        registry,
        event_log,
    );
}

fn check_provided_postfix_operator(
    expression: &Expression,
    operator: &Operator,
    context: &TypeContext,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    registry: &SignatureRegistry,
    event_log: &mut EventLog,
) {
    let key = DisambiguationKey::PostfixOperator(operator.text.clone());
    let actuals = vec![key_for_expression(expression)];
    let Some(rule) = find_provided_symbol_rule(
        &key,
        NamedOperatorKind::BothColon,
        &actuals,
        context,
        registry,
    ) else {
        return;
    };

    let owner_actual = provided_symbol_owner_actual(NamedOperatorKind::LeftColon, &actuals);
    check_provided_symbol_target(
        rule,
        &actuals,
        owner_actual.as_deref(),
        context,
        path,
        locator,
        registry,
        event_log,
    );
}

fn check_provided_member(
    owner: &Expression,
    name: &str,
    arguments: &[Expression],
    context: &TypeContext,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    registry: &SignatureRegistry,
    event_log: &mut EventLog,
) {
    let key = DisambiguationKey::Function {
        name: name.to_owned(),
        arity: arguments.len(),
    };
    let owner_actual = key_for_expression(owner);
    let actuals = arguments.iter().map(key_for_expression).collect::<Vec<_>>();
    let Some(rule) =
        find_member_provided_symbol_rule(&key, &owner_actual, &actuals, context, registry)
    else {
        emit_error(
            event_log,
            path,
            locator.locate_symbol(name),
            format!(
                "Could not resolve member `{name}` for `{}`",
                context.normalize_key(&owner_actual)
            ),
        );
        return;
    };

    check_provided_symbol_target(
        rule,
        &actuals,
        Some(&owner_actual),
        context,
        path,
        locator,
        registry,
        event_log,
    );
}

fn provided_symbol_owner_actual(kind: NamedOperatorKind, actuals: &[String]) -> Option<String> {
    match kind {
        NamedOperatorKind::Plain => None,
        NamedOperatorKind::LeftColon | NamedOperatorKind::BothColon => actuals.first().cloned(),
        NamedOperatorKind::RightColon => actuals.last().cloned(),
    }
}

fn find_provided_symbol_rule<'a>(
    key: &DisambiguationKey,
    kind: NamedOperatorKind,
    actuals: &[String],
    context: &TypeContext,
    registry: &'a SignatureRegistry,
) -> Option<&'a ProvidedSymbolRule> {
    registry.provided_symbols.iter().find(|rule| {
        let owner_actual = provided_symbol_owner_actual(kind, actuals);
        disambiguation_keys_match(key, &rule.key)
            && rule.parameters.len() == actuals.len()
            && provided_symbol_owner_matches(
                kind,
                &rule.owner_signature,
                actuals,
                context,
                registry,
            )
            && provided_symbol_source_matches(rule, owner_actual.as_deref(), context)
    })
}

fn find_member_provided_symbol_rule<'a>(
    key: &DisambiguationKey,
    owner_actual: &str,
    actuals: &[String],
    context: &TypeContext,
    registry: &'a SignatureRegistry,
) -> Option<&'a ProvidedSymbolRule> {
    registry.provided_symbols.iter().find(|rule| {
        disambiguation_keys_match(key, &rule.key)
            && rule.parameters.len() == actuals.len()
            && has_type_signature(owner_actual, &rule.owner_signature, context, registry)
            && provided_symbol_source_matches(rule, Some(owner_actual), context)
    })
}

fn provided_symbol_source_matches(
    rule: &ProvidedSymbolRule,
    owner_actual: Option<&str>,
    context: &TypeContext,
) -> bool {
    rule.source_subject.is_none()
        || owner_actual.is_some_and(|actual| context.collection_literal(actual).is_some())
}

fn provided_symbol_owner_matches(
    kind: NamedOperatorKind,
    owner_signature: &str,
    actuals: &[String],
    context: &TypeContext,
    registry: &SignatureRegistry,
) -> bool {
    match kind {
        NamedOperatorKind::Plain => false,
        NamedOperatorKind::LeftColon => actuals
            .first()
            .is_some_and(|actual| has_type_signature(actual, owner_signature, context, registry)),
        NamedOperatorKind::RightColon => actuals
            .last()
            .is_some_and(|actual| has_type_signature(actual, owner_signature, context, registry)),
        NamedOperatorKind::BothColon => actuals
            .iter()
            .all(|actual| has_type_signature(actual, owner_signature, context, registry)),
    }
}

fn check_provided_symbol_target(
    rule: &ProvidedSymbolRule,
    actuals: &[String],
    owner_actual: Option<&str>,
    context: &TypeContext,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    registry: &SignatureRegistry,
    event_log: &mut EventLog,
) {
    let mut child = context.clone();
    for parameter in &rule.parameters {
        child.declare_name(parameter.clone());
    }
    for (parameter, actual) in rule.parameters.iter().zip(actuals) {
        child.add_substitution(parameter.clone(), context.normalize_key(actual));
    }
    if let Some(owner_actual) = owner_actual {
        child.declare_name(rule.owner_subject.clone());
        child.add_substitution(
            rule.owner_subject.clone(),
            context.normalize_key(owner_actual),
        );
        if let Some(source_subject) = &rule.source_subject {
            child.declare_name(source_subject.clone());
            child.add_substitution(source_subject.clone(), context.normalize_key(owner_actual));
        }
    }
    check_expression(&rule.target, &child, path, locator, registry, event_log);
}

fn effective_key_for_expression(
    expression: &Expression,
    context: &TypeContext,
    registry: &SignatureRegistry,
) -> String {
    let mut resolving = HashSet::new();
    effective_key_for_expression_inner(expression, context, registry, &mut resolving)
}

fn effective_key_for_expression_inner(
    expression: &Expression,
    context: &TypeContext,
    registry: &SignatureRegistry,
    resolving: &mut HashSet<String>,
) -> String {
    if let ExpressionKind::Grouped { expression, .. } | ExpressionKind::Labeled { expression, .. } =
        &expression.kind
    {
        return effective_key_for_expression_inner(expression, context, registry, resolving);
    }

    let raw_key = context.normalize_key(&key_for_expression(expression));
    if !resolving.insert(raw_key.clone()) {
        return raw_key;
    }

    let result = match &expression.kind {
        ExpressionKind::FunctionCall { name, arguments } => {
            effective_key_for_function_call(name, arguments, context, registry, resolving)
        }
        ExpressionKind::MemberCall {
            owner,
            name,
            arguments,
        } => effective_key_for_member_call(owner, name, arguments, context, registry, resolving),
        ExpressionKind::MemberAccess { owner, name } => {
            effective_key_for_member_access(owner, name, context, registry, resolving)
        }
        ExpressionKind::Prefix {
            operator,
            expression,
        } => {
            effective_key_for_prefix_expression(operator, expression, context, registry, resolving)
        }
        ExpressionKind::Postfix {
            expression,
            operator,
        } => {
            effective_key_for_postfix_expression(expression, operator, context, registry, resolving)
        }
        ExpressionKind::Binary {
            left,
            operator,
            right,
        } => {
            effective_key_for_binary_expression(left, operator, right, context, registry, resolving)
        }
        _ => None,
    }
    .unwrap_or_else(|| raw_key.clone());

    resolving.remove(&raw_key);
    result
}

fn effective_keys_for_expressions(
    expressions: &[Expression],
    context: &TypeContext,
    registry: &SignatureRegistry,
    resolving: &mut HashSet<String>,
) -> Vec<String> {
    expressions
        .iter()
        .map(|expression| {
            effective_key_for_expression_inner(expression, context, registry, resolving)
        })
        .collect()
}

fn effective_key_for_function_call(
    name: &str,
    arguments: &[Expression],
    context: &TypeContext,
    registry: &SignatureRegistry,
    resolving: &mut HashSet<String>,
) -> Option<String> {
    let key = DisambiguationKey::Function {
        name: name.to_owned(),
        arity: arguments.len(),
    };
    let actuals = effective_keys_for_expressions(arguments, context, registry, resolving);
    effective_key_for_disambiguated_target(&key, &actuals, context, registry, resolving)
}

fn effective_key_for_member_call(
    owner: &Expression,
    name: &str,
    arguments: &[Expression],
    context: &TypeContext,
    registry: &SignatureRegistry,
    resolving: &mut HashSet<String>,
) -> Option<String> {
    let key = DisambiguationKey::Function {
        name: name.to_owned(),
        arity: arguments.len(),
    };
    let owner_actual = effective_key_for_expression_inner(owner, context, registry, resolving);
    let actuals = effective_keys_for_expressions(arguments, context, registry, resolving);
    let rule = find_member_provided_symbol_rule(&key, &owner_actual, &actuals, context, registry)?;
    Some(effective_key_for_provided_symbol_target(
        rule,
        &actuals,
        Some(&owner_actual),
        context,
        registry,
        resolving,
    ))
}

fn effective_key_for_member_access(
    owner: &Expression,
    name: &str,
    context: &TypeContext,
    registry: &SignatureRegistry,
    resolving: &mut HashSet<String>,
) -> Option<String> {
    effective_key_for_member_call(owner, name, &[], context, registry, resolving)
}

fn effective_key_for_prefix_expression(
    operator: &UnaryOperator,
    expression: &Expression,
    context: &TypeContext,
    registry: &SignatureRegistry,
    resolving: &mut HashSet<String>,
) -> Option<String> {
    let (key, _, _) = disambiguation_key_for_prefix_operator(operator);
    let actuals = vec![effective_key_for_expression_inner(
        expression, context, registry, resolving,
    )];
    if let Some(key) =
        effective_key_for_disambiguated_target(&key, &actuals, context, registry, resolving)
    {
        return Some(key);
    }

    let rule = find_provided_symbol_rule(
        &key,
        NamedOperatorKind::BothColon,
        &actuals,
        context,
        registry,
    )?;
    let owner_actual = provided_symbol_owner_actual(NamedOperatorKind::LeftColon, &actuals);
    Some(effective_key_for_provided_symbol_target(
        rule,
        &actuals,
        owner_actual.as_deref(),
        context,
        registry,
        resolving,
    ))
}

fn effective_key_for_postfix_expression(
    expression: &Expression,
    operator: &Operator,
    context: &TypeContext,
    registry: &SignatureRegistry,
    resolving: &mut HashSet<String>,
) -> Option<String> {
    let key = DisambiguationKey::PostfixOperator(operator.text.clone());
    let actuals = vec![effective_key_for_expression_inner(
        expression, context, registry, resolving,
    )];
    if let Some(key) =
        effective_key_for_disambiguated_target(&key, &actuals, context, registry, resolving)
    {
        return Some(key);
    }

    let rule = find_provided_symbol_rule(
        &key,
        NamedOperatorKind::BothColon,
        &actuals,
        context,
        registry,
    )?;
    let owner_actual = provided_symbol_owner_actual(NamedOperatorKind::LeftColon, &actuals);
    Some(effective_key_for_provided_symbol_target(
        rule,
        &actuals,
        owner_actual.as_deref(),
        context,
        registry,
        resolving,
    ))
}

fn effective_key_for_binary_expression(
    left: &Expression,
    operator: &BinaryOperator,
    right: &Expression,
    context: &TypeContext,
    registry: &SignatureRegistry,
    resolving: &mut HashSet<String>,
) -> Option<String> {
    let (symbol, kind) = binary_operator_symbol_and_kind(operator);
    let actuals = vec![
        effective_key_for_expression_inner(left, context, registry, resolving),
        effective_key_for_expression_inner(right, context, registry, resolving),
    ];

    if let Some(kind) = provided_binary_operator_kind(&symbol, kind) {
        if let Some(rule) = find_provided_symbol_rule(
            &DisambiguationKey::BinaryOperator(symbol.clone()),
            kind,
            &actuals,
            context,
            registry,
        ) {
            let owner_actual = provided_symbol_owner_actual(kind, &actuals);
            return Some(effective_key_for_provided_symbol_target(
                rule,
                &actuals,
                owner_actual.as_deref(),
                context,
                registry,
                resolving,
            ));
        }
    }

    if binary_operator_uses_provided_by_default(operator) {
        return None;
    }

    let (key, _, _) = disambiguation_key_for_binary_operator(operator)?;
    effective_key_for_disambiguated_target(&key, &actuals, context, registry, resolving)
}

fn effective_key_for_disambiguated_target(
    key: &DisambiguationKey,
    actuals: &[String],
    context: &TypeContext,
    registry: &SignatureRegistry,
    resolving: &mut HashSet<String>,
) -> Option<String> {
    let rule = registry
        .disambiguations
        .iter()
        .find(|rule| disambiguation_keys_match(key, &rule.key))?;
    if rule.parameters.len() != actuals.len()
        || context
            .active_disambiguations
            .iter()
            .any(|active| disambiguation_keys_match(active, &rule.key))
    {
        return None;
    }

    for branch in &rule.branches {
        if disambiguation_branch_matches(rule, branch, actuals, context, registry) {
            return effective_key_for_disambiguation_target(
                rule,
                branch.substitutions.as_slice(),
                actuals,
                &branch.to,
                context,
                registry,
                resolving,
            );
        }
    }

    let expression = rule.else_expression.as_ref()?;
    effective_key_for_disambiguation_target(
        rule,
        &[],
        actuals,
        expression,
        context,
        registry,
        resolving,
    )
}

fn effective_key_for_disambiguation_target(
    rule: &DisambiguationRule,
    branch_substitutions: &[(String, String)],
    actuals: &[String],
    expression: &Expression,
    context: &TypeContext,
    registry: &SignatureRegistry,
    resolving: &mut HashSet<String>,
) -> Option<String> {
    let mut child = context.activate_disambiguation(&rule.key)?;
    let substitutions = disambiguation_substitutions(rule, actuals, context);
    for parameter in &rule.parameters {
        child.declare_name(parameter.clone());
    }
    for (parameter, actual) in rule.parameters.iter().zip(actuals) {
        child.add_substitution(parameter.clone(), context.normalize_key(actual));
    }
    for (left, right) in branch_substitutions {
        child.add_substitution(
            substitute_key(left, &substitutions),
            substitute_key(right, &substitutions),
        );
    }

    Some(effective_key_for_expression_inner(
        expression, &child, registry, resolving,
    ))
}

fn effective_key_for_provided_symbol_target(
    rule: &ProvidedSymbolRule,
    actuals: &[String],
    owner_actual: Option<&str>,
    context: &TypeContext,
    registry: &SignatureRegistry,
    resolving: &mut HashSet<String>,
) -> String {
    let mut child = context.clone();
    for parameter in &rule.parameters {
        child.declare_name(parameter.clone());
    }
    for (parameter, actual) in rule.parameters.iter().zip(actuals) {
        child.add_substitution(parameter.clone(), context.normalize_key(actual));
    }
    if let Some(owner_actual) = owner_actual {
        child.declare_name(rule.owner_subject.clone());
        child.add_substitution(
            rule.owner_subject.clone(),
            context.normalize_key(owner_actual),
        );
        if let Some(source_subject) = &rule.source_subject {
            child.declare_name(source_subject.clone());
            child.add_substitution(source_subject.clone(), context.normalize_key(owner_actual));
        }
    }

    effective_key_for_expression_inner(&rule.target, &child, registry, resolving)
}

fn binary_operator_symbol_and_kind(operator: &BinaryOperator) -> (String, NamedOperatorKind) {
    match operator {
        BinaryOperator::Equality(operator)
        | BinaryOperator::Special(operator)
        | BinaryOperator::Add(operator)
        | BinaryOperator::Subtract(operator)
        | BinaryOperator::Multiply(operator)
        | BinaryOperator::Divide(operator)
        | BinaryOperator::Power(operator) => (operator.text.clone(), operator.kind),
        BinaryOperator::Named(operator) => (operator.name.clone(), operator.kind),
    }
}

fn provided_binary_operator_kind(
    symbol: &str,
    kind: NamedOperatorKind,
) -> Option<NamedOperatorKind> {
    match (symbol, kind) {
        ("=", NamedOperatorKind::Plain) => Some(NamedOperatorKind::BothColon),
        ("!=", NamedOperatorKind::Plain) => Some(NamedOperatorKind::BothColon),
        (_, NamedOperatorKind::Plain) => None,
        (_, kind) => Some(kind),
    }
}

fn binary_operator_uses_provided_by_default(operator: &BinaryOperator) -> bool {
    let (symbol, kind) = binary_operator_symbol_and_kind(operator);
    matches!(symbol.as_str(), "=" | "!=") && kind == NamedOperatorKind::Plain
}

fn resolution_kind_label(kind: NamedOperatorKind) -> &'static str {
    match kind {
        NamedOperatorKind::Plain => "the local or global scope",
        NamedOperatorKind::LeftColon => "the left operand type",
        NamedOperatorKind::RightColon => "the right operand type",
        NamedOperatorKind::BothColon => "the common operand type",
    }
}

fn check_disambiguated_expression(
    key: &DisambiguationKey,
    actuals: &[String],
    label: &str,
    position: Option<SourcePosition>,
    context: &TypeContext,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    registry: &SignatureRegistry,
    event_log: &mut EventLog,
) {
    let Some(rule) = registry
        .disambiguations
        .iter()
        .find(|rule| disambiguation_keys_match(key, &rule.key))
    else {
        return;
    };
    if rule.parameters.len() != actuals.len() {
        return;
    }
    if context
        .active_disambiguations
        .iter()
        .any(|active| disambiguation_keys_match(active, &rule.key))
    {
        return;
    }

    for branch in &rule.branches {
        if disambiguation_branch_matches(rule, branch, actuals, context, registry) {
            check_disambiguation_target(
                rule,
                branch.substitutions.as_slice(),
                actuals,
                &branch.to,
                context,
                path,
                locator,
                registry,
                event_log,
            );
            return;
        }
    }

    if let Some(expression) = &rule.else_expression {
        check_disambiguation_target(
            rule,
            &[],
            actuals,
            expression,
            context,
            path,
            locator,
            registry,
            event_log,
        );
        return;
    }

    emit_error(
        event_log,
        path,
        position,
        format!(
            "Could not disambiguate {label} for arguments `{}`",
            actuals.join("`, `")
        ),
    );
}

fn check_disambiguation_target(
    rule: &DisambiguationRule,
    branch_substitutions: &[(String, String)],
    actuals: &[String],
    expression: &Expression,
    context: &TypeContext,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    registry: &SignatureRegistry,
    event_log: &mut EventLog,
) {
    let Some(mut child) = context.activate_disambiguation(&rule.key) else {
        return;
    };
    let substitutions = disambiguation_substitutions(rule, actuals, context);
    for parameter in &rule.parameters {
        child.declare_name(parameter.clone());
    }
    for (parameter, actual) in rule.parameters.iter().zip(actuals) {
        child.add_substitution(parameter.clone(), context.normalize_key(actual));
    }
    for (left, right) in branch_substitutions {
        child.add_substitution(
            substitute_key(left, &substitutions),
            substitute_key(right, &substitutions),
        );
    }
    check_expression(expression, &child, path, locator, registry, event_log);
}

fn disambiguation_branch_matches(
    rule: &DisambiguationRule,
    branch: &DisambiguationBranch,
    actuals: &[String],
    context: &TypeContext,
    registry: &SignatureRegistry,
) -> bool {
    let substitutions = disambiguation_substitutions(rule, actuals, context);
    let mut requirement_context = context.clone();
    for (left, right) in &branch.substitutions {
        requirement_context.add_substitution(
            substitute_key(left, &substitutions),
            substitute_key(right, &substitutions),
        );
    }

    branch.requirements.iter().all(|requirement| {
        let instantiated = substitute_fact(requirement, &substitutions);
        prove_fact_without_viewable(&instantiated, &requirement_context, registry)
    })
}

fn disambiguation_substitutions(
    rule: &DisambiguationRule,
    actuals: &[String],
    context: &TypeContext,
) -> HashMap<String, String> {
    rule.parameters
        .iter()
        .zip(actuals)
        .map(|(parameter, actual)| (parameter.clone(), context.normalize_key(actual)))
        .collect()
}

fn disambiguation_keys_match(left: &DisambiguationKey, right: &DisambiguationKey) -> bool {
    left == right
        || equivalent_disambiguation_keys(left)
            .iter()
            .any(|key| key == right)
}

fn has_disambiguation_for_key(key: &DisambiguationKey, registry: &SignatureRegistry) -> bool {
    registry
        .disambiguations
        .iter()
        .any(|rule| disambiguation_keys_match(key, &rule.key))
}

fn equivalent_disambiguation_keys(key: &DisambiguationKey) -> Vec<DisambiguationKey> {
    match key {
        DisambiguationKey::BinaryOperator(operator) => vec![DisambiguationKey::Function {
            name: function_name_for_operator(operator),
            arity: 2,
        }],
        DisambiguationKey::Function { name, arity: 2 } => {
            vec![DisambiguationKey::BinaryOperator(function_operator_name(
                name,
            ))]
        }
        DisambiguationKey::PrefixOperator(operator)
        | DisambiguationKey::PostfixOperator(operator) => vec![DisambiguationKey::Function {
            name: function_name_for_operator(operator),
            arity: 1,
        }],
        DisambiguationKey::Function { name, arity: 1 } => vec![
            DisambiguationKey::PrefixOperator(function_operator_name(name)),
            DisambiguationKey::PostfixOperator(function_operator_name(name)),
        ],
        DisambiguationKey::Function { .. } => Vec::new(),
    }
}

fn function_name_for_operator(operator: &str) -> String {
    if is_plain_function_name(operator) {
        operator.to_owned()
    } else {
        format!("`{operator}`")
    }
}

fn function_operator_name(name: &str) -> String {
    unstrop_operator_name(name).unwrap_or_else(|| name.to_owned())
}

fn unstrop_operator_name(name: &str) -> Option<String> {
    name.strip_prefix('`')
        .and_then(|rest| rest.strip_suffix('`'))
        .map(ToOwned::to_owned)
}

fn is_plain_function_name(value: &str) -> bool {
    !value.is_empty()
        && value
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
}

fn disambiguation_key_for_binary_operator(
    operator: &BinaryOperator,
) -> Option<(DisambiguationKey, String, String)> {
    match operator {
        BinaryOperator::Special(operator)
        | BinaryOperator::Add(operator)
        | BinaryOperator::Subtract(operator)
        | BinaryOperator::Multiply(operator)
        | BinaryOperator::Divide(operator)
        | BinaryOperator::Power(operator)
            if operator.kind == NamedOperatorKind::Plain =>
        {
            Some((
                DisambiguationKey::BinaryOperator(operator.text.clone()),
                format!("operator `{}`", operator.text),
                operator.text.clone(),
            ))
        }
        BinaryOperator::Equality(_)
        | BinaryOperator::Special(_)
        | BinaryOperator::Add(_)
        | BinaryOperator::Subtract(_)
        | BinaryOperator::Multiply(_)
        | BinaryOperator::Divide(_)
        | BinaryOperator::Power(_) => None,
        BinaryOperator::Named(operator) if operator.kind == NamedOperatorKind::Plain => Some((
            DisambiguationKey::BinaryOperator(operator.name.clone()),
            format!("operator `|{}|`", operator.name),
            operator.name.clone(),
        )),
        BinaryOperator::Named(_) => None,
    }
}

fn disambiguation_key_for_prefix_operator(
    operator: &UnaryOperator,
) -> (DisambiguationKey, String, String) {
    match operator {
        UnaryOperator::Arithmetic(operator) => (
            DisambiguationKey::PrefixOperator(operator.text.clone()),
            format!("operator `{}`", operator.text),
            operator.text.clone(),
        ),
        UnaryOperator::Named(operator) => (
            DisambiguationKey::PrefixOperator(operator.text.clone()),
            format!("operator `{}|`", operator.text),
            operator.text.clone(),
        ),
    }
}

fn check_function_call_result(
    subject: &Expression,
    ty: &TypeExpression,
    context: &TypeContext,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    registry: &SignatureRegistry,
    event_log: &mut EventLog,
) {
    let ExpressionKind::FunctionCall { name, .. } = &subject.kind else {
        return;
    };
    let Some(required) = fact_from_type_assertion(subject, ty) else {
        return;
    };
    check_function_call_result_fact(
        name, subject, required, context, path, locator, registry, event_log,
    );
}

fn check_function_call_spec_result(
    statement: &SpecStatement,
    context: &TypeContext,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    registry: &SignatureRegistry,
    event_log: &mut EventLog,
) {
    let ExpressionKind::FunctionCall { name, .. } = &statement.subject.kind else {
        return;
    };
    let required = TypeFact::Spec {
        subject: key_for_expression(&statement.subject),
        operator: statement.operator.clone(),
        target: statement.name.clone(),
    };
    check_function_call_result_fact(
        name,
        &statement.subject,
        required,
        context,
        path,
        locator,
        registry,
        event_log,
    );
}

fn check_function_call_result_fact(
    name: &str,
    subject: &Expression,
    required: TypeFact,
    context: &TypeContext,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    registry: &SignatureRegistry,
    event_log: &mut EventLog,
) {
    let function_types = function_type_facts_for_subject(name, context, registry);
    let mut found_matching_arity = false;
    for function_type in &function_types {
        if function_type_matches_call_arity(function_type, function_call_arity(subject)) {
            found_matching_arity = true;
        }
        let mut seen = HashSet::new();
        if function_type_implies_required(
            function_type,
            &required,
            context,
            registry,
            &mut seen,
            true,
        ) {
            return;
        }
    }

    if found_matching_arity {
        emit_error(
            event_log,
            path,
            locator.locate_symbol(name),
            format!(
                "Could not establish function call result `{}`",
                format_fact(&required)
            ),
        );
    }
}

fn check_command_expression(
    command: &CommandExpression,
    context: &TypeContext,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    registry: &SignatureRegistry,
    event_log: &mut EventLog,
) {
    check_inactive_expression_tail(&command.tail, context, path, locator, registry, event_log);
    let active_command = active_command_expression(command, context);
    let shape = shape_for_command_expression(&active_command);
    let position = locator.locate_reference(&shape);
    let actuals = command_expression_arguments(&active_command)
        .into_iter()
        .map(|expression| effective_key_for_expression(expression, context, registry))
        .collect::<Vec<_>>();
    check_command_requirements(
        &shape.signature,
        &actuals,
        context,
        path,
        position,
        registry,
        event_log,
    );
}

fn check_command_type_expression(
    command: &CommandExpression,
    context: &TypeContext,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    registry: &SignatureRegistry,
    event_log: &mut EventLog,
) {
    check_inactive_expression_tail(&command.tail, context, path, locator, registry, event_log);
    let active_command = active_command_expression(command, context);
    let shape = shape_for_command_expression(&active_command);
    let position = locator.locate_reference(&shape);
    let actuals = command_expression_arguments(&active_command)
        .into_iter()
        .map(|expression| effective_key_for_expression(expression, context, registry))
        .collect::<Vec<_>>();
    if command_type_is_nominal_without_arguments(&shape.signature, &actuals, registry) {
        return;
    }
    check_command_requirements(
        &shape.signature,
        &actuals,
        context,
        path,
        position,
        registry,
        event_log,
    );
}

fn check_command_predicate(
    command: &CommandExpression,
    context: &TypeContext,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    registry: &SignatureRegistry,
    event_log: &mut EventLog,
) {
    check_command_expression(command, context, path, locator, registry, event_log);
}

fn check_infix_command(
    left: &Expression,
    command: &InfixCommand,
    right: &Expression,
    context: &TypeContext,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    registry: &SignatureRegistry,
    event_log: &mut EventLog,
) {
    check_inactive_expression_tail(&command.tail, context, path, locator, registry, event_log);
    let active_command = active_infix_command(command, context);
    let shape = shape_for_infix_command(&active_command);
    let position = locator.locate_reference(&shape);
    let mut actuals = Vec::new();
    actuals.push(effective_key_for_expression(left, context, registry));
    actuals.extend(
        infix_command_arguments(&active_command)
            .into_iter()
            .map(|expression| effective_key_for_expression(expression, context, registry)),
    );
    actuals.push(effective_key_for_expression(right, context, registry));
    check_command_requirements(
        &shape.signature,
        &actuals,
        context,
        path,
        position,
        registry,
        event_log,
    );
}

fn check_refined_command_type_expression(
    command: &RefinedCommandExpression,
    context: &TypeContext,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    registry: &SignatureRegistry,
    event_log: &mut EventLog,
) {
    check_inactive_refined_command_expression_arguments(
        command, context, path, locator, registry, event_log,
    );
    let active_command = active_refined_command_expression(command, context);
    let shape = shape_for_refined_command_expression(&active_command);
    let position = locator.locate_reference(&shape);
    let actuals = refined_command_expression_arguments(&active_command)
        .into_iter()
        .map(|expression| effective_key_for_expression(expression, context, registry))
        .collect::<Vec<_>>();
    if command_type_is_nominal_without_arguments(&shape.signature, &actuals, registry) {
        return;
    }
    check_command_requirements(
        &shape.signature,
        &actuals,
        context,
        path,
        position,
        registry,
        event_log,
    );
}

fn check_refined_command_expression(
    command: &RefinedCommandExpression,
    context: &TypeContext,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    registry: &SignatureRegistry,
    event_log: &mut EventLog,
) {
    check_inactive_refined_command_expression_arguments(
        command, context, path, locator, registry, event_log,
    );
    let active_command = active_refined_command_expression(command, context);
    let shape = shape_for_refined_command_expression(&active_command);
    let position = locator.locate_reference(&shape);
    let actuals = refined_command_expression_arguments(&active_command)
        .into_iter()
        .map(|expression| effective_key_for_expression(expression, context, registry))
        .collect::<Vec<_>>();
    check_command_requirements(
        &shape.signature,
        &actuals,
        context,
        path,
        position,
        registry,
        event_log,
    );
}

fn active_command_expression(
    command: &CommandExpression,
    context: &TypeContext,
) -> CommandExpression {
    let mut active = command.clone();
    active.tail = active_expression_tail(&command.tail, context);
    active
}

fn active_infix_command(command: &InfixCommand, context: &TypeContext) -> InfixCommand {
    let mut active = command.clone();
    active.tail = active_expression_tail(&command.tail, context);
    active
}

fn active_infix_spec(spec: &InfixSpec, context: &TypeContext) -> InfixSpec {
    let mut active = spec.clone();
    active.tail = active_expression_tail(&spec.tail, context);
    active
}

fn active_refined_command_expression(
    command: &RefinedCommandExpression,
    context: &TypeContext,
) -> RefinedCommandExpression {
    let mut active = command.clone();
    active.tail = active_expression_tail(&command.tail, context);
    active.parts = command
        .parts
        .iter()
        .cloned()
        .map(|mut part| {
            part.tail = active_expression_tail(&part.tail, context);
            part
        })
        .collect();
    active
}

fn check_inactive_refined_command_expression_arguments(
    command: &RefinedCommandExpression,
    context: &TypeContext,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    registry: &SignatureRegistry,
    event_log: &mut EventLog,
) {
    for part in &command.parts {
        check_inactive_expression_tail(&part.tail, context, path, locator, registry, event_log);
    }
    check_inactive_expression_tail(&command.tail, context, path, locator, registry, event_log);
}

fn check_inactive_expression_tail(
    tail: &[CommandExpressionTailPart],
    context: &TypeContext,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    registry: &SignatureRegistry,
    event_log: &mut EventLog,
) {
    for part in tail
        .iter()
        .filter(|part| part.optional && !expression_tail_part_is_active(part, context))
    {
        for expression in part.args.iter().flat_map(|args| args.expressions.iter()) {
            check_expression(expression, context, path, locator, registry, event_log);
        }
    }
}

fn active_expression_tail(
    tail: &[CommandExpressionTailPart],
    context: &TypeContext,
) -> Vec<CommandExpressionTailPart> {
    tail.iter()
        .filter(|part| expression_tail_part_is_active(part, context))
        .cloned()
        .collect()
}

fn expression_tail_part_is_active(part: &CommandExpressionTailPart, context: &TypeContext) -> bool {
    !part.optional
        || part
            .args
            .iter()
            .flat_map(|args| args.expressions.iter())
            .all(|expression| expression_names_are_defined(expression, context))
}

fn expression_names_are_defined(expression: &Expression, context: &TypeContext) -> bool {
    let mut names = Vec::new();
    collect_defined_expression_names(expression, &mut names);
    names
        .iter()
        .all(|name| is_literal_name(name) || context.has_name(name))
}

fn collect_defined_expression_names(expression: &Expression, names: &mut Vec<String>) {
    match &expression.kind {
        ExpressionKind::Name(name) => names.push(name.clone()),
        ExpressionKind::FunctionCall { name, arguments } => {
            names.push(name.clone());
            for argument in arguments {
                collect_defined_expression_names(argument, names);
            }
        }
        ExpressionKind::FunctionNamedCall { name, elements } => {
            names.push(name.clone());
            for element in elements {
                match &element.lhs {
                    FunctionNamedExpressionElementLhs::Name(name) => names.push(name.clone()),
                    FunctionNamedExpressionElementLhs::SubsetCall(subset) => {
                        collect_defined_subset_call_names(subset, names);
                    }
                }
                collect_defined_expression_names(&element.expression, names);
            }
        }
        ExpressionKind::MemberCall {
            owner, arguments, ..
        } => {
            collect_defined_expression_names(owner, names);
            for argument in arguments {
                collect_defined_expression_names(argument, names);
            }
        }
        ExpressionKind::MemberAccess { owner, .. } => {
            collect_defined_expression_names(owner, names)
        }
        ExpressionKind::Tuple(elements) => {
            for element in elements {
                if let TupleExpressionElement::Expression(expression) = element {
                    collect_defined_expression_names(expression, names);
                }
            }
        }
        ExpressionKind::Set(set) => {
            collect_defined_set_target_names(&set.target, names);
            for spec in &set.specs {
                collect_defined_expression_names(spec, names);
            }
            if let Some(predicate) = &set.predicate {
                collect_defined_expression_names(predicate, names);
            }
        }
        ExpressionKind::Grouped { expression, .. }
        | ExpressionKind::Labeled { expression, .. }
        | ExpressionKind::Prefix { expression, .. }
        | ExpressionKind::Postfix { expression, .. } => {
            collect_defined_expression_names(expression, names)
        }
        ExpressionKind::SubsetCall(subset) => collect_defined_subset_call_names(subset, names),
        ExpressionKind::Command(command) => {
            for expression in command_expression_arguments(command) {
                collect_defined_expression_names(expression, names);
            }
        }
        ExpressionKind::InfixCommand {
            left,
            command,
            right,
        } => {
            collect_defined_expression_names(left, names);
            for expression in infix_command_arguments(command) {
                collect_defined_expression_names(expression, names);
            }
            collect_defined_expression_names(right, names);
        }
        ExpressionKind::InfixSpecStatement { left, spec, right } => {
            collect_defined_expression_names(left, names);
            for expression in infix_spec_arguments(spec) {
                collect_defined_expression_names(expression, names);
            }
            collect_defined_expression_names(right, names);
        }
        ExpressionKind::Binary { left, right, .. } => {
            collect_defined_expression_names(left, names);
            collect_defined_expression_names(right, names);
        }
        ExpressionKind::SpecStatement(statement) | ExpressionKind::SpecPredicate(statement) => {
            collect_defined_expression_names(&statement.subject, names);
            names.push(statement.name.clone());
        }
        ExpressionKind::MemberOf {
            subject,
            collection,
        } => {
            collect_defined_expression_names(subject, names);
            collect_defined_expression_names(collection, names);
        }
        ExpressionKind::IsPredicate { subject, command }
        | ExpressionKind::IsNotPredicate { subject, command } => {
            collect_defined_expression_names(subject, names);
            for expression in command_expression_arguments(command) {
                collect_defined_expression_names(expression, names);
            }
        }
        ExpressionKind::IsBuiltinPredicate { subject, ty }
        | ExpressionKind::IsNotBuiltinPredicate { subject, ty } => {
            collect_defined_expression_names(subject, names);
            collect_defined_type_expression_names(ty, names);
        }
        ExpressionKind::IsRefinedPredicate { subject, command }
        | ExpressionKind::IsNotRefinedPredicate { subject, command } => {
            collect_defined_expression_names(subject, names);
            for expression in refined_command_expression_arguments(command) {
                collect_defined_expression_names(expression, names);
            }
        }
        ExpressionKind::IsType { subject, ty } => {
            collect_defined_expression_names(subject, names);
            collect_defined_type_expression_names(ty, names);
        }
    }
}

fn collect_defined_type_expression_names(ty: &TypeExpression, names: &mut Vec<String>) {
    match ty {
        TypeExpression::Builtin { .. } => {}
        TypeExpression::Command(command) => {
            for expression in command_expression_arguments(command) {
                collect_defined_expression_names(expression, names);
            }
        }
        TypeExpression::RefinedCommand(command) => {
            for expression in refined_command_expression_arguments(command) {
                collect_defined_expression_names(expression, names);
            }
        }
        TypeExpression::Function(function_type) => {
            for spec in function_type
                .inputs
                .iter()
                .chain(std::iter::once(&function_type.output))
            {
                collect_defined_function_type_spec_names(spec, names);
            }
        }
        TypeExpression::Coercion { ty, literal, .. } => {
            collect_defined_type_expression_names(ty, names);
            collect_defined_set_target_names(&literal.target, names);
            for spec in &literal.specs {
                collect_defined_expression_names(spec, names);
            }
            if let Some(predicate) = &literal.predicate {
                collect_defined_expression_names(predicate, names);
            }
        }
    }
}

fn collect_defined_function_type_spec_names(spec: &FunctionTypeSpec, names: &mut Vec<String>) {
    match &spec.kind {
        FunctionTypeSpecKind::Is(ty) => collect_defined_type_expression_names(ty, names),
        FunctionTypeSpecKind::Spec { target, .. } => names.push(target.clone()),
    }
}

fn collect_defined_set_target_names(target: &SetTarget, names: &mut Vec<String>) {
    match &target.kind {
        SetTargetKind::Name(name) => names.push(name.clone()),
        SetTargetKind::PlaceholderForm(form) => collect_defined_placeholder_form_names(form, names),
        SetTargetKind::Alias { name, target } => {
            names.push(name.clone());
            collect_defined_set_target_names(target, names);
        }
        SetTargetKind::Function {
            name: function_name,
            arguments,
        } => {
            names.push(function_name.clone());
            for argument in arguments {
                collect_defined_set_target_names(argument, names);
            }
        }
        SetTargetKind::Tuple(elements) => {
            for element in elements {
                if let SetTargetElement::Target(target) = element {
                    collect_defined_set_target_names(target, names);
                }
            }
        }
    }
}

fn collect_defined_placeholder_form_names(form: &PlaceholderForm, names: &mut Vec<String>) {
    match &form.kind {
        PlaceholderFormKind::Placeholder(placeholder) => names.push(placeholder.name.clone()),
        PlaceholderFormKind::Function {
            placeholder,
            arguments,
        } => {
            names.push(placeholder.name.clone());
            names.extend(arguments.iter().map(|argument| argument.name.clone()));
        }
    }
}

fn collect_defined_subset_call_names(subset: &SubsetCall, names: &mut Vec<String>) {
    match subset {
        SubsetCall::One { target, first, .. } => {
            names.push(target.clone());
            names.push(first.clone());
        }
        SubsetCall::Two {
            target,
            first,
            second,
            ..
        } => {
            names.push(target.clone());
            names.push(first.clone());
            names.push(second.clone());
        }
        SubsetCall::Nested {
            target,
            outer,
            inner_target,
            ..
        } => {
            names.push(target.clone());
            names.push(outer.clone());
            names.push(inner_target.clone());
        }
    }
}

fn command_type_is_nominal_without_arguments(
    signature: &str,
    actuals: &[String],
    registry: &SignatureRegistry,
) -> bool {
    actuals.is_empty()
        && registry
            .definitions
            .get(signature)
            .is_some_and(|definition| definition.kind == DefinitionKind::Describes)
}

fn validate_optional_enables(
    enables: &Option<EnablesSection>,
    context: &TypeContext,
    owner_shapes: &[HeaderShape],
    owner_subject: &str,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    registry: &SignatureRegistry,
    event_log: &mut EventLog,
) {
    let Some(enables) = enables else {
        return;
    };

    for item in &enables.arguments {
        match item {
            EnablesItem::Capability(group) => validate_capability_alias(
                &group.capability.argument,
                context,
                owner_shapes,
                owner_subject,
                path,
                locator,
                registry,
                event_log,
            ),
            EnablesItem::FromCapability(group) => {
                let child = context_with_from_declaration(
                    &group.from.argument,
                    context,
                    path,
                    locator,
                    registry,
                    event_log,
                );
                validate_capability_alias(
                    &group.capability.argument,
                    &child,
                    owner_shapes,
                    owner_subject,
                    path,
                    locator,
                    registry,
                    event_log,
                );
            }
            EnablesItem::FromAs(group) => {
                let child = context_with_from_declaration(
                    &group.from.argument,
                    context,
                    path,
                    locator,
                    registry,
                    event_log,
                );
                check_expression(
                    &group.as_.argument.left,
                    &child,
                    path,
                    locator,
                    registry,
                    event_log,
                );
                check_expression(
                    &group.as_.argument.right,
                    &child,
                    path,
                    locator,
                    registry,
                    event_log,
                );
            }
            EnablesItem::Viewable(group) => {
                validate_viewable_group(
                    group,
                    context,
                    owner_shapes,
                    owner_subject,
                    path,
                    locator,
                    registry,
                    event_log,
                );
            }
            EnablesItem::Connection(_) => {}
        }
    }
}

fn validate_viewable_group(
    group: &ViewableGroup,
    context: &TypeContext,
    owner_shapes: &[HeaderShape],
    owner_subject: &str,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    registry: &SignatureRegistry,
    event_log: &mut EventLog,
) {
    let mut child = context.clone();
    for owner_shape in owner_shapes {
        child.add_fact(TypeFact::Is {
            subject: owner_subject.to_owned(),
            ty: owner_shape.type_key.clone(),
            signature: owner_shape.shape.signature.clone(),
        });
    }
    declare_declaration_statement_subjects(&group.as_.argument, &mut child);
    complete_introduced_declaration_statement(
        &group.as_.argument,
        &mut child,
        path,
        locator,
        registry,
        event_log,
    );
    if !matches!(
        group.as_.argument.relation,
        Some(DeclarationRelation::Is(_))
    ) {
        emit_error(
            event_log,
            path,
            locator.locate_symbol(&primary_subject_key(&group.as_.argument.subject)),
            "`viewable:` `as:` must specify the target type using `is`",
        );
    }
    if let Some(states) = &group.states {
        check_clause(&states.argument, &child, path, locator, registry, event_log);
    }
}

fn context_with_from_declaration(
    statement: &DeclarationStatement,
    context: &TypeContext,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    registry: &SignatureRegistry,
    event_log: &mut EventLog,
) -> TypeContext {
    let mut child = context.clone();
    declare_declaration_statement_subjects(statement, &mut child);
    complete_introduced_declaration_statement(
        statement, &mut child, path, locator, registry, event_log,
    );
    declare_declaration_collection_literal_target(statement, &mut child);
    child
}

fn declare_declaration_collection_literal_target(
    statement: &DeclarationStatement,
    context: &mut TypeContext,
) {
    if let Some(Expression {
        kind: ExpressionKind::Set(set),
        ..
    }) = &statement.definition
    {
        declare_set_target(&set.target, context);
    }
    if let Some(DeclarationRelation::Is(TypeExpression::Coercion { literal, .. })) =
        &statement.relation
    {
        declare_set_target(&literal.target, context);
    }
}

fn validate_optional_requires(
    requires: &Option<RequiresSection>,
    context: &TypeContext,
    owner_shapes: Option<&[HeaderShape]>,
    owner_subject: Option<&str>,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    registry: &SignatureRegistry,
    event_log: &mut EventLog,
) {
    let Some(requires) = requires else {
        return;
    };

    for item in &requires.arguments {
        match item {
            RequiresItem::Capability(group) => {
                if let (Some(owner_shapes), Some(owner_subject)) = (owner_shapes, owner_subject) {
                    validate_capability_alias(
                        &group.capability.argument,
                        context,
                        owner_shapes,
                        owner_subject,
                        path,
                        locator,
                        registry,
                        event_log,
                    );
                }
            }
            RequiresItem::Definition(group) => validate_definition_requirement(
                &group.definition.argument,
                context,
                path,
                locator,
                registry,
                event_log,
            ),
        }
    }
}

fn validate_capability_alias(
    capability: &AliasKind,
    context: &TypeContext,
    owner_shapes: &[HeaderShape],
    owner_subject: &str,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    registry: &SignatureRegistry,
    event_log: &mut EventLog,
) {
    match capability {
        AliasKind::SpecOperator(alias) => validate_spec_operator_alias(
            alias,
            context,
            owner_subject,
            path,
            locator,
            registry,
            event_log,
        ),
        AliasKind::Expression(alias) => validate_provided_expression_alias(
            alias,
            context,
            owner_shapes,
            path,
            locator,
            registry,
            event_log,
        ),
    }
}

fn validate_definition_requirement(
    requirement: &DefinitionRequirement,
    context: &TypeContext,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    registry: &SignatureRegistry,
    event_log: &mut EventLog,
) {
    check_inactive_expression_tail(
        &requirement.command.tail,
        context,
        path,
        locator,
        registry,
        event_log,
    );
    let active_command = active_command_expression(&requirement.command, context);
    let shape = shape_for_command_expression(&active_command);
    let position = locator.locate_reference(&shape);
    let actuals = command_expression_arguments(&active_command)
        .into_iter()
        .map(|expression| {
            check_expression(expression, context, path, locator, registry, event_log);
            effective_key_for_expression(expression, context, registry)
        })
        .collect::<Vec<_>>();
    check_command_requirements(
        &shape.signature,
        &actuals,
        context,
        path,
        position,
        registry,
        event_log,
    );
    check_type_expression(&requirement.ty, context, path, locator, registry, event_log);

    if !signature_has_kind(&shape.signature, DefinitionKind::Defines, registry) {
        emit_error(
            event_log,
            path,
            position,
            format!(
                "Required definition `{}` must reference a `Defines:` entry",
                key_for_command_expression(&active_command)
            ),
        );
        return;
    }

    let subject = key_for_command_expression(&active_command);
    let Some(required) = fact_from_type_key_assertion(subject.clone(), &requirement.ty, context)
    else {
        return;
    };
    let required = context.normalize_fact(&required);
    let established = defined_output_facts_for_key(&subject, context, registry)
        .iter()
        .any(|fact| {
            let mut seen = HashSet::new();
            fact_implies(fact, &required, context, registry, &mut seen)
        });
    if established {
        return;
    }

    emit_error(
        event_log,
        path,
        position,
        format!(
            "Required definition `{}` does not establish `{}`",
            subject,
            format_fact(&required)
        ),
    );
}

fn validate_spec_operator_alias(
    alias: &SpecOperatorAlias,
    context: &TypeContext,
    owner_subject: &str,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    registry: &SignatureRegistry,
    event_log: &mut EventLog,
) {
    check_name(
        &alias.placeholder_spec.name,
        context,
        path,
        locator,
        event_log,
    );
    if context.normalize_key(&alias.placeholder_spec.name) != context.normalize_key(owner_subject) {
        emit_error(
            event_log,
            path,
            locator.locate_symbol(&alias.placeholder_spec.name),
            format!(
                "Provided spec operator target `{}` must be the described item `{}`",
                alias.placeholder_spec.name, owner_subject
            ),
        );
    }

    let mut child = context.clone();
    declare_placeholder_form(&alias.placeholder_spec.placeholder_form, &mut child);

    match &alias.target {
        SpecOperatorAliasTarget::IsOrSpec(target) => {
            check_is_or_spec_alias_target(target, &child, path, locator, registry, event_log);
        }
        SpecOperatorAliasTarget::MemberOf(expression) => {
            check_expression(expression, &child, path, locator, registry, event_log);
        }
        SpecOperatorAliasTarget::Builtin(_) => {}
    }
}

fn validate_provided_expression_alias(
    alias: &ExpressionAlias,
    context: &TypeContext,
    owner_shapes: &[HeaderShape],
    path: &Path,
    locator: &mut SourceLocator<'_>,
    registry: &SignatureRegistry,
    event_log: &mut EventLog,
) {
    let mut child = context.clone();
    declare_expression_alias_lhs(&alias.lhs, &mut child);
    assume_provided_expression_alias_lhs_owner_types(&alias.lhs, owner_shapes, &mut child);
    check_expression(
        &alias.expression,
        &child,
        path,
        locator,
        registry,
        event_log,
    );
}

fn declare_expression_alias_lhs(lhs: &ExpressionAliasLhs, context: &mut TypeContext) {
    match lhs {
        ExpressionAliasLhs::Form(form) => declare_form_or_declaration(form, context),
        ExpressionAliasLhs::Command(command) => {
            for form in command_header_forms(command) {
                declare_form_or_declaration(form, context);
            }
        }
        ExpressionAliasLhs::InfixCommand(command) => {
            for form in infix_header_forms(command) {
                declare_form_or_declaration(form, context);
            }
        }
    }
}

fn assume_provided_expression_alias_lhs_owner_types(
    lhs: &ExpressionAliasLhs,
    owner_shapes: &[HeaderShape],
    context: &mut TypeContext,
) {
    let ExpressionAliasLhs::Form(form) = lhs else {
        return;
    };
    match &form.kind {
        FormOrDeclarationKind::InfixOperator { left, right, .. } => {
            assume_owner_type(&left.name, owner_shapes, context);
            assume_owner_type(&right.name, owner_shapes, context);
        }
        FormOrDeclarationKind::PrefixOperator { placeholder, .. }
        | FormOrDeclarationKind::PostfixOperator { placeholder, .. } => {
            assume_owner_type(&placeholder.name, owner_shapes, context);
        }
        FormOrDeclarationKind::Name(_)
        | FormOrDeclarationKind::FunctionDeclaration { .. }
        | FormOrDeclarationKind::TupleDeclaration { .. }
        | FormOrDeclarationKind::SetDeclaration { .. } => {}
    }
}

fn assume_owner_type(subject: &str, owner_shapes: &[HeaderShape], context: &mut TypeContext) {
    for owner_shape in owner_shapes {
        context.add_fact(TypeFact::Is {
            subject: subject.to_owned(),
            ty: owner_shape.type_key.clone(),
            signature: owner_shape.shape.signature.clone(),
        });
    }
}

fn check_is_or_spec_alias_target(
    target: &IsOrSpec,
    context: &TypeContext,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    registry: &SignatureRegistry,
    event_log: &mut EventLog,
) {
    match target {
        IsOrSpec::Is(statement) => {
            check_is_subject(&statement.subject, context, path, locator, event_log);
            check_type_expression_requirements(
                &statement.ty,
                context,
                path,
                locator,
                registry,
                event_log,
            );
        }
        IsOrSpec::Spec(statement) => {
            check_spec_subject(&statement.subject, context, path, locator, event_log);
            check_name(&statement.name, context, path, locator, event_log);
            for fact in facts_from_is_or_spec(target) {
                check_spec_fact_supported(
                    &fact,
                    context,
                    path,
                    locator.locate_symbol(&statement.name),
                    registry,
                    event_log,
                );
            }
        }
    }
}

fn check_type_expression_requirements(
    ty: &TypeExpression,
    context: &TypeContext,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    registry: &SignatureRegistry,
    event_log: &mut EventLog,
) {
    match ty {
        TypeExpression::Builtin { .. } => {}
        TypeExpression::Command(command) => {
            check_command_expression(command, context, path, locator, registry, event_log);
            let active_command = active_command_expression(command, context);
            for expression in command_expression_arguments(&active_command) {
                check_expression(expression, context, path, locator, registry, event_log);
            }
        }
        TypeExpression::RefinedCommand(command) => {
            check_refined_command_expression(command, context, path, locator, registry, event_log);
            let active_command = active_refined_command_expression(command, context);
            for expression in refined_command_expression_arguments(&active_command) {
                check_expression(expression, context, path, locator, registry, event_log);
            }
        }
        TypeExpression::Function(function_type) => {
            for spec in function_type
                .inputs
                .iter()
                .chain(std::iter::once(&function_type.output))
            {
                check_function_type_spec(spec, context, path, locator, registry, event_log);
            }
        }
        TypeExpression::Coercion { ty, literal, .. } => {
            check_type_expression_requirements(ty, context, path, locator, registry, event_log);
            check_set_literal(literal, context, path, locator, registry, event_log);
        }
    }
}

fn check_command_requirements(
    signature: &str,
    actuals: &[String],
    context: &TypeContext,
    path: &Path,
    position: Option<SourcePosition>,
    registry: &SignatureRegistry,
    event_log: &mut EventLog,
) {
    let Some(info) = registry.type_infos.get(signature) else {
        return;
    };

    let substitutions = info
        .parameters
        .iter()
        .zip(actuals)
        .map(|(name, actual)| (name.clone(), context.normalize_key(actual)))
        .collect::<HashMap<_, _>>();

    let mut requirement_context = context.clone();
    for (left, right) in &info.substitutions {
        if info
            .hidden_parameters
            .iter()
            .any(|name| key_mentions_name(left, name) || key_mentions_name(right, name))
        {
            continue;
        }
        requirement_context.add_substitution(
            substitute_key(left, &substitutions),
            substitute_key(right, &substitutions),
        );
    }

    for requirement in &info.requirements {
        if info
            .hidden_parameters
            .iter()
            .any(|name| fact_mentions_name(requirement, name))
        {
            continue;
        }
        let instantiated = substitute_fact(requirement, &substitutions);
        if !prove_fact(&instantiated, &requirement_context, registry) {
            emit_error(
                event_log,
                path,
                position,
                format!(
                    "Could not establish requirement `{}` for command `{signature}`",
                    format_fact(&instantiated)
                ),
            );
        }
    }
}

fn fact_mentions_name(fact: &TypeFact, name: &str) -> bool {
    match fact {
        TypeFact::Is { subject, ty, .. } => {
            key_mentions_name(subject, name) || key_mentions_name(ty, name)
        }
        TypeFact::Spec {
            subject, target, ..
        } => key_mentions_name(subject, name) || key_mentions_name(target, name),
        TypeFact::InfixSpec {
            subject,
            args,
            target,
            ..
        } => {
            key_mentions_name(subject, name)
                || args.iter().any(|arg| key_mentions_name(arg, name))
                || key_mentions_name(target, name)
        }
        TypeFact::RefinedIs {
            subject,
            ty,
            base_ty,
            ..
        } => {
            key_mentions_name(subject, name)
                || key_mentions_name(ty, name)
                || key_mentions_name(base_ty, name)
        }
        TypeFact::MemberOf {
            subject,
            collection,
        } => key_mentions_name(subject, name) || key_mentions_name(collection, name),
        TypeFact::FunctionType {
            subject,
            inputs,
            output,
            ..
        } => {
            key_mentions_name(subject, name)
                || inputs
                    .iter()
                    .any(|spec| function_type_spec_mentions_name(spec, name))
                || function_type_spec_mentions_name(output, name)
        }
    }
}

fn function_type_spec_mentions_name(spec: &FunctionTypeFactSpec, name: &str) -> bool {
    match spec {
        FunctionTypeFactSpec::Is { ty, .. } => key_mentions_name(ty, name),
        FunctionTypeFactSpec::Spec { target, .. } => key_mentions_name(target, name),
    }
}

fn prove_fact(required: &TypeFact, context: &TypeContext, registry: &SignatureRegistry) -> bool {
    prove_fact_with_options(required, context, registry, true)
}

fn prove_fact_without_viewable(
    required: &TypeFact,
    context: &TypeContext,
    registry: &SignatureRegistry,
) -> bool {
    prove_fact_with_options(required, context, registry, false)
}

fn prove_fact_with_options(
    required: &TypeFact,
    context: &TypeContext,
    registry: &SignatureRegistry,
    allow_viewable: bool,
) -> bool {
    let required = context.normalize_fact(required);
    if builtin_fact_holds(&required, registry) {
        return true;
    }
    if context
        .facts
        .iter()
        .any(|fact| context.normalize_fact(fact) == required)
    {
        return true;
    }

    let mut seen = HashSet::new();
    if defined_output_facts_for_key(fact_subject(&required), context, registry)
        .iter()
        .any(|fact| {
            fact_implies_with_options(
                fact,
                &required,
                context,
                registry,
                &mut seen,
                allow_viewable,
            )
        })
    {
        return true;
    }

    context.facts.iter().any(|fact| {
        fact_implies_with_options(
            fact,
            &required,
            context,
            registry,
            &mut seen,
            allow_viewable,
        )
    })
}

fn builtin_fact_holds(required: &TypeFact, registry: &SignatureRegistry) -> bool {
    let TypeFact::Is {
        subject, signature, ..
    } = required
    else {
        return false;
    };

    match signature.as_str() {
        BUILTIN_OPAQUE_SIGNATURE => true,
        BUILTIN_EXPRESSION_SIGNATURE => true,
        BUILTIN_STATEMENT_SIGNATURE => key_is_statement(subject, registry),
        BUILTIN_SPECIFICATION_SIGNATURE => key_is_specification(subject),
        BUILTIN_TYPE_SIGNATURE => key_is_type(subject, registry),
        _ => false,
    }
}

fn key_is_type(key: &str, registry: &SignatureRegistry) -> bool {
    command_signature_from_key(key)
        .as_deref()
        .is_some_and(|signature| signature_has_kind(signature, DefinitionKind::Describes, registry))
        || infix_command_signatures_from_key(key)
            .iter()
            .any(|signature| signature_has_kind(signature, DefinitionKind::Describes, registry))
}

fn key_is_statement(key: &str, registry: &SignatureRegistry) -> bool {
    key_contains_top_level(key, " is? ")
        || key_contains_top_level(key, " is_not? ")
        || key_contains_top_level(key, " = ")
        || key_contains_top_level(key, " != ")
        || key_contains_top_level_quoted_spec(key, true)
        || key_contains_top_level_infix_spec(key, true)
        || key_is_states_command_reference(key, registry)
}

fn key_is_specification(key: &str) -> bool {
    key_contains_top_level(key, " is ")
        || key_contains_top_level_quoted_spec(key, false)
        || key_contains_top_level_infix_spec(key, false)
}

fn key_contains_top_level(key: &str, pattern: &str) -> bool {
    let mut index = 0;
    let mut paren_depth = 0usize;
    let mut brace_depth = 0usize;
    let mut bracket_depth = 0usize;

    while index < key.len() {
        let rest = &key[index..];
        if rest.starts_with(pattern) && paren_depth == 0 && brace_depth == 0 && bracket_depth == 0 {
            return true;
        }

        let Some(ch) = rest.chars().next() else {
            return false;
        };
        match ch {
            '(' => paren_depth += 1,
            ')' => paren_depth = paren_depth.saturating_sub(1),
            '{' => brace_depth += 1,
            '}' => brace_depth = brace_depth.saturating_sub(1),
            '[' => bracket_depth += 1,
            ']' => bracket_depth = bracket_depth.saturating_sub(1),
            _ => {}
        }
        index += ch.len_utf8();
    }

    false
}

fn key_contains_top_level_quoted_spec(key: &str, predicate: bool) -> bool {
    let mut search_start = 0;
    while search_start < key.len() {
        let Some(relative_start) = key[search_start..].find('"') else {
            return false;
        };
        let start = search_start + relative_start;
        if !key_is_top_level_at(key, start) {
            search_start = start + '"'.len_utf8();
            continue;
        }

        let after_open = start + '"'.len_utf8();
        let Some(relative_end) = key[after_open..].find('"') else {
            return false;
        };
        let after_close = after_open + relative_end + '"'.len_utf8();
        if key[after_close..].starts_with('?') == predicate {
            return true;
        }

        search_start = after_close;
    }

    false
}

fn key_contains_top_level_infix_spec(key: &str, predicate: bool) -> bool {
    let mut search_start = 0;
    while search_start < key.len() {
        let Some(relative_start) = key[search_start..].find("\\:") else {
            return false;
        };
        let start = search_start + relative_start;
        if !key_is_top_level_at(key, start) {
            search_start = start + "\\:".len();
            continue;
        }

        let Some((end, is_predicate)) = find_infix_spec_key_end(key, start) else {
            return false;
        };
        if is_predicate == predicate {
            return true;
        }

        search_start = end;
    }

    false
}

fn key_is_top_level_at(key: &str, target: usize) -> bool {
    let mut index = 0;
    let mut paren_depth = 0usize;
    let mut brace_depth = 0usize;
    let mut bracket_depth = 0usize;

    while index < target {
        let Some(ch) = key[index..].chars().next() else {
            return false;
        };
        match ch {
            '(' => paren_depth += 1,
            ')' => paren_depth = paren_depth.saturating_sub(1),
            '{' => brace_depth += 1,
            '}' => brace_depth = brace_depth.saturating_sub(1),
            '[' => bracket_depth += 1,
            ']' => bracket_depth = bracket_depth.saturating_sub(1),
            _ => {}
        }
        index += ch.len_utf8();
    }

    paren_depth == 0 && brace_depth == 0 && bracket_depth == 0
}

fn find_infix_spec_key_end(key: &str, start: usize) -> Option<(usize, bool)> {
    let mut index = start + "\\:".len();
    let mut paren_depth = 0usize;
    let mut brace_depth = 0usize;
    let mut bracket_depth = 0usize;

    while index < key.len() {
        let rest = &key[index..];
        if paren_depth == 0 && brace_depth == 0 && bracket_depth == 0 {
            if rest.starts_with("?:/") {
                return Some((index + "?:/".len(), true));
            }
            if rest.starts_with(":/") {
                return Some((index + ":/".len(), false));
            }
        }

        let ch = rest.chars().next()?;
        match ch {
            '(' => paren_depth += 1,
            ')' => paren_depth = paren_depth.saturating_sub(1),
            '{' => brace_depth += 1,
            '}' => brace_depth = brace_depth.saturating_sub(1),
            '[' => bracket_depth += 1,
            ']' => bracket_depth = bracket_depth.saturating_sub(1),
            _ => {}
        }
        index += ch.len_utf8();
    }

    None
}

fn key_is_states_command_reference(key: &str, registry: &SignatureRegistry) -> bool {
    command_signature_from_key(key)
        .as_deref()
        .is_some_and(|signature| signature_has_kind(signature, DefinitionKind::States, registry))
        || infix_command_signatures_from_key(key)
            .iter()
            .any(|signature| signature_has_kind(signature, DefinitionKind::States, registry))
}

fn signature_has_kind(signature: &str, kind: DefinitionKind, registry: &SignatureRegistry) -> bool {
    registry
        .definitions
        .get(signature)
        .is_some_and(|definition| definition.kind == kind)
}

fn command_signature_from_key(key: &str) -> Option<String> {
    if !key.starts_with('\\') || key.starts_with("\\.") || key.starts_with("\\:") {
        return None;
    }

    let mut signature = String::new();
    let mut index = 0;
    while index < key.len() {
        let rest = &key[index..];
        if rest.starts_with('{') {
            index += find_balanced_group_end(rest, '{', '}')?;
            continue;
        }
        if rest.starts_with('(') {
            index += find_balanced_group_end(rest, '(', ')')?;
            continue;
        }

        let ch = rest.chars().next()?;
        signature.push(ch);
        index += ch.len_utf8();
    }

    Some(signature)
}

fn infix_command_signatures_from_key(key: &str) -> Vec<String> {
    let mut signatures = Vec::new();
    let mut search_start = 0;

    while search_start < key.len() {
        let Some(relative_start) = key[search_start..].find("\\.") else {
            break;
        };
        let start = search_start + relative_start;
        let Some(end) = find_infix_command_key_end(key, start) else {
            break;
        };
        if let Some(signature) = infix_command_signature_from_key_segment(&key[start..end]) {
            signatures.push(signature);
        }
        search_start = end;
    }

    signatures
}

fn find_infix_command_key_end(key: &str, start: usize) -> Option<usize> {
    let mut index = start + "\\.".len();
    let mut paren_depth = 0usize;
    let mut brace_depth = 0usize;
    let mut bracket_depth = 0usize;

    while index < key.len() {
        let rest = &key[index..];
        if rest.starts_with("./") && paren_depth == 0 && brace_depth == 0 && bracket_depth == 0 {
            return Some(index + "./".len());
        }

        let ch = rest.chars().next()?;
        match ch {
            '(' => paren_depth += 1,
            ')' => paren_depth = paren_depth.saturating_sub(1),
            '{' => brace_depth += 1,
            '}' => brace_depth = brace_depth.saturating_sub(1),
            '[' => bracket_depth += 1,
            ']' => bracket_depth = bracket_depth.saturating_sub(1),
            _ => {}
        }
        index += ch.len_utf8();
    }

    None
}

fn infix_command_signature_from_key_segment(segment: &str) -> Option<String> {
    let body = segment.strip_prefix("\\.")?.strip_suffix("./")?;
    let mut signature = "\\.".to_owned();
    let mut index = 0;

    while index < body.len() {
        let rest = &body[index..];
        if rest.starts_with('{') {
            index += find_balanced_group_end(rest, '{', '}')?;
            continue;
        }

        let ch = rest.chars().next()?;
        signature.push(ch);
        index += ch.len_utf8();
    }

    signature.push_str("./");
    Some(signature)
}

fn fact_implies(
    fact: &TypeFact,
    required: &TypeFact,
    context: &TypeContext,
    registry: &SignatureRegistry,
    seen: &mut HashSet<TypeFact>,
) -> bool {
    fact_implies_with_options(fact, required, context, registry, seen, true)
}

fn fact_implies_with_options(
    fact: &TypeFact,
    required: &TypeFact,
    context: &TypeContext,
    registry: &SignatureRegistry,
    seen: &mut HashSet<TypeFact>,
    allow_viewable: bool,
) -> bool {
    let fact = context.normalize_fact(fact);
    if &fact == required {
        return true;
    }
    if !seen.insert(fact.clone()) {
        return false;
    }

    if function_type_implies_required(&fact, required, context, registry, seen, allow_viewable) {
        return true;
    }

    if cast_as_fact_implies_required(&fact, required, context, registry, seen, allow_viewable) {
        return true;
    }

    if allow_viewable && viewable_fact_implies_required(&fact, required, context, registry, seen) {
        return true;
    }

    for extended in reduce_extension_fact(&fact, context, registry) {
        if fact_implies_with_options(&extended, required, context, registry, seen, allow_viewable) {
            return true;
        }
    }

    for reduced in reduce_refined_fact_with_options(&fact, context, registry, allow_viewable) {
        if fact_implies_with_options(&reduced, required, context, registry, seen, allow_viewable) {
            return true;
        }
    }

    if matches!(fact, TypeFact::Spec { .. } | TypeFact::MemberOf { .. }) {
        let mut spec_seen = HashSet::new();
        for reduced in reduce_spec_or_member_fact(&fact, context, registry, &mut spec_seen) {
            if fact_implies_with_options(
                &reduced,
                required,
                context,
                registry,
                seen,
                allow_viewable,
            ) {
                return true;
            }
        }
    }

    false
}

fn viewable_fact_implies_required(
    fact: &TypeFact,
    required: &TypeFact,
    context: &TypeContext,
    registry: &SignatureRegistry,
    seen: &mut HashSet<TypeFact>,
) -> bool {
    let fact = context.normalize_fact(fact);
    let TypeFact::Is {
        subject,
        ty,
        signature,
    } = &fact
    else {
        return false;
    };
    let actuals = actuals_for_type_key(signature, ty).unwrap_or_default();

    registry
        .viewable_rules
        .iter()
        .filter(|rule| rule.source_signature == *signature)
        .any(|rule| {
            let mut substitutions = rule
                .parameters
                .iter()
                .zip(&actuals)
                .map(|(name, actual)| (name.clone(), context.normalize_key(actual)))
                .collect::<HashMap<_, _>>();
            substitutions.insert(rule.source_subject.clone(), subject.clone());
            substitutions.insert(rule.target_subject.clone(), subject.clone());
            let viewed = context.normalize_fact(&substitute_fact(&rule.target, &substitutions));
            fact_implies(&viewed, required, context, registry, seen)
        })
}

fn cast_as_fact_implies_required(
    fact: &TypeFact,
    required: &TypeFact,
    context: &TypeContext,
    registry: &SignatureRegistry,
    seen: &mut HashSet<TypeFact>,
    allow_viewable: bool,
) -> bool {
    let TypeFact::Is {
        subject, signature, ..
    } = fact
    else {
        return false;
    };
    let subject = context.normalize_key(subject);
    let Some(literal) = context.collection_literal(&subject) else {
        return false;
    };

    for rule in registry
        .cast_as_rules
        .iter()
        .filter(|rule| rule.owner_signature == *signature)
    {
        let mut substitutions = HashMap::from([
            (rule.owner_subject.clone(), subject.clone()),
            (rule.source_subject.clone(), subject.clone()),
        ]);
        let required_subject = context.normalize_key(fact_subject(required));
        if !bind_cast_expression_to_key(&rule.left, &required_subject, &mut substitutions, context)
        {
            continue;
        }
        if !bind_cast_expression_to_key(&rule.right, &required_subject, &mut substitutions, context)
        {
            continue;
        }

        for fact in facts_from_collection_literal_cast(literal, &substitutions, context) {
            if fact_implies_with_options(&fact, required, context, registry, seen, allow_viewable) {
                return true;
            }
        }
    }

    false
}

fn bind_cast_expression_to_key(
    pattern: &Expression,
    actual: &str,
    substitutions: &mut HashMap<String, String>,
    context: &TypeContext,
) -> bool {
    match &pattern.kind {
        ExpressionKind::Name(name) => bind_cast_name_to_key(name, actual, substitutions, context),
        ExpressionKind::FunctionCall { name, arguments } => {
            let Some((actual_name, actual_arguments)) = function_call_parts_from_key(actual) else {
                return false;
            };
            let pattern_name = context.normalize_key(&substitute_key(name, substitutions));
            if context.normalize_key(&actual_name) != pattern_name {
                return false;
            }
            if arguments.len() != actual_arguments.len() {
                return false;
            }
            arguments
                .iter()
                .zip(actual_arguments)
                .all(|(argument, actual)| {
                    bind_cast_expression_to_key(argument, &actual, substitutions, context)
                })
        }
        ExpressionKind::Tuple(elements) => {
            let Some(actual_arguments) = tuple_arguments_from_key(actual) else {
                return false;
            };
            if elements.len() != actual_arguments.len() {
                return false;
            }
            elements
                .iter()
                .zip(actual_arguments)
                .all(|(element, actual)| {
                    let TupleExpressionElement::Expression(expression) = element else {
                        return false;
                    };
                    bind_cast_expression_to_key(expression, &actual, substitutions, context)
                })
        }
        ExpressionKind::Grouped { expression, .. } | ExpressionKind::Labeled { expression, .. } => {
            bind_cast_expression_to_key(expression, actual, substitutions, context)
        }
        _ => {
            context.normalize_key(&substitute_key(&key_for_expression(pattern), substitutions))
                == context.normalize_key(actual)
        }
    }
}

fn bind_cast_name_to_key(
    name: &str,
    actual: &str,
    substitutions: &mut HashMap<String, String>,
    context: &TypeContext,
) -> bool {
    if let Some(bound) = substitutions.get(name) {
        return context.normalize_key(bound) == context.normalize_key(actual);
    }

    substitutions.insert(name.to_owned(), actual.to_owned());
    true
}

fn tuple_arguments_from_key(key: &str) -> Option<Vec<String>> {
    let key = key.trim();
    let inner = key.strip_prefix('(')?.strip_suffix(')')?;
    Some(split_key_arg_list(inner))
}

fn facts_from_collection_literal_cast(
    literal: &SetExpression,
    substitutions: &HashMap<String, String>,
    context: &TypeContext,
) -> Vec<TypeFact> {
    let mut child = context.clone();
    declare_set_target(&literal.target, &mut child);

    let mut result = Vec::new();
    for spec in &literal.specs {
        let Some(fact) = fact_from_expression_in_context(spec, &child) else {
            continue;
        };
        child.add_fact(fact.clone());
        result.push(child.normalize_fact(&substitute_fact(&fact, substitutions)));
    }
    result
}

fn reduce_extension_fact(
    fact: &TypeFact,
    context: &TypeContext,
    registry: &SignatureRegistry,
) -> Vec<TypeFact> {
    let (subject, signature, actuals) = match fact {
        TypeFact::Is {
            subject,
            ty,
            signature,
        } => (
            subject.clone(),
            signature.clone(),
            actuals_for_type_key(signature, ty).unwrap_or_default(),
        ),
        TypeFact::InfixSpec {
            subject,
            signature,
            args,
            target,
        } => {
            let mut actuals = Vec::with_capacity(args.len() + 2);
            actuals.push(subject.clone());
            actuals.extend(args.iter().cloned());
            actuals.push(target.clone());
            (subject.clone(), signature.clone(), actuals)
        }
        _ => return Vec::new(),
    };
    registry
        .extension_rules
        .iter()
        .filter(|rule| rule.subtype_signature == signature.as_str())
        .map(|rule| {
            let mut substitutions = rule
                .parameters
                .iter()
                .zip(&actuals)
                .map(|(name, actual)| (name.clone(), context.normalize_key(actual)))
                .collect::<HashMap<_, _>>();
            substitutions.insert(rule.subject.clone(), subject.clone());
            context.normalize_fact(&substitute_fact(&rule.target, &substitutions))
        })
        .collect()
}

fn reduce_refined_fact(
    fact: &TypeFact,
    context: &TypeContext,
    registry: &SignatureRegistry,
) -> Vec<TypeFact> {
    reduce_refined_fact_with_options(fact, context, registry, true)
}

fn reduce_refined_fact_with_options(
    fact: &TypeFact,
    context: &TypeContext,
    registry: &SignatureRegistry,
    allow_viewable: bool,
) -> Vec<TypeFact> {
    let fact = context.normalize_fact(fact);
    let TypeFact::RefinedIs {
        subject,
        ty,
        signature,
        base_ty,
        base_signature,
    } = &fact
    else {
        return Vec::new();
    };

    let mut result = vec![TypeFact::Is {
        subject: subject.clone(),
        ty: base_ty.clone(),
        signature: base_signature.clone(),
    }];

    result.extend(refined_part_facts(
        subject,
        &ty,
        &signature,
        base_ty,
        base_signature,
    ));
    result.extend(reduce_refinement_extension_fact(
        subject,
        &ty,
        &signature,
        base_ty,
        base_signature,
        context,
        registry,
        allow_viewable,
    ));
    result
}

fn refined_part_facts(
    subject: &str,
    ty: &str,
    signature: &str,
    base_ty: &str,
    base_signature: &str,
) -> Vec<TypeFact> {
    let (Some(signature_segments), Some(ty_segments)) =
        (split_refined_key(signature), split_refined_key(ty))
    else {
        return Vec::new();
    };
    if signature_segments.len() != ty_segments.len() || signature_segments.len() < 3 {
        return Vec::new();
    }

    let mut result = Vec::new();
    let last = signature_segments.len() - 1;
    for part_index in 0..last {
        let candidate_signature = format!(
            "\\{}::{}",
            signature_segments[part_index], signature_segments[last]
        );
        if candidate_signature == signature {
            continue;
        }

        result.push(TypeFact::RefinedIs {
            subject: subject.to_owned(),
            ty: format!("\\{}::{}", ty_segments[part_index], ty_segments[last]),
            signature: candidate_signature,
            base_ty: base_ty.to_owned(),
            base_signature: base_signature.to_owned(),
        });
    }
    result
}

fn reduce_refinement_extension_fact(
    subject: &str,
    ty: &str,
    signature: &str,
    base_ty: &str,
    base_signature: &str,
    context: &TypeContext,
    registry: &SignatureRegistry,
    allow_viewable: bool,
) -> Vec<TypeFact> {
    let actuals = actuals_for_refined_type_key(signature, ty).unwrap_or_default();
    registry
        .refinement_extension_rules
        .iter()
        .filter(|rule| {
            refinement_extension_rule_matches(
                signature,
                base_ty,
                base_signature,
                rule,
                context,
                registry,
                allow_viewable,
            )
        })
        .map(|rule| {
            let mut substitutions = rule
                .parameters
                .iter()
                .zip(&actuals)
                .map(|(name, actual)| (name.clone(), context.normalize_key(actual)))
                .collect::<HashMap<_, _>>();
            substitutions.insert(rule.subject.clone(), subject.to_owned());
            refinement_extension_target_fact(
                &rule.target,
                base_ty,
                base_signature,
                &substitutions,
                context,
            )
        })
        .collect()
}

fn refinement_extension_rule_matches(
    signature: &str,
    base_ty: &str,
    base_signature: &str,
    rule: &RefinementExtensionRule,
    context: &TypeContext,
    registry: &SignatureRegistry,
    allow_viewable: bool,
) -> bool {
    if rule.subtype_signature == signature {
        return true;
    }

    let (Some(fact_segments), Some(rule_segments)) = (
        split_refined_key(signature),
        split_refined_key(&rule.subtype_signature),
    ) else {
        return false;
    };
    if fact_segments.len() < 2 || rule_segments.len() < 2 {
        return false;
    }
    if fact_segments[..fact_segments.len() - 1] != rule_segments[..rule_segments.len() - 1] {
        return false;
    }

    let rule_base_signature = format!("\\{}", rule_segments.last().unwrap());
    if base_signature == rule_base_signature {
        return true;
    }

    let base_fact = TypeFact::Is {
        subject: "#".to_owned(),
        ty: base_ty.to_owned(),
        signature: base_signature.to_owned(),
    };
    let required = TypeFact::Is {
        subject: "#".to_owned(),
        ty: rule_base_signature.clone(),
        signature: rule_base_signature,
    };
    let mut seen = HashSet::new();
    fact_implies_with_options(
        &base_fact,
        &required,
        context,
        registry,
        &mut seen,
        allow_viewable,
    )
}

fn refinement_extension_target_fact(
    target: &RefinementExtensionTarget,
    base_ty: &str,
    base_signature: &str,
    substitutions: &HashMap<String, String>,
    context: &TypeContext,
) -> TypeFact {
    match target {
        RefinementExtensionTarget::Fact(fact) => {
            context.normalize_fact(&substitute_fact(fact, substitutions))
        }
        RefinementExtensionTarget::DynamicRefinedIs { subject, command } => {
            let subject = substitute_key(subject, substitutions);
            TypeFact::RefinedIs {
                subject: context.normalize_key(&subject),
                ty: context.normalize_key(&substitute_key(
                    &key_for_refined_command_with_tail(command, base_ty.trim_start_matches('\\')),
                    substitutions,
                )),
                signature: refined_command_signature_with_tail(
                    command,
                    base_signature.trim_start_matches('\\'),
                ),
                base_ty: context.normalize_key(base_ty),
                base_signature: base_signature.to_owned(),
            }
        }
    }
}

fn reduce_spec_fact(
    fact: &TypeFact,
    context: &TypeContext,
    registry: &SignatureRegistry,
    seen: &mut HashSet<TypeFact>,
) -> Vec<TypeFact> {
    let fact = context.normalize_fact(fact);
    if !seen.insert(fact.clone()) {
        return Vec::new();
    }

    let TypeFact::Spec {
        subject,
        operator,
        target,
    } = &fact
    else {
        return Vec::new();
    };

    let mut result = Vec::new();
    for rule in &registry.spec_rules {
        if &rule.operator != operator {
            continue;
        }

        if !has_type_signature(target, &rule.owner_signature, context, registry) {
            continue;
        }

        if rule.source_requires_literal && context.collection_literal(target).is_none() {
            continue;
        }

        let mut substitutions = HashMap::from([
            (rule.placeholder.clone(), subject.clone()),
            (rule.target.clone(), target.clone()),
        ]);
        if let Some(source_subject) = &rule.source_subject {
            substitutions.insert(source_subject.clone(), target.clone());
        }

        match &rule.target_alias {
            SpecOperatorAliasTarget::Builtin(_) => {}
            SpecOperatorAliasTarget::IsOrSpec(target_alias) => {
                for next in facts_from_is_or_spec(target_alias) {
                    let next = substitute_fact(&next, &substitutions);
                    let next = context.normalize_fact(&next);
                    result.push(next.clone());
                    if matches!(next, TypeFact::Spec { .. } | TypeFact::MemberOf { .. }) {
                        result.extend(reduce_spec_or_member_fact(&next, context, registry, seen));
                    }
                }
            }
            SpecOperatorAliasTarget::MemberOf(target_alias) => {
                if rule.source_subject.is_none() {
                    continue;
                }
                if let Some(next) = fact_from_expression(target_alias) {
                    let next = substitute_fact(&next, &substitutions);
                    let next = context.normalize_fact(&next);
                    result.push(next.clone());
                    if matches!(next, TypeFact::Spec { .. } | TypeFact::MemberOf { .. }) {
                        result.extend(reduce_spec_or_member_fact(&next, context, registry, seen));
                    }
                }
            }
        }
    }

    result
}

fn reduce_spec_or_member_fact(
    fact: &TypeFact,
    context: &TypeContext,
    registry: &SignatureRegistry,
    seen: &mut HashSet<TypeFact>,
) -> Vec<TypeFact> {
    match fact {
        TypeFact::Spec { .. } => reduce_spec_fact(fact, context, registry, seen),
        TypeFact::MemberOf { .. } => reduce_member_of_fact(fact, context, registry, seen),
        _ => Vec::new(),
    }
}

fn reduce_member_of_fact(
    fact: &TypeFact,
    context: &TypeContext,
    registry: &SignatureRegistry,
    seen: &mut HashSet<TypeFact>,
) -> Vec<TypeFact> {
    let fact = context.normalize_fact(fact);
    if !seen.insert(fact.clone()) {
        return Vec::new();
    }

    let TypeFact::MemberOf {
        subject,
        collection,
    } = &fact
    else {
        return Vec::new();
    };

    if let Some(literal) = context.collection_literal(collection) {
        let facts = facts_from_collection_literal_membership(subject, literal, context);
        if !facts.is_empty() {
            return facts;
        }
    }

    if collection_has_registered_collection_type(collection, context, registry) {
        return vec![opaque_type_fact(subject)];
    }

    Vec::new()
}

fn facts_from_collection_literal_membership(
    subject: &str,
    literal: &SetExpression,
    context: &TypeContext,
) -> Vec<TypeFact> {
    let Some(pattern) = collection_literal_member_pattern(&literal.target) else {
        return Vec::new();
    };

    let mut child = context.clone();
    declare_set_target(&literal.target, &mut child);

    let substitutions = HashMap::from([(pattern.clone(), subject.to_owned())]);
    let mut result = Vec::new();
    for spec in &literal.specs {
        let Some(fact) = fact_from_expression_in_context(spec, &child) else {
            continue;
        };
        child.add_fact(fact.clone());
        if child.normalize_key(fact_subject(&fact)) == child.normalize_key(&pattern) {
            result.push(child.normalize_fact(&substitute_fact(&fact, &substitutions)));
        }
    }
    result
}

fn collection_literal_member_pattern(target: &SetTarget) -> Option<String> {
    match &target.kind {
        SetTargetKind::Name(name) => Some(name.clone()),
        SetTargetKind::PlaceholderForm(form) => Some(key_for_placeholder_form(form)),
        SetTargetKind::Alias { name, .. } => Some(name.clone()),
        SetTargetKind::Function { .. } | SetTargetKind::Tuple(_) => {
            Some(key_for_set_target(target))
        }
    }
}

fn collection_has_registered_collection_type(
    collection: &str,
    context: &TypeContext,
    registry: &SignatureRegistry,
) -> bool {
    registry
        .collection_type_signatures
        .iter()
        .any(|signature| has_type_signature(collection, signature, context, registry))
}

fn opaque_type_fact(subject: &str) -> TypeFact {
    TypeFact::Is {
        subject: subject.to_owned(),
        ty: BUILTIN_OPAQUE_SIGNATURE.to_owned(),
        signature: BUILTIN_OPAQUE_SIGNATURE.to_owned(),
    }
}

fn defined_output_facts_for_key(
    key: &str,
    context: &TypeContext,
    registry: &SignatureRegistry,
) -> Vec<TypeFact> {
    let key = context.normalize_key(key);
    let mut result = Vec::new();

    if let Some((signature, actuals)) = command_signature_and_actuals_from_key(&key) {
        result.extend(defined_output_facts_for_signature(
            &signature, &actuals, &key, context, registry,
        ));
    }

    if let Some((signature, actuals)) = infix_command_signature_and_actuals_from_key(&key) {
        result.extend(defined_output_facts_for_signature(
            &signature, &actuals, &key, context, registry,
        ));
    }

    result
}

fn defined_output_facts_for_signature(
    signature: &str,
    actuals: &[String],
    key: &str,
    context: &TypeContext,
    registry: &SignatureRegistry,
) -> Vec<TypeFact> {
    let Some(info) = registry.type_infos.get(signature) else {
        return Vec::new();
    };
    if info.outputs.is_empty() {
        return Vec::new();
    }

    let mut base_substitutions = info
        .parameters
        .iter()
        .zip(actuals)
        .map(|(name, actual)| (name.clone(), context.normalize_key(actual)))
        .collect::<HashMap<_, _>>();
    for (index, name) in info.hidden_parameters.iter().enumerate() {
        base_substitutions.insert(name.clone(), "#".repeat(index + 1));
    }

    let mut output_context = context.clone();
    for (left, right) in &info.substitutions {
        output_context.add_substitution(
            substitute_key(left, &base_substitutions),
            substitute_key(right, &base_substitutions),
        );
    }

    info.outputs
        .iter()
        .map(|output| {
            let mut substitutions = base_substitutions.clone();
            substitutions.insert(fact_subject(output).to_owned(), key.to_owned());
            output_context.normalize_fact(&substitute_fact(output, &substitutions))
        })
        .collect()
}

fn command_signature_and_actuals_from_key(key: &str) -> Option<(String, Vec<String>)> {
    let signature = command_signature_from_key(key)?;
    let actuals = actuals_for_type_key(&signature, key)?;
    Some((signature, actuals))
}

fn infix_command_signature_and_actuals_from_key(key: &str) -> Option<(String, Vec<String>)> {
    let mut search_start = 0;

    while search_start < key.len() {
        let relative_start = key[search_start..].find("\\.")?;
        let start = search_start + relative_start;
        if !key_is_top_level_at(key, start) {
            search_start = start + "\\.".len();
            continue;
        }

        let end = find_infix_command_key_end(key, start)?;
        let left = key[..start].trim();
        let right = key[end..].trim();
        if left.is_empty() || right.is_empty() {
            return None;
        }

        let segment = &key[start..end];
        let signature = infix_command_signature_from_key_segment(segment)?;
        let mut actuals = Vec::new();
        actuals.push(left.to_owned());
        actuals.extend(actuals_for_infix_command_key_segment(&signature, segment)?);
        actuals.push(right.to_owned());
        return Some((signature, actuals));
    }

    None
}

fn actuals_for_infix_command_key_segment(signature: &str, segment: &str) -> Option<Vec<String>> {
    let signature_body = signature.strip_prefix("\\.")?.strip_suffix("./")?;
    let mut rest = segment.strip_prefix("\\.")?.strip_suffix("./")?;
    let mut actuals = Vec::new();
    let parts = signature_body.split(':').collect::<Vec<_>>();
    let first = parts.first()?;

    rest = rest.strip_prefix(first)?;
    collect_adjacent_key_args(&mut rest, &mut actuals)?;

    for part in parts.iter().skip(1) {
        rest = rest.strip_prefix(':')?;
        rest = rest.strip_prefix(part)?;
        collect_adjacent_key_args(&mut rest, &mut actuals)?;
    }

    rest.is_empty().then_some(actuals)
}

fn has_type_signature(
    subject: &str,
    signature: &str,
    context: &TypeContext,
    registry: &SignatureRegistry,
) -> bool {
    let subject = context.normalize_key(subject);
    let mut seen = HashSet::new();
    context.facts.iter().any(|fact| {
        fact_has_type_signature(fact, &subject, signature, context, registry, &mut seen)
    }) || defined_output_facts_for_key(&subject, context, registry)
        .iter()
        .any(|fact| {
            fact_has_type_signature(fact, &subject, signature, context, registry, &mut seen)
        })
}

fn fact_has_type_signature(
    fact: &TypeFact,
    subject: &str,
    signature: &str,
    context: &TypeContext,
    registry: &SignatureRegistry,
    seen: &mut HashSet<TypeFact>,
) -> bool {
    let fact = context.normalize_fact(fact);
    if !seen.insert(fact.clone()) {
        return false;
    }

    if matches!(
        &fact,
        TypeFact::Is {
            subject: fact_subject,
            signature: fact_signature,
            ..
        } if fact_subject == subject && fact_signature == signature
    ) {
        return true;
    }
    if matches!(
        &fact,
        TypeFact::RefinedIs {
            subject: fact_subject,
            signature: fact_signature,
            ..
        } if fact_subject == subject && fact_signature == signature
    ) {
        return true;
    }

    command_requirement_facts(&fact, context, registry)
        .iter()
        .any(|fact| fact_has_type_signature(fact, subject, signature, context, registry, seen))
        || reduce_extension_fact(&fact, context, registry)
            .iter()
            .any(|fact| fact_has_type_signature(fact, subject, signature, context, registry, seen))
        || reduce_refined_fact(&fact, context, registry)
            .iter()
            .any(|fact| fact_has_type_signature(fact, subject, signature, context, registry, seen))
}

fn command_requirement_facts(
    fact: &TypeFact,
    context: &TypeContext,
    registry: &SignatureRegistry,
) -> Vec<TypeFact> {
    let Some((signature, actuals)) = command_fact_signature_and_actuals(fact) else {
        return Vec::new();
    };
    let Some(info) = registry.type_infos.get(&signature) else {
        return Vec::new();
    };

    let mut substitutions = info
        .parameters
        .iter()
        .zip(actuals)
        .map(|(name, actual)| (name.clone(), context.normalize_key(&actual)))
        .collect::<HashMap<_, _>>();
    for (index, name) in info.hidden_parameters.iter().enumerate() {
        substitutions.insert(name.clone(), "#".repeat(index + 1));
    }

    let mut requirement_context = context.clone();
    for (left, right) in &info.substitutions {
        requirement_context.add_substitution(
            substitute_key(left, &substitutions),
            substitute_key(right, &substitutions),
        );
    }

    info.requirements
        .iter()
        .map(|requirement| {
            requirement_context.normalize_fact(&substitute_fact(requirement, &substitutions))
        })
        .collect()
}

fn command_fact_signature_and_actuals(fact: &TypeFact) -> Option<(String, Vec<String>)> {
    match fact {
        TypeFact::Is { ty, signature, .. } => {
            Some((signature.clone(), actuals_for_type_key(signature, ty)?))
        }
        TypeFact::RefinedIs { ty, signature, .. } => Some((
            signature.clone(),
            actuals_for_refined_type_key(signature, ty)?,
        )),
        TypeFact::InfixSpec {
            subject,
            signature,
            args,
            target,
        } => {
            let mut actuals = Vec::with_capacity(args.len() + 2);
            actuals.push(subject.clone());
            actuals.extend(args.iter().cloned());
            actuals.push(target.clone());
            Some((signature.clone(), actuals))
        }
        TypeFact::Spec { .. } | TypeFact::MemberOf { .. } | TypeFact::FunctionType { .. } => None,
    }
}

fn function_type_facts_for_subject(
    subject: &str,
    context: &TypeContext,
    registry: &SignatureRegistry,
) -> Vec<TypeFact> {
    let subject = context.normalize_key(subject);
    let mut seen = HashSet::new();
    let mut result = Vec::new();

    for fact in &context.facts {
        collect_function_type_facts(fact, &subject, context, registry, &mut seen, &mut result);
    }

    result
}

fn collect_function_type_facts(
    fact: &TypeFact,
    subject: &str,
    context: &TypeContext,
    registry: &SignatureRegistry,
    seen: &mut HashSet<TypeFact>,
    result: &mut Vec<TypeFact>,
) {
    let fact = context.normalize_fact(fact);
    if !seen.insert(fact.clone()) {
        return;
    }

    if matches!(
        &fact,
        TypeFact::FunctionType {
            subject: fact_subject,
            ..
        } if fact_subject == subject
    ) {
        result.push(fact.clone());
    }

    for extended in reduce_extension_fact(&fact, context, registry) {
        collect_function_type_facts(&extended, subject, context, registry, seen, result);
    }
}

fn function_type_implies_required(
    fact: &TypeFact,
    required: &TypeFact,
    context: &TypeContext,
    registry: &SignatureRegistry,
    seen: &mut HashSet<TypeFact>,
    allow_viewable: bool,
) -> bool {
    let TypeFact::FunctionType {
        subject,
        inputs,
        output,
        variadic_tuple_input,
    } = fact
    else {
        return false;
    };

    let Some((function_name, arguments)) = function_call_parts_from_fact(required) else {
        return false;
    };
    if context.normalize_key(&function_name) != context.normalize_key(subject) {
        return false;
    }

    let Some(argument_subjects) =
        function_type_argument_subjects_from_keys(inputs.len(), *variadic_tuple_input, &arguments)
    else {
        return false;
    };

    for (input, argument) in inputs.iter().zip(argument_subjects) {
        let required_input = instantiate_function_type_spec(input, &argument);
        if !prove_fact_with_options(&required_input, context, registry, allow_viewable) {
            return false;
        }
    }

    let output_subject = fact_subject(required);
    let output_fact = instantiate_function_type_spec(output, output_subject);
    fact_implies_with_options(
        &output_fact,
        required,
        context,
        registry,
        seen,
        allow_viewable,
    )
}

fn function_call_parts_from_fact(fact: &TypeFact) -> Option<(String, Vec<String>)> {
    function_call_parts_from_key(fact_subject(fact))
}

fn fact_subject(fact: &TypeFact) -> &str {
    match fact {
        TypeFact::Is { subject, .. }
        | TypeFact::Spec { subject, .. }
        | TypeFact::InfixSpec { subject, .. }
        | TypeFact::RefinedIs { subject, .. }
        | TypeFact::MemberOf { subject, .. }
        | TypeFact::FunctionType { subject, .. } => subject,
    }
}

fn function_call_parts_from_key(key: &str) -> Option<(String, Vec<String>)> {
    let open_index = key.find('(')?;
    let name = key[..open_index].trim();
    if name.is_empty() {
        return None;
    }

    let rest = &key[open_index..];
    let end = find_balanced_group_end(rest, '(', ')')?;
    if end != rest.len() {
        return None;
    }

    let inside = &rest['('.len_utf8()..end - ')'.len_utf8()];
    Some((name.to_owned(), split_key_arg_list(inside)))
}

fn function_type_matches_call_arity(fact: &TypeFact, arity: usize) -> bool {
    match fact {
        TypeFact::FunctionType {
            inputs,
            variadic_tuple_input,
            ..
        } => {
            if *variadic_tuple_input {
                arity > 0 && inputs.len() == 1
            } else {
                inputs.len() == arity
            }
        }
        _ => false,
    }
}

fn function_type_argument_subjects_from_expressions(
    input_count: usize,
    variadic_tuple_input: bool,
    arguments: &[Expression],
    context: &TypeContext,
    registry: &SignatureRegistry,
) -> Option<Vec<String>> {
    let argument_keys = arguments
        .iter()
        .map(|argument| effective_key_for_expression(argument, context, registry))
        .collect::<Vec<_>>();
    function_type_argument_subjects_from_keys(input_count, variadic_tuple_input, &argument_keys)
}

fn function_type_argument_subjects_from_keys(
    input_count: usize,
    variadic_tuple_input: bool,
    arguments: &[String],
) -> Option<Vec<String>> {
    if variadic_tuple_input {
        if input_count == 1 && !arguments.is_empty() {
            return Some(vec![tuple_key_for_function_arguments(arguments)]);
        }
        return None;
    }

    if input_count == arguments.len() {
        Some(arguments.to_vec())
    } else {
        None
    }
}

fn tuple_key_for_function_arguments(arguments: &[String]) -> String {
    if arguments.len() == 1 {
        arguments[0].clone()
    } else {
        format!("({})", arguments.join(","))
    }
}

fn function_call_arity(expression: &Expression) -> usize {
    match &expression.kind {
        ExpressionKind::FunctionCall { arguments, .. } => arguments.len(),
        ExpressionKind::MemberCall { arguments, .. } => arguments.len(),
        ExpressionKind::MemberAccess { .. } => 0,
        _ => 0,
    }
}

fn instantiate_function_type_spec(spec: &FunctionTypeFactSpec, subject: &str) -> TypeFact {
    match spec {
        FunctionTypeFactSpec::Is { ty, signature } => TypeFact::Is {
            subject: subject.to_owned(),
            ty: ty.clone(),
            signature: signature.clone(),
        },
        FunctionTypeFactSpec::Spec { operator, target } => TypeFact::Spec {
            subject: subject.to_owned(),
            operator: operator.clone(),
            target: target.clone(),
        },
    }
}

#[derive(Clone, Default)]
struct TypeContext {
    facts: Vec<TypeFact>,
    substitutions: Vec<(String, String)>,
    collection_literals: HashMap<String, SetExpression>,
    symbols: HashSet<String>,
    active_disambiguations: Vec<DisambiguationKey>,
    defer_unresolved_provided_symbols: bool,
}

impl TypeContext {
    fn add_fact(&mut self, fact: TypeFact) {
        self.facts.push(fact);
    }

    fn add_substitution(&mut self, left: String, right: String) {
        self.substitutions.push((left, right));
    }

    fn add_collection_literal(&mut self, subject: String, literal: SetExpression) {
        self.collection_literals
            .insert(subject.clone(), literal.clone());
        let normalized = self.normalize_key(&subject);
        self.collection_literals.insert(normalized, literal);
    }

    fn collection_literal(&self, subject: &str) -> Option<&SetExpression> {
        self.collection_literals
            .get(subject)
            .or_else(|| self.collection_literals.get(&self.normalize_key(subject)))
    }

    fn declare_name(&mut self, name: impl Into<String>) {
        self.symbols.insert(name.into());
    }

    fn has_name(&self, name: &str) -> bool {
        self.symbols.contains(name)
    }

    fn activate_disambiguation(&self, key: &DisambiguationKey) -> Option<Self> {
        if self.active_disambiguations.contains(key) {
            return None;
        }

        let mut child = self.clone();
        child.active_disambiguations.push(key.clone());
        Some(child)
    }

    fn normalize_fact(&self, fact: &TypeFact) -> TypeFact {
        match fact {
            TypeFact::Is {
                subject,
                ty,
                signature,
            } => TypeFact::Is {
                subject: self.normalize_key(subject),
                ty: self.normalize_key(ty),
                signature: signature.clone(),
            },
            TypeFact::Spec {
                subject,
                operator,
                target,
            } => TypeFact::Spec {
                subject: self.normalize_key(subject),
                operator: operator.clone(),
                target: self.normalize_key(target),
            },
            TypeFact::InfixSpec {
                subject,
                signature,
                args,
                target,
            } => TypeFact::InfixSpec {
                subject: self.normalize_key(subject),
                signature: signature.clone(),
                args: args.iter().map(|arg| self.normalize_key(arg)).collect(),
                target: self.normalize_key(target),
            },
            TypeFact::RefinedIs {
                subject,
                ty,
                signature,
                base_ty,
                base_signature,
            } => TypeFact::RefinedIs {
                subject: self.normalize_key(subject),
                ty: self.normalize_key(ty),
                signature: signature.clone(),
                base_ty: self.normalize_key(base_ty),
                base_signature: base_signature.clone(),
            },
            TypeFact::MemberOf {
                subject,
                collection,
            } => TypeFact::MemberOf {
                subject: self.normalize_key(subject),
                collection: self.normalize_key(collection),
            },
            TypeFact::FunctionType {
                subject,
                inputs,
                output,
                variadic_tuple_input,
            } => TypeFact::FunctionType {
                subject: self.normalize_key(subject),
                inputs: inputs
                    .iter()
                    .map(|spec| self.normalize_function_type_spec(spec))
                    .collect(),
                output: self.normalize_function_type_spec(output),
                variadic_tuple_input: *variadic_tuple_input,
            },
        }
    }

    fn normalize_function_type_spec(&self, spec: &FunctionTypeFactSpec) -> FunctionTypeFactSpec {
        match spec {
            FunctionTypeFactSpec::Is { ty, signature } => FunctionTypeFactSpec::Is {
                ty: self.normalize_key(ty),
                signature: signature.clone(),
            },
            FunctionTypeFactSpec::Spec { operator, target } => FunctionTypeFactSpec::Spec {
                operator: operator.clone(),
                target: self.normalize_key(target),
            },
        }
    }

    fn normalize_key(&self, key: &str) -> String {
        let mut map = HashMap::new();
        for (left, right) in &self.substitutions {
            let representative = left.min(right).clone();
            map.insert(left.clone(), representative.clone());
            map.insert(right.clone(), representative);
        }

        let mut result = key.to_owned();
        for _ in 0..self.substitutions.len().saturating_add(1) {
            let next = substitute_key(&result, &map);
            if next == result {
                break;
            }
            result = next;
        }
        result
    }
}

fn check_name(
    name: &str,
    context: &TypeContext,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    event_log: &mut EventLog,
) {
    if is_literal_name(name) || context.has_name(name) {
        return;
    }

    emit_error(
        event_log,
        path,
        locator.locate_symbol(name),
        format!("Unrecognized symbol `{name}`"),
    );
}

fn is_literal_name(name: &str) -> bool {
    name.chars().all(|ch| ch.is_ascii_digit())
}

fn declare_header_symbols(header: &CommandHeader, context: &mut TypeContext) {
    for form in header_forms(header) {
        declare_form_or_declaration(form, context);
    }
}

fn check_is_subject(
    subject: &IsSubject,
    context: &TypeContext,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    event_log: &mut EventLog,
) {
    match &subject.kind {
        IsSubjectKind::Forms(forms) => {
            for form in forms {
                match form {
                    IsSubjectForm::Form(form) => {
                        check_form_or_declaration(form, context, path, locator, event_log);
                    }
                    IsSubjectForm::PlaceholderForm(form) => {
                        check_placeholder_form(form, context, path, locator, event_log);
                    }
                }
            }
        }
        IsSubjectKind::Operator(_) => {}
    }
}

fn check_spec_subject(
    subject: &SpecSubject,
    context: &TypeContext,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    event_log: &mut EventLog,
) {
    match &subject.kind {
        SpecSubjectKind::Form(form) => {
            check_form_or_declaration(form, context, path, locator, event_log);
        }
        SpecSubjectKind::Operator(_) => {}
    }
}

fn check_form_or_declaration(
    form: &FormOrDeclaration,
    context: &TypeContext,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    event_log: &mut EventLog,
) {
    match &form.kind {
        FormOrDeclarationKind::Name(name) => {
            check_name(name, context, path, locator, event_log);
        }
        FormOrDeclarationKind::FunctionDeclaration { name, form } => {
            check_name(
                name.as_ref().unwrap_or(&form.name),
                context,
                path,
                locator,
                event_log,
            );
        }
        FormOrDeclarationKind::TupleDeclaration { name, form } => {
            if let Some(name) = name {
                check_name(name, context, path, locator, event_log);
            } else {
                for element in &form.elements {
                    if let TupleFormElement::Form(form) = element {
                        check_form_or_declaration(form, context, path, locator, event_log);
                    }
                }
            }
        }
        FormOrDeclarationKind::SetDeclaration { name, form } => {
            if let Some(name) = name {
                check_name(name, context, path, locator, event_log);
            } else {
                check_placeholder_form(&form.placeholder_form, context, path, locator, event_log);
            }
        }
        FormOrDeclarationKind::InfixOperator { .. }
        | FormOrDeclarationKind::PrefixOperator { .. }
        | FormOrDeclarationKind::PostfixOperator { .. } => {}
    }
}

fn check_placeholder_form(
    form: &PlaceholderForm,
    context: &TypeContext,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    event_log: &mut EventLog,
) {
    match &form.kind {
        PlaceholderFormKind::Placeholder(placeholder) => {
            check_name(&placeholder.name, context, path, locator, event_log);
        }
        PlaceholderFormKind::Function {
            placeholder,
            arguments,
        } => {
            check_name(&placeholder.name, context, path, locator, event_log);
            for argument in arguments {
                check_name(&argument.name, context, path, locator, event_log);
            }
        }
    }
}

fn check_subset_call(
    subset: &SubsetCall,
    context: &TypeContext,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    event_log: &mut EventLog,
) {
    match subset {
        SubsetCall::One { target, first, .. } => {
            check_name(target, context, path, locator, event_log);
            check_name(first, context, path, locator, event_log);
        }
        SubsetCall::Two {
            target,
            first,
            second,
            ..
        } => {
            check_name(target, context, path, locator, event_log);
            check_name(first, context, path, locator, event_log);
            check_name(second, context, path, locator, event_log);
        }
        SubsetCall::Nested {
            target,
            outer,
            inner_target,
            ..
        } => {
            check_name(target, context, path, locator, event_log);
            check_name(outer, context, path, locator, event_log);
            check_name(inner_target, context, path, locator, event_log);
        }
    }
}

fn assume_fact_expression(
    expression: &Expression,
    context: &mut TypeContext,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    registry: &SignatureRegistry,
    event_log: &mut EventLog,
) {
    match &expression.kind {
        ExpressionKind::IsType { subject, ty } => {
            check_type_expression(ty, context, path, locator, registry, event_log);
            declare_names_from_expression(subject, context);
            register_coerced_collection_literal(subject, ty, context);
        }
        ExpressionKind::SpecStatement(statement) => {
            check_name(&statement.name, context, path, locator, event_log);
            declare_names_from_expression(&statement.subject, context);
            if let Some(fact) = fact_from_expression_in_context(expression, context) {
                check_spec_fact_supported(
                    &fact,
                    context,
                    path,
                    locator.locate_symbol(&statement.name),
                    registry,
                    event_log,
                );
            }
        }
        ExpressionKind::MemberOf {
            subject,
            collection,
        } => {
            declare_names_from_expression(subject, context);
            check_expression(collection, context, path, locator, registry, event_log);
            register_expression_collection_literal(collection, context);
        }
        ExpressionKind::InfixSpecStatement { left, spec, right } if !spec.predicate => {
            check_inactive_expression_tail(&spec.tail, context, path, locator, registry, event_log);
            let active_spec = active_infix_spec(spec, context);
            for expression in infix_spec_arguments(&active_spec) {
                check_expression(expression, context, path, locator, registry, event_log);
            }
            check_expression(right, context, path, locator, registry, event_log);
            declare_names_from_expression(left, context);
            if let Some(fact) =
                fact_from_infix_spec_statement_in_context(left, spec, right, context)
            {
                check_spec_fact_supported(
                    &fact,
                    context,
                    path,
                    locator.locate_reference(&shape_for_infix_spec(&active_spec)),
                    registry,
                    event_log,
                );
            }
        }
        _ => check_expression(expression, context, path, locator, registry, event_log),
    }
}

fn declare_is_subject(subject: &IsSubject, context: &mut TypeContext) {
    match &subject.kind {
        IsSubjectKind::Forms(forms) => {
            for form in forms {
                match form {
                    IsSubjectForm::Form(form) => declare_form_or_declaration(form, context),
                    IsSubjectForm::PlaceholderForm(form) => declare_placeholder_form(form, context),
                }
            }
        }
        IsSubjectKind::Operator(_) => {}
    }
}

fn declare_form_or_declaration(form: &FormOrDeclaration, context: &mut TypeContext) {
    match &form.kind {
        FormOrDeclarationKind::Name(name) => context.declare_name(name.clone()),
        FormOrDeclarationKind::FunctionDeclaration { name, form } => {
            context.declare_name(name.as_ref().unwrap_or(&form.name).clone());
            if let Some(placeholder) = &form.magnetic_placeholder {
                context.declare_name(placeholder.name.clone());
            }
            for placeholder in &form.placeholders {
                context.declare_name(placeholder.name.clone());
            }
        }
        FormOrDeclarationKind::TupleDeclaration { name, form } => {
            if let Some(name) = name {
                context.declare_name(name.clone());
            }
            for element in &form.elements {
                if let TupleFormElement::Form(form) = element {
                    declare_form_or_declaration(form, context);
                }
            }
        }
        FormOrDeclarationKind::SetDeclaration { name, form } => {
            if let Some(name) = name {
                context.declare_name(name.clone());
            }
            declare_placeholder_form(&form.placeholder_form, context);
        }
        FormOrDeclarationKind::InfixOperator { left, right, .. } => {
            context.declare_name(left.name.clone());
            context.declare_name(right.name.clone());
        }
        FormOrDeclarationKind::PrefixOperator { placeholder, .. }
        | FormOrDeclarationKind::PostfixOperator { placeholder, .. } => {
            context.declare_name(placeholder.name.clone());
        }
    }
}

fn declare_placeholder_form(form: &PlaceholderForm, context: &mut TypeContext) {
    match &form.kind {
        PlaceholderFormKind::Placeholder(placeholder) => {
            context.declare_name(placeholder.name.clone());
        }
        PlaceholderFormKind::Function {
            placeholder,
            arguments,
        } => {
            context.declare_name(placeholder.name.clone());
            for argument in arguments {
                context.declare_name(argument.name.clone());
            }
        }
    }
}

fn declare_set_target(target: &SetTarget, context: &mut TypeContext) {
    match &target.kind {
        SetTargetKind::Name(name) => context.declare_name(name.clone()),
        SetTargetKind::PlaceholderForm(form) => declare_placeholder_form(form, context),
        SetTargetKind::Alias { name, target } => {
            context.declare_name(name.clone());
            declare_set_target(target, context);
        }
        SetTargetKind::Function { name, arguments } => {
            context.declare_name(name.clone());
            for argument in arguments {
                declare_set_target(argument, context);
            }
        }
        SetTargetKind::Tuple(elements) => {
            for element in elements {
                if let SetTargetElement::Target(target) = element {
                    declare_set_target(target, context);
                }
            }
        }
    }
}

fn declare_names_from_expression(expression: &Expression, context: &mut TypeContext) {
    match &expression.kind {
        ExpressionKind::Name(name) => context.declare_name(name.clone()),
        ExpressionKind::FunctionCall { name, arguments } => {
            context.declare_name(name.clone());
            for argument in arguments {
                declare_names_from_expression(argument, context);
            }
        }
        ExpressionKind::FunctionNamedCall { name, elements } => {
            context.declare_name(name.clone());
            for element in elements {
                match &element.lhs {
                    FunctionNamedExpressionElementLhs::Name(name) => {
                        context.declare_name(name.clone());
                    }
                    FunctionNamedExpressionElementLhs::SubsetCall(subset) => {
                        declare_subset_call_names(subset, context);
                    }
                }
                declare_names_from_expression(&element.expression, context);
            }
        }
        ExpressionKind::MemberCall {
            owner, arguments, ..
        } => {
            declare_names_from_expression(owner, context);
            for argument in arguments {
                declare_names_from_expression(argument, context);
            }
        }
        ExpressionKind::MemberAccess { owner, .. } => {
            declare_names_from_expression(owner, context);
        }
        ExpressionKind::Tuple(elements) => {
            for element in elements {
                if let TupleExpressionElement::Expression(expression) = element {
                    declare_names_from_expression(expression, context);
                }
            }
        }
        ExpressionKind::Set(set) => {
            declare_set_target(&set.target, context);
            for spec in &set.specs {
                declare_names_from_expression(spec, context);
            }
            if let Some(predicate) = &set.predicate {
                declare_names_from_expression(predicate, context);
            }
        }
        ExpressionKind::Grouped { expression, .. }
        | ExpressionKind::Labeled { expression, .. }
        | ExpressionKind::Prefix { expression, .. }
        | ExpressionKind::Postfix { expression, .. } => {
            declare_names_from_expression(expression, context);
        }
        ExpressionKind::SubsetCall(subset) => declare_subset_call_names(subset, context),
        ExpressionKind::Command(command) => {
            let active_command = active_command_expression(command, context);
            for expression in command_expression_arguments(&active_command) {
                declare_names_from_expression(expression, context);
            }
        }
        ExpressionKind::InfixCommand {
            left,
            command,
            right,
        } => {
            declare_names_from_expression(left, context);
            let active_command = active_infix_command(command, context);
            for expression in infix_command_arguments(&active_command) {
                declare_names_from_expression(expression, context);
            }
            declare_names_from_expression(right, context);
        }
        ExpressionKind::InfixSpecStatement { left, spec, right } => {
            declare_names_from_expression(left, context);
            let active_spec = active_infix_spec(spec, context);
            for expression in infix_spec_arguments(&active_spec) {
                declare_names_from_expression(expression, context);
            }
            declare_names_from_expression(right, context);
        }
        ExpressionKind::Binary { left, right, .. } => {
            declare_names_from_expression(left, context);
            declare_names_from_expression(right, context);
        }
        ExpressionKind::SpecStatement(statement) => {
            declare_names_from_expression(&statement.subject, context);
            context.declare_name(statement.name.clone());
        }
        ExpressionKind::SpecPredicate(statement) => {
            declare_names_from_expression(&statement.subject, context);
            context.declare_name(statement.name.clone());
        }
        ExpressionKind::MemberOf {
            subject,
            collection,
        } => {
            declare_names_from_expression(subject, context);
            declare_names_from_expression(collection, context);
        }
        ExpressionKind::IsPredicate { subject, command }
        | ExpressionKind::IsNotPredicate { subject, command } => {
            declare_names_from_expression(subject, context);
            let active_command = active_command_expression(command, context);
            for expression in command_expression_arguments(&active_command) {
                declare_names_from_expression(expression, context);
            }
        }
        ExpressionKind::IsBuiltinPredicate { subject, ty }
        | ExpressionKind::IsNotBuiltinPredicate { subject, ty } => {
            declare_names_from_expression(subject, context);
            declare_names_from_type_expression(ty, context);
        }
        ExpressionKind::IsRefinedPredicate { subject, command }
        | ExpressionKind::IsNotRefinedPredicate { subject, command } => {
            declare_names_from_expression(subject, context);
            let active_command = active_refined_command_expression(command, context);
            for expression in refined_command_expression_arguments(&active_command) {
                declare_names_from_expression(expression, context);
            }
        }
        ExpressionKind::IsType { subject, ty } => {
            declare_names_from_expression(subject, context);
            declare_names_from_type_expression(ty, context);
        }
    }
}

fn declare_names_from_type_expression(ty: &TypeExpression, context: &mut TypeContext) {
    match ty {
        TypeExpression::Builtin { .. } => {}
        TypeExpression::Command(command) => {
            let active_command = active_command_expression(command, context);
            for expression in command_expression_arguments(&active_command) {
                declare_names_from_expression(expression, context);
            }
        }
        TypeExpression::RefinedCommand(command) => {
            let active_command = active_refined_command_expression(command, context);
            for expression in refined_command_expression_arguments(&active_command) {
                declare_names_from_expression(expression, context);
            }
        }
        TypeExpression::Function(function_type) => {
            for spec in function_type
                .inputs
                .iter()
                .chain(std::iter::once(&function_type.output))
            {
                declare_names_from_function_type_spec(spec, context);
            }
        }
        TypeExpression::Coercion { ty, literal, .. } => {
            declare_names_from_type_expression(ty, context);
            declare_set_target(&literal.target, context);
            for spec in &literal.specs {
                declare_names_from_expression(spec, context);
            }
            if let Some(predicate) = &literal.predicate {
                declare_names_from_expression(predicate, context);
            }
        }
    }
}

fn declare_names_from_function_type_spec(spec: &FunctionTypeSpec, context: &mut TypeContext) {
    match &spec.kind {
        FunctionTypeSpecKind::Is(ty) => declare_names_from_type_expression(ty, context),
        FunctionTypeSpecKind::Spec { target, .. } => context.declare_name(target.clone()),
    }
}

fn declare_subset_call_names(subset: &SubsetCall, context: &mut TypeContext) {
    match subset {
        SubsetCall::One { target, first, .. } => {
            context.declare_name(target.clone());
            context.declare_name(first.clone());
        }
        SubsetCall::Two {
            target,
            first,
            second,
            ..
        } => {
            context.declare_name(target.clone());
            context.declare_name(first.clone());
            context.declare_name(second.clone());
        }
        SubsetCall::Nested {
            target,
            outer,
            inner_target,
            ..
        } => {
            context.declare_name(target.clone());
            context.declare_name(outer.clone());
            context.declare_name(inner_target.clone());
        }
    }
}

fn substitute_fact(fact: &TypeFact, substitutions: &HashMap<String, String>) -> TypeFact {
    match fact {
        TypeFact::Is {
            subject,
            ty,
            signature,
        } => TypeFact::Is {
            subject: substitute_key(subject, substitutions),
            ty: substitute_key(ty, substitutions),
            signature: signature.clone(),
        },
        TypeFact::Spec {
            subject,
            operator,
            target,
        } => TypeFact::Spec {
            subject: substitute_key(subject, substitutions),
            operator: operator.clone(),
            target: substitute_key(target, substitutions),
        },
        TypeFact::InfixSpec {
            subject,
            signature,
            args,
            target,
        } => TypeFact::InfixSpec {
            subject: substitute_key(subject, substitutions),
            signature: signature.clone(),
            args: args
                .iter()
                .map(|arg| substitute_key(arg, substitutions))
                .collect(),
            target: substitute_key(target, substitutions),
        },
        TypeFact::RefinedIs {
            subject,
            ty,
            signature,
            base_ty,
            base_signature,
        } => TypeFact::RefinedIs {
            subject: substitute_key(subject, substitutions),
            ty: substitute_key(ty, substitutions),
            signature: signature.clone(),
            base_ty: substitute_key(base_ty, substitutions),
            base_signature: base_signature.clone(),
        },
        TypeFact::MemberOf {
            subject,
            collection,
        } => TypeFact::MemberOf {
            subject: substitute_key(subject, substitutions),
            collection: substitute_key(collection, substitutions),
        },
        TypeFact::FunctionType {
            subject,
            inputs,
            output,
            variadic_tuple_input,
        } => TypeFact::FunctionType {
            subject: substitute_key(subject, substitutions),
            inputs: inputs
                .iter()
                .map(|spec| substitute_function_type_spec(spec, substitutions))
                .collect(),
            output: substitute_function_type_spec(output, substitutions),
            variadic_tuple_input: *variadic_tuple_input,
        },
    }
}

fn substitute_function_type_spec(
    spec: &FunctionTypeFactSpec,
    substitutions: &HashMap<String, String>,
) -> FunctionTypeFactSpec {
    match spec {
        FunctionTypeFactSpec::Is { ty, signature } => FunctionTypeFactSpec::Is {
            ty: substitute_key(ty, substitutions),
            signature: signature.clone(),
        },
        FunctionTypeFactSpec::Spec { operator, target } => FunctionTypeFactSpec::Spec {
            operator: operator.clone(),
            target: substitute_key(target, substitutions),
        },
    }
}

fn substitute_key(key: &str, substitutions: &HashMap<String, String>) -> String {
    if substitutions.is_empty() {
        return key.to_owned();
    }

    let mut result = String::new();
    let mut index = 0;
    while index < key.len() {
        let rest = &key[index..];
        let mut replacement = None;
        for (name, value) in substitutions {
            if rest.starts_with(name)
                && is_name_boundary(key, index, false)
                && is_name_boundary(key, index + name.len(), true)
            {
                replacement = Some((name.len(), value.as_str()));
                break;
            }
        }

        if let Some((len, value)) = replacement {
            result.push_str(value);
            index += len;
            continue;
        }

        let ch = rest.chars().next().expect("non-empty rest");
        result.push(ch);
        index += ch.len_utf8();
    }
    result
}

fn key_mentions_name(key: &str, name: &str) -> bool {
    if name.is_empty() {
        return false;
    }

    key.match_indices(name).any(|(index, _)| {
        is_name_boundary(key, index, false) && is_name_boundary(key, index + name.len(), true)
    })
}

fn is_name_boundary(text: &str, index: usize, after: bool) -> bool {
    if index == 0 || index == text.len() {
        return true;
    }
    let ch = if after {
        text[index..].chars().next()
    } else {
        text[..index].chars().next_back()
    };
    match ch {
        Some('\\') if !after => false,
        Some(ch) => !ch.is_ascii_alphanumeric() && ch != '_',
        None => true,
    }
}

fn facts_from_is_or_via_item(item: &IsOrViaItem) -> Vec<TypeFact> {
    match item {
        IsOrViaItem::IsVia(statement) => facts_from_is_statement(&statement.is_statement),
        IsOrViaItem::Declaration(statement) => facts_from_declaration_statement(statement),
    }
}

fn facts_from_is_or_via_item_in_context(
    item: &IsOrViaItem,
    context: &TypeContext,
) -> Vec<TypeFact> {
    match item {
        IsOrViaItem::IsVia(statement) => facts_from_is_statement(&statement.is_statement),
        IsOrViaItem::Declaration(statement) => {
            facts_from_declaration_statement_in_context(statement, context)
        }
    }
}

#[derive(Clone, Debug)]
struct DescribedFunctionTarget {
    subject: String,
    inputs: Vec<String>,
    output: String,
    variadic_tuple_input: bool,
}

fn function_type_fact_from_describes_specifies(
    target: &DescribesTarget,
    specifies: &DescribesSpecifiesSection,
    context: &TypeContext,
) -> Option<TypeFact> {
    let target = described_function_target(target)?;
    let specs = function_type_specs_from_describes_specifies(specifies, context);
    let inputs = target
        .inputs
        .iter()
        .map(|name| specs.get(name).cloned())
        .collect::<Option<Vec<_>>>()?;
    let output = specs.get(&target.output).cloned()?;

    Some(TypeFact::FunctionType {
        subject: target.subject,
        inputs,
        output,
        variadic_tuple_input: target.variadic_tuple_input,
    })
}

fn described_function_target(target: &DescribesTarget) -> Option<DescribedFunctionTarget> {
    let DescribesTarget::Declaration(statement) = target else {
        return None;
    };

    let IsSubjectKind::Forms(forms) = &statement.subject.kind else {
        return None;
    };
    let [
        IsSubjectForm::Form(FormOrDeclaration {
            kind: FormOrDeclarationKind::FunctionDeclaration { name, form },
            ..
        }),
    ] = forms.as_slice()
    else {
        return None;
    };

    let output = statement
        .expansion
        .as_ref()
        .and_then(single_placeholder_subject_key)?;
    let inputs = function_form_parameters(form);
    if inputs.is_empty() {
        return None;
    }

    Some(DescribedFunctionTarget {
        subject: name.clone().unwrap_or_else(|| form.name.clone()),
        inputs,
        output,
        variadic_tuple_input: form.magnetic_placeholder.is_some(),
    })
}

fn function_type_specs_from_describes_specifies(
    specifies: &DescribesSpecifiesSection,
    context: &TypeContext,
) -> HashMap<String, FunctionTypeFactSpec> {
    let mut specs = HashMap::new();
    for item in &specifies.arguments {
        for fact in facts_from_is_or_via_item_in_context(item, context) {
            if let Some((subject, spec)) = function_type_spec_from_fact(&fact) {
                specs.insert(subject, spec);
            }
        }
    }
    specs
}

fn function_type_spec_from_fact(fact: &TypeFact) -> Option<(String, FunctionTypeFactSpec)> {
    match fact {
        TypeFact::Is {
            subject,
            ty,
            signature,
        } => Some((
            subject.clone(),
            FunctionTypeFactSpec::Is {
                ty: ty.clone(),
                signature: signature.clone(),
            },
        )),
        TypeFact::Spec {
            subject,
            operator,
            target,
        } => Some((
            subject.clone(),
            FunctionTypeFactSpec::Spec {
                operator: operator.clone(),
                target: target.clone(),
            },
        )),
        TypeFact::InfixSpec { .. }
        | TypeFact::RefinedIs { .. }
        | TypeFact::MemberOf { .. }
        | TypeFact::FunctionType { .. } => None,
    }
}

fn facts_from_is_or_spec(spec: &IsOrSpec) -> Vec<TypeFact> {
    match spec {
        IsOrSpec::Is(statement) => facts_from_is_statement(statement),
        IsOrSpec::Spec(statement) => vec![TypeFact::Spec {
            subject: key_for_spec_subject(&statement.subject),
            operator: statement.operator.clone(),
            target: statement.name.clone(),
        }],
    }
}

fn facts_from_declaration_statement(statement: &DeclarationStatement) -> Vec<TypeFact> {
    let Some(relation) = &statement.relation else {
        return Vec::new();
    };

    match relation {
        DeclarationRelation::Is(ty) => facts_from_declaration_is(statement, ty),
        DeclarationRelation::Spec { operator, target } => declaration_subject_keys(statement)
            .into_iter()
            .map(|subject| TypeFact::Spec {
                subject,
                operator: operator.clone(),
                target: key_for_expression(target),
            })
            .collect(),
        DeclarationRelation::InfixSpec { spec, target } => {
            let shape = shape_for_infix_spec(spec);
            let args = infix_spec_arguments(spec)
                .into_iter()
                .map(key_for_expression)
                .collect::<Vec<_>>();
            declaration_subject_keys(statement)
                .into_iter()
                .map(|subject| TypeFact::InfixSpec {
                    subject,
                    signature: shape.signature.clone(),
                    args: args.clone(),
                    target: key_for_expression(target),
                })
                .collect()
        }
    }
}

fn facts_from_declaration_statement_in_context(
    statement: &DeclarationStatement,
    context: &TypeContext,
) -> Vec<TypeFact> {
    let Some(relation) = &statement.relation else {
        return Vec::new();
    };

    match relation {
        DeclarationRelation::Is(ty) => facts_from_declaration_is_in_context(statement, ty, context),
        DeclarationRelation::Spec { operator, target } => declaration_subject_keys(statement)
            .into_iter()
            .map(|subject| TypeFact::Spec {
                subject,
                operator: operator.clone(),
                target: key_for_expression(target),
            })
            .collect(),
        DeclarationRelation::InfixSpec { spec, target } => {
            let active_spec = active_infix_spec(spec, context);
            let shape = shape_for_infix_spec(&active_spec);
            let args = infix_spec_arguments(&active_spec)
                .into_iter()
                .map(key_for_expression)
                .collect::<Vec<_>>();
            declaration_subject_keys(statement)
                .into_iter()
                .map(|subject| TypeFact::InfixSpec {
                    subject,
                    signature: shape.signature.clone(),
                    args: args.clone(),
                    target: key_for_expression(target),
                })
                .collect()
        }
    }
}

fn facts_from_declaration_is(
    statement: &DeclarationStatement,
    ty: &TypeExpression,
) -> Vec<TypeFact> {
    if let TypeExpression::Function(function_type) = ty {
        let (Some(inputs), Some(output)) = (
            function_type_inputs_as_facts(function_type),
            function_type_spec_as_fact(&function_type.output),
        ) else {
            return Vec::new();
        };
        return declaration_subject_keys(statement)
            .into_iter()
            .map(|subject| TypeFact::FunctionType {
                subject,
                inputs: inputs.clone(),
                output: output.clone(),
                variadic_tuple_input: false,
            })
            .collect();
    }

    if let TypeExpression::RefinedCommand(command) = ty {
        return declaration_subject_keys(statement)
            .into_iter()
            .map(|subject| refined_fact_from_command(subject, command))
            .collect();
    }

    let Some((ty, signature)) = key_for_type_expression(ty) else {
        return Vec::new();
    };
    declaration_subject_keys(statement)
        .into_iter()
        .map(|subject| TypeFact::Is {
            subject,
            ty: ty.clone(),
            signature: signature.clone(),
        })
        .collect()
}

fn facts_from_declaration_is_in_context(
    statement: &DeclarationStatement,
    ty: &TypeExpression,
    context: &TypeContext,
) -> Vec<TypeFact> {
    if let TypeExpression::Function(function_type) = ty {
        let (Some(inputs), Some(output)) = (
            function_type_inputs_as_facts(function_type),
            function_type_spec_as_fact(&function_type.output),
        ) else {
            return Vec::new();
        };
        return declaration_subject_keys(statement)
            .into_iter()
            .map(|subject| TypeFact::FunctionType {
                subject,
                inputs: inputs.clone(),
                output: output.clone(),
                variadic_tuple_input: false,
            })
            .collect();
    }

    if let TypeExpression::RefinedCommand(command) = ty {
        let active_command = active_refined_command_expression(command, context);
        return declaration_subject_keys(statement)
            .into_iter()
            .map(|subject| refined_fact_from_command(subject, &active_command))
            .collect();
    }

    let Some((ty, signature)) = key_for_type_expression_in_context(ty, context) else {
        return Vec::new();
    };
    declaration_subject_keys(statement)
        .into_iter()
        .map(|subject| TypeFact::Is {
            subject,
            ty: ty.clone(),
            signature: signature.clone(),
        })
        .collect()
}

fn declaration_substitution(statement: &DeclarationStatement) -> Option<(String, String)> {
    let definition = statement.definition.as_ref()?;
    let left = if is_single_function_declaration(&statement.subject) {
        statement
            .expansion
            .as_ref()
            .and_then(single_placeholder_subject_key)
            .unwrap_or_else(|| primary_subject_key(&statement.subject))
    } else {
        primary_subject_key(&statement.subject)
    };
    Some((left, key_for_expression(definition)))
}

fn facts_from_is_statement(statement: &IsStatement) -> Vec<TypeFact> {
    if let TypeExpression::Function(function_type) = &statement.ty {
        let (Some(inputs), Some(output)) = (
            function_type_inputs_as_facts(function_type),
            function_type_spec_as_fact(&function_type.output),
        ) else {
            return Vec::new();
        };
        return subject_keys_for_is_subject(&statement.subject)
            .into_iter()
            .map(|subject| TypeFact::FunctionType {
                subject,
                inputs: inputs.clone(),
                output: output.clone(),
                variadic_tuple_input: false,
            })
            .collect();
    }

    if let TypeExpression::RefinedCommand(command) = &statement.ty {
        return subject_keys_for_is_subject(&statement.subject)
            .into_iter()
            .map(|subject| refined_fact_from_command(subject, command))
            .collect();
    }

    let Some((ty, signature)) = key_for_type_expression(&statement.ty) else {
        return Vec::new();
    };
    subject_keys_for_is_subject(&statement.subject)
        .into_iter()
        .map(|subject| TypeFact::Is {
            subject,
            ty: ty.clone(),
            signature: signature.clone(),
        })
        .collect()
}

fn fact_from_expression(expression: &Expression) -> Option<TypeFact> {
    match &expression.kind {
        ExpressionKind::IsType { subject, ty } => fact_from_type_assertion(subject, ty),
        ExpressionKind::SpecStatement(statement) => Some(TypeFact::Spec {
            subject: key_for_expression(&statement.subject),
            operator: statement.operator.clone(),
            target: statement.name.clone(),
        }),
        ExpressionKind::InfixSpecStatement { left, spec, right } if !spec.predicate => {
            fact_from_infix_spec_statement(left, spec, right)
        }
        ExpressionKind::MemberOf {
            subject,
            collection,
        } => Some(TypeFact::MemberOf {
            subject: key_for_expression(subject),
            collection: key_for_expression(collection),
        }),
        ExpressionKind::IsRefinedPredicate { subject, command } => Some(refined_fact_from_command(
            key_for_expression(subject),
            command,
        )),
        _ => None,
    }
}

fn fact_from_expression_in_context(
    expression: &Expression,
    context: &TypeContext,
) -> Option<TypeFact> {
    match &expression.kind {
        ExpressionKind::IsType { subject, ty } => {
            fact_from_type_assertion_in_context(subject, ty, context)
        }
        ExpressionKind::SpecStatement(statement) => Some(TypeFact::Spec {
            subject: key_for_expression(&statement.subject),
            operator: statement.operator.clone(),
            target: statement.name.clone(),
        }),
        ExpressionKind::InfixSpecStatement { left, spec, right } if !spec.predicate => {
            fact_from_infix_spec_statement_in_context(left, spec, right, context)
        }
        ExpressionKind::MemberOf {
            subject,
            collection,
        } => Some(TypeFact::MemberOf {
            subject: context.normalize_key(&key_for_expression(subject)),
            collection: context.normalize_key(&key_for_expression(collection)),
        }),
        ExpressionKind::IsRefinedPredicate { subject, command } => {
            let active_command = active_refined_command_expression(command, context);
            Some(refined_fact_from_command(
                key_for_expression(subject),
                &active_command,
            ))
        }
        _ => None,
    }
}

fn fact_from_infix_spec_statement(
    left: &Expression,
    spec: &InfixSpec,
    right: &Expression,
) -> Option<TypeFact> {
    let shape = shape_for_infix_spec(spec);
    Some(TypeFact::InfixSpec {
        subject: key_for_expression(left),
        signature: shape.signature,
        args: infix_spec_arguments(spec)
            .into_iter()
            .map(key_for_expression)
            .collect(),
        target: key_for_expression(right),
    })
}

fn fact_from_infix_spec_statement_in_context(
    left: &Expression,
    spec: &InfixSpec,
    right: &Expression,
    context: &TypeContext,
) -> Option<TypeFact> {
    let active_spec = active_infix_spec(spec, context);
    let shape = shape_for_infix_spec(&active_spec);
    Some(TypeFact::InfixSpec {
        subject: key_for_expression(left),
        signature: shape.signature,
        args: infix_spec_arguments(&active_spec)
            .into_iter()
            .map(key_for_expression)
            .collect(),
        target: key_for_expression(right),
    })
}

fn fact_from_type_assertion(subject: &Expression, ty: &TypeExpression) -> Option<TypeFact> {
    if let TypeExpression::Function(function_type) = ty {
        let inputs = function_type_inputs_as_facts(function_type)?;
        let output = function_type_spec_as_fact(&function_type.output)?;
        return Some(TypeFact::FunctionType {
            subject: key_for_expression(subject),
            inputs,
            output,
            variadic_tuple_input: false,
        });
    }

    if let TypeExpression::RefinedCommand(command) = ty {
        return Some(refined_fact_from_command(
            key_for_expression(subject),
            command,
        ));
    }

    let (ty, signature) = key_for_type_expression(ty)?;
    Some(TypeFact::Is {
        subject: key_for_expression(subject),
        ty,
        signature,
    })
}

fn fact_from_type_assertion_in_context(
    subject: &Expression,
    ty: &TypeExpression,
    context: &TypeContext,
) -> Option<TypeFact> {
    if let TypeExpression::Function(function_type) = ty {
        let inputs = function_type_inputs_as_facts(function_type)?;
        let output = function_type_spec_as_fact(&function_type.output)?;
        return Some(TypeFact::FunctionType {
            subject: key_for_expression(subject),
            inputs,
            output,
            variadic_tuple_input: false,
        });
    }

    if let TypeExpression::RefinedCommand(command) = ty {
        let active_command = active_refined_command_expression(command, context);
        return Some(refined_fact_from_command(
            key_for_expression(subject),
            &active_command,
        ));
    }

    let (ty, signature) = key_for_type_expression_in_context(ty, context)?;
    Some(TypeFact::Is {
        subject: key_for_expression(subject),
        ty,
        signature,
    })
}

fn fact_from_type_key_assertion(
    subject: String,
    ty: &TypeExpression,
    context: &TypeContext,
) -> Option<TypeFact> {
    if let TypeExpression::Function(function_type) = ty {
        let inputs = function_type_inputs_as_facts(function_type)?;
        let output = function_type_spec_as_fact(&function_type.output)?;
        return Some(TypeFact::FunctionType {
            subject,
            inputs,
            output,
            variadic_tuple_input: false,
        });
    }

    if let TypeExpression::RefinedCommand(command) = ty {
        let active_command = active_refined_command_expression(command, context);
        return Some(refined_fact_from_command(subject, &active_command));
    }

    let (ty, signature) = key_for_type_expression_in_context(ty, context)?;
    Some(TypeFact::Is {
        subject,
        ty,
        signature,
    })
}

fn refined_fact_from_command(subject: String, command: &RefinedCommandExpression) -> TypeFact {
    TypeFact::RefinedIs {
        subject,
        ty: key_for_refined_command_expression(command),
        signature: shape_for_refined_command_expression(command).signature,
        base_ty: key_for_refined_command_base(command),
        base_signature: shape_for_refined_command_base(command).signature,
    }
}

fn function_type_inputs_as_facts(
    function_type: &FunctionType,
) -> Option<Vec<FunctionTypeFactSpec>> {
    function_type
        .inputs
        .iter()
        .map(function_type_spec_as_fact)
        .collect()
}

fn function_type_spec_as_fact(spec: &FunctionTypeSpec) -> Option<FunctionTypeFactSpec> {
    match &spec.kind {
        FunctionTypeSpecKind::Is(ty) => {
            let (ty, signature) = key_for_type_expression(ty)?;
            Some(FunctionTypeFactSpec::Is { ty, signature })
        }
        FunctionTypeSpecKind::Spec { operator, target } => Some(FunctionTypeFactSpec::Spec {
            operator: operator.clone(),
            target: target.clone(),
        }),
    }
}

#[derive(Clone, Debug, Default)]
struct WhenParameters {
    required: HashSet<String>,
    allowed: HashSet<String>,
}

impl WhenParameters {
    fn require(&mut self, parameter: String) {
        self.allowed.insert(parameter.clone());
        self.required.insert(parameter);
    }

    fn allow(&mut self, parameter: String) {
        self.allowed.insert(parameter);
    }
}

fn header_when_parameters(header: &CommandHeader) -> WhenParameters {
    let mut parameters = WhenParameters::default();
    collect_header_form_parameters(header, &mut parameters);
    parameters
}

fn collect_header_form_parameters(header: &CommandHeader, parameters: &mut WhenParameters) {
    match header {
        CommandHeader::Command(command) => {
            collect_curly_heading_parameters(&command.head_args, parameters);
            collect_tail_parameters(&command.tail, parameters);
        }
        CommandHeader::Infix(command) => {
            require_optional_form_when_parameter(command.left.as_ref(), parameters);
            collect_curly_heading_parameters(&command.head_args, parameters);
            collect_tail_parameters(&command.tail, parameters);
            require_optional_form_when_parameter(command.right.as_ref(), parameters);
        }
        CommandHeader::InfixSpec(spec) => {
            require_form_when_parameter(&spec.left, parameters);
            collect_curly_heading_parameters(&spec.head_args, parameters);
            collect_tail_parameters(&spec.tail, parameters);
            require_form_when_parameter(&spec.right, parameters);
        }
        CommandHeader::Refined(command) => {
            for part in &command.parts {
                collect_tail_parameters(&part.tail, parameters);
            }
            collect_curly_heading_parameters(&command.head_args, parameters);
            collect_tail_parameters(&command.tail, parameters);
        }
    }
}

fn collect_curly_heading_parameters(groups: &[CurlyHeadingArgs], parameters: &mut WhenParameters) {
    for form in groups.iter().flat_map(|group| group.forms.iter()) {
        require_form_when_parameter(form, parameters);
    }
}

fn collect_tail_parameters(parts: &[CommandHeaderTailPart], parameters: &mut WhenParameters) {
    for form in parts
        .iter()
        .flat_map(|part| part.args.iter().map(move |group| (part.optional, group)))
        .flat_map(|(optional, group)| group.forms.iter().map(move |form| (optional, form)))
    {
        let (optional, form) = form;
        if optional {
            allow_form_when_parameter(form, parameters);
        } else {
            require_form_when_parameter(form, parameters);
        }
    }
}

fn require_optional_form_when_parameter(
    form: Option<&FormOrDeclaration>,
    parameters: &mut WhenParameters,
) {
    if let Some(form) = form {
        require_form_when_parameter(form, parameters);
    }
}

fn require_form_when_parameter(form: &FormOrDeclaration, parameters: &mut WhenParameters) {
    for parameter in form_when_parameter_names(form) {
        parameters.require(parameter);
    }
}

fn allow_form_when_parameter(form: &FormOrDeclaration, parameters: &mut WhenParameters) {
    for parameter in form_when_parameter_names(form) {
        parameters.allow(parameter);
    }
}

fn form_when_parameter_names(form: &FormOrDeclaration) -> HashSet<String> {
    let mut parameters = HashSet::new();
    if let Some(name) = primary_form_name(form) {
        parameters.insert(name);
    }
    if let FormOrDeclarationKind::TupleDeclaration { form, .. } = &form.kind {
        collect_tuple_form_when_parameters(form, &mut parameters);
    }
    parameters
}

fn collect_tuple_form_when_parameters(form: &TupleForm, parameters: &mut HashSet<String>) {
    for element in &form.elements {
        match element {
            TupleFormElement::Form(form) => {
                if let FormOrDeclarationKind::TupleDeclaration { name: None, form } = &form.kind {
                    collect_tuple_form_when_parameters(form, parameters);
                } else if let Some(name) = primary_form_name(form) {
                    parameters.insert(name);
                } else {
                    parameters.insert(key_for_form_or_declaration(form));
                }
            }
            TupleFormElement::Operator(operator) => {
                parameters.insert(operator.text.clone());
            }
        }
    }
}

fn subject_keys_for_is_subject(subject: &IsSubject) -> Vec<String> {
    match &subject.kind {
        IsSubjectKind::Forms(forms) => forms
            .iter()
            .map(|form| match form {
                IsSubjectForm::Form(form) => key_for_form_or_declaration(form),
                IsSubjectForm::PlaceholderForm(form) => key_for_placeholder_form(form),
            })
            .collect(),
        IsSubjectKind::Operator(operator) => vec![operator.text.clone()],
    }
}

fn declaration_subject_keys(statement: &DeclarationStatement) -> Vec<String> {
    match &statement.subject.kind {
        IsSubjectKind::Forms(forms) => forms
            .iter()
            .map(|form| match form {
                IsSubjectForm::Form(form) => {
                    primary_form_name(form).unwrap_or_else(|| key_for_form_or_declaration(form))
                }
                IsSubjectForm::PlaceholderForm(form) => key_for_placeholder_form(form),
            })
            .collect(),
        IsSubjectKind::Operator(operator) => vec![operator.text.clone()],
    }
}

fn primary_subject_key(subject: &IsSubject) -> String {
    match &subject.kind {
        IsSubjectKind::Forms(forms) => forms
            .iter()
            .find_map(|form| match form {
                IsSubjectForm::Form(form) => Some(
                    primary_form_name(form).unwrap_or_else(|| key_for_form_or_declaration(form)),
                ),
                IsSubjectForm::PlaceholderForm(form) => Some(key_for_placeholder_form(form)),
            })
            .unwrap_or_default(),
        IsSubjectKind::Operator(operator) => operator.text.clone(),
    }
}

fn single_placeholder_subject_key(subject: &IsSubject) -> Option<String> {
    match &subject.kind {
        IsSubjectKind::Forms(forms) if forms.len() == 1 => match &forms[0] {
            IsSubjectForm::PlaceholderForm(form) => Some(key_for_placeholder_form(form)),
            _ => None,
        },
        _ => None,
    }
}

fn is_single_function_declaration(subject: &IsSubject) -> bool {
    match &subject.kind {
        IsSubjectKind::Forms(forms) if forms.len() == 1 => matches!(
            &forms[0],
            IsSubjectForm::Form(FormOrDeclaration {
                kind: FormOrDeclarationKind::FunctionDeclaration { .. },
                ..
            })
        ),
        _ => false,
    }
}

fn key_for_spec_subject(subject: &SpecSubject) -> String {
    match &subject.kind {
        SpecSubjectKind::Form(form) => key_for_form_or_declaration(form),
        SpecSubjectKind::Operator(operator) => operator.text.clone(),
    }
}

fn key_for_type_expression(ty: &TypeExpression) -> Option<(String, String)> {
    match ty {
        TypeExpression::Builtin { chain, .. } => {
            let signature = format!("\\\\{}", format_chain(chain));
            Some((signature.clone(), signature))
        }
        TypeExpression::Command(command) => Some((
            key_for_command_expression(command),
            shape_for_command_expression(command).signature,
        )),
        TypeExpression::Coercion { ty, .. } => key_for_type_expression(ty),
        TypeExpression::RefinedCommand(_) | TypeExpression::Function(_) => None,
    }
}

fn key_for_type_expression_in_context(
    ty: &TypeExpression,
    context: &TypeContext,
) -> Option<(String, String)> {
    match ty {
        TypeExpression::Builtin { chain, .. } => {
            let signature = format!("\\\\{}", format_chain(chain));
            Some((signature.clone(), signature))
        }
        TypeExpression::Command(command) => {
            let active_command = active_command_expression(command, context);
            Some((
                key_for_command_expression(&active_command),
                shape_for_command_expression(&active_command).signature,
            ))
        }
        TypeExpression::Coercion { ty, .. } => key_for_type_expression_in_context(ty, context),
        TypeExpression::RefinedCommand(_) | TypeExpression::Function(_) => None,
    }
}

fn key_for_expression(expression: &Expression) -> String {
    match &expression.kind {
        ExpressionKind::Name(name) => name.clone(),
        ExpressionKind::FunctionCall { name, arguments } => {
            format!(
                "{}({})",
                name,
                arguments
                    .iter()
                    .map(key_for_expression)
                    .collect::<Vec<_>>()
                    .join(",")
            )
        }
        ExpressionKind::FunctionNamedCall { name, elements } => {
            format!(
                "{}[|{}|]",
                name,
                elements
                    .iter()
                    .map(|element| format!(
                        "{}:={}",
                        key_for_named_expression_lhs(&element.lhs),
                        key_for_expression(&element.expression)
                    ))
                    .collect::<Vec<_>>()
                    .join(",")
            )
        }
        ExpressionKind::MemberCall {
            owner,
            name,
            arguments,
        } => format!(
            "{}.{}({})",
            key_for_expression(owner),
            name,
            arguments
                .iter()
                .map(key_for_expression)
                .collect::<Vec<_>>()
                .join(",")
        ),
        ExpressionKind::MemberAccess { owner, name } => {
            format!("{}.{}", key_for_expression(owner), name)
        }
        ExpressionKind::Tuple(elements) => format!(
            "({})",
            elements
                .iter()
                .map(|element| match element {
                    TupleExpressionElement::Expression(expression) =>
                        key_for_expression(expression),
                    TupleExpressionElement::Operator(operator) => operator.text.clone(),
                })
                .collect::<Vec<_>>()
                .join(",")
        ),
        ExpressionKind::Set(set) => key_for_set_expression(set),
        ExpressionKind::Grouped { expression, .. } => key_for_expression(expression),
        ExpressionKind::Labeled { expression, .. } => key_for_expression(expression),
        ExpressionKind::SubsetCall(subset) => format!("{subset:?}"),
        ExpressionKind::Command(command) => key_for_command_expression(command),
        ExpressionKind::InfixCommand {
            left,
            command,
            right,
        } => format!(
            "{}{}{}",
            key_for_expression(left),
            key_for_infix_command(command),
            key_for_expression(right)
        ),
        ExpressionKind::InfixSpecStatement { left, spec, right } => format!(
            "{}{}{}",
            key_for_expression(left),
            key_for_infix_spec(spec),
            key_for_expression(right)
        ),
        ExpressionKind::Prefix {
            operator,
            expression,
        } => format!(
            "{}{}",
            key_for_unary_operator(operator),
            key_for_expression(expression)
        ),
        ExpressionKind::Postfix {
            expression,
            operator,
        } => format!("{}{}", key_for_expression(expression), operator.text),
        ExpressionKind::Binary {
            left,
            operator,
            right,
        } => format!(
            "{} {} {}",
            key_for_expression(left),
            key_for_binary_operator(operator),
            key_for_expression(right)
        ),
        ExpressionKind::SpecStatement(statement) => format!(
            "{}\"{}\"{}",
            key_for_expression(&statement.subject),
            statement.operator,
            statement.name
        ),
        ExpressionKind::MemberOf {
            subject,
            collection,
        } => format!(
            "{} member_of {}",
            key_for_expression(subject),
            key_for_expression(collection)
        ),
        ExpressionKind::SpecPredicate(statement) => format!(
            "{}\"{}\"?{}",
            key_for_expression(&statement.subject),
            statement.operator,
            statement.name
        ),
        ExpressionKind::IsPredicate { subject, command } => {
            format!(
                "{} is? {}",
                key_for_expression(subject),
                key_for_command_expression(command)
            )
        }
        ExpressionKind::IsNotPredicate { subject, command } => format!(
            "{} is_not? {}",
            key_for_expression(subject),
            key_for_command_expression(command)
        ),
        ExpressionKind::IsBuiltinPredicate { subject, ty } => format!(
            "{} is? {}",
            key_for_expression(subject),
            key_for_type_expression(ty)
                .map(|(key, _)| key)
                .unwrap_or_else(|| key_for_non_command_type_expression(ty))
        ),
        ExpressionKind::IsNotBuiltinPredicate { subject, ty } => format!(
            "{} is_not? {}",
            key_for_expression(subject),
            key_for_type_expression(ty)
                .map(|(key, _)| key)
                .unwrap_or_else(|| key_for_non_command_type_expression(ty))
        ),
        ExpressionKind::IsRefinedPredicate { subject, command } => format!(
            "{} is? {}",
            key_for_expression(subject),
            key_for_refined_command_expression(command)
        ),
        ExpressionKind::IsNotRefinedPredicate { subject, command } => format!(
            "{} is_not? {}",
            key_for_expression(subject),
            key_for_refined_command_expression(command)
        ),
        ExpressionKind::IsType { subject, ty } => format!(
            "{} is {}",
            key_for_expression(subject),
            key_for_type_expression(ty)
                .map(|(key, _)| key)
                .unwrap_or_else(|| key_for_non_command_type_expression(ty))
        ),
    }
}

fn key_for_unary_operator(operator: &UnaryOperator) -> String {
    match operator {
        UnaryOperator::Arithmetic(operator) | UnaryOperator::Named(operator) => {
            operator.text.clone()
        }
    }
}

fn key_for_binary_operator(operator: &BinaryOperator) -> String {
    match operator {
        BinaryOperator::Equality(operator)
        | BinaryOperator::Special(operator)
        | BinaryOperator::Add(operator)
        | BinaryOperator::Subtract(operator)
        | BinaryOperator::Multiply(operator)
        | BinaryOperator::Divide(operator)
        | BinaryOperator::Power(operator) => {
            key_for_binary_operator_parts(&operator.text, operator.kind, false)
        }
        BinaryOperator::Named(operator) => {
            key_for_binary_operator_parts(&operator.name, operator.kind, true)
        }
    }
}

fn key_for_binary_operator_parts(symbol: &str, kind: NamedOperatorKind, named: bool) -> String {
    let body = if named {
        format!("|{symbol}|")
    } else {
        symbol.to_owned()
    };
    match kind {
        NamedOperatorKind::Plain => body,
        NamedOperatorKind::LeftColon => format!(":{body}"),
        NamedOperatorKind::RightColon => format!("{body}:"),
        NamedOperatorKind::BothColon => format!(":{body}:"),
    }
}

fn key_for_non_command_type_expression(ty: &TypeExpression) -> String {
    match ty {
        TypeExpression::Builtin { chain, .. } => format!("\\\\{}", format_chain(chain)),
        TypeExpression::Command(command) => key_for_command_expression(command),
        TypeExpression::RefinedCommand(command) => key_for_refined_command_expression(command),
        TypeExpression::Function(function_type) => format_function_type(
            &function_type
                .inputs
                .iter()
                .filter_map(function_type_spec_as_fact)
                .collect::<Vec<_>>(),
            &function_type_spec_as_fact(&function_type.output).unwrap_or(
                FunctionTypeFactSpec::Spec {
                    operator: "?".to_owned(),
                    target: "?".to_owned(),
                },
            ),
        ),
        TypeExpression::Coercion { ty, literal, .. } => {
            format!(
                "{}@{}",
                key_for_non_command_type_expression(ty),
                key_for_set_expression(literal)
            )
        }
    }
}

fn key_for_named_expression_lhs(lhs: &FunctionNamedExpressionElementLhs) -> String {
    format!("{lhs:?}")
}

fn key_for_form_or_declaration(form: &FormOrDeclaration) -> String {
    match &form.kind {
        FormOrDeclarationKind::Name(name) => name.clone(),
        FormOrDeclarationKind::FunctionDeclaration { name, form } => {
            let name = name.as_ref().unwrap_or(&form.name);
            let args = form
                .magnetic_placeholder
                .iter()
                .map(|placeholder| placeholder.name.clone())
                .chain(
                    form.placeholders
                        .iter()
                        .map(|placeholder| placeholder.name.clone()),
                )
                .collect::<Vec<_>>()
                .join(",");
            if args.is_empty() {
                name.clone()
            } else {
                format!("{name}({args})")
            }
        }
        FormOrDeclarationKind::TupleDeclaration { name, form } => {
            let tuple = format!(
                "({})",
                form.elements
                    .iter()
                    .map(|element| match element {
                        TupleFormElement::Form(form) => key_for_form_or_declaration(form),
                        TupleFormElement::Operator(operator) => operator.text.clone(),
                    })
                    .collect::<Vec<_>>()
                    .join(",")
            );
            name.as_ref()
                .map(|name| format!("{name}:={tuple}"))
                .unwrap_or(tuple)
        }
        FormOrDeclarationKind::SetDeclaration { name, form } => {
            let set = format!("{{{}}}", key_for_placeholder_form(&form.placeholder_form));
            name.as_ref()
                .map(|name| format!("{name}:={set}"))
                .unwrap_or(set)
        }
        FormOrDeclarationKind::InfixOperator {
            left,
            operator,
            right,
        } => format!("{}{}{}", left.name, operator.text, right.name),
        FormOrDeclarationKind::PrefixOperator {
            operator,
            placeholder,
        } => format!("{}{}", operator.text, placeholder.name),
        FormOrDeclarationKind::PostfixOperator {
            placeholder,
            operator,
        } => format!("{}{}", placeholder.name, operator.text),
    }
}

fn disambiguation_key_and_parameters(
    form: &FormOrDeclaration,
) -> Option<(DisambiguationKey, Vec<String>)> {
    match &form.kind {
        FormOrDeclarationKind::FunctionDeclaration { name: None, form } => {
            let parameters = function_form_parameters(form);
            Some((
                DisambiguationKey::Function {
                    name: form.name.clone(),
                    arity: parameters.len(),
                },
                parameters,
            ))
        }
        FormOrDeclarationKind::InfixOperator {
            left,
            operator,
            right,
        } => Some((
            DisambiguationKey::BinaryOperator(operator.text.clone()),
            vec![left.name.clone(), right.name.clone()],
        )),
        FormOrDeclarationKind::PrefixOperator {
            operator,
            placeholder,
        } => Some((
            DisambiguationKey::PrefixOperator(operator.text.clone()),
            vec![placeholder.name.clone()],
        )),
        FormOrDeclarationKind::PostfixOperator {
            placeholder,
            operator,
        } => Some((
            DisambiguationKey::PostfixOperator(operator.text.clone()),
            vec![placeholder.name.clone()],
        )),
        FormOrDeclarationKind::Name(_)
        | FormOrDeclarationKind::FunctionDeclaration { name: Some(_), .. }
        | FormOrDeclarationKind::TupleDeclaration { .. }
        | FormOrDeclarationKind::SetDeclaration { .. } => None,
    }
}

fn provided_symbol_key_and_parameters(
    lhs: &ExpressionAliasLhs,
) -> Option<(DisambiguationKey, Vec<String>)> {
    match lhs {
        ExpressionAliasLhs::Form(FormOrDeclaration {
            kind: FormOrDeclarationKind::Name(name),
            ..
        }) => Some((
            DisambiguationKey::Function {
                name: name.clone(),
                arity: 0,
            },
            Vec::new(),
        )),
        ExpressionAliasLhs::Form(form) => disambiguation_key_and_parameters(form),
        ExpressionAliasLhs::Command(CommandHeaderNode {
            chain, paren_args, ..
        }) => {
            let name = format!("\\{}", format_chain(chain));
            let parameters = paren_args
                .first()
                .map(|args| {
                    args.forms
                        .iter()
                        .map(key_for_form_or_declaration)
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            Some((
                DisambiguationKey::Function {
                    name,
                    arity: parameters.len(),
                },
                parameters,
            ))
        }
        ExpressionAliasLhs::InfixCommand(_) => None,
    }
}

fn function_form_parameters(form: &FunctionForm) -> Vec<String> {
    form.magnetic_placeholder
        .iter()
        .map(|placeholder| placeholder.name.clone())
        .chain(
            form.placeholders
                .iter()
                .map(|placeholder| placeholder.name.clone()),
        )
        .collect()
}

fn key_for_placeholder_form(form: &PlaceholderForm) -> String {
    match &form.kind {
        PlaceholderFormKind::Placeholder(placeholder) => placeholder.name.clone(),
        PlaceholderFormKind::Function {
            placeholder,
            arguments,
        } => format!(
            "{}({})",
            placeholder.name,
            arguments
                .iter()
                .map(|argument| argument.name.clone())
                .collect::<Vec<_>>()
                .join(",")
        ),
    }
}

fn key_for_set_target(target: &SetTarget) -> String {
    match &target.kind {
        SetTargetKind::Name(name) => name.clone(),
        SetTargetKind::PlaceholderForm(form) => key_for_placeholder_form(form),
        SetTargetKind::Alias { name, target } => {
            format!("{name}:={}", key_for_set_target(target))
        }
        SetTargetKind::Function { name, arguments } => format!(
            "{}({})",
            name,
            arguments
                .iter()
                .map(key_for_set_target)
                .collect::<Vec<_>>()
                .join(",")
        ),
        SetTargetKind::Tuple(elements) => format!(
            "({})",
            elements
                .iter()
                .map(|element| match element {
                    SetTargetElement::Target(target) => key_for_set_target(target),
                    SetTargetElement::Operator(operator) => operator.text.clone(),
                })
                .collect::<Vec<_>>()
                .join(",")
        ),
    }
}

fn key_for_set_expression(set: &SetExpression) -> String {
    let mut key = format!(
        "{{{}:{}}}",
        key_for_set_target(&set.target),
        set.specs
            .iter()
            .map(key_for_expression)
            .collect::<Vec<_>>()
            .join(",")
    );
    if let Some(predicate) = &set.predicate {
        key.push('|');
        key.push_str(&key_for_expression(predicate));
    }
    key
}

fn key_for_command_expression(command: &CommandExpression) -> String {
    let mut key = format!("\\{}", format_chain(&command.chain));
    append_expression_args(&mut key, &command.head_args);
    for tail in &command.tail {
        key.push(':');
        key.push_str(&format_chain(&tail.chain));
        append_expression_args(&mut key, &tail.args);
    }
    for args in &command.paren_args {
        key.push('(');
        key.push_str(
            &args
                .expressions
                .iter()
                .map(key_for_expression)
                .collect::<Vec<_>>()
                .join(","),
        );
        key.push(')');
    }
    key
}

fn key_for_refined_command_expression(command: &RefinedCommandExpression) -> String {
    let mut key = "\\".to_string();
    if let Some(prefix) = &command.prefix_chain {
        key.push_str(&format_chain(prefix));
        key.push_str("::");
    }
    for (index, part) in command.parts.iter().enumerate() {
        if index > 0 {
            key.push_str("::");
        }
        key.push_str(&format_chain(&part.chain));
        for tail in &part.tail {
            key.push(':');
            key.push_str(&format_chain(&tail.chain));
            append_expression_args(&mut key, &tail.args);
        }
    }
    key.push_str("::");
    key.push_str(&format_refined_tail(&command.refined_tail));
    append_expression_args(&mut key, &command.head_args);
    for tail in &command.tail {
        key.push(':');
        key.push_str(&format_chain(&tail.chain));
        append_expression_args(&mut key, &tail.args);
    }
    for args in &command.paren_args {
        key.push('(');
        key.push_str(
            &args
                .expressions
                .iter()
                .map(key_for_expression)
                .collect::<Vec<_>>()
                .join(","),
        );
        key.push(')');
    }
    key
}

fn key_for_refined_command_base(command: &RefinedCommandExpression) -> String {
    let mut key = format!("\\{}", format_refined_tail(&command.refined_tail));
    append_expression_args(&mut key, &command.head_args);
    for tail in &command.tail {
        key.push(':');
        key.push_str(&format_chain(&tail.chain));
        append_expression_args(&mut key, &tail.args);
    }
    for args in &command.paren_args {
        key.push('(');
        key.push_str(
            &args
                .expressions
                .iter()
                .map(key_for_expression)
                .collect::<Vec<_>>()
                .join(","),
        );
        key.push(')');
    }
    key
}

fn key_for_refined_command_with_tail(
    command: &RefinedCommandExpression,
    tail_text: &str,
) -> String {
    let mut key = "\\".to_string();
    if let Some(prefix) = &command.prefix_chain {
        key.push_str(&format_chain(prefix));
        key.push_str("::");
    }
    for (index, part) in command.parts.iter().enumerate() {
        if index > 0 {
            key.push_str("::");
        }
        key.push_str(&format_chain(&part.chain));
        for tail in &part.tail {
            key.push(':');
            key.push_str(&format_chain(&tail.chain));
            append_expression_args(&mut key, &tail.args);
        }
    }
    key.push_str("::");
    key.push_str(tail_text);
    append_expression_args(&mut key, &command.head_args);
    for tail in &command.tail {
        key.push(':');
        key.push_str(&format_chain(&tail.chain));
        append_expression_args(&mut key, &tail.args);
    }
    for args in &command.paren_args {
        key.push('(');
        key.push_str(
            &args
                .expressions
                .iter()
                .map(key_for_expression)
                .collect::<Vec<_>>()
                .join(","),
        );
        key.push(')');
    }
    key
}

fn refined_command_signature_with_tail(
    command: &RefinedCommandExpression,
    tail_text: &str,
) -> String {
    let mut signature = "\\".to_string();
    if let Some(prefix) = &command.prefix_chain {
        signature.push_str(&format_chain(prefix));
        signature.push_str("::");
    }
    let mut arg_groups = Vec::new();
    for (index, part) in command.parts.iter().enumerate() {
        if index > 0 {
            signature.push_str("::");
        }
        signature.push_str(&format_chain(&part.chain));
        add_expression_tail(&mut signature, &mut arg_groups, &part.tail);
    }
    signature.push_str("::");
    signature.push_str(tail_text);
    add_expression_tail(&mut signature, &mut arg_groups, &command.tail);
    signature
}

fn key_for_infix_command(command: &InfixCommand) -> String {
    let mut key = format!("\\.{}", format_chain(&command.chain));
    append_expression_args(&mut key, &command.head_args);
    for tail in &command.tail {
        key.push(':');
        key.push_str(&format_chain(&tail.chain));
        append_expression_args(&mut key, &tail.args);
    }
    key.push_str("./");
    key
}

fn key_for_infix_spec(spec: &InfixSpec) -> String {
    let mut key = format!("\\:{}", format_chain(&spec.chain));
    append_expression_args(&mut key, &spec.head_args);
    for tail in &spec.tail {
        key.push(':');
        key.push_str(&format_chain(&tail.chain));
        append_expression_args(&mut key, &tail.args);
    }
    if spec.predicate {
        key.push_str("?:/");
    } else {
        key.push_str(":/");
    }
    key
}

fn append_expression_args(key: &mut String, groups: &[CurlyExpressionArgs]) {
    for args in groups {
        key.push('{');
        key.push_str(
            &args
                .expressions
                .iter()
                .map(key_for_expression)
                .collect::<Vec<_>>()
                .join(","),
        );
        key.push('}');
    }
}

fn command_expression_arguments(command: &CommandExpression) -> Vec<&Expression> {
    command
        .head_args
        .iter()
        .flat_map(|args| args.expressions.iter())
        .chain(
            command
                .tail
                .iter()
                .flat_map(|tail| tail.args.iter())
                .flat_map(|args| args.expressions.iter()),
        )
        .chain(
            command
                .paren_args
                .iter()
                .flat_map(|args| args.expressions.iter()),
        )
        .collect()
}

fn infix_command_arguments(command: &InfixCommand) -> Vec<&Expression> {
    command
        .head_args
        .iter()
        .flat_map(|args| args.expressions.iter())
        .chain(
            command
                .tail
                .iter()
                .flat_map(|tail| tail.args.iter())
                .flat_map(|args| args.expressions.iter()),
        )
        .collect()
}

fn infix_spec_arguments(spec: &InfixSpec) -> Vec<&Expression> {
    spec.head_args
        .iter()
        .flat_map(|args| args.expressions.iter())
        .chain(
            spec.tail
                .iter()
                .flat_map(|tail| tail.args.iter())
                .flat_map(|args| args.expressions.iter()),
        )
        .collect()
}

fn refined_command_expression_arguments(command: &RefinedCommandExpression) -> Vec<&Expression> {
    command
        .parts
        .iter()
        .flat_map(|part| part.tail.iter())
        .flat_map(|tail| tail.args.iter())
        .flat_map(|args| args.expressions.iter())
        .chain(
            command
                .head_args
                .iter()
                .flat_map(|args| args.expressions.iter()),
        )
        .chain(
            command
                .tail
                .iter()
                .flat_map(|tail| tail.args.iter())
                .flat_map(|args| args.expressions.iter()),
        )
        .chain(
            command
                .paren_args
                .iter()
                .flat_map(|args| args.expressions.iter()),
        )
        .collect()
}

fn actuals_for_type_key(signature: &str, ty: &str) -> Option<Vec<String>> {
    if signature.starts_with("\\.") || signature.contains("::") {
        return None;
    }

    let parts = signature.split(':').collect::<Vec<_>>();
    let first = parts.first()?;
    let mut rest = ty.strip_prefix(first)?;
    let mut actuals = Vec::new();
    collect_adjacent_key_args(&mut rest, &mut actuals)?;

    for part in parts.iter().skip(1) {
        rest = rest.strip_prefix(':')?;
        rest = rest.strip_prefix(part)?;
        collect_adjacent_key_args(&mut rest, &mut actuals)?;
    }

    rest.is_empty().then_some(actuals)
}

fn actuals_for_refined_type_key(signature: &str, ty: &str) -> Option<Vec<String>> {
    let signature_segments = split_refined_key(signature)?;
    let ty_segments = split_refined_key(ty)?;
    if signature_segments.len() != ty_segments.len() {
        return None;
    }

    let mut actuals = Vec::new();
    for (signature_segment, ty_segment) in signature_segments.iter().zip(&ty_segments) {
        collect_segment_actuals(signature_segment, ty_segment, &mut actuals)?;
    }
    Some(actuals)
}

fn collect_segment_actuals(
    signature_segment: &str,
    ty_segment: &str,
    actuals: &mut Vec<String>,
) -> Option<()> {
    let parts = signature_segment.split(':').collect::<Vec<_>>();
    let first = parts.first()?;
    let mut rest = ty_segment.strip_prefix(first)?;
    collect_adjacent_key_args(&mut rest, actuals)?;

    for part in parts.iter().skip(1) {
        rest = rest.strip_prefix(':')?;
        rest = rest.strip_prefix(part)?;
        collect_adjacent_key_args(&mut rest, actuals)?;
    }

    rest.is_empty().then_some(())
}

fn split_refined_key(key: &str) -> Option<Vec<String>> {
    let body = key.strip_prefix('\\')?;
    let mut segments = Vec::new();
    let mut start = 0;
    let mut index = 0;
    let mut paren_depth = 0usize;
    let mut brace_depth = 0usize;
    let mut bracket_depth = 0usize;

    while index < body.len() {
        let rest = &body[index..];
        if rest.starts_with("::") && paren_depth == 0 && brace_depth == 0 && bracket_depth == 0 {
            segments.push(body[start..index].to_owned());
            index += "::".len();
            start = index;
            continue;
        }

        let ch = rest.chars().next()?;
        match ch {
            '(' => paren_depth += 1,
            ')' => paren_depth = paren_depth.saturating_sub(1),
            '{' => brace_depth += 1,
            '}' => brace_depth = brace_depth.saturating_sub(1),
            '[' => bracket_depth += 1,
            ']' => bracket_depth = bracket_depth.saturating_sub(1),
            _ => {}
        }
        index += ch.len_utf8();
    }

    segments.push(body[start..].to_owned());
    (segments.len() >= 2).then_some(segments)
}

fn collect_adjacent_key_args(rest: &mut &str, actuals: &mut Vec<String>) -> Option<()> {
    loop {
        let Some(open) = rest.chars().next() else {
            return Some(());
        };
        let close = match open {
            '{' => '}',
            '(' => ')',
            _ => return Some(()),
        };
        let end = find_balanced_group_end(rest, open, close)?;
        let inside = &rest[open.len_utf8()..end - close.len_utf8()];
        actuals.extend(split_key_arg_list(inside));
        *rest = &rest[end..];
    }
}

fn split_key_arg_list(input: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut start = 0;
    let mut paren_depth = 0usize;
    let mut brace_depth = 0usize;
    let mut bracket_depth = 0usize;

    for (index, ch) in input.char_indices() {
        match ch {
            ',' if paren_depth == 0 && brace_depth == 0 && bracket_depth == 0 => {
                let arg = input[start..index].trim();
                if !arg.is_empty() {
                    args.push(arg.to_owned());
                }
                start = index + ch.len_utf8();
            }
            '(' => paren_depth += 1,
            ')' => paren_depth = paren_depth.saturating_sub(1),
            '{' => brace_depth += 1,
            '}' => brace_depth = brace_depth.saturating_sub(1),
            '[' => bracket_depth += 1,
            ']' => bracket_depth = bracket_depth.saturating_sub(1),
            _ => {}
        }
    }

    let tail = input[start..].trim();
    if !tail.is_empty() {
        args.push(tail.to_owned());
    }
    args
}

fn header_forms(header: &CommandHeader) -> Vec<&FormOrDeclaration> {
    match header {
        CommandHeader::Command(command) => command_header_forms(command),
        CommandHeader::Infix(command) => infix_header_forms(command),
        CommandHeader::InfixSpec(spec) => infix_spec_header_forms(spec),
        CommandHeader::Refined(command) => refined_header_forms(command),
    }
}

fn command_header_forms(command: &CommandHeaderNode) -> Vec<&FormOrDeclaration> {
    command
        .head_args
        .iter()
        .flat_map(|args| args.forms.iter())
        .chain(
            command
                .tail
                .iter()
                .flat_map(|tail| tail.args.iter())
                .flat_map(|args| args.forms.iter()),
        )
        .chain(command.paren_args.iter().flat_map(|args| args.forms.iter()))
        .collect()
}

fn infix_header_forms(command: &InfixCommandHeader) -> Vec<&FormOrDeclaration> {
    command
        .left
        .iter()
        .chain(command.head_args.iter().flat_map(|args| args.forms.iter()))
        .chain(
            command
                .tail
                .iter()
                .flat_map(|tail| tail.args.iter())
                .flat_map(|args| args.forms.iter()),
        )
        .chain(command.right.iter())
        .collect()
}

fn infix_spec_header_forms(spec: &InfixSpecHeader) -> Vec<&FormOrDeclaration> {
    std::iter::once(&spec.left)
        .chain(spec.head_args.iter().flat_map(|args| args.forms.iter()))
        .chain(
            spec.tail
                .iter()
                .flat_map(|tail| tail.args.iter())
                .flat_map(|args| args.forms.iter()),
        )
        .chain(std::iter::once(&spec.right))
        .collect()
}

fn refined_header_forms(command: &RefinedCommandHeader) -> Vec<&FormOrDeclaration> {
    command
        .head_args
        .iter()
        .flat_map(|args| args.forms.iter())
        .chain(
            command
                .tail
                .iter()
                .flat_map(|tail| tail.args.iter())
                .flat_map(|args| args.forms.iter()),
        )
        .chain(command.paren_args.iter().flat_map(|args| args.forms.iter()))
        .collect()
}

fn primary_form_name(form: &FormOrDeclaration) -> Option<String> {
    match &form.kind {
        FormOrDeclarationKind::Name(name) => Some(name.clone()),
        FormOrDeclarationKind::FunctionDeclaration { name, form } => {
            Some(name.as_ref().unwrap_or(&form.name).clone())
        }
        FormOrDeclarationKind::TupleDeclaration { name, .. }
        | FormOrDeclarationKind::SetDeclaration { name, .. } => name.clone(),
        FormOrDeclarationKind::InfixOperator { .. }
        | FormOrDeclarationKind::PrefixOperator { .. }
        | FormOrDeclarationKind::PostfixOperator { .. } => None,
    }
}

fn described_target_subject_key(target: &DescribesTarget) -> String {
    match target {
        DescribesTarget::Form(form) => {
            primary_form_name(form).unwrap_or_else(|| key_for_form_or_declaration(form))
        }
        DescribesTarget::Declaration(statement) => primary_subject_key(&statement.subject),
    }
}

fn placeholder_pattern_name(form: &PlaceholderForm) -> Option<String> {
    match &form.kind {
        PlaceholderFormKind::Placeholder(placeholder) => Some(placeholder.name.clone()),
        PlaceholderFormKind::Function { placeholder, .. } => Some(placeholder.name.clone()),
    }
}

fn format_fact(fact: &TypeFact) -> String {
    match fact {
        TypeFact::Is { subject, ty, .. } => format!("{subject} is {ty}"),
        TypeFact::Spec {
            subject,
            operator,
            target,
        } => format!("{subject} \"{operator}\" {target}"),
        TypeFact::InfixSpec {
            subject,
            signature,
            args,
            target,
        } => {
            let rendered_args = if args.is_empty() {
                String::new()
            } else {
                format!("{{{}}}", args.join(", "))
            };
            format!("{subject} {signature}{rendered_args} {target}")
        }
        TypeFact::RefinedIs { subject, ty, .. } => format!("{subject} is {ty}"),
        TypeFact::MemberOf {
            subject,
            collection,
        } => format!("{subject} member_of {collection}"),
        TypeFact::FunctionType {
            subject,
            inputs,
            output,
            ..
        } => format!("{subject} is {}", format_function_type(inputs, output)),
    }
}

fn format_function_type(inputs: &[FunctionTypeFactSpec], output: &FunctionTypeFactSpec) -> String {
    format!(
        "({}) => ({})",
        inputs
            .iter()
            .map(format_function_type_spec)
            .collect::<Vec<_>>()
            .join(", "),
        format_function_type_spec(output)
    )
}

fn format_function_type_spec(spec: &FunctionTypeFactSpec) -> String {
    match spec {
        FunctionTypeFactSpec::Is { ty, .. } => format!("_ is {ty}"),
        FunctionTypeFactSpec::Spec { operator, target } => format!("_ \"{operator}\" {target}"),
    }
}

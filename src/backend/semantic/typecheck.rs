use super::*;
use std::collections::{HashMap, HashSet};

pub(super) fn collect_definition_type_metadata(
    item: &TopLevelItem,
    header_shape: &HeaderShape,
    registry: &mut SignatureRegistry,
) {
    let Some(info) = definition_type_info(item, header_shape) else {
        return;
    };

    collect_type_extension_rules(item, &info, registry);
    collect_spec_operator_rules(item, &info, registry);
    registry
        .type_infos
        .insert(header_shape.shape.signature.clone(), info);
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
            Some(&group.describes.argument),
        )),
        TopLevelItem::Defines(group) => Some(type_info_from_parts(
            header_shape,
            &group.heading,
            group.using.as_ref(),
            group.when.as_ref(),
            None,
        )),
        TopLevelItem::Refines(group) => Some(type_info_from_parts(
            header_shape,
            &group.heading,
            group.using.as_ref(),
            group.when.as_ref(),
            None,
        )),
        TopLevelItem::States(group) => Some(type_info_from_parts(
            header_shape,
            &group.heading,
            group.using.as_ref(),
            group.when.as_ref(),
            None,
        )),
        TopLevelItem::Axiom(group) => group
            .heading
            .as_ref()
            .map(|heading| type_info_from_parts(header_shape, heading, None, None, None)),
        TopLevelItem::Theorem(group) => group
            .heading
            .as_ref()
            .map(|heading| type_info_from_parts(header_shape, heading, None, None, None)),
        TopLevelItem::Corollary(group) => group
            .heading
            .as_ref()
            .map(|heading| type_info_from_parts(header_shape, heading, None, None, None)),
        TopLevelItem::Lemma(group) => group
            .heading
            .as_ref()
            .map(|heading| type_info_from_parts(header_shape, heading, None, None, None)),
        TopLevelItem::Conjecture(group) => group
            .heading
            .as_ref()
            .map(|heading| type_info_from_parts(header_shape, heading, None, None, None)),
        _ => None,
    }
}

fn type_info_from_parts(
    header_shape: &HeaderShape,
    _heading: &CommandHeader,
    using: Option<&UsingSection>,
    when: Option<&WhenSection>,
    described: Option<&FormOrDeclaration>,
) -> DefinitionTypeInfo {
    let mut context = TypeContext::default();

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

    DefinitionTypeInfo {
        signature: header_shape.shape.signature.clone(),
        parameters: header_shape.parameters.clone(),
        hidden_parameters: header_shape.hidden_parameters.clone(),
        requirements,
        substitutions: context.substitutions,
        described: described.map(key_for_form_or_declaration),
    }
}

fn collect_spec_operator_rules(
    item: &TopLevelItem,
    info: &DefinitionTypeInfo,
    registry: &mut SignatureRegistry,
) {
    let (Some(_described), Some(provides)) = (info.described.as_ref(), provides_section(item))
    else {
        return;
    };

    for item in &provides.arguments {
        let ProvidesItem::Symbol(group) = item else {
            continue;
        };
        let AliasKind::SpecOperator(alias) = &group.symbol.argument else {
            continue;
        };
        if let Some(rule) = spec_operator_rule_from_alias(alias, info) {
            registry.spec_rules.push(rule);
        }
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

fn extends_item(item: &TopLevelItem) -> Option<&IsOrViaItem> {
    match item {
        TopLevelItem::Describes(group) => group.extends.as_ref().map(|section| &section.argument),
        _ => None,
    }
}

fn provides_section(item: &TopLevelItem) -> Option<&ProvidesSection> {
    match item {
        TopLevelItem::Describes(group) => group.provides.as_ref(),
        TopLevelItem::Defines(group) => group.provides.as_ref(),
        TopLevelItem::Refines(group) => group.provides.as_ref(),
        TopLevelItem::States(group) => group.provides.as_ref(),
        _ => None,
    }
}

fn spec_operator_rule_from_alias(
    alias: &SpecOperatorAlias,
    info: &DefinitionTypeInfo,
) -> Option<SpecOperatorRule> {
    let placeholder = placeholder_pattern_name(&alias.placeholder_spec.placeholder_form)?;
    Some(SpecOperatorRule {
        owner_signature: info.signature.clone(),
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
            declare_form_or_declaration(&group.describes.argument, &mut context);
            assume_described_type(&group.heading, &group.describes.argument, &mut context);
            assume_optional_using(
                &group.using,
                &mut context,
                path,
                locator,
                registry,
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
            validate_optional_provides(
                &group.provides,
                &context,
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
            validate_refines_target_matches_heading(group, path, locator, event_log);
            assume_optional_using(
                &group.using,
                &mut context,
                path,
                locator,
                registry,
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
                &group.refines.argument,
                &mut context,
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
        | TopLevelItem::Section(_)
        | TopLevelItem::Subsection(_)
        | TopLevelItem::Subsubsection(_)
        | TopLevelItem::Text(_)
        | TopLevelItem::Person(_)
        | TopLevelItem::Resource(_) => {}
    }
}

fn validate_refines_target_matches_heading(
    group: &RefinesGroup,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    event_log: &mut EventLog,
) {
    let CommandHeader::Refined(header) = &group.heading else {
        return;
    };

    let Some(DeclarationRelation::Is(TypeExpression::Command(target))) =
        &group.refines.argument.relation
    else {
        emit_error(
            event_log,
            path,
            locator.locate_heading(&shape_for_header(&group.heading)),
            "Refines entries with refined headings must have the form `Refines: ... is <refined target>`",
        );
        return;
    };

    if refined_header_target_matches_command(header, target) {
        return;
    }

    emit_error(
        event_log,
        path,
        locator.locate_heading(&shape_for_header(&group.heading)),
        "Refines target must exactly match the command after `::` in the refined heading",
    );
}

fn refined_header_target_matches_command(
    header: &RefinedCommandHeader,
    target: &CommandExpression,
) -> bool {
    match &header.refined_tail {
        RefinedTail::Chain(chain) if chains_match(chain, &target.chain) => {}
        RefinedTail::Name { name, .. } if name == &format_chain(&target.chain) => {}
        _ => return false,
    }

    heading_expression_groups_match(&header.head_args, &target.head_args)
        && heading_expression_tails_match(&header.tail, &target.tail)
        && paren_heading_expression_groups_match(&header.paren_args, &target.paren_args)
}

fn heading_expression_tails_match(
    heading: &[CommandHeaderTailPart],
    expression: &[CommandExpressionTailPart],
) -> bool {
    heading.len() == expression.len()
        && heading.iter().zip(expression).all(|(heading, expression)| {
            chains_match(&heading.chain, &expression.chain)
                && heading.optional == expression.optional
                && heading_expression_groups_match(&heading.args, &expression.args)
        })
}

fn chains_match(left: &Chain, right: &Chain) -> bool {
    left.parts == right.parts
}

fn heading_expression_groups_match(
    heading: &[CurlyHeadingArgs],
    expression: &[CurlyExpressionArgs],
) -> bool {
    heading.len() == expression.len()
        && heading.iter().zip(expression).all(|(heading, expression)| {
            heading.forms.len() == expression.expressions.len()
                && heading
                    .forms
                    .iter()
                    .zip(&expression.expressions)
                    .all(|(form, expression)| {
                        key_for_form_or_declaration(form) == key_for_expression(expression)
                    })
        })
}

fn paren_heading_expression_groups_match(
    heading: &[ParenHeadingArgs],
    expression: &[ParenExpressionArgs],
) -> bool {
    heading.len() == expression.len()
        && heading.iter().zip(expression).all(|(heading, expression)| {
            heading.forms.len() == expression.expressions.len()
                && heading
                    .forms
                    .iter()
                    .zip(&expression.expressions)
                    .all(|(form, expression)| {
                        key_for_form_or_declaration(form) == key_for_expression(expression)
                    })
        })
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
    described: &FormOrDeclaration,
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

    let subject =
        primary_form_name(described).unwrap_or_else(|| key_for_form_or_declaration(described));

    for header_shape in shapes_for_header(heading) {
        context.add_fact(TypeFact::Is {
            subject: subject.clone(),
            ty: header_shape.type_key,
            signature: header_shape.shape.signature,
        });
    }
}

fn validate_spec_infix_describes_header(
    heading: &CommandHeader,
    described: &FormOrDeclaration,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    event_log: &mut EventLog,
) {
    let CommandHeader::InfixSpec(header) = heading else {
        return;
    };

    if key_for_form_or_declaration(&header.left) == key_for_form_or_declaration(described) {
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
            assume_binding_or_spec(
                &group.exists.argument,
                &mut child,
                path,
                locator,
                registry,
                event_log,
            );
            for clause in &group.such_that.arguments {
                assume_clause(clause, &mut child, path, locator, registry, event_log);
            }
        }
        Clause::ExistsUnique(group) => {
            let mut child = context.clone();
            assume_binding_or_spec(
                &group.exists_unique.argument,
                &mut child,
                path,
                locator,
                registry,
                event_log,
            );
            for clause in &group.such_that.arguments {
                assume_clause(clause, &mut child, path, locator, registry, event_log);
            }
        }
        Clause::ForAll(group) => {
            let mut child = context.clone();
            assume_binding_or_spec(
                &group.for_all.argument,
                &mut child,
                path,
                locator,
                registry,
                event_log,
            );
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
            assume_declaration_statement(
                &group.given.argument,
                &mut child,
                path,
                locator,
                registry,
                event_log,
            );
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
    if let Some(definition) = &statement.definition {
        check_expression(definition, context, path, locator, registry, event_log);
    }
    if let Some((left, right)) = declaration_substitution(statement) {
        context.add_substitution(left, right);
    }
    for fact in facts_from_declaration_statement_in_context(statement, context) {
        context.add_fact(fact);
    }
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
        }
        ExpressionKind::Postfix {
            expression,
            operator,
        } => {
            check_expression(expression, context, path, locator, registry, event_log);
            check_disambiguated_postfix(
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
            check_disambiguated_binary(
                left, operator, right, context, path, locator, registry, event_log,
            );
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
        ExpressionKind::IsPredicate { subject, command }
        | ExpressionKind::IsNotPredicate { subject, command } => {
            check_expression(subject, context, path, locator, registry, event_log);
            check_command_predicate(command, context, path, locator, registry, event_log);
            let active_command = active_command_expression(command, context);
            for expression in command_expression_arguments(&active_command) {
                check_expression(expression, context, path, locator, registry, event_log);
            }
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
    for function_type in function_types {
        let TypeFact::FunctionType {
            inputs, output: _, ..
        } = function_type
        else {
            continue;
        };
        if inputs.len() != arguments.len() {
            continue;
        }

        for (input, argument) in inputs.iter().zip(arguments) {
            let required = instantiate_function_type_spec(input, &key_for_expression(argument));
            if !prove_fact(&required, context, registry) {
                emit_error(
                    event_log,
                    path,
                    locator.locate_symbol(name),
                    format!(
                        "Could not prove requirement `{}` for function `{name}`",
                        format_fact(&required)
                    ),
                );
            }
        }
    }
}

fn has_function_call_disambiguation(
    name: &str,
    arity: usize,
    registry: &SignatureRegistry,
) -> bool {
    registry.disambiguations.iter().any(|rule| {
        rule.key
            == DisambiguationKey::Function {
                name: name.to_owned(),
                arity,
            }
    })
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
    if !registry.disambiguations.iter().any(|rule| rule.key == key) {
        return;
    }
    let actuals = arguments.iter().map(key_for_expression).collect::<Vec<_>>();
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
    if !registry.disambiguations.iter().any(|rule| rule.key == key) {
        return;
    }
    let actuals = vec![key_for_expression(left), key_for_expression(right)];
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
    if !registry.disambiguations.iter().any(|rule| rule.key == key) {
        return;
    }
    let actuals = vec![key_for_expression(expression)];
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
    if !registry.disambiguations.iter().any(|rule| rule.key == key) {
        return;
    }
    let actuals = vec![key_for_expression(expression)];
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
        .find(|rule| rule.key == *key)
    else {
        return;
    };
    if rule.parameters.len() != actuals.len() {
        return;
    }
    if context.active_disambiguations.contains(key) {
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
        prove_fact(&instantiated, &requirement_context, registry)
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

fn disambiguation_key_for_binary_operator(
    operator: &BinaryOperator,
) -> Option<(DisambiguationKey, String, String)> {
    match operator {
        BinaryOperator::Equality(operator)
        | BinaryOperator::Special(operator)
        | BinaryOperator::Add(operator)
        | BinaryOperator::Subtract(operator)
        | BinaryOperator::Multiply(operator)
        | BinaryOperator::Divide(operator)
        | BinaryOperator::Power(operator) => Some((
            DisambiguationKey::BinaryOperator(operator.text.clone()),
            format!("operator `{}`", operator.text),
            operator.text.clone(),
        )),
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
        if function_type_call_arity(function_type) == Some(function_call_arity(subject)) {
            found_matching_arity = true;
        }
        let mut seen = HashSet::new();
        if function_type_implies_required(function_type, &required, context, registry, &mut seen) {
            return;
        }
    }

    if found_matching_arity {
        emit_error(
            event_log,
            path,
            locator.locate_symbol(name),
            format!(
                "Could not prove function call result `{}`",
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
    let active_command = active_command_expression(command, context);
    let shape = shape_for_command_expression(&active_command);
    let position = locator.locate_reference(&shape);
    let actuals = command_expression_arguments(&active_command)
        .into_iter()
        .map(key_for_expression)
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
    let active_command = active_command_expression(command, context);
    let shape = shape_for_command_expression(&active_command);
    let position = locator.locate_reference(&shape);
    let actuals = command_expression_arguments(&active_command)
        .into_iter()
        .map(key_for_expression)
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
    let active_command = active_infix_command(command, context);
    let shape = shape_for_infix_command(&active_command);
    let position = locator.locate_reference(&shape);
    let mut actuals = Vec::new();
    actuals.push(key_for_expression(left));
    actuals.extend(
        infix_command_arguments(&active_command)
            .into_iter()
            .map(key_for_expression),
    );
    actuals.push(key_for_expression(right));
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
    let active_command = active_refined_command_expression(command, context);
    let shape = shape_for_refined_command_expression(&active_command);
    let position = locator.locate_reference(&shape);
    let actuals = refined_command_expression_arguments(&active_command)
        .into_iter()
        .map(key_for_expression)
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
    let active_command = active_refined_command_expression(command, context);
    let shape = shape_for_refined_command_expression(&active_command);
    let position = locator.locate_reference(&shape);
    let actuals = refined_command_expression_arguments(&active_command)
        .into_iter()
        .map(key_for_expression)
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
    collect_expression_names(expression, &mut names);
    names
        .iter()
        .all(|name| is_literal_name(name) || context.has_name(name))
}

fn collect_expression_names(expression: &Expression, names: &mut Vec<String>) {
    match &expression.kind {
        ExpressionKind::Name(name) => names.push(name.clone()),
        ExpressionKind::FunctionCall { name, arguments } => {
            names.push(name.clone());
            for argument in arguments {
                collect_expression_names(argument, names);
            }
        }
        ExpressionKind::FunctionNamedCall { name, elements } => {
            names.push(name.clone());
            for element in elements {
                match &element.lhs {
                    FunctionNamedExpressionElementLhs::Name(name) => names.push(name.clone()),
                    FunctionNamedExpressionElementLhs::SubsetCall(subset) => {
                        collect_subset_call_names(subset, names);
                    }
                }
                collect_expression_names(&element.expression, names);
            }
        }
        ExpressionKind::Tuple(elements) => {
            for element in elements {
                if let TupleExpressionElement::Expression(expression) = element {
                    collect_expression_names(expression, names);
                }
            }
        }
        ExpressionKind::Set(set) => {
            collect_set_target_names(&set.target, names);
            for spec in &set.specs {
                collect_expression_names(spec, names);
            }
            if let Some(predicate) = &set.predicate {
                collect_expression_names(predicate, names);
            }
        }
        ExpressionKind::Grouped { expression, .. }
        | ExpressionKind::Labeled { expression, .. }
        | ExpressionKind::Prefix { expression, .. }
        | ExpressionKind::Postfix { expression, .. } => collect_expression_names(expression, names),
        ExpressionKind::SubsetCall(subset) => collect_subset_call_names(subset, names),
        ExpressionKind::Command(command) => {
            for expression in command_expression_arguments(command) {
                collect_expression_names(expression, names);
            }
        }
        ExpressionKind::InfixCommand {
            left,
            command,
            right,
        } => {
            collect_expression_names(left, names);
            for expression in infix_command_arguments(command) {
                collect_expression_names(expression, names);
            }
            collect_expression_names(right, names);
        }
        ExpressionKind::InfixSpecStatement { left, spec, right } => {
            collect_expression_names(left, names);
            for expression in infix_spec_arguments(spec) {
                collect_expression_names(expression, names);
            }
            collect_expression_names(right, names);
        }
        ExpressionKind::Binary { left, right, .. } => {
            collect_expression_names(left, names);
            collect_expression_names(right, names);
        }
        ExpressionKind::SpecStatement(statement) | ExpressionKind::SpecPredicate(statement) => {
            collect_expression_names(&statement.subject, names);
            names.push(statement.name.clone());
        }
        ExpressionKind::IsPredicate { subject, command }
        | ExpressionKind::IsNotPredicate { subject, command } => {
            collect_expression_names(subject, names);
            for expression in command_expression_arguments(command) {
                collect_expression_names(expression, names);
            }
        }
        ExpressionKind::IsRefinedPredicate { subject, command }
        | ExpressionKind::IsNotRefinedPredicate { subject, command } => {
            collect_expression_names(subject, names);
            for expression in refined_command_expression_arguments(command) {
                collect_expression_names(expression, names);
            }
        }
        ExpressionKind::IsType { subject, ty } => {
            collect_expression_names(subject, names);
            collect_type_expression_names(ty, names);
        }
    }
}

fn collect_type_expression_names(ty: &TypeExpression, names: &mut Vec<String>) {
    match ty {
        TypeExpression::Builtin { .. } => {}
        TypeExpression::Command(command) => {
            for expression in command_expression_arguments(command) {
                collect_expression_names(expression, names);
            }
        }
        TypeExpression::RefinedCommand(command) => {
            for expression in refined_command_expression_arguments(command) {
                collect_expression_names(expression, names);
            }
        }
        TypeExpression::Function(function_type) => {
            for spec in function_type
                .inputs
                .iter()
                .chain(std::iter::once(&function_type.output))
            {
                collect_function_type_spec_names(spec, names);
            }
        }
    }
}

fn collect_function_type_spec_names(spec: &FunctionTypeSpec, names: &mut Vec<String>) {
    match &spec.kind {
        FunctionTypeSpecKind::Is(ty) => collect_type_expression_names(ty, names),
        FunctionTypeSpecKind::Spec { target, .. } => names.push(target.clone()),
    }
}

fn collect_set_target_names(target: &SetTarget, names: &mut Vec<String>) {
    match &target.kind {
        SetTargetKind::Name(name) => names.push(name.clone()),
        SetTargetKind::PlaceholderForm(form) => collect_placeholder_form_names(form, names),
        SetTargetKind::Function {
            name: function_name,
            arguments,
        } => {
            names.push(function_name.clone());
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

fn collect_placeholder_form_names(form: &PlaceholderForm, names: &mut Vec<String>) {
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

fn collect_subset_call_names(subset: &SubsetCall, names: &mut Vec<String>) {
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

fn validate_optional_provides(
    provides: &Option<ProvidesSection>,
    context: &TypeContext,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    registry: &SignatureRegistry,
    event_log: &mut EventLog,
) {
    let Some(provides) = provides else {
        return;
    };

    for item in &provides.arguments {
        let ProvidesItem::Symbol(group) = item else {
            continue;
        };
        let AliasKind::SpecOperator(alias) = &group.symbol.argument else {
            continue;
        };
        validate_spec_operator_alias(alias, context, path, locator, registry, event_log);
    }
}

fn validate_spec_operator_alias(
    alias: &SpecOperatorAlias,
    context: &TypeContext,
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

    let mut child = context.clone();
    declare_placeholder_form(&alias.placeholder_spec.placeholder_form, &mut child);

    if let SpecOperatorAliasTarget::IsOrSpec(target) = &alias.target {
        check_is_or_spec_alias_target(target, &child, path, locator, registry, event_log);
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
                    "Could not prove requirement `{}` for command `{signature}`",
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
        TypeFact::FunctionType {
            subject,
            inputs,
            output,
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
    let required = context.normalize_fact(required);
    if builtin_fact_holds(&required, registry) {
        return true;
    }

    let mut seen = HashSet::new();
    context
        .facts
        .iter()
        .any(|fact| fact_implies(fact, &required, context, registry, &mut seen))
}

fn builtin_fact_holds(required: &TypeFact, registry: &SignatureRegistry) -> bool {
    let TypeFact::Is {
        subject, signature, ..
    } = required
    else {
        return false;
    };

    match signature.as_str() {
        BUILTIN_EXPRESSION_SIGNATURE => true,
        BUILTIN_STATEMENT_SIGNATURE => key_is_statement(subject, registry),
        BUILTIN_SPECIFICATION_SIGNATURE => key_is_specification(subject),
        _ => false,
    }
}

fn key_is_statement(key: &str, registry: &SignatureRegistry) -> bool {
    key_contains_top_level(key, " is? ")
        || key_contains_top_level(key, " is_not? ")
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
    let fact = context.normalize_fact(fact);
    if &fact == required {
        return true;
    }
    if !seen.insert(fact.clone()) {
        return false;
    }

    if function_type_implies_required(&fact, required, context, registry, seen) {
        return true;
    }

    for extended in reduce_extension_fact(&fact, context, registry) {
        if fact_implies(&extended, required, context, registry, seen) {
            return true;
        }
    }

    if matches!(fact, TypeFact::Spec { .. }) {
        let mut spec_seen = HashSet::new();
        for reduced in reduce_spec_fact(&fact, context, registry, &mut spec_seen) {
            if fact_implies(&reduced, required, context, registry, seen) {
                return true;
            }
        }
    }

    false
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

        let substitutions = HashMap::from([
            (rule.placeholder.clone(), subject.clone()),
            (rule.target.clone(), target.clone()),
        ]);

        match &rule.target_alias {
            SpecOperatorAliasTarget::Builtin(_) => {}
            SpecOperatorAliasTarget::IsOrSpec(target_alias) => {
                for next in facts_from_is_or_spec(target_alias) {
                    let next = substitute_fact(&next, &substitutions);
                    let next = context.normalize_fact(&next);
                    result.push(next.clone());
                    if matches!(next, TypeFact::Spec { .. }) {
                        result.extend(reduce_spec_fact(&next, context, registry, seen));
                    }
                }
            }
        }
    }

    result
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

    command_requirement_facts(&fact, context, registry)
        .iter()
        .any(|fact| fact_has_type_signature(fact, subject, signature, context, registry, seen))
        || reduce_extension_fact(&fact, context, registry)
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
        TypeFact::Spec { .. } | TypeFact::FunctionType { .. } => None,
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
) -> bool {
    let TypeFact::FunctionType {
        subject,
        inputs,
        output,
    } = fact
    else {
        return false;
    };

    let Some((function_name, arguments)) = function_call_parts_from_fact(required) else {
        return false;
    };
    if context.normalize_key(&function_name) != context.normalize_key(subject)
        || arguments.len() != inputs.len()
    {
        return false;
    }

    for (input, argument) in inputs.iter().zip(&arguments) {
        let required_input = instantiate_function_type_spec(input, argument);
        if !prove_fact(&required_input, context, registry) {
            return false;
        }
    }

    let output_subject = fact_subject(required);
    let output_fact = instantiate_function_type_spec(output, output_subject);
    fact_implies(&output_fact, required, context, registry, seen)
}

fn function_call_parts_from_fact(fact: &TypeFact) -> Option<(String, Vec<String>)> {
    function_call_parts_from_key(fact_subject(fact))
}

fn fact_subject(fact: &TypeFact) -> &str {
    match fact {
        TypeFact::Is { subject, .. }
        | TypeFact::Spec { subject, .. }
        | TypeFact::InfixSpec { subject, .. }
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

fn function_type_call_arity(fact: &TypeFact) -> Option<usize> {
    match fact {
        TypeFact::FunctionType { inputs, .. } => Some(inputs.len()),
        _ => None,
    }
}

fn function_call_arity(expression: &Expression) -> usize {
    match &expression.kind {
        ExpressionKind::FunctionCall { arguments, .. } => arguments.len(),
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
    symbols: HashSet<String>,
    active_disambiguations: Vec<DisambiguationKey>,
}

impl TypeContext {
    fn add_fact(&mut self, fact: TypeFact) {
        self.facts.push(fact);
    }

    fn add_substitution(&mut self, left: String, right: String) {
        self.substitutions.push((left, right));
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
            TypeFact::FunctionType {
                subject,
                inputs,
                output,
            } => TypeFact::FunctionType {
                subject: self.normalize_key(subject),
                inputs: inputs
                    .iter()
                    .map(|spec| self.normalize_function_type_spec(spec))
                    .collect(),
                output: self.normalize_function_type_spec(output),
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
        ExpressionKind::InfixSpecStatement { left, spec, right } if !spec.predicate => {
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
        ExpressionKind::IsPredicate { subject, command }
        | ExpressionKind::IsNotPredicate { subject, command } => {
            declare_names_from_expression(subject, context);
            let active_command = active_command_expression(command, context);
            for expression in command_expression_arguments(&active_command) {
                declare_names_from_expression(expression, context);
            }
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
        TypeFact::FunctionType {
            subject,
            inputs,
            output,
        } => TypeFact::FunctionType {
            subject: substitute_key(subject, substitutions),
            inputs: inputs
                .iter()
                .map(|spec| substitute_function_type_spec(spec, substitutions))
                .collect(),
            output: substitute_function_type_spec(output, substitutions),
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
            })
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
            })
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
            })
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
        });
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
        });
    }

    let (ty, signature) = key_for_type_expression_in_context(ty, context)?;
    Some(TypeFact::Is {
        subject: key_for_expression(subject),
        ty,
        signature,
    })
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
        ExpressionKind::Set(set) => format!(
            "{{{}:{}}}",
            key_for_set_target(&set.target),
            set.specs
                .iter()
                .map(key_for_expression)
                .collect::<Vec<_>>()
                .join(",")
        ),
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
            "{}{:?}{}",
            key_for_expression(left),
            operator,
            key_for_expression(right)
        ),
        ExpressionKind::SpecStatement(statement) => format!(
            "{}\"{}\"{}",
            key_for_expression(&statement.subject),
            statement.operator,
            statement.name
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

fn key_for_non_command_type_expression(ty: &TypeExpression) -> String {
    match ty {
        TypeExpression::Builtin { chain, .. } => format!("\\\\{}", format_chain(chain)),
        TypeExpression::Command(command) => key_for_command_expression(command),
        TypeExpression::RefinedCommand(_) => "<refined>".to_owned(),
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
        TypeFact::FunctionType {
            subject,
            inputs,
            output,
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

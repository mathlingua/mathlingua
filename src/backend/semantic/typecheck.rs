use super::*;
use std::collections::{HashMap, HashSet};

/// Collects type-checking metadata for a definition item.
pub(super) fn collect_definition_type_metadata(
    item: &TopLevelItem,
    shape: &SignatureShape,
    registry: &mut SignatureRegistry,
) {
    let Some(info) = definition_type_info(item, shape) else {
        return;
    };

    collect_spec_operator_rules(item, &info, registry);
    registry.type_infos.insert(shape.signature.clone(), info);
}

/// Runs dependent type checks across one parsed source file.
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

fn definition_type_info(item: &TopLevelItem, shape: &SignatureShape) -> Option<DefinitionTypeInfo> {
    match item {
        TopLevelItem::Describes(group) => Some(type_info_from_parts(
            shape,
            &group.heading,
            group.using.as_ref(),
            group.when.as_ref(),
            Some(&group.describes.argument),
        )),
        TopLevelItem::Defines(group) => Some(type_info_from_parts(
            shape,
            &group.heading,
            group.using.as_ref(),
            group.when.as_ref(),
            None,
        )),
        TopLevelItem::Refines(group) => Some(type_info_from_parts(
            shape,
            &group.heading,
            group.using.as_ref(),
            group.when.as_ref(),
            None,
        )),
        TopLevelItem::States(group) => Some(type_info_from_parts(
            shape,
            &group.heading,
            group.using.as_ref(),
            group.when.as_ref(),
            None,
        )),
        TopLevelItem::Axiom(group) => group
            .heading
            .as_ref()
            .map(|heading| type_info_from_parts(shape, heading, None, None, None)),
        TopLevelItem::Theorem(group) => group
            .heading
            .as_ref()
            .map(|heading| type_info_from_parts(shape, heading, None, None, None)),
        TopLevelItem::Corollary(group) => group
            .heading
            .as_ref()
            .map(|heading| type_info_from_parts(shape, heading, None, None, None)),
        TopLevelItem::Lemma(group) => group
            .heading
            .as_ref()
            .map(|heading| type_info_from_parts(shape, heading, None, None, None)),
        TopLevelItem::Conjecture(group) => group
            .heading
            .as_ref()
            .map(|heading| type_info_from_parts(shape, heading, None, None, None)),
        _ => None,
    }
}

fn type_info_from_parts(
    shape: &SignatureShape,
    heading: &CommandHeader,
    using: Option<&UsingSection>,
    when: Option<&WhenSection>,
    described: Option<&FormOrDeclaration>,
) -> DefinitionTypeInfo {
    let mut context = TypeContext::default();

    if let Some(using) = using {
        for spec in &using.arguments {
            for fact in facts_from_is_or_spec(spec) {
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
        signature: shape.signature.clone(),
        parameters: header_parameter_names(heading),
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
        TopLevelItem::Describes(group) => {
            let mut context = TypeContext::default();
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
        | TopLevelItem::Person(_)
        | TopLevelItem::Resource(_) => {}
    }
}

struct TheoremLikeSections<'a> {
    given: Option<&'a GivenSection>,
    where_: Option<&'a WhereSection>,
    then: &'a ThenSection,
    iff: Option<&'a IffSection>,
}

impl<'a> TheoremLikeSections<'a> {
    fn new(
        given: Option<&'a GivenSection>,
        where_: Option<&'a WhereSection>,
        then: &'a ThenSection,
        iff: Option<&'a IffSection>,
    ) -> Self {
        Self {
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

    if let Some(given) = sections.given {
        for spec in &given.arguments {
            check_is_or_refined_spec(spec, &context, path, locator, registry, event_log);
            for fact in facts_from_is_or_refined_spec(spec) {
                context.add_fact(fact);
            }
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
        for spec in &using.arguments {
            check_is_or_spec(spec, context, path, locator, registry, event_log);
            for fact in facts_from_is_or_spec(spec) {
                context.add_fact(fact);
            }
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
    check_clause(clause, context, path, locator, registry, event_log);
    collect_clause_assumptions(clause, context);
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
            check_is_or_refined_spec(
                &group.given.argument,
                &child,
                path,
                locator,
                registry,
                event_log,
            );
            for fact in facts_from_is_or_refined_spec(&group.given.argument) {
                child.add_fact(fact);
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
        Clause::Binding(binding) => {
            check_expression(&binding.left, context, path, locator, registry, event_log);
            check_expression(&binding.right, context, path, locator, registry, event_log);
        }
        Clause::IsOrSpec(spec) => {
            check_is_or_spec(spec, context, path, locator, registry, event_log)
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
        BindingOrSpec::Binding(binding) => {
            check_expression(&binding.left, context, path, locator, registry, event_log);
            check_expression(&binding.right, context, path, locator, registry, event_log);
            context.add_substitution(
                key_for_expression(&binding.left),
                key_for_expression(&binding.right),
            );
        }
        BindingOrSpec::IsOrSpec(spec) => {
            check_is_or_spec(spec, context, path, locator, registry, event_log);
            for fact in facts_from_is_or_spec(spec) {
                context.add_fact(fact);
            }
        }
    }
}

fn collect_clause_assumptions(clause: &Clause, context: &mut TypeContext) {
    match clause {
        Clause::Binding(binding) => {
            context.add_substitution(
                key_for_expression(&binding.left),
                key_for_expression(&binding.right),
            );
        }
        Clause::IsOrSpec(spec) => {
            for fact in facts_from_is_or_spec(spec) {
                context.add_fact(fact);
            }
        }
        Clause::Expression(expression) => {
            if let Some(fact) = fact_from_expression(expression) {
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

fn check_is_or_spec(
    spec: &IsOrSpec,
    context: &TypeContext,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    registry: &SignatureRegistry,
    event_log: &mut EventLog,
) {
    match spec {
        IsOrSpec::Is(statement) => {
            check_is_statement(statement, context, path, locator, registry, event_log)
        }
        IsOrSpec::Spec(_) => {}
    }
}

fn check_is_or_refined_spec(
    spec: &IsOrRefinedStatementSpec,
    context: &TypeContext,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    registry: &SignatureRegistry,
    event_log: &mut EventLog,
) {
    match spec {
        IsOrRefinedStatementSpec::Is(statement) => {
            check_is_statement(statement, context, path, locator, registry, event_log);
        }
        IsOrRefinedStatementSpec::Spec(_) => {}
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
        ExpressionKind::Name(_) => {}
        ExpressionKind::FunctionCall { arguments, .. } => {
            for argument in arguments {
                check_expression(argument, context, path, locator, registry, event_log);
            }
        }
        ExpressionKind::FunctionNamedCall { elements, .. } => {
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
            if let Some(predicate) = &set.predicate {
                check_expression(predicate, context, path, locator, registry, event_log);
            }
        }
        ExpressionKind::Grouped { expression, .. }
        | ExpressionKind::Labeled { expression, .. }
        | ExpressionKind::Prefix { expression, .. } => {
            check_expression(expression, context, path, locator, registry, event_log);
        }
        ExpressionKind::SubsetCall(_) => {}
        ExpressionKind::Command(command) => {
            check_command_expression(command, context, path, locator, registry, event_log);
            for expression in command_expression_arguments(command) {
                check_expression(expression, context, path, locator, registry, event_log);
            }
        }
        ExpressionKind::InfixCommand {
            left,
            command,
            right,
        } => {
            check_expression(left, context, path, locator, registry, event_log);
            check_infix_command(command, context, path, locator, registry, event_log);
            for expression in infix_command_arguments(command) {
                check_expression(expression, context, path, locator, registry, event_log);
            }
            check_expression(right, context, path, locator, registry, event_log);
        }
        ExpressionKind::Binary { left, right, .. } => {
            check_expression(left, context, path, locator, registry, event_log);
            check_expression(right, context, path, locator, registry, event_log);
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
        }
        ExpressionKind::IsPredicate { subject, command }
        | ExpressionKind::IsNotPredicate { subject, command } => {
            check_expression(subject, context, path, locator, registry, event_log);
            check_command_expression(command, context, path, locator, registry, event_log);
        }
        ExpressionKind::IsType { subject, ty } => {
            check_expression(subject, context, path, locator, registry, event_log);
            check_type_expression(ty, context, path, locator, registry, event_log);
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
        TypeExpression::Command(command) => {
            check_command_expression(command, context, path, locator, registry, event_log);
        }
        TypeExpression::RefinedCommand(_) => {}
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
    let shape = shape_for_command_expression(command);
    let position = locator.locate_reference(&shape);
    let actuals = command_expression_arguments(command)
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

fn check_infix_command(
    command: &InfixCommand,
    context: &TypeContext,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    registry: &SignatureRegistry,
    event_log: &mut EventLog,
) {
    let shape = shape_for_infix_command(command);
    let position = locator.locate_reference(&shape);
    let actuals = infix_command_arguments(command)
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
        requirement_context.add_substitution(
            substitute_key(left, &substitutions),
            substitute_key(right, &substitutions),
        );
    }

    for requirement in &info.requirements {
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

fn prove_fact(required: &TypeFact, context: &TypeContext, registry: &SignatureRegistry) -> bool {
    let required = context.normalize_fact(required);
    if context
        .facts
        .iter()
        .any(|fact| context.normalize_fact(fact) == required)
    {
        return true;
    }

    let mut seen = HashSet::new();
    for fact in &context.facts {
        let TypeFact::Spec { .. } = fact else {
            continue;
        };
        for reduced in reduce_spec_fact(fact, context, registry, &mut seen) {
            if context.normalize_fact(&reduced) == required {
                return true;
            }
        }
    }

    false
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

        if !has_type_signature(target, &rule.owner_signature, context) {
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

fn has_type_signature(subject: &str, signature: &str, context: &TypeContext) -> bool {
    context
        .facts
        .iter()
        .any(|fact| match context.normalize_fact(fact) {
            TypeFact::Is {
                subject: fact_subject,
                signature: fact_signature,
                ..
            } => fact_subject == context.normalize_key(subject) && fact_signature == signature,
            TypeFact::Spec { .. } => false,
        })
}

#[derive(Clone, Default)]
struct TypeContext {
    facts: Vec<TypeFact>,
    substitutions: Vec<(String, String)>,
}

impl TypeContext {
    fn add_fact(&mut self, fact: TypeFact) {
        self.facts.push(fact);
    }

    fn add_substitution(&mut self, left: String, right: String) {
        self.substitutions.push((left, right));
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

fn facts_from_is_or_refined_spec(spec: &IsOrRefinedStatementSpec) -> Vec<TypeFact> {
    match spec {
        IsOrRefinedStatementSpec::Is(statement) => facts_from_is_statement(statement),
        IsOrRefinedStatementSpec::Spec(statement) => vec![TypeFact::Spec {
            subject: key_for_spec_subject(&statement.subject),
            operator: statement.operator.clone(),
            target: statement.name.clone(),
        }],
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

fn facts_from_is_statement(statement: &IsStatement) -> Vec<TypeFact> {
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
        ExpressionKind::IsType { subject, ty } => {
            let (ty, signature) = key_for_type_expression(ty)?;
            Some(TypeFact::Is {
                subject: key_for_expression(subject),
                ty,
                signature,
            })
        }
        ExpressionKind::SpecStatement(statement) => Some(TypeFact::Spec {
            subject: key_for_expression(&statement.subject),
            operator: statement.operator.clone(),
            target: statement.name.clone(),
        }),
        _ => None,
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

fn key_for_spec_subject(subject: &SpecSubject) -> String {
    match &subject.kind {
        SpecSubjectKind::Form(form) => key_for_form_or_declaration(form),
        SpecSubjectKind::Operator(operator) => operator.text.clone(),
    }
}

fn key_for_type_expression(ty: &TypeExpression) -> Option<(String, String)> {
    match ty {
        TypeExpression::Command(command) => Some((
            key_for_command_expression(command),
            shape_for_command_expression(command).signature,
        )),
        TypeExpression::RefinedCommand(_) => None,
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
            set.spec.name,
            key_for_placeholder_form(&set.target)
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
        ExpressionKind::Prefix {
            operator,
            expression,
        } => format!("{operator:?}{}", key_for_expression(expression)),
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
        ExpressionKind::IsType { subject, ty } => format!(
            "{} is {}",
            key_for_expression(subject),
            key_for_type_expression(ty)
                .map(|(key, _)| key)
                .unwrap_or_else(|| "<refined>".to_owned())
        ),
    }
}

fn key_for_named_expression_lhs(
    lhs: &crate::frontend::formulation::ast::FunctionNamedExpressionElementLhs,
) -> String {
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

fn key_for_infix_command(command: &InfixCommand) -> String {
    let mut key = format!("\\:{}", format_chain(&command.chain));
    append_expression_args(&mut key, &command.head_args);
    for tail in &command.tail {
        key.push(':');
        key.push_str(&format_chain(&tail.chain));
        append_expression_args(&mut key, &tail.args);
    }
    key.push_str(":/");
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

fn header_parameter_names(header: &CommandHeader) -> Vec<String> {
    match header {
        CommandHeader::Command(command) => command_header_parameter_names(command),
        CommandHeader::Infix(command) => infix_header_parameter_names(command),
        CommandHeader::Refined(command) => refined_header_parameter_names(command),
    }
}

fn command_header_parameter_names(command: &CommandHeaderNode) -> Vec<String> {
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
        .filter_map(primary_form_name)
        .collect()
}

fn infix_header_parameter_names(command: &InfixCommandHeader) -> Vec<String> {
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
        .filter_map(primary_form_name)
        .collect()
}

fn refined_header_parameter_names(command: &RefinedCommandHeader) -> Vec<String> {
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
        .filter_map(primary_form_name)
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
    }
}

use super::*;
use std::collections::{BTreeSet, HashMap, HashSet};
use std::rc::Rc;

pub(super) fn collect_numeric_specifications(
    files: &[ParsedSourceFile],
    registry: &mut SignatureRegistry,
    event_log: &mut EventLog,
) {
    for file in files {
        for (item_index, item) in file.document.items.iter().enumerate() {
            let TopLevelItem::Specify(group) = item else {
                continue;
            };
            let row = file
                .item_ids
                .get(item_index)
                .map(|id| id.group_row)
                .unwrap_or_default();
            for item in &group.specify.arguments {
                let (label, specification, target) = match item {
                    SpecifyItem::Decimal(group) => (
                        "decimal",
                        &group.is_.argument,
                        &mut registry.numeric_specifications.decimal,
                    ),
                    SpecifyItem::ZeroOrPositiveInt(group) => (
                        "zeroOrPositiveInt",
                        &group.is_.argument,
                        &mut registry.numeric_specifications.zero_or_positive_int,
                    ),
                    SpecifyItem::PositiveInt(group) => (
                        "positiveInt",
                        &group.is_.argument,
                        &mut registry.numeric_specifications.positive_int,
                    ),
                    SpecifyItem::Int(group) => (
                        "int",
                        &group.is_.argument,
                        &mut registry.numeric_specifications.int,
                    ),
                };
                let Some((ty, signature)) = key_for_type_expression(specification) else {
                    event_log.user_error_at_file_row(
                        Some(ORIGIN),
                        file.path.clone(),
                        row,
                        format!("Specify `{label}: is:` must name a nominal or built-in type"),
                    );
                    continue;
                };
                if target.is_some() {
                    event_log.user_error_at_file_row(
                        Some(ORIGIN),
                        file.path.clone(),
                        row,
                        format!("Specify `{label}:` may be declared only once per collection"),
                    );
                    continue;
                }
                *target = Some(NumericTypeSpecification { ty, signature });
            }
        }
    }
}

pub(super) fn collect_definition_type_metadata(
    item: &TopLevelItem,
    header_shape: &HeaderShape,
    registry: &mut SignatureRegistry,
) {
    let Some(info) = definition_type_info(item, header_shape, registry) else {
        return;
    };

    collect_type_extension_rules(item, &info, registry);
    collect_refinement_extension_rules(item, &info, registry);
    collect_spec_operator_rules(item, &info, registry);
    collect_provided_symbol_rules(item, &info, registry);
    collect_cast_as_rules(item, &info, registry);
    collect_viewable_rules(item, &info, registry);
    collect_abstraction_rules(item, &info, registry);
    collect_collection_type_signature(item, &info, registry);
    collect_abstract_declaration(item, header_shape, registry);
    let mut info = info;
    inherit_realized_component_types(item, &mut info, registry);
    collect_collection_body(item, header_shape, registry);
    collect_equivalence_class(item, header_shape, registry);
    registry.definition_summaries.insert(
        header_shape.shape.signature.clone(),
        DefinitionSummary {
            target_shape: target_shape_of_item(item),
        },
    );
    registry
        .type_infos
        .insert(header_shape.shape.signature.clone(), info);
}

/// Gives a realization the component types of the declaration it realizes.
///
/// Realizing a declaration means supplying values for the symbols it specified,
/// so the components keep the types the declaration gave them while the
/// realization's own `means:` says what they are. Copying the declaration's list
/// wholesale also keeps the components in tuple order, which is how they are
/// matched to a destructuring binding.
fn inherit_realized_component_types(
    item: &TopLevelItem,
    info: &mut DefinitionTypeInfo,
    registry: &SignatureRegistry,
) {
    let TopLevelItem::Realizes(group) = item else {
        return;
    };
    let Some(definition) = group.realizes.argument.definition.as_ref() else {
        return;
    };
    let Some(signature) = command_signature_from_key(&key_for_expression(definition)) else {
        return;
    };
    let Some(declared) = registry.type_infos.get(&signature) else {
        return;
    };
    if declared.component_types.is_empty() {
        return;
    }
    info.component_types = declared.component_types.clone();
    info.component_shapes = declared.component_shapes.clone();
}

/// Records what a `Declares:` marked `abstractly:` leaves for a `Realizes:` to
/// supply: every `means:` item that states a specification but no value.
fn collect_abstract_declaration(
    item: &TopLevelItem,
    header_shape: &HeaderShape,
    registry: &mut SignatureRegistry,
) {
    let TopLevelItem::Declares(group) = item else {
        return;
    };
    if !group.abstractly {
        return;
    }
    let expressed = expresses_bound_symbols(&group.expresses);
    let context = TypeContext::default();
    let mut abstract_facts = Vec::new();
    for statement in means_statements(&group.means) {
        if !statement_is_abstract(statement, &expressed) {
            continue;
        }
        abstract_facts.extend(facts_from_declaration_statement_in_context(
            statement, &context,
        ));
    }

    registry.abstract_declarations.insert(
        header_shape.shape.signature.clone(),
        AbstractDeclaration { abstract_facts },
    );
}

/// The declaration statements a `means:` section states, skipping items that
/// carry no statement of their own (a `have:` group).
fn means_statements(
    means: &Option<DeclaresMeansSection>,
) -> impl Iterator<Item = &DeclarationStatement> {
    means
        .iter()
        .flat_map(|means| means.arguments.iter())
        .filter_map(means_item_statement)
}

fn means_item_statement(item: &IsOrViaItem) -> Option<&DeclarationStatement> {
    match item {
        IsOrViaItem::Declaration(statement) => Some(statement),
        IsOrViaItem::Labeled { item, .. } => means_item_statement(item),
        IsOrViaItem::IsVia(_) | IsOrViaItem::Have(_) => None,
    }
}

/// Whether a `means:` item leaves its subject abstract: it states a type or
/// specification but supplies no value, directly or through `expresses:`.
fn statement_is_abstract(statement: &DeclarationStatement, expressed: &BTreeSet<String>) -> bool {
    statement.definition.is_none()
        && statement.relation.is_some()
        && !declaration_subject_keys(statement)
            .iter()
            .all(|subject| expressed.contains(subject))
}

/// The symbols an `expresses:` section supplies a value for.
fn expresses_bound_symbols(expresses: &Option<ExpressesSection>) -> BTreeSet<String> {
    let mut bound = BTreeSet::new();
    let Some(expresses) = expresses else {
        return bound;
    };
    for clause in &expresses.arguments {
        collect_clause_bound_symbols(clause, &mut bound);
    }
    bound
}

/// The symbols a clause supplies a value for. A `piecewise:` group defines the
/// mapping its branches assign to, so its own assignments count too.
fn collect_clause_bound_symbols(clause: &Clause, bound: &mut BTreeSet<String>) {
    match clause {
        Clause::Declaration(statement) => {
            if statement.definition.is_some() {
                bound.extend(declaration_subject_keys(statement));
            }
        }
        Clause::Expression(expression) => {
            if let ExpressionKind::Binary { operator, left, .. } = &expression.kind
                && matches!(operator, BinaryOperator::Equality(_))
            {
                bound.insert(key_for_expression(left));
            }
        }
        Clause::Piecewise(group) => {
            for clause in &group.then.arguments {
                collect_clause_bound_symbols(clause, bound);
            }
            if let Some(else_) = &group.else_ {
                for clause in &else_.arguments {
                    collect_clause_bound_symbols(clause, bound);
                }
            }
        }
        _ => {}
    }
}

/// Records a set-defining command's body (`Declares: X := {x_ : ...} is \set`) so
/// membership in a use of the command can reduce to the body's element condition.
fn collect_collection_body(
    item: &TopLevelItem,
    header_shape: &HeaderShape,
    registry: &mut SignatureRegistry,
) {
    let TopLevelItem::Declares(group) = item else {
        return;
    };
    let Some(definition) = &group.declares.argument.definition else {
        return;
    };
    let body = match &definition.kind {
        ExpressionKind::Set(set) => Some(set),
        _ => cast_expression_set_literal(definition),
    };
    if let Some(body) = body {
        registry
            .collection_bodies
            .insert(header_shape.shape.signature.clone(), body.clone());
    }
}

/// Record the equivalence class declared by a top-level `Equivalent:` item: the
/// class-naming header plus each `to:` member, each paired with the header
/// parameters its arguments use. `to:` members whose arguments are not all bare
/// names are skipped here (they are reported by `validate_equivalent_item`).
fn collect_equivalence_class(
    item: &TopLevelItem,
    header_shape: &HeaderShape,
    registry: &mut SignatureRegistry,
) {
    let TopLevelItem::Equivalent(group) = item else {
        return;
    };
    // A multi-shape header collects the class once, under its primary shape.
    if registry
        .equivalence_classes
        .iter()
        .any(|class| class.member(&header_shape.shape.signature).is_some())
    {
        return;
    }

    let mut members = vec![EquivalenceMember {
        signature: header_shape.shape.signature.clone(),
        params: header_shape.parameters.clone(),
    }];
    for expression in &group.to.arguments {
        let ExpressionKind::Command(command) = &expression.kind else {
            continue;
        };
        let Some(params) = command_bare_name_arguments(command) else {
            continue;
        };
        members.push(EquivalenceMember {
            signature: shape_for_command_expression(command).signature,
            params,
        });
    }

    if members.len() >= 2 {
        registry
            .equivalence_classes
            .push(EquivalenceClass { members });
    }
}

/// The bare `Name` arguments of a command, in order, or `None` if any argument is
/// not a bare name.
fn command_bare_name_arguments(command: &CommandExpression) -> Option<Vec<String>> {
    command_argument_expressions(command)
        .into_iter()
        .map(|argument| match &argument.kind {
            ExpressionKind::Name(name) => Some(name.clone()),
            _ => None,
        })
        .collect()
}

/// The form of the object `item` declares (rule 2 of `Equivalent:`).
fn target_shape_of_item(item: &TopLevelItem) -> TargetShape {
    match item {
        TopLevelItem::Defines(group) => defines_target_shape(&group.defines.argument),
        TopLevelItem::Declares(group) => is_subject_shape(&group.declares.argument.subject),
        TopLevelItem::Realizes(group) => is_subject_shape(&group.realizes.argument.subject),
        TopLevelItem::Refines(group) => group
            .refines
            .argument
            .expansion
            .as_ref()
            .map(is_subject_shape)
            .unwrap_or_else(|| is_subject_shape(&group.refines.argument.subject)),
        TopLevelItem::States(_) => TargetShape::Statement,
        _ => TargetShape::Other,
    }
}

fn defines_target_shape(target: &DefinesTarget) -> TargetShape {
    match target {
        DefinesTarget::Form(form) => form_shape(form),
        DefinesTarget::Declaration(statement) => is_subject_shape(&statement.subject),
    }
}

fn is_subject_shape(subject: &IsSubject) -> TargetShape {
    match &subject.kind {
        IsSubjectKind::Operator(_) => TargetShape::Operator,
        IsSubjectKind::Forms(forms) => match forms.as_slice() {
            [IsSubjectForm::Form(form)] => form_shape(form),
            _ => TargetShape::Other,
        },
    }
}

fn form_shape(form: &FormOrDeclaration) -> TargetShape {
    match &form.kind {
        FormOrDeclarationKind::Name(_) | FormOrDeclarationKind::MappingParameter { .. } => {
            TargetShape::Name
        }
        FormOrDeclarationKind::FunctionDeclaration { form, .. } => {
            TargetShape::Function(form.placeholders.len())
        }
        FormOrDeclarationKind::TupleDeclaration { form, .. } => {
            TargetShape::Tuple(form.elements.len())
        }
        FormOrDeclarationKind::SetDeclaration { .. } => TargetShape::Set,
        FormOrDeclarationKind::InfixOperator { .. }
        | FormOrDeclarationKind::PrefixOperator { .. }
        | FormOrDeclarationKind::PostfixOperator { .. } => TargetShape::Operator,
    }
}

fn collect_collection_type_signature(
    item: &TopLevelItem,
    info: &DefinitionTypeInfo,
    registry: &mut SignatureRegistry,
) {
    let TopLevelItem::Defines(group) = item else {
        return;
    };
    if !defines_target_is_collection(&group.defines.argument) {
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

fn defines_target_is_collection(target: &DefinesTarget) -> bool {
    match target {
        DefinesTarget::Form(FormOrDeclaration {
            kind: FormOrDeclarationKind::SetDeclaration { .. },
            ..
        }) => true,
        DefinesTarget::Declaration(statement) => declaration_has_collection_literal(statement),
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
    ) || statement
        .definition
        .as_ref()
        .and_then(cast_expression_set_literal)
        .is_some()
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
    for (index, item) in file.document.items.iter().enumerate() {
        begin_type_info_item(file, index, registry);
        validate_top_level_item_types(item, file.path.as_path(), &mut locator, registry, event_log);
    }
}

/// Narrows an active type-info recording to the rows of the item about to be
/// walked, so a formulation spelled identically in two items cannot claim the
/// other item's line.
fn begin_type_info_item(file: &ParsedSourceFile, index: usize, registry: &SignatureRegistry) {
    let mut slot = registry.recorder.borrow_mut();
    let Some(recorder) = slot.as_mut() else {
        return;
    };
    let start = file
        .item_ids
        .get(index)
        .map(|id| id.group_row)
        .unwrap_or_default();
    let end = file
        .item_ids
        .get(index + 1)
        .map(|id| id.group_row)
        .unwrap_or(usize::MAX);
    recorder.begin_item(start..end);
}

/// Records the types of `expression` and every sub-expression beneath it, when
/// it is the formulation of a line a type-info pass is collecting.
///
/// Called wherever the walk first reaches a whole formulation — a clause being
/// checked, a clause being assumed — and does nothing at all when no type-info
/// pass is running.
fn record_line_types(expression: &Expression, context: &TypeContext, registry: &SignatureRegistry) {
    let claim = match registry.recorder.borrow_mut().as_mut() {
        Some(recorder) => recorder.claim_expression(expression),
        None => return,
    };
    let Some(claim) = claim else { return };

    // Resolved outside the borrow above: resolving a type reads the registry,
    // and the recorder is borrowed again only to file the finished entries.
    let mut entries = Vec::new();
    collect_type_entries(
        expression,
        0,
        Some(claim.text.clone()),
        context,
        registry,
        &mut entries,
    );
    if let Some(recorder) = registry.recorder.borrow_mut().as_mut() {
        recorder.record(claim, entries);
    }
}

/// The declaration-statement counterpart of [`record_line_types`].
fn record_declaration_line_types(
    statement: &DeclarationStatement,
    context: &TypeContext,
    registry: &SignatureRegistry,
) {
    let claim = match registry.recorder.borrow_mut().as_mut() {
        Some(recorder) => recorder.claim_declaration(statement),
        None => return,
    };
    let Some(claim) = claim else { return };

    let mut entries = vec![TypeEntry {
        depth: 0,
        text: claim.text.clone(),
        types: facts_from_declaration_statement(statement)
            .iter()
            .map(|fact| format!("asserts {}", format_fact(fact)))
            .collect(),
    }];
    if let Some(definition) = &statement.definition {
        collect_type_entries(definition, 1, None, context, registry, &mut entries);
    }
    match &statement.relation {
        Some(DeclarationRelation::Spec { target, .. })
        | Some(DeclarationRelation::InfixSpec { target, .. }) => {
            collect_type_entries(target, 1, None, context, registry, &mut entries);
        }
        Some(DeclarationRelation::Is(_)) | None => {}
    }
    if let Some(recorder) = registry.recorder.borrow_mut().as_mut() {
        recorder.record(claim, entries);
    }
}

/// Appends `expression` and, beneath it, each of its sub-expressions.
fn collect_type_entries(
    expression: &Expression,
    depth: usize,
    label: Option<String>,
    context: &TypeContext,
    registry: &SignatureRegistry,
    entries: &mut Vec<TypeEntry>,
) {
    entries.push(TypeEntry {
        depth,
        text: label.unwrap_or_else(|| key_for_expression(expression)),
        types: resolved_type_predicates(expression, context, registry),
    });
    for child in sub_expressions(expression) {
        collect_type_entries(child, depth + 1, None, context, registry, entries);
    }
}

/// The immediate sub-expressions of `expression`, in source order.
fn sub_expressions(expression: &Expression) -> Vec<&Expression> {
    match &expression.kind {
        ExpressionKind::Name(_)
        | ExpressionKind::InferredName(_)
        | ExpressionKind::VariadicSlice(_)
        | ExpressionKind::SubsetCall(_)
        | ExpressionKind::BuiltinCommand(_) => Vec::new(),
        ExpressionKind::VariadicAssignment { value, .. } => vec![value.as_ref()],
        ExpressionKind::FunctionCall { arguments, .. } => arguments.iter().collect(),
        ExpressionKind::FunctionNamedCall { elements, .. } => {
            elements.iter().map(|element| &element.expression).collect()
        }
        ExpressionKind::MemberCall {
            owner, arguments, ..
        } => std::iter::once(owner.as_ref()).chain(arguments).collect(),
        ExpressionKind::MemberAccess { owner, .. } => vec![owner.as_ref()],
        ExpressionKind::Tuple(elements) => elements
            .iter()
            .filter_map(|element| match element {
                TupleExpressionElement::Expression(expression) => Some(expression),
                TupleExpressionElement::Operator(_) => None,
            })
            .collect(),
        ExpressionKind::Set(set) => set.specs.iter().collect(),
        ExpressionKind::Grouped { expression, .. } | ExpressionKind::Labeled { expression, .. } => {
            vec![expression.as_ref()]
        }
        ExpressionKind::IndexedCall(call) => call.indices.iter().collect(),
        ExpressionKind::Command(command) => command_expression_arguments(command),
        ExpressionKind::InfixCommand {
            left,
            command,
            right,
        } => std::iter::once(left.as_ref())
            .chain(infix_command_arguments(command))
            .chain(std::iter::once(right.as_ref()))
            .collect(),
        ExpressionKind::InfixSpecStatement { left, right, .. } => {
            vec![left.as_ref(), right.as_ref()]
        }
        ExpressionKind::Prefix { expression, .. } | ExpressionKind::Postfix { expression, .. } => {
            vec![expression.as_ref()]
        }
        ExpressionKind::Binary { left, right, .. } => vec![left.as_ref(), right.as_ref()],
        ExpressionKind::SpecStatement(statement) | ExpressionKind::SpecPredicate(statement) => {
            vec![statement.subject.as_ref()]
        }
        ExpressionKind::SpecStatementExpr {
            subject, target, ..
        } => vec![subject.as_ref(), target.as_ref()],
        ExpressionKind::SpecLiteral(literal) => match &literal.form {
            SpecLiteralForm::Spec { target, .. } => vec![target.as_ref()],
            SpecLiteralForm::Is(_) => Vec::new(),
        },
        ExpressionKind::Satisfies { subject, spec } => vec![subject.as_ref(), spec.as_ref()],
        ExpressionKind::Mapping { lhs, rhs } => vec![lhs.as_ref(), rhs.as_ref()],
        ExpressionKind::IsPredicate { subject, command }
        | ExpressionKind::IsNotPredicate { subject, command } => std::iter::once(subject.as_ref())
            .chain(command_expression_arguments(command))
            .collect(),
        ExpressionKind::IsRefinedPredicate { subject, .. }
        | ExpressionKind::IsNotRefinedPredicate { subject, .. }
        | ExpressionKind::IsBuiltinPredicate { subject, .. }
        | ExpressionKind::IsNotBuiltinPredicate { subject, .. }
        | ExpressionKind::IsType { subject, .. } => vec![subject.as_ref()],
        ExpressionKind::Build { value, .. } => vec![value.as_ref()],
        ExpressionKind::MemberOf {
            subject,
            collection,
        } => vec![subject.as_ref(), collection.as_ref()],
    }
}

/// What the checker knows about `expression`, rendered as predicates about it.
///
/// A value reports the facts its result carries; a name that carries none of its
/// own reports what the context declared about it; and a statement — which has
/// no value type — reports the fact it asserts.
fn resolved_type_predicates(
    expression: &Expression,
    context: &TypeContext,
    registry: &SignatureRegistry,
) -> Vec<String> {
    let subject = effective_key_for_expression(expression, context, registry);
    let mut resolving = HashSet::new();
    let mut facts =
        expression_result_facts(expression, &subject, context, registry, &mut resolving);

    if facts.is_empty() {
        let normalized = context.normalize_key(&subject);
        facts = context
            .facts
            .iter()
            .filter(|fact| {
                let fact_subject = fact_subject(fact);
                fact_subject == subject || fact_subject == normalized
            })
            .cloned()
            .collect();
    }

    if facts.is_empty() {
        if let Some(asserted) = fact_from_expression_in_context(expression, context) {
            return vec![format!("asserts {}", format_fact(&asserted))];
        }
        if is_statement_shaped(expression) || is_statement_command(&subject, registry) {
            return vec![format!("is {BUILTIN_STATEMENT_SIGNATURE}")];
        }
    }

    let mut types: Vec<String> = facts.iter().map(format_fact_predicate).collect();
    types.sort();
    types.dedup();
    types
}

/// Whether an expression asserts something rather than denoting a value, judged
/// from its shape alone. Used only as a fallback, when no fact could be resolved
/// and none could be read off the expression either.
fn is_statement_shaped(expression: &Expression) -> bool {
    match &expression.kind {
        ExpressionKind::IsType { .. }
        | ExpressionKind::IsPredicate { .. }
        | ExpressionKind::IsNotPredicate { .. }
        | ExpressionKind::IsBuiltinPredicate { .. }
        | ExpressionKind::IsNotBuiltinPredicate { .. }
        | ExpressionKind::IsRefinedPredicate { .. }
        | ExpressionKind::IsNotRefinedPredicate { .. }
        | ExpressionKind::SpecStatement(_)
        | ExpressionKind::SpecPredicate(_)
        | ExpressionKind::SpecStatementExpr { .. }
        | ExpressionKind::InfixSpecStatement { .. }
        | ExpressionKind::MemberOf { .. }
        | ExpressionKind::Satisfies { .. } => true,
        ExpressionKind::Binary { operator, .. } => {
            matches!(operator, BinaryOperator::Equality(_))
        }
        ExpressionKind::Grouped { expression, .. } | ExpressionKind::Labeled { expression, .. } => {
            is_statement_shaped(expression)
        }
        _ => false,
    }
}

/// Whether an expression's key names a command declared by an item that states
/// something — those have no value type; they are statements.
fn is_statement_command(key: &str, registry: &SignatureRegistry) -> bool {
    let Some(signature) = command_signature_from_key(key) else {
        return false;
    };
    registry.definitions.get(&signature).is_some_and(|entry| {
        matches!(
            entry.kind,
            DefinitionKind::States
                | DefinitionKind::Axiom
                | DefinitionKind::Theorem
                | DefinitionKind::Conjecture
        )
    })
}

fn definition_type_info(
    item: &TopLevelItem,
    header_shape: &HeaderShape,
    registry: &SignatureRegistry,
) -> Option<DefinitionTypeInfo> {
    match item {
        TopLevelItem::Defines(group) => Some(type_info_from_parts(
            header_shape,
            &group.heading,
            group.using.as_ref(),
            None,
            group.when.as_ref(),
            None,
            Some(&group.defines),
            group.means.as_ref(),
            group.extends.as_ref(),
            None,
            registry,
        )),
        TopLevelItem::Declares(group) => Some(type_info_from_parts(
            header_shape,
            &group.heading,
            group.using.as_ref(),
            None,
            group.when.as_ref(),
            Some(&group.declares.argument),
            None,
            None,
            None,
            group.means.as_ref(),
            registry,
        )),
        TopLevelItem::Realizes(group) => Some(type_info_from_parts(
            header_shape,
            &group.heading,
            group.using.as_ref(),
            None,
            group.when.as_ref(),
            Some(&group.realizes.argument),
            None,
            None,
            None,
            group.means.as_ref(),
            registry,
        )),
        TopLevelItem::Refines(group) => Some(type_info_from_parts(
            header_shape,
            &group.heading,
            group.using.as_ref(),
            None,
            group.when.as_ref(),
            None,
            None,
            None,
            None,
            None,
            registry,
        )),
        TopLevelItem::States(group) => Some(type_info_from_parts(
            header_shape,
            &group.heading,
            group.using.as_ref(),
            None,
            group.when.as_ref(),
            None,
            None,
            None,
            None,
            None,
            registry,
        )),
        TopLevelItem::Axiom(group) => group.heading.as_ref().map(|heading| {
            type_info_from_parts(
                header_shape,
                heading,
                None,
                group.given.as_ref(),
                None,
                None,
                None,
                None,
                None,
                None,
                registry,
            )
        }),
        TopLevelItem::Theorem(group) => group.heading.as_ref().map(|heading| {
            type_info_from_parts(
                header_shape,
                heading,
                None,
                group.given.as_ref(),
                None,
                None,
                None,
                None,
                None,
                None,
                registry,
            )
        }),
        TopLevelItem::Conjecture(group) => group.heading.as_ref().map(|heading| {
            type_info_from_parts(
                header_shape,
                heading,
                None,
                group.given.as_ref(),
                None,
                None,
                None,
                None,
                None,
                None,
                registry,
            )
        }),
        TopLevelItem::Equivalent(group) => Some(type_info_from_parts(
            header_shape,
            &group.heading,
            group.using.as_ref(),
            None,
            group.when.as_ref(),
            None,
            None,
            None,
            None,
            None,
            registry,
        )),
        _ => None,
    }
}

fn type_info_from_parts(
    header_shape: &HeaderShape,
    heading: &CommandHeader,
    using: Option<&UsingSection>,
    given: Option<&GivenSection>,
    when: Option<&WhenSection>,
    declares: Option<&DeclarationStatement>,
    defines: Option<&DefinesSection>,
    defines_means: Option<&DefinesMeansSection>,
    extends: Option<&ExtendsSection>,
    declares_means: Option<&DeclaresMeansSection>,
    registry: &SignatureRegistry,
) -> DefinitionTypeInfo {
    let described = defines.map(|defines| &defines.argument);
    let mut context = TypeContext::default();
    declare_header_symbols(heading, &mut context, registry);
    let using_parameters = collect_using_parameter_names(using);
    let given_parameters = collect_given_parameter_names(given);

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

    if let Some(given) = given {
        for statement in &given.arguments {
            declare_is_subject(&statement.subject, &mut context);
            if let Some(expansion) = &statement.expansion {
                declare_is_subject(expansion, &mut context);
            }
            for fact in facts_from_declaration_statement_in_context(statement, &context) {
                context.add_fact(fact);
            }
            if let Some((left, right)) = declaration_substitution(statement) {
                context.add_substitution(left, right);
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
    let mut outputs: Vec<TypeFact> = declares
        .map(|statement| {
            facts_from_declaration_statement_in_context(statement, &context)
                .into_iter()
                .map(|fact| context.normalize_fact(&fact))
                .collect()
        })
        .unwrap_or_default();
    if let Some(fact) = described.zip(defines_means).and_then(|(target, declares)| {
        function_type_fact_from_defines_means(target, declares, &context)
    }) {
        outputs.push(context.normalize_fact(&fact));
    }

    let component_types = match (defines, declares) {
        (Some(defines), _) => {
            component_type_facts(defines, extends, defines_means, &context, registry)
        }
        // A `Declares:`/`Realizes:` takes its component types from `means:`, so a
        // value of the declaration can be destructured the same way.
        (None, Some(statement)) => {
            declares_component_type_facts(statement, declares_means, &context)
        }
        (None, None) => Vec::new(),
    };
    let component_shapes = match (described, declares) {
        (Some(target), _) => defines_target_component_shapes(target),
        (None, Some(statement)) => declaration_component_shapes(statement),
        (None, None) => Vec::new(),
    };
    let set_element_target = described.and_then(defines_target_set_target).cloned();
    let set_element_types = set_element_target
        .as_ref()
        .map(|target| set_element_type_facts(target, defines_means, &context))
        .unwrap_or_default();
    let parameter_destructurings = destructured_parameters(heading, &context);
    let inferred_parameters = collect_inferred_parameter_names(when);

    DefinitionTypeInfo {
        signature: header_shape.shape.signature.clone(),
        type_key: header_shape.type_key.clone(),
        parameters: header_shape.parameters.clone(),
        arg_groups: header_shape.shape.arg_groups.clone(),
        variadic_parameters: header_variadic_parameters(heading)
            .into_iter()
            .filter(|variadic| header_shape.parameters.contains(&variadic.name))
            .cloned()
            .collect(),
        hidden_parameters: header_shape.hidden_parameters.clone(),
        using_parameters,
        given_parameters,
        requirements,
        outputs,
        substitutions: context.substitutions,
        described: described.map(described_target_subject_key),
        component_types,
        component_shapes,
        set_element_target,
        set_element_types,
        parameter_destructurings,
        inferred_parameters,
    }
}

fn defines_target_set_target(target: &DefinesTarget) -> Option<&SetTarget> {
    let form = match target {
        DefinesTarget::Form(form) => Some(form),
        DefinesTarget::Declaration(statement) => is_subject_first_form(&statement.subject)
            .or_else(|| statement.expansion.as_ref().and_then(is_subject_first_form)),
    }?;
    match &form.kind {
        FormOrDeclarationKind::SetDeclaration { form, .. } => Some(&form.target),
        _ => None,
    }
}

fn set_element_type_facts(
    target: &SetTarget,
    means: Option<&DefinesMeansSection>,
    context: &TypeContext,
) -> Vec<TypeFact> {
    let Some(means) = means else {
        return Vec::new();
    };
    let mut target_names = BTreeSet::new();
    collect_set_target_names(target, &mut target_names);
    means
        .arguments
        .iter()
        .flat_map(|item| facts_from_is_or_via_item_in_context(item, context))
        .filter(|fact| target_names.contains(fact_subject(fact)))
        .map(|fact| context.normalize_fact(&fact))
        .collect()
}

/// The `?`-suffixed inferred parameter names appearing in a definition's `when:`
/// requirements (e.g. `A`, `B` in `g is \function:on{A?}:to{B?}`).
fn collect_inferred_parameter_names(when: Option<&WhenSection>) -> Vec<String> {
    let mut names = Vec::new();
    if let Some(when) = when {
        for clause in &when.arguments {
            if let Clause::Declaration(statement) = clause
                && let Some(DeclarationRelation::Is(ty)) = &statement.relation
            {
                collect_inferred_names_in_type_expression(ty, &mut names);
            }
        }
    }
    names.sort();
    names.dedup();
    names
}

fn collect_inferred_names_in_type_expression(ty: &TypeExpression, names: &mut Vec<String>) {
    match ty {
        TypeExpression::Tuple(tuple) => {
            for spec in &tuple.elements {
                collect_inferred_names_in_function_type_spec(spec, names);
            }
            return;
        }
        TypeExpression::Set(set) => {
            match &set.element {
                SetTypeElement::Spec(spec) => {
                    collect_inferred_names_in_function_type_spec(spec, names)
                }
                SetTypeElement::Tuple(tuple) => {
                    for spec in &tuple.elements {
                        collect_inferred_names_in_function_type_spec(spec, names);
                    }
                }
            }
            return;
        }
        TypeExpression::Function(function) => {
            for spec in function
                .inputs
                .iter()
                .chain(std::iter::once(&function.output))
            {
                if let FunctionTypeSpecKind::Is(ty) = &spec.kind {
                    collect_inferred_names_in_type_expression(ty, names);
                }
            }
            return;
        }
        _ => {}
    }
    let arguments = match ty {
        TypeExpression::Command(command) => command_expression_arguments(command),
        TypeExpression::RefinedCommand(command) => refined_command_expression_arguments(command),
        _ => return,
    };
    for argument in arguments {
        if let ExpressionKind::InferredName(name) = &argument.kind {
            names.push(name.clone());
        }
    }
}

fn collect_inferred_names_in_function_type_spec(spec: &FunctionTypeSpec, names: &mut Vec<String>) {
    if let FunctionTypeSpecKind::Is(ty) = &spec.kind {
        collect_inferred_names_in_type_expression(ty, names);
    }
}

/// Type facts for the components of a destructuring describes target, in tuple
/// order. Each component's type is drawn from the definition's `is … via …`
/// clauses first (its components inherit the extended type's component types)
/// and then `means:` for any component no `via` covers. Facts are normalized
/// so they can be re-substituted when another definition destructures a value of
/// this type.
fn component_type_facts(
    defines: &DefinesSection,
    extends: Option<&ExtendsSection>,
    means: Option<&DefinesMeansSection>,
    context: &TypeContext,
    registry: &SignatureRegistry,
) -> Vec<TypeFact> {
    let component_names = defines_target_component_names(&defines.argument);
    if component_names.is_empty() {
        return Vec::new();
    }

    let mut fact_context = context.clone();
    for clause in extends_clauses(defines, extends) {
        for fact in facts_from_extends_clause_in_context(clause, &fact_context) {
            fact_context.add_fact(fact);
        }
        for fact in facts_from_extends_via(clause, &fact_context, registry) {
            fact_context.add_fact(fact);
        }
    }
    if let Some(means) = means {
        for item in &means.arguments {
            for fact in facts_from_is_or_via_item_in_context(item, &fact_context) {
                fact_context.add_fact(fact);
            }
        }
    }

    component_names
        .iter()
        .filter_map(|name| {
            fact_context
                .facts
                .iter()
                .find(|fact| fact_subject(fact) == name)
                .map(|fact| fact_context.normalize_fact(fact))
        })
        .collect()
}

/// Type facts for the components of a destructuring `Declares:`/`Realizes:`
/// target, in tuple order, taken from the group's `means:`.
///
/// This is the `Declares` counterpart of [`component_type_facts`]: it is what
/// lets `Y ::= (N, 0, succ) := \naturals` type `N`, `0` and `succ`.
fn declares_component_type_facts(
    statement: &DeclarationStatement,
    means: Option<&DeclaresMeansSection>,
    context: &TypeContext,
) -> Vec<TypeFact> {
    let component_names = declaration_component_names(statement);
    if component_names.is_empty() {
        return Vec::new();
    }

    let mut fact_context = context.clone();
    for item in means.into_iter().flat_map(|means| means.arguments.iter()) {
        for fact in facts_from_is_or_via_item_in_context(item, &fact_context) {
            fact_context.add_fact(fact);
        }
    }

    component_names
        .iter()
        .filter_map(|name| {
            fact_context
                .facts
                .iter()
                .find(|fact| fact_subject(fact) == name)
                .map(|fact| fact_context.normalize_fact(fact))
        })
        .collect()
}

/// The tuple form a declaration destructures, e.g. `(N, 0, succ(n_))` for
/// `Nb ::= (N, 0, succ(n_))`.
fn declaration_tuple_form(statement: &DeclarationStatement) -> Option<&TupleForm> {
    statement
        .expansion
        .as_ref()
        .and_then(is_subject_first_form)
        .and_then(form_or_declaration_tuple_form)
}

fn declaration_component_names(statement: &DeclarationStatement) -> Vec<String> {
    declaration_tuple_form(statement)
        .map(tuple_form_component_names)
        .unwrap_or_default()
}

fn declaration_component_shapes(statement: &DeclarationStatement) -> Vec<TargetShape> {
    declaration_tuple_form(statement)
        .map(tuple_form_component_shapes)
        .unwrap_or_default()
}

/// The type facts one subtype clause states.
///
/// An operator-form subject (`x_ * y_ is \function:…`) states its facts about
/// the operator itself, since `*` is how such a target is named everywhere
/// else — in `means:`, in a `Refines:` of the same form, and in member
/// access.
fn facts_from_extends_clause(clause: ExtendsClause<'_>) -> Vec<TypeFact> {
    retarget_operator_subject(
        facts_from_declaration_statement(clause.statement),
        clause.statement,
    )
}

fn facts_from_extends_clause_in_context(
    clause: ExtendsClause<'_>,
    context: &TypeContext,
) -> Vec<TypeFact> {
    retarget_operator_subject(
        facts_from_declaration_statement_in_context(clause.statement, context),
        clause.statement,
    )
}

/// Rewrites facts about an operator form (`x_ * y_`) to be about the operator
/// (`*`).
fn retarget_operator_subject(
    facts: Vec<TypeFact>,
    statement: &DeclarationStatement,
) -> Vec<TypeFact> {
    let Some(operator) = subject_operator_name(&statement.subject) else {
        return facts;
    };
    let substitutions = HashMap::from([(primary_subject_key(&statement.subject), operator)]);
    facts
        .iter()
        .map(|fact| substitute_fact(fact, &substitutions))
        .collect()
}

/// The operator an operator-form subject (`x_ * y_`, `-x_`, `x_!`) names.
fn subject_operator_name(subject: &IsSubject) -> Option<String> {
    let form = is_subject_first_form(subject)?;
    match &form.kind {
        FormOrDeclarationKind::InfixOperator { operator, .. }
        | FormOrDeclarationKind::PrefixOperator { operator, .. }
        | FormOrDeclarationKind::PostfixOperator { operator, .. } => Some(operator.text.clone()),
        _ => None,
    }
}

/// The type facts a `<subject> is <Type> via <via>` clause assigns to the `via`
/// symbols. `via X` with a plain `\set` gives `X is \set`. `via (X, *)` onto a
/// tuple type maps the extended type's components positionally, so
/// `S is \magma via (X, *)` yields `X is \set` and `* is \binary.operation:on{S}`
/// by following `\magma`'s own component types (with its subject replaced by `S`).
fn facts_from_extends_via(
    clause: ExtendsClause<'_>,
    context: &TypeContext,
    registry: &SignatureRegistry,
) -> Vec<TypeFact> {
    let Some(via) = clause.via else {
        return Vec::new();
    };
    let Some(DeclarationRelation::Is(ty)) = &clause.statement.relation else {
        return Vec::new();
    };
    let subject = primary_subject_key(&clause.statement.subject);
    match &via.kind {
        FormOrDeclarationKind::Name(name) => {
            fact_from_type_key_assertion(name.clone(), ty, context)
                .into_iter()
                .collect()
        }
        FormOrDeclarationKind::TupleDeclaration { form, .. } => {
            let via_names = tuple_form_component_names(form);
            let Some((_, signature)) = key_for_type_expression(ty) else {
                return Vec::new();
            };
            let Some(info) = registry.type_infos.get(&signature) else {
                return Vec::new();
            };
            instantiate_component_type_facts(info, &subject, &via_names, context)
        }
        _ => Vec::new(),
    }
}

/// Adds the component types the definition's `is … via …` clauses assign
/// (`X is \set`, etc.) to the checking context, so the definition's own body
/// (e.g. `means: e "in" X`) can rely on them.
fn assume_extends_via_facts(
    defines: &DefinesSection,
    extends: Option<&ExtendsSection>,
    context: &mut TypeContext,
    registry: &SignatureRegistry,
) {
    for clause in extends_clauses(defines, extends) {
        for fact in facts_from_extends_via(clause, context, registry) {
            context.add_fact(fact);
        }
    }
}

fn collect_using_parameter_names(using: Option<&UsingSection>) -> Vec<String> {
    using
        .map(|using| collect_declaration_parameter_names(&using.arguments))
        .unwrap_or_default()
}

fn collect_given_parameter_names(given: Option<&GivenSection>) -> Vec<String> {
    given
        .map(|given| collect_declaration_parameter_names(&given.arguments))
        .unwrap_or_default()
}

fn collect_declaration_parameter_names(statements: &[DeclarationStatement]) -> Vec<String> {
    let mut names = BTreeSet::new();
    for statement in statements {
        collect_declaration_statement_covered_symbols(statement, &mut names);
    }
    names.into_iter().collect()
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
            && item_defines_collection(item)
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

fn item_defines_collection(item: &TopLevelItem) -> bool {
    matches!(
        item,
        TopLevelItem::Defines(group) if defines_target_is_collection(&group.defines.argument)
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
        let EnablesItem::Relation(group) = item else {
            continue;
        };
        if !relation_group_has_kind(group, RelationKind::Coercion) {
            continue;
        };
        let Some((target_subject, target @ TypeFact::Is { .. })) =
            view_target_from_relationship_declaration(&group.to.argument, source_subject)
        else {
            continue;
        };
        registry.viewable_rules.push(ViewableRule {
            source_signature: info.signature.clone(),
            source_subject: source_subject.clone(),
            parameters: info.parameters.clone(),
            target_subject,
            target,
        });
    }
}

fn collect_abstraction_rules(
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
        let EnablesItem::Relation(group) = item else {
            continue;
        };
        if !relation_group_has_kind(group, RelationKind::Encoding) {
            continue;
        };
        let Some((_, target @ TypeFact::Is { .. })) =
            view_target_from_relationship_declaration(&group.to.argument, source_subject)
        else {
            continue;
        };
        registry.abstraction_rules.push(AbstractionRule {
            source_signature: info.signature.clone(),
            source_subject: source_subject.clone(),
            parameters: info.parameters.clone(),
            target,
        });
    }
}

fn relation_group_has_kind(group: &EnablesRelationGroup, kind: RelationKind) -> bool {
    group
        .represents
        .as_ref()
        .is_some_and(|section| section.arguments.iter().any(|argument| *argument == kind))
}

fn view_target_from_relationship_declaration(
    declaration: &RelationshipDeclaration,
    source_subject: &str,
) -> Option<(String, TypeFact)> {
    match declaration {
        RelationshipDeclaration::Command(command) => {
            let ty = TypeExpression::Command(command.clone());
            let (ty, signature) = key_for_type_expression(&ty)?;
            let subject = source_subject.to_owned();
            Some((
                subject.clone(),
                TypeFact::Is {
                    subject,
                    ty,
                    signature,
                },
            ))
        }
        RelationshipDeclaration::Declaration(statement) => {
            facts_from_declaration_statement(statement)
                .into_iter()
                .find_map(|fact| {
                    if matches!(fact, TypeFact::Is { .. }) {
                        Some((fact_subject(&fact).to_owned(), fact))
                    } else {
                        None
                    }
                })
        }
    }
}

fn collect_type_extension_rules(
    item: &TopLevelItem,
    info: &DefinitionTypeInfo,
    registry: &mut SignatureRegistry,
) {
    let Some((defines, extends)) = defines_extends_parts(item) else {
        return;
    };

    for fact in extends_clauses(defines, extends)
        .into_iter()
        .flat_map(facts_from_extends_clause)
    {
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

fn defines_extends_parts(
    item: &TopLevelItem,
) -> Option<(&DefinesSection, Option<&ExtendsSection>)> {
    match item {
        TopLevelItem::Defines(group) => Some((&group.defines, group.extends.as_ref())),
        _ => None,
    }
}

fn enables_section(item: &TopLevelItem) -> Option<&EnablesSection> {
    match item {
        TopLevelItem::Defines(group) => group.enables.as_ref(),
        TopLevelItem::Declares(group) => group.enables.as_ref(),
        TopLevelItem::Realizes(group) => group.enables.as_ref(),
        TopLevelItem::Refines(group) => group.enables.as_ref(),
        TopLevelItem::States(group) => group.enables.as_ref(),
        _ => None,
    }
}

fn requires_section(item: &TopLevelItem) -> Option<&RequiresSection> {
    match item {
        TopLevelItem::Defines(group) => group.requires.as_ref(),
        TopLevelItem::Declares(group) => group.requires.as_ref(),
        TopLevelItem::Realizes(group) => group.requires.as_ref(),
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
            EnablesItem::FromAs(_) | EnablesItem::Relation(_) => None,
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

/// The definition kinds a top-level `Equivalent:` may group.
const EQUIVALENT_MEMBER_KINDS: [DefinitionKind; 4] = [
    DefinitionKind::Defines,
    DefinitionKind::Declares,
    DefinitionKind::States,
    DefinitionKind::Refines,
];

/// One `to:` command of an `Equivalent:` item, as needed for validation.
struct EquivalentMember<'a> {
    signature: String,
    /// The header parameters used as arguments, in order — `Some` only when every
    /// argument was a bare header parameter (so a `when:` check is meaningful).
    actuals: Option<Vec<String>>,
    command_context: Option<&'a CommandContext>,
}

/// Every argument expression across a command's head, tail, and paren argument
/// groups, in order.
fn command_argument_expressions(command: &CommandExpression) -> Vec<&Expression> {
    let mut arguments = Vec::new();
    for group in &command.head_args {
        arguments.extend(&group.expressions);
    }
    for part in &command.tail {
        for group in &part.args {
            arguments.extend(&group.expressions);
        }
    }
    for group in &command.paren_args {
        arguments.extend(&group.expressions);
    }
    arguments
}

/// Validate a top-level `Equivalent:` item.
///
/// Phase 1: local validation only — parameter-exactness of the `to:` commands,
/// that they agree in kind/shape/target/capabilities, and that this item's
/// `when:` is compatible with each member's requirements. It does NOT yet make
/// the members interchangeable to the type checker (that is Phase 2).
fn validate_equivalent_item(
    group: &EquivalentGroup,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    registry: &SignatureRegistry,
    event_log: &mut EventLog,
) {
    let position = locator.locate_heading(&shape_for_header(&group.heading));

    // The header parameters; `to:` commands may use only these, directly.
    let header_parameters: HashSet<String> = shapes_for_header(&group.heading)
        .into_iter()
        .flat_map(|shape| shape.parameters)
        .collect();

    // Establish the `using:`/`when:` scope (this also validates their references).
    let mut context = TypeContext::default();
    declare_header_symbols(&group.heading, &mut context, registry);
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

    // Collect the members, validating command-ness and parameter-exactness.
    let mut members: Vec<EquivalentMember> = Vec::new();
    for expression in &group.to.arguments {
        let ExpressionKind::Command(command) = &expression.kind else {
            emit_error(event_log, path, position, "`to:` entries must be commands");
            continue;
        };
        let mut actuals = Some(Vec::new());
        for argument in command_argument_expressions(command) {
            match &argument.kind {
                ExpressionKind::Name(name) if header_parameters.contains(name) => {
                    if let Some(list) = actuals.as_mut() {
                        list.push(name.clone());
                    }
                }
                ExpressionKind::Name(name) => {
                    actuals = None;
                    emit_error(
                        event_log,
                        path,
                        position,
                        format!(
                            "`to:` command uses `{name}`, which is not a parameter of the `Equivalent:` header"
                        ),
                    );
                }
                _ => {
                    actuals = None;
                    emit_error(
                        event_log,
                        path,
                        position,
                        "`to:` command arguments must be header parameters used directly, not expressions",
                    );
                }
            }
        }
        members.push(EquivalentMember {
            signature: shape_for_command_expression(command).signature,
            actuals,
            command_context: command.context.as_ref(),
        });
    }

    // Rule 1: every member is defined, is one of the supported kinds, and they all
    // share the same kind.
    let mut member_kind: Option<DefinitionKind> = None;
    for member in &members {
        let signature = &member.signature;
        let Some(kind) = registry.definitions.get(signature).map(|entry| entry.kind) else {
            continue; // undefined command — already reported by reference validation
        };
        if !EQUIVALENT_MEMBER_KINDS.contains(&kind) {
            emit_error(
                event_log,
                path,
                position,
                format!(
                    "`Equivalent:` `to:` items must be Defines/Declares/States/Refines, but `{signature}` is a {}",
                    kind.label()
                ),
            );
            continue;
        }
        match member_kind {
            None => member_kind = Some(kind),
            Some(first) if first == kind => {}
            Some(first) => emit_error(
                event_log,
                path,
                position,
                format!(
                    "`Equivalent:` mixes a {} (`{signature}`) with a {}; all `to:` items must be the same kind",
                    kind.label(),
                    first.label()
                ),
            ),
        }
    }

    validate_equivalent_member_agreement(
        &members,
        member_kind,
        position,
        path,
        registry,
        event_log,
    );
    validate_equivalent_when_compatibility(&members, &context, position, path, registry, event_log);
}

/// Rules 2–6: the `to:` members must agree in target shape and — depending on
/// their shared kind — in the type they define (`is`, Declares), the type they
/// extend (`extends:`, Defines), the base they refine (Refines), plus their
/// provided-capability set.
fn validate_equivalent_member_agreement(
    members: &[EquivalentMember],
    member_kind: Option<DefinitionKind>,
    position: Option<SourcePosition>,
    path: &Path,
    registry: &SignatureRegistry,
    event_log: &mut EventLog,
) {
    let Some(kind) = member_kind else {
        return; // no defined members, or a kind conflict was already reported
    };

    // Compare only the members actually defined with the shared kind; undefined
    // members and kind conflicts were already reported.
    let defined: Vec<&str> = members
        .iter()
        .filter(|member| {
            registry
                .definitions
                .get(member.signature.as_str())
                .map(|entry| entry.kind)
                == Some(kind)
        })
        .map(|member| member.signature.as_str())
        .collect();
    if defined.len() < 2 {
        return;
    }

    // Rule 2: same target shape.
    require_uniform_members(
        &defined,
        position,
        path,
        event_log,
        |signature| {
            registry
                .definition_summaries
                .get(signature)
                .map(|summary| summary.target_shape.clone())
                .unwrap_or(TargetShape::Other)
        },
        "declare targets of different shapes",
    );

    // Rules 3–5: members must share their core type identity. Which registry facts
    // express that identity, and the wording of a divergence, depend on the kind.
    let identity_divergence = match kind {
        DefinitionKind::Declares => Some("define values of different types"),
        DefinitionKind::Defines => Some("extend different types"),
        DefinitionKind::Refines => Some("refine different base types"),
        _ => None,
    };
    if let Some(divergence) = identity_divergence {
        require_uniform_members(
            &defined,
            position,
            path,
            event_log,
            |signature| member_type_identity(signature, kind, registry),
            divergence,
        );
    }

    // Rule 6: members must provide the same set of capabilities (existence only).
    require_uniform_members(
        &defined,
        position,
        path,
        event_log,
        |signature| member_capability_keys(signature, registry),
        "provide different capabilities",
    );
}

/// Emit one error (naming the two diverging members) if `key_of` is not constant
/// across `members`.
fn require_uniform_members<K: PartialEq>(
    members: &[&str],
    position: Option<SourcePosition>,
    path: &Path,
    event_log: &mut EventLog,
    key_of: impl Fn(&str) -> K,
    divergence: &str,
) {
    let Some((first, rest)) = members.split_first() else {
        return;
    };
    let first_key = key_of(first);
    for member in rest {
        if key_of(member) != first_key {
            emit_error(
                event_log,
                path,
                position,
                format!("`Equivalent:` `to:` items `{first}` and `{member}` {divergence}"),
            );
            return;
        }
    }
}

/// The set of type signatures that fix a member's core identity for rules 3–5:
/// the `is` type of a Declares, the extended type of a Defines target, or the
/// base type of a Refines. Uses only global type signatures (never definition-local
/// symbol names), so structurally identical types compare equal.
fn member_type_identity(
    signature: &str,
    kind: DefinitionKind,
    registry: &SignatureRegistry,
) -> BTreeSet<String> {
    match kind {
        DefinitionKind::Declares => registry
            .type_infos
            .get(signature)
            .map(|info| {
                info.outputs
                    .iter()
                    .filter_map(type_fact_type_signature)
                    .collect()
            })
            .unwrap_or_default(),
        DefinitionKind::Defines => registry
            .extension_rules
            .iter()
            .filter(|rule| rule.subtype_signature == signature)
            .filter_map(|rule| type_fact_type_signature(&rule.target))
            .collect(),
        DefinitionKind::Refines => registry
            .refinement_extension_rules
            .iter()
            .filter(|rule| rule.subtype_signature == signature)
            .map(|rule| match &rule.target {
                RefinementExtensionTarget::Fact(fact) => {
                    type_fact_type_signature(fact).unwrap_or_else(|| "\\\\other".to_string())
                }
                RefinementExtensionTarget::DynamicRefinedIs { .. } => "\\\\dynamic".to_string(),
            })
            .collect(),
        _ => BTreeSet::new(),
    }
}

/// The global type signature a fact asserts its subject to be, or `None` for
/// facts that don't pin the subject to a named type.
fn type_fact_type_signature(fact: &TypeFact) -> Option<String> {
    match fact {
        TypeFact::Is { signature, .. } => Some(signature.clone()),
        TypeFact::RefinedIs { base_signature, .. } => Some(base_signature.clone()),
        _ => None,
    }
}

/// The set of capabilities a member provides (rule 6): named symbols keyed by
/// name and arity, and spec operators. Only existence is compared, per the spec.
fn member_capability_keys(signature: &str, registry: &SignatureRegistry) -> BTreeSet<String> {
    let mut keys = BTreeSet::new();
    for rule in &registry.provided_symbols {
        if rule.owner_signature == signature {
            keys.insert(format!("symbol {}", disambiguation_key_label(&rule.key)));
        }
    }
    for rule in &registry.spec_rules {
        if rule.owner_signature == signature {
            keys.insert(format!("operator {}", rule.operator));
        }
    }
    keys
}

fn disambiguation_key_label(key: &DisambiguationKey) -> String {
    match key {
        DisambiguationKey::BinaryOperator(operator) => format!("binop {operator}"),
        DisambiguationKey::PrefixOperator(operator) => format!("prefix {operator}"),
        DisambiguationKey::PostfixOperator(operator) => format!("postfix {operator}"),
        DisambiguationKey::Function { name, arity } => format!("fn {name}/{arity}"),
    }
}

/// The `Equivalent:`'s own `when:`/`using:` scope (captured in `context`) must
/// guarantee each member command's requirements — otherwise a member could not be
/// formed under the conditions the equivalence claims to hold. Reuses the standard
/// call-site requirement check, treating each `to:` command as a call within the
/// `Equivalent:`'s scope.
fn validate_equivalent_when_compatibility(
    members: &[EquivalentMember],
    context: &TypeContext,
    position: Option<SourcePosition>,
    path: &Path,
    registry: &SignatureRegistry,
    event_log: &mut EventLog,
) {
    for member in members {
        let Some(actuals) = &member.actuals else {
            continue; // parameter-exactness already failed; a requirement check is moot
        };
        check_command_requirements(
            &member.signature,
            actuals,
            None,
            member.command_context,
            context,
            path,
            position,
            registry,
            event_log,
        );
    }
}

fn validate_top_level_item_types(
    item: &TopLevelItem,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    registry: &SignatureRegistry,
    event_log: &mut EventLog,
) {
    locator.begin_item();
    anchor_top_level_item(item, locator);

    match item {
        TopLevelItem::Disambiguates(group) => {
            validate_disambiguates(group, path, locator, registry, event_log);
        }
        TopLevelItem::Defines(group) => {
            let mut context = TypeContext::default();
            context.set_justifications(build_justification_map(&group.justification));
            validate_spec_infix_defines_header(
                &group.heading,
                &group.defines.argument,
                path,
                locator,
                event_log,
            );
            declare_header_symbols(&group.heading, &mut context, registry);
            declare_defines_target(&group.defines.argument, &mut context);
            assume_described_type(&group.heading, &group.defines.argument, &mut context);
            assume_optional_using(
                &group.using,
                &mut context,
                path,
                locator,
                registry,
                event_log,
            );
            let when_parameters = defines_when_parameters_from_usage(group);
            validate_when_section(&group.when, &when_parameters, path, locator, event_log);
            assume_optional_clauses(
                &group.when,
                &mut context,
                path,
                locator,
                registry,
                event_log,
            );
            assume_destructured_parameter_components(&group.heading, &mut context, registry);
            assume_extends_via_facts(
                &group.defines,
                group.extends.as_ref(),
                &mut context,
                registry,
            );
            assume_optional_defines_means(
                &group.means,
                &mut context,
                path,
                locator,
                registry,
                event_log,
            );
            assume_defines_function_type(
                &group.defines.argument,
                &group.means,
                &mut context,
            );
            validate_defines_justification_usage(group, path, locator, event_log);
            // The target and its `extends:` clauses are checked here, rather
            // than where the target's symbols are declared, because the type a
            // definition extends may name symbols that only `using:`/`when:`
            // and the `via` facts bring into scope — as in `Defines: x "in" X`
            // under `when: M is \magma`.
            check_defines_target(
                &group.defines,
                &context,
                path,
                locator,
                registry,
                event_log,
            );
            check_optional_extends(
                &group.extends,
                &context,
                path,
                locator,
                registry,
                event_log,
            );
            validate_defines_target_symbol_specifications(group, path, locator, event_log);
            validate_optional_requires(
                &group.requires,
                &context,
                Some(&shapes_for_header(&group.heading)),
                Some(&described_target_subject_key(&group.defines.argument)),
                path,
                locator,
                registry,
                event_log,
            );
            validate_optional_enables(
                &group.enables,
                &context,
                &shapes_for_header(&group.heading),
                &described_target_subject_key(&group.defines.argument),
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
        TopLevelItem::Declares(group) => {
            let mut context = TypeContext::default();
            declare_header_symbols(&group.heading, &mut context, registry);
            declare_declaration_statement_subjects(&group.declares.argument, &mut context);
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
            assume_destructured_parameter_components(&group.heading, &mut context, registry);
            complete_introduced_declaration_statement(
                &group.declares.argument,
                &mut context,
                path,
                locator,
                registry,
                event_log,
            );
            assume_destructured_declaration_components(
                &group.declares.argument,
                &mut context,
                registry,
            );
            assume_optional_means(&group.means, &mut context, path, locator, registry, event_log);
            validate_declares_target_symbol_specifications(group, path, locator, event_log);
            validate_declares_means_items(group, path, locator, event_log);
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
            assume_optional_expresses(
                &group.expresses,
                &mut context,
                path,
                locator,
                registry,
                event_log,
            );
        }
        TopLevelItem::Realizes(group) => {
            let mut context = TypeContext::default();
            declare_header_symbols(&group.heading, &mut context, registry);
            declare_declaration_statement_subjects(&group.realizes.argument, &mut context);
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
            assume_destructured_parameter_components(&group.heading, &mut context, registry);
            assume_realized_declaration_components(group, &mut context, registry);
            assume_optional_means(&group.means, &mut context, path, locator, registry, event_log);
            validate_realizes_target(group, path, locator, registry, event_log);
            validate_concrete_means_items(
                &group.means,
                &group.expresses,
                "leave it abstract in the declaration this realizes",
                path,
                locator,
                event_log,
            );
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
            assume_optional_expresses(
                &group.expresses,
                &mut context,
                path,
                locator,
                registry,
                event_log,
            );
        }
        TopLevelItem::Refines(group) => {
            let mut context = TypeContext::default();
            declare_header_symbols(&group.heading, &mut context, registry);
            declare_declaration_statement_subjects(&group.refines.argument, &mut context);
            validate_refines_target(group, path, locator, registry, event_log);
            validate_refined_spec_infix_header(group, path, locator, event_log);
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
                &refines_when_parameters_from_usage(group),
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
            assume_refines_destructured_components(
                &group.heading,
                &group.refines.argument,
                &mut context,
                registry,
            );
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
            check_refines_marker(group, path, locator, registry, event_log);
            validate_refines_target_symbol_specifications(
                group, path, locator, registry, event_log,
            );
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
            declare_header_symbols(&group.heading, &mut context, registry);
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
            reject_specification_clauses(&group.that.arguments, path, locator, event_log);
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
        TopLevelItem::Equivalent(group) => {
            validate_equivalent_item(group, path, locator, registry, event_log);
        }
        TopLevelItem::Relation(group) => {
            // Assume the `using:` declarations and the two related declarations
            // (introducing their subjects and facts) and the `when:` specs, then
            // check the `means:` statement against that scope. Like a theorem, the
            // statement is checked for valid symbols/references, not proven.
            let mut context = TypeContext::default();
            assume_optional_using(
                &group.using,
                &mut context,
                path,
                locator,
                registry,
                event_log,
            );
            assume_relation_subject(
                &group.between.argument,
                &mut context,
                path,
                locator,
                registry,
                event_log,
            );
            assume_relation_subject(
                &group.and_.argument,
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
            // Only a statement `means:` is checked; a prose `Text` description is not.
            if let Some(RelationMeans::Statement(clause)) =
                group.means.as_ref().map(|means| &means.argument)
            {
                check_clause(clause, &context, path, locator, registry, event_log);
            }
        }
        TopLevelItem::Specify(_)
        | TopLevelItem::Title(_)
        | TopLevelItem::SectionTitle(_)
        | TopLevelItem::SubsectionTitle(_)
        | TopLevelItem::Text(_)
        | TopLevelItem::Writing(_)
        | TopLevelItem::Person(_)
        | TopLevelItem::Resource(_)
        // A `Topic:` only names a documentation topic (heading, prose, optional
        // parent, optional rendering override); there is nothing to type-check.
        | TopLevelItem::Topic(_)
        // `Text*` placeholders are opaque prose; the checker never inspects them.
        | TopLevelItem::TextItem(_) => {}
    }
}

fn anchor_top_level_item(item: &TopLevelItem, locator: &mut SourceLocator<'_>) {
    let heading = match item {
        TopLevelItem::Defines(group) => Some(&group.heading),
        TopLevelItem::Declares(group) => Some(&group.heading),
        TopLevelItem::Realizes(group) => Some(&group.heading),
        TopLevelItem::Refines(group) => Some(&group.heading),
        TopLevelItem::States(group) => Some(&group.heading),
        TopLevelItem::Axiom(group) => group.heading.as_ref(),
        TopLevelItem::Theorem(group) => group.heading.as_ref(),
        TopLevelItem::Conjecture(group) => group.heading.as_ref(),
        TopLevelItem::Equivalent(group) => Some(&group.heading),
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
            "The `extends:` subject must match the `Refines:` subject",
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
                "`[[...]]` in a `Refines` `extends:` clause must name the `Refines:` subject",
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

/// The base type signature (`\group`) of a refined command heading such as
/// `[\(finite)::group]`.
fn refined_command_header_base_signature(command: &RefinedCommandHeader) -> String {
    format!("\\{}", format_refined_tail(&command.refined_tail))
}

/// The `::`-joined adjective chains of a refined command heading, e.g.
/// `\(finite)::group` -> `finite`, `\(injective, surjective)::function` ->
/// `injective::surjective`.
fn refined_command_header_adjective_key(command: &RefinedCommandHeader) -> String {
    command
        .parts
        .iter()
        .map(|part| format_chain(&part.chain))
        .collect::<Vec<_>>()
        .join("::")
}

/// The `::`-joined adjective chains of a refined command expression (the form of
/// a `Refines:` `extends:` target), rendered the same way as
/// [`refined_command_header_adjective_key`] so the two can be compared.
fn refined_command_expression_adjective_key(command: &RefinedCommandExpression) -> String {
    command
        .parts
        .iter()
        .map(|part| format_chain(&part.chain))
        .collect::<Vec<_>>()
        .join("::")
}

/// The type signature a subtype's `extends:` rule points at (the supertype), for
/// the fact kinds an `extends:` clause can produce.
fn extension_rule_supertype_signature(fact: &TypeFact) -> Option<String> {
    match fact {
        TypeFact::Is { signature, .. } => Some(signature.clone()),
        TypeFact::RefinedIs { base_signature, .. } => Some(base_signature.clone()),
        _ => None,
    }
}

/// The direct supertypes (parents) of `base_signature`, taken from the type
/// extension rules the registry collected for the type the base's own
/// `Defines:` target extends.
fn direct_parent_signatures(base_signature: &str, registry: &SignatureRegistry) -> Vec<String> {
    registry
        .extension_rules
        .iter()
        .filter(|rule| rule.subtype_signature == base_signature)
        .filter_map(|rule| extension_rule_supertype_signature(&rule.target))
        .collect()
}

/// Whether an `implicitly:`-marked group's `extends:` clause literally names the
/// parent type's refinement: the same adjective(s) applied to a direct supertype
/// of the refined base type.
fn implicit_extends_names_parent_refinement(
    group: &RefinesGroup,
    heading: &RefinedCommandHeader,
    parents: &[String],
) -> bool {
    let Some(extends) = &group.extends else {
        return false;
    };
    let Some(DeclarationRelation::Is(TypeExpression::RefinedCommand(target))) =
        &extends.argument.relation
    else {
        return false;
    };
    if refined_command_header_adjective_key(heading)
        != refined_command_expression_adjective_key(target)
    {
        return false;
    }
    let target_base = format!("\\{}", format_refined_tail(&target.refined_tail));
    parents.contains(&target_base)
}

/// Validates the optional `implicitly:`/`explicitly:` marker on a `Refines:`
/// group.
///
/// Both markers are only meaningful when the refined base type is a subtype of
/// another type (so that a supertype refinement could be inherited).  Given that:
///
///   * `implicitly:` asserts the group merely restates the inherited definition,
///     so its body must contain nothing beyond the inherited `extends:` clause
///     (plus scaffolding `using:`/`when:`), and that `extends:` clause must
///     literally name the parent type's refinement — the same adjective(s)
///     applied to a direct supertype of the refined base type.
///   * `explicitly:` asserts the group overrides the inherited definition, so it
///     must add at least one property beyond the inherited `extends:` clause;
///     otherwise it is the trivial case that should be marked `implicitly:`.
fn check_refines_marker(
    group: &RefinesGroup,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    registry: &SignatureRegistry,
    event_log: &mut EventLog,
) {
    let Some(marker) = group.refinement_kind else {
        return;
    };
    let CommandHeader::Refined(heading) = &group.heading else {
        return;
    };
    let location = locator.locate_heading(&shape_for_header(&group.heading));

    let base_signature = refined_command_header_base_signature(heading);
    let parents = direct_parent_signatures(&base_signature, registry);

    if parents.is_empty() {
        emit_error(
            event_log,
            path,
            location,
            "`implicitly:` and `explicitly:` may only be used when the `Refines:` base type is a \
             subtype of another type (its `Defines:` target names a type it extends); the base \
             here is not",
        );
        return;
    }

    let has_extends = group.extends.is_some();
    let adds_properties = group.satisfies.is_some()
        || group.requires.is_some()
        || group.enables.is_some()
        || group.justification.is_some();

    match marker {
        RefinementKind::Implicit => {
            if !has_extends {
                emit_error(
                    event_log,
                    path,
                    location,
                    "A `Refines:` marked `implicitly:` must restate the inherited definition with an \
                     `extends:` clause naming the supertype's refinement",
                );
            } else if adds_properties {
                emit_error(
                    event_log,
                    path,
                    location,
                    "A `Refines:` marked `implicitly:` must contain only the inherited `extends:` \
                     clause; it must not add `satisfies:`, `Requires:`, `Enables:`, or `Justification:`. \
                     Mark it `explicitly:` if the definition is meant to differ",
                );
            } else if !implicit_extends_names_parent_refinement(group, heading, &parents) {
                emit_error(
                    event_log,
                    path,
                    location,
                    "A `Refines:` marked `implicitly:` must name the parent type's refinement in its \
                     `extends:` clause: the same adjective(s) applied to a supertype of the refined \
                     base type",
                );
            }
        }
        RefinementKind::Explicit => {
            if !adds_properties {
                emit_error(
                    event_log,
                    path,
                    location,
                    "A `Refines:` marked `explicitly:` must add at least one property beyond the \
                     inherited `extends:` clause (for example a `satisfies:` section); the trivial \
                     case should be marked `implicitly:`",
                );
            }
        }
    }
}

/// Reports each `means:` item of a non-abstract `Declares:` that states a type
/// but supplies no value.
///
/// A concrete declaration has to say what its parts *are*, either directly with
/// `:=` or indirectly in `expresses:` (`C is \real.function` plus a `piecewise:`
/// that defines `C`). A specification with neither is what `abstractly:` is for:
/// it leaves the symbol for a `Realizes:` to supply.
fn validate_declares_means_items(
    group: &DeclaresGroup,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    event_log: &mut EventLog,
) {
    if group.abstractly {
        return;
    }
    validate_concrete_means_items(
        &group.means,
        &group.expresses,
        "mark this `Declares:` `abstractly:`",
        path,
        locator,
        event_log,
    );
}

/// Reports each `means:` item of a group that must be concrete — a `Declares:`
/// without `abstractly:`, or a `Realizes:` — that states a specification but
/// supplies no value.
fn validate_concrete_means_items(
    means: &Option<DeclaresMeansSection>,
    expresses: &Option<ExpressesSection>,
    remedy: &str,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    event_log: &mut EventLog,
) {
    let expressed = expresses_bound_symbols(expresses);
    for statement in means_statements(means) {
        if statement.definition.is_none() && statement.relation.is_none() {
            for subject in declaration_subject_keys(statement) {
                emit_error(
                    event_log,
                    path,
                    locator.locate_symbol(&subject),
                    format!(
                        "`means:` item `{subject}` must either define its subject with `:=` or state its type"
                    ),
                );
            }
            continue;
        }
        if !statement_is_abstract(statement, &expressed) {
            continue;
        }
        for subject in declaration_subject_keys(statement) {
            emit_error(
                event_log,
                path,
                locator.locate_symbol(&subject),
                format!(
                    "`{subject}` states a specification but no value; define it with `:=`, define it in `expresses:`, or {remedy}"
                ),
            );
        }
    }
}

/// The abstract declaration a `Realizes:` target names, if it names one.
///
/// The target is written `Realizes: Nb := \naturals` — a `:=` because a
/// `Declares:` introduces a value, where a `Defines:` type would be named with
/// `is`.
fn realized_declaration<'a>(
    group: &RealizesGroup,
    registry: &'a SignatureRegistry,
) -> Option<(String, &'a AbstractDeclaration)> {
    let definition = group.realizes.argument.definition.as_ref()?;
    let signature = command_signature_from_key(&key_for_expression(definition))?;
    let declaration = registry.abstract_declarations.get(&signature)?;
    Some((signature, declaration))
}

/// Validates that a `Realizes:` names an abstract declaration and supplies every
/// symbol that declaration left abstract.
fn validate_realizes_target(
    group: &RealizesGroup,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    registry: &SignatureRegistry,
    event_log: &mut EventLog,
) {
    let statement = &group.realizes.argument;
    let subject = primary_subject_key(&statement.subject);
    let Some(definition) = &statement.definition else {
        emit_error(
            event_log,
            path,
            locator.locate_symbol(&subject),
            "A `Realizes:` must name the declaration it realizes with `:=`, as in `Realizes: Nb := \\naturals`".to_owned(),
        );
        return;
    };

    let key = key_for_expression(definition);
    let Some(signature) = command_signature_from_key(&key) else {
        emit_error(
            event_log,
            path,
            locator.locate_symbol(&subject),
            format!("`Realizes:` must name a command; `{key}` is not one"),
        );
        return;
    };

    if registry.abstract_declarations.contains_key(&signature) {
        validate_realized_symbols(group, &signature, path, locator, registry, event_log);
        return;
    }

    // The command exists but is the wrong kind, or does not exist at all — the
    // reference walk reports the latter, so only the former is added here.
    if let Some(entry) = registry.definitions.get(&signature) {
        let label = entry.kind.label();
        emit_error(
            event_log,
            path,
            locator.locate_symbol(&subject),
            format!(
                "`Realizes:` must name a `Declares:` marked `abstractly:`; `{key}` is a `{label}:`"
            ),
        );
    }
}

/// Reports each symbol the realized declaration left abstract that this
/// `Realizes:` does not supply.
fn validate_realized_symbols(
    group: &RealizesGroup,
    signature: &str,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    registry: &SignatureRegistry,
    event_log: &mut EventLog,
) {
    let Some(declaration) = registry.abstract_declarations.get(signature) else {
        return;
    };
    let realized = realized_symbols(group);
    let subject = primary_subject_key(&group.realizes.argument.subject);
    for fact in &declaration.abstract_facts {
        let abstract_symbol = fact_subject(fact);
        if realized.contains(abstract_symbol) {
            continue;
        }
        emit_error(
            event_log,
            path,
            locator.locate_symbol(&subject),
            format!(
                "Missing realization for abstract symbol `{abstract_symbol}`; a `Realizes:` must supply every symbol its declaration leaves abstract"
            ),
        );
    }
}

/// The symbols a `Realizes:` supplies, from its `means:` items and `expresses:`.
fn realized_symbols(group: &RealizesGroup) -> BTreeSet<String> {
    let mut realized = expresses_bound_symbols(&group.expresses);
    for statement in means_statements(&group.means) {
        realized.extend(declaration_subject_keys(statement));
    }
    realized
}

/// Brings the realized declaration's abstract facts into scope, so a `means:`
/// item can be checked against the specification it is realizing.
fn assume_realized_declaration_components(
    group: &RealizesGroup,
    context: &mut TypeContext,
    registry: &SignatureRegistry,
) {
    let Some((_, declaration)) = realized_declaration(group, registry) else {
        return;
    };
    for fact in &declaration.abstract_facts {
        context.declare_name(fact_subject(fact));
        context.add_fact(fact.clone());
    }
}

fn validate_refines_target(
    group: &RefinesGroup,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    registry: &SignatureRegistry,
    event_log: &mut EventLog,
) {
    let statement = &group.refines.argument;
    if statement.relation.is_some() || statement.definition.is_some() {
        emit_error(
            event_log,
            path,
            locator.locate_heading(&shape_for_header(&group.heading)),
            "`Refines:` must have the form `Refines: <form>` or `Refines: <name> ::= (<matching components>)`; the refined target is inferred from the heading",
        );
        return;
    }

    let Some(expansion) = &statement.expansion else {
        return;
    };
    if is_subject_shape(&statement.subject) != TargetShape::Name {
        emit_error(
            event_log,
            path,
            locator.locate_heading(&shape_for_header(&group.heading)),
            "The left side of a destructuring `Refines:` entry must be a single name",
        );
        return;
    }
    let Some(tuple) = is_subject_first_form(expansion).and_then(form_or_declaration_tuple_form)
    else {
        emit_refines_destructuring_mismatch(group, None, path, locator, registry, event_log);
        return;
    };
    let actual = tuple_form_component_shapes(tuple);
    if matching_refines_base_info(&group.heading, &actual, registry).is_none() {
        emit_refines_destructuring_mismatch(
            group,
            Some(&actual),
            path,
            locator,
            registry,
            event_log,
        );
    }
}

fn emit_refines_destructuring_mismatch(
    group: &RefinesGroup,
    actual: Option<&[TargetShape]>,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    registry: &SignatureRegistry,
    event_log: &mut EventLog,
) {
    let expected = refines_base_infos(&group.heading, registry)
        .into_iter()
        .find(|info| !info.component_shapes.is_empty())
        .map(|info| format_component_shapes(&info.component_shapes))
        .unwrap_or_else(|| "a non-destructured form".to_owned());
    let actual = actual
        .map(format_component_shapes)
        .unwrap_or_else(|| "a non-tuple expansion".to_owned());
    emit_error(
        event_log,
        path,
        locator.locate_heading(&shape_for_header(&group.heading)),
        format!(
            "`Refines:` destructuring has shape {actual}, but the base `Defines:` target has shape {expected}"
        ),
    );
}

fn format_component_shapes(shapes: &[TargetShape]) -> String {
    format!(
        "({})",
        shapes
            .iter()
            .map(|shape| match shape {
                TargetShape::Name => "value".to_owned(),
                TargetShape::Function(arity) => format!("function/{arity}"),
                TargetShape::Tuple(arity) => format!("tuple/{arity}"),
                TargetShape::Set => "set".to_owned(),
                TargetShape::Operator => "operator".to_owned(),
                TargetShape::Statement => "statement".to_owned(),
                TargetShape::Other => "other".to_owned(),
            })
            .collect::<Vec<_>>()
            .join(", ")
    )
}

fn validate_refined_spec_infix_header(
    group: &RefinesGroup,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    event_log: &mut EventLog,
) {
    let CommandHeader::InfixSpec(header) = &group.heading else {
        return;
    };
    if header.refinement.is_none()
        || form_or_declaration_subject_key(&header.left)
            == primary_subject_key(&group.refines.argument.subject)
    {
        return;
    }

    emit_error(
        event_log,
        path,
        locator.locate_heading(&shape_for_header(&group.heading)),
        "Refined spec-infix heading left operand must match the `Refines:` argument",
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
    described: &DefinesTarget,
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
    if let CommandHeader::InfixSpec(spec) = heading
        && spec.refinement.is_some()
    {
        let mut base = spec.clone();
        base.refinement = None;
        for header_shape in shapes_for_infix_spec_header(&base) {
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

fn refines_base_infos<'a>(
    heading: &CommandHeader,
    registry: &'a SignatureRegistry,
) -> Vec<&'a DefinitionTypeInfo> {
    if !matches!(heading, CommandHeader::Refined(_)) {
        return Vec::new();
    }
    shapes_for_header(heading)
        .into_iter()
        .filter_map(|shape| refined_header_base_type_fact_parts(&shape))
        .filter_map(|(_, signature)| registry.type_infos.get(&signature))
        .collect()
}

fn matching_refines_base_info<'a>(
    heading: &CommandHeader,
    component_shapes: &[TargetShape],
    registry: &'a SignatureRegistry,
) -> Option<&'a DefinitionTypeInfo> {
    refines_base_infos(heading, registry)
        .into_iter()
        .find(|info| info.component_shapes == component_shapes)
}

/// Binds a valid `Refines: G ::= (X, *, e)` expansion from the component
/// metadata of the base type named by the refined heading. Component names are
/// local aliases; their shapes and positions must match the base `Defines:`
/// target, while their types are instantiated positionally.
fn assume_refines_destructured_components(
    heading: &CommandHeader,
    statement: &DeclarationStatement,
    context: &mut TypeContext,
    registry: &SignatureRegistry,
) {
    let Some(tuple) = statement
        .expansion
        .as_ref()
        .and_then(is_subject_first_form)
        .and_then(form_or_declaration_tuple_form)
    else {
        return;
    };
    let component_shapes = tuple_form_component_shapes(tuple);
    let Some(info) = matching_refines_base_info(heading, &component_shapes, registry) else {
        return;
    };
    let component_names = tuple_form_component_names(tuple);
    let subject = primary_subject_key(&statement.subject);
    for component in &component_names {
        context.declare_name(component.clone());
    }
    for fact in instantiate_component_type_facts(info, &subject, &component_names, context) {
        context.add_fact(fact);
    }
    context.add_destructured_components(subject, component_names);
}

fn refined_header_base_type_fact_parts(header_shape: &HeaderShape) -> Option<(String, String)> {
    let signature_segments = split_refined_key(&header_shape.shape.signature)?;
    let type_key_segments = split_refined_key(&header_shape.type_key)?;
    let signature = format!("\\{}", signature_segments.last()?);
    let ty = format!("\\{}", type_key_segments.last()?);
    Some((ty, signature))
}

fn validate_spec_infix_defines_header(
    heading: &CommandHeader,
    described: &DefinesTarget,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    event_log: &mut EventLog,
) {
    let CommandHeader::InfixSpec(header) = heading else {
        return;
    };

    if form_or_declaration_subject_key(&header.left) == described_target_subject_key(described) {
        return;
    }

    emit_error(
        event_log,
        path,
        locator.locate_heading(&shape_for_infix_spec_header(header)),
        "Spec-infix Defines heading left operand must match the Defines argument",
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
        declare_header_symbols(heading, &mut context, registry);
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

    reject_specification_clauses(&sections.then.arguments, path, locator, event_log);
    for clause in &sections.then.arguments {
        check_clause(clause, &context, path, locator, registry, event_log);
    }

    if let Some(iff) = sections.iff {
        reject_specification_clauses(&iff.arguments, path, locator, event_log);
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

/// Assumes and checks the items of a `Declares:`/`Realizes:` `means:` section.
fn assume_optional_means(
    means: &Option<DeclaresMeansSection>,
    context: &mut TypeContext,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    registry: &SignatureRegistry,
    event_log: &mut EventLog,
) {
    if let Some(means) = means {
        for item in &means.arguments {
            assume_is_or_via_item(item, context, path, locator, registry, event_log);
        }
    }
}

/// Assumes an `expresses:` section: declaration clauses introduce their symbols
/// and facts, and every other clause is checked in place.
fn assume_optional_expresses(
    expresses: &Option<ExpressesSection>,
    context: &mut TypeContext,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    registry: &SignatureRegistry,
    event_log: &mut EventLog,
) {
    let Some(expresses) = expresses else {
        return;
    };
    for clause in &expresses.arguments {
        if let Clause::Declaration(statement) = clause {
            assume_declaration_statement(statement, context, path, locator, registry, event_log);
        } else {
            check_clause(clause, context, path, locator, registry, event_log);
        }
    }
}

fn assume_optional_defines_means(
    means: &Option<DefinesMeansSection>,
    context: &mut TypeContext,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    registry: &SignatureRegistry,
    event_log: &mut EventLog,
) {
    if let Some(means) = means {
        for item in &means.arguments {
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
        // Check the `have:` specification against its `asserting:` facts, then
        // contribute its typing facts to the surrounding context (this is the
        // sole pass over `means:` items, so the check happens here).
        IsOrViaItem::Have(group) => {
            check_have_group(group, context, path, locator, registry, event_log);
            for statement in have_group_declarations(group) {
                declare_declaration_statement_subjects(statement, context);
                for fact in facts_from_declaration_statement_in_context(statement, context) {
                    context.add_fact(fact);
                }
            }
        }
        // A labeled specification whose `[:label:]` matches a `Justification:`
        // entry is established via that entry's `have:`/`asserting:`; then its
        // facts are contributed. An unmatched label is checked inline as normal.
        IsOrViaItem::Labeled { label, item } => {
            if establish_labeled_specification(
                label, item, context, path, locator, registry, event_log,
            ) {
                assume_is_or_via_item_facts(item, context);
            } else {
                assume_is_or_via_item(item, context, path, locator, registry, event_log);
            }
        }
    }
}

/// Checks a `have:`/`asserting:` group. The `asserting:` items are taken as true
/// and used to check the `have:` items; `because:`/`by:` are justification whose
/// command and theorem references are reference-validated elsewhere but which are
/// never proven as logical consequences.
fn check_have_group(
    group: &HaveGroup,
    context: &TypeContext,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    registry: &SignatureRegistry,
    event_log: &mut EventLog,
) {
    let mut asserted = context.clone();
    for clause in &group.asserting.arguments {
        assume_asserted_clause(clause, &mut asserted, path, locator, registry, event_log);
    }
    for clause in &group.have.arguments {
        check_clause(clause, &asserted, path, locator, registry, event_log);
    }
}

/// Builds the `[label] -> have:/asserting: group` map for a `Justification:`
/// section. Entries without a `[label]` heading are skipped (unreferenceable).
fn build_justification_map(
    justification: &Option<JustificationSection>,
) -> HashMap<String, HaveGroup> {
    let mut map = HashMap::new();
    if let Some(section) = justification {
        for group in &section.arguments {
            if let Some(heading) = &group.heading {
                map.insert(heading.parts.join("."), group.clone());
            }
        }
    }
    map
}

/// The canonical key of the specification an `IsOrViaItem` states, used to check
/// that a labeled item and the `Justification:` entry it references restate the
/// same specification. `None` for shapes without a simple key.
fn spec_key_for_is_or_via_item(item: &IsOrViaItem) -> Option<String> {
    match item {
        IsOrViaItem::Declaration(statement) => Some(key_for_declaration_statement(statement)),
        IsOrViaItem::Labeled { item, .. } => spec_key_for_is_or_via_item(item),
        IsOrViaItem::IsVia(_) | IsOrViaItem::Have(_) => None,
    }
}

/// The canonical key of the specification a `have:` clause states.
fn spec_key_for_clause(clause: &Clause) -> Option<String> {
    match clause {
        Clause::Declaration(statement) => Some(key_for_declaration_statement(statement)),
        Clause::Expression(expression) => Some(key_for_expression(expression)),
        _ => None,
    }
}

/// Whether a `Justification:` entry's `have:` restates exactly the labeled item it
/// justifies (a single `have:` clause equal to the item). An item without a simple
/// key is treated as matching so no spurious mismatch is reported.
fn justification_have_matches_item(group: &HaveGroup, item: &IsOrViaItem) -> bool {
    let Some(item_key) = spec_key_for_is_or_via_item(item) else {
        return true;
    };
    justification_have_matches_key(group, &item_key)
}

fn justification_have_matches_key(group: &HaveGroup, item_key: &str) -> bool {
    let have_keys: Vec<String> = group
        .have
        .arguments
        .iter()
        .filter_map(spec_key_for_clause)
        .collect();
    have_keys.len() == 1 && have_keys[0] == item_key
}

/// Resolves labels stored directly on a declaration statement. Statement
/// parsers retain these labels in every declaration-bearing context, including
/// ordinary clauses and quantifier bindings.
fn establish_labeled_declaration_statement(
    statement: &DeclarationStatement,
    context: &TypeContext,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    registry: &SignatureRegistry,
    event_log: &mut EventLog,
) -> bool {
    let item_key = key_for_declaration_statement(statement);
    for label in statement.labels.iter().rev() {
        let key = label.parts.join(".");
        let Some(group) = context.justification(&key).cloned() else {
            continue;
        };
        if !justification_have_matches_key(&group, &item_key) {
            emit_error(
                event_log,
                path,
                declaration_subject_keys(statement)
                    .into_iter()
                    .next()
                    .and_then(|name| locator.locate_symbol(&name)),
                format!(
                    "The `have:` of `Justification:` entry `[{key}]` must restate the labeled specification exactly"
                ),
            );
        }
        check_have_group(&group, context, path, locator, registry, event_log);
        return true;
    }
    false
}

/// Checks a labeled expression under the assertions supplied by its matching
/// `Justification:` entry. This works at any expression depth because labeled
/// expressions are ordinary primary expressions in the formulation grammar.
fn establish_labeled_expression(
    label: &Label,
    expression: &Expression,
    context: &TypeContext,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    registry: &SignatureRegistry,
    event_log: &mut EventLog,
) -> bool {
    let key = label.parts.join(".");
    let Some(group) = context.justification(&key).cloned() else {
        return false;
    };
    if !justification_have_matches_key(&group, &key_for_expression(expression)) {
        emit_error(
            event_log,
            path,
            None,
            format!(
                "The `have:` of `Justification:` entry `[{key}]` must restate the labeled expression exactly"
            ),
        );
    }
    check_have_group(&group, context, path, locator, registry, event_log);
    true
}

/// The subject key of an `IsOrViaItem`, for locating diagnostics.
fn is_or_via_item_subject_key(item: &IsOrViaItem) -> Option<String> {
    match item {
        IsOrViaItem::Declaration(statement) => {
            declaration_subject_keys(statement).into_iter().next()
        }
        IsOrViaItem::Labeled { item, .. } => is_or_via_item_subject_key(item),
        IsOrViaItem::IsVia(_) | IsOrViaItem::Have(_) => None,
    }
}

/// Contributes the typing facts of an `IsOrViaItem` to `context` without checking
/// it — used after a labeled item has already been established via a
/// `Justification:` entry, so the specification is taken as given.
fn assume_is_or_via_item_facts(item: &IsOrViaItem, context: &mut TypeContext) {
    match item {
        IsOrViaItem::IsVia(statement) => {
            declare_is_subject(&statement.is_statement.subject, context);
            for fact in facts_from_is_statement(&statement.is_statement) {
                context.add_fact(fact);
            }
        }
        IsOrViaItem::Declaration(statement) => {
            declare_declaration_statement_subjects(statement, context);
            for fact in facts_from_declaration_statement_in_context(statement, context) {
                context.add_fact(fact);
            }
        }
        IsOrViaItem::Have(group) => {
            for statement in have_group_declarations(group) {
                declare_declaration_statement_subjects(statement, context);
                for fact in facts_from_declaration_statement_in_context(statement, context) {
                    context.add_fact(fact);
                }
            }
        }
        IsOrViaItem::Labeled { item, .. } => assume_is_or_via_item_facts(item, context),
    }
}

/// Resolves a `[:label:]`-labeled specification against the `Justification:`
/// entries in `context`. When the label matches an entry, verifies the entry
/// restates `item`, establishes `item` via the entry's `have:`/`asserting:`, and
/// returns `true`; a label with no matching entry returns `false` so the caller
/// checks the item inline.
fn establish_labeled_specification(
    label: &[String],
    item: &IsOrViaItem,
    context: &TypeContext,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    registry: &SignatureRegistry,
    event_log: &mut EventLog,
) -> bool {
    let key = label.join(".");
    let Some(group) = context.justification(&key).cloned() else {
        return false;
    };
    if !justification_have_matches_item(&group, item) {
        let position =
            is_or_via_item_subject_key(item).and_then(|name| locator.locate_symbol(&name));
        emit_error(
            event_log,
            path,
            position,
            format!(
                "The `have:` of `Justification:` entry `[{key}]` must restate the labeled \
                 specification exactly"
            ),
        );
    }
    check_have_group(&group, context, path, locator, registry, event_log);
    true
}

/// Collects the labels referenced by labeled specifications inside an
/// `IsOrViaItem` (the `[:label:]` of any `Labeled` wrapper).
fn collect_is_or_via_referenced_labels(item: &IsOrViaItem, labels: &mut BTreeSet<String>) {
    match item {
        IsOrViaItem::Labeled { label, item } => {
            labels.insert(label.join("."));
            collect_is_or_via_referenced_labels(item, labels);
        }
        IsOrViaItem::Declaration(statement) => {
            collect_declaration_referenced_labels(statement, labels)
        }
        IsOrViaItem::IsVia(_) | IsOrViaItem::Have(_) => {}
    }
}

fn collect_declaration_referenced_labels(
    statement: &DeclarationStatement,
    labels: &mut BTreeSet<String>,
) {
    labels.extend(statement.labels.iter().map(|label| label.parts.join(".")));
    if let Some(definition) = &statement.definition {
        collect_expression_referenced_labels(definition, labels);
    }
    match &statement.relation {
        Some(DeclarationRelation::Spec { target, .. }) => {
            collect_expression_referenced_labels(target, labels)
        }
        Some(DeclarationRelation::InfixSpec { spec, target }) => {
            for expression in infix_spec_arguments(spec) {
                collect_expression_referenced_labels(expression, labels);
            }
            collect_expression_referenced_labels(target, labels);
        }
        Some(DeclarationRelation::Is(ty)) => collect_type_referenced_labels(ty, labels),
        None => {}
    }
}

fn collect_expression_referenced_labels(expression: &Expression, labels: &mut BTreeSet<String>) {
    match &expression.kind {
        ExpressionKind::Name(_)
        | ExpressionKind::InferredName(_)
        | ExpressionKind::VariadicSlice(_)
        | ExpressionKind::SubsetCall(_) => {}
        ExpressionKind::IndexedCall(call) => {
            for index in &call.indices {
                collect_expression_referenced_labels(index, labels);
            }
        }
        ExpressionKind::VariadicAssignment { value, .. } => {
            collect_expression_referenced_labels(value, labels)
        }
        ExpressionKind::FunctionCall { arguments, .. } => {
            for argument in arguments {
                collect_expression_referenced_labels(argument, labels);
            }
        }
        ExpressionKind::FunctionNamedCall { elements, .. } => {
            for element in elements {
                collect_expression_referenced_labels(&element.expression, labels);
            }
        }
        ExpressionKind::MemberCall {
            owner, arguments, ..
        } => {
            collect_expression_referenced_labels(owner, labels);
            for argument in arguments {
                collect_expression_referenced_labels(argument, labels);
            }
        }
        ExpressionKind::MemberAccess { owner, .. }
        | ExpressionKind::Prefix {
            expression: owner, ..
        }
        | ExpressionKind::Postfix {
            expression: owner, ..
        }
        | ExpressionKind::Grouped {
            expression: owner, ..
        } => collect_expression_referenced_labels(owner, labels),
        ExpressionKind::Labeled {
            expression: inner,
            label,
        } => {
            labels.insert(label.parts.join("."));
            collect_expression_referenced_labels(inner, labels);
        }
        ExpressionKind::Tuple(elements) => {
            for element in elements {
                if let TupleExpressionElement::Expression(expression) = element {
                    collect_expression_referenced_labels(expression, labels);
                }
            }
        }
        ExpressionKind::Set(set) => {
            collect_set_target_referenced_labels(&set.target, labels);
            for spec in &set.specs {
                collect_expression_referenced_labels(spec, labels);
            }
            if let Some(predicate) = &set.predicate {
                match predicate {
                    SetPredicate::Expression(expression) => {
                        collect_expression_referenced_labels(expression, labels)
                    }
                    SetPredicate::Definition { target, value, .. } => {
                        collect_set_target_referenced_labels(target, labels);
                        collect_expression_referenced_labels(value, labels);
                    }
                }
            }
        }
        ExpressionKind::Command(command) => {
            collect_command_referenced_labels(command, labels);
        }
        ExpressionKind::BuiltinCommand(command) => {
            for args in command
                .head_args
                .iter()
                .chain(command.tail.iter().flat_map(|tail| tail.args.iter()))
            {
                for argument in &args.arguments {
                    match argument {
                        BuiltinCommandArgument::Expression(expression) => {
                            collect_expression_referenced_labels(expression, labels)
                        }
                        BuiltinCommandArgument::Declaration(statement) => {
                            collect_declaration_referenced_labels(statement, labels)
                        }
                        BuiltinCommandArgument::Text(_) => {}
                    }
                }
            }
        }
        ExpressionKind::InfixCommand {
            left,
            command,
            right,
        } => {
            collect_expression_referenced_labels(left, labels);
            for argument in infix_command_arguments(command) {
                collect_expression_referenced_labels(argument, labels);
            }
            collect_expression_referenced_labels(right, labels);
        }
        ExpressionKind::InfixSpecStatement { left, spec, right } => {
            collect_expression_referenced_labels(left, labels);
            for argument in infix_spec_arguments(spec) {
                collect_expression_referenced_labels(argument, labels);
            }
            collect_expression_referenced_labels(right, labels);
        }
        ExpressionKind::Binary { left, right, .. }
        | ExpressionKind::Satisfies {
            subject: left,
            spec: right,
        }
        | ExpressionKind::Mapping {
            lhs: left,
            rhs: right,
        }
        | ExpressionKind::MemberOf {
            subject: left,
            collection: right,
        } => {
            collect_expression_referenced_labels(left, labels);
            collect_expression_referenced_labels(right, labels);
        }
        ExpressionKind::SpecStatement(statement) | ExpressionKind::SpecPredicate(statement) => {
            collect_expression_referenced_labels(&statement.subject, labels)
        }
        ExpressionKind::SpecStatementExpr {
            subject, target, ..
        } => {
            collect_expression_referenced_labels(subject, labels);
            collect_expression_referenced_labels(target, labels);
        }
        ExpressionKind::SpecLiteral(literal) => match &literal.form {
            SpecLiteralForm::Is(ty) => collect_type_referenced_labels(ty, labels),
            SpecLiteralForm::Spec { target, .. } => {
                collect_expression_referenced_labels(target, labels)
            }
        },
        ExpressionKind::IsPredicate { subject, command }
        | ExpressionKind::IsNotPredicate { subject, command } => {
            collect_expression_referenced_labels(subject, labels);
            for argument in command_expression_arguments(command) {
                collect_expression_referenced_labels(argument, labels);
            }
        }
        ExpressionKind::IsRefinedPredicate { subject, command }
        | ExpressionKind::IsNotRefinedPredicate { subject, command } => {
            collect_expression_referenced_labels(subject, labels);
            for argument in refined_command_expression_arguments(command) {
                collect_expression_referenced_labels(argument, labels);
            }
        }
        ExpressionKind::IsBuiltinPredicate { subject, ty }
        | ExpressionKind::IsNotBuiltinPredicate { subject, ty }
        | ExpressionKind::IsType { subject, ty } => {
            collect_expression_referenced_labels(subject, labels);
            collect_type_referenced_labels(ty, labels);
        }
        ExpressionKind::Build { ty, value, .. } => {
            collect_type_referenced_labels(ty, labels);
            collect_expression_referenced_labels(value, labels);
        }
    }
}

fn collect_command_referenced_labels(command: &CommandExpression, labels: &mut BTreeSet<String>) {
    for argument in command_expression_arguments(command) {
        collect_expression_referenced_labels(argument, labels);
    }
    if let Some(command_context) = &command.context {
        for argument in &command_context.arguments {
            match argument {
                CommandContextArgument::Assignment { value, .. }
                | CommandContextArgument::Expression(value) => {
                    collect_expression_referenced_labels(value, labels)
                }
                CommandContextArgument::Declaration(statement) => {
                    collect_declaration_referenced_labels(statement, labels)
                }
                CommandContextArgument::Text(_) => {}
            }
        }
    }
}

fn collect_type_referenced_labels(ty: &TypeExpression, labels: &mut BTreeSet<String>) {
    match ty {
        TypeExpression::Builtin { .. } | TypeExpression::Parameter { .. } => {}
        TypeExpression::Command(command) => collect_command_referenced_labels(command, labels),
        TypeExpression::RefinedCommand(command) => {
            for argument in refined_command_expression_arguments(command) {
                collect_expression_referenced_labels(argument, labels);
            }
        }
        TypeExpression::Tuple(tuple) => {
            for spec in &tuple.elements {
                collect_function_type_spec_referenced_labels(spec, labels);
            }
        }
        TypeExpression::Set(set) => match &set.element {
            SetTypeElement::Spec(spec) => {
                collect_function_type_spec_referenced_labels(spec, labels)
            }
            SetTypeElement::Tuple(tuple) => {
                for spec in &tuple.elements {
                    collect_function_type_spec_referenced_labels(spec, labels);
                }
            }
        },
        TypeExpression::Function(function) => {
            for spec in function
                .inputs
                .iter()
                .chain(std::iter::once(&function.output))
            {
                collect_function_type_spec_referenced_labels(spec, labels);
            }
        }
    }
}

fn collect_function_type_spec_referenced_labels(
    spec: &FunctionTypeSpec,
    labels: &mut BTreeSet<String>,
) {
    match &spec.kind {
        FunctionTypeSpecKind::Is(ty) => collect_type_referenced_labels(ty, labels),
        FunctionTypeSpecKind::Spec { target, .. } => {
            collect_expression_referenced_labels(target, labels)
        }
    }
}

fn collect_set_target_referenced_labels(target: &SetTarget, labels: &mut BTreeSet<String>) {
    match &target.kind {
        SetTargetKind::Name(_) | SetTargetKind::PlaceholderForm(_) => {}
        SetTargetKind::Expression { expression, .. } => {
            collect_expression_referenced_labels(expression, labels)
        }
        SetTargetKind::Alias { target, .. } | SetTargetKind::Introduction { target, .. } => {
            collect_set_target_referenced_labels(target, labels)
        }
        SetTargetKind::Function { arguments, .. } => {
            for argument in arguments {
                collect_set_target_referenced_labels(argument, labels);
            }
        }
        SetTargetKind::Tuple(elements) => {
            for element in elements {
                if let SetTargetElement::Target(target) = element {
                    collect_set_target_referenced_labels(target, labels);
                }
            }
        }
    }
}

fn collect_clause_referenced_labels(clause: &Clause, labels: &mut BTreeSet<String>) {
    match clause {
        Clause::Not(group) => collect_clause_referenced_labels(&group.not.argument, labels),
        Clause::AllOf(group) => {
            for clause in &group.all_of.arguments {
                collect_clause_referenced_labels(clause, labels);
            }
        }
        Clause::AnyOf(group) => {
            for clause in &group.any_of.arguments {
                collect_clause_referenced_labels(clause, labels);
            }
        }
        Clause::OneOf(group) => {
            for clause in &group.one_of.arguments {
                collect_clause_referenced_labels(clause, labels);
            }
        }
        Clause::Equivalently(group) => {
            for clause in &group.equivalently.arguments {
                collect_clause_referenced_labels(clause, labels);
            }
        }
        Clause::Exists(group) => {
            for item in &group.exists.arguments {
                let BindingOrSpec::Declaration(statement) = item;
                collect_declaration_referenced_labels(statement, labels);
            }
            if let Some(section) = &group.such_that {
                for clause in &section.arguments {
                    collect_clause_referenced_labels(clause, labels);
                }
            }
        }
        Clause::ExistsUnique(group) => {
            for item in &group.exists_unique.arguments {
                let BindingOrSpec::Declaration(statement) = item;
                collect_declaration_referenced_labels(statement, labels);
            }
            if let Some(section) = &group.such_that {
                for clause in &section.arguments {
                    collect_clause_referenced_labels(clause, labels);
                }
            }
        }
        Clause::ForAll(group) => {
            for item in &group.for_all.arguments {
                let BindingOrSpec::Declaration(statement) = item;
                collect_declaration_referenced_labels(statement, labels);
            }
            if let Some(section) = &group.where_ {
                for clause in &section.arguments {
                    collect_clause_referenced_labels(clause, labels);
                }
            }
            for clause in &group.then.arguments {
                collect_clause_referenced_labels(clause, labels);
            }
        }
        Clause::Let(group) => {
            for item in &group.let_.arguments {
                let BindingOrSpec::Declaration(statement) = item;
                collect_declaration_referenced_labels(statement, labels);
            }
            if let Some(section) = &group.where_ {
                for clause in &section.arguments {
                    collect_clause_referenced_labels(clause, labels);
                }
            }
            for clause in &group.then.arguments {
                collect_clause_referenced_labels(clause, labels);
            }
        }
        Clause::If(group) => {
            for clause in group
                .if_
                .arguments
                .iter()
                .chain(group.then.arguments.iter())
            {
                collect_clause_referenced_labels(clause, labels);
            }
        }
        Clause::Iff(group) => {
            for clause in group
                .iff
                .arguments
                .iter()
                .chain(group.then.arguments.iter())
            {
                collect_clause_referenced_labels(clause, labels);
            }
        }
        Clause::Piecewise(group) => {
            for clause in group
                .if_
                .arguments
                .iter()
                .chain(group.then.arguments.iter())
            {
                collect_clause_referenced_labels(clause, labels);
            }
            if let Some(section) = &group.else_ {
                for clause in &section.arguments {
                    collect_clause_referenced_labels(clause, labels);
                }
            }
        }
        Clause::Given(group) => {
            for statement in &group.given.arguments {
                collect_declaration_referenced_labels(statement, labels);
            }
            if let Some(section) = &group.where_ {
                for clause in &section.arguments {
                    collect_clause_referenced_labels(clause, labels);
                }
            }
            for clause in &group.then.arguments {
                collect_clause_referenced_labels(clause, labels);
            }
        }
        Clause::Have(group) => {
            for clause in group
                .have
                .arguments
                .iter()
                .chain(group.asserting.arguments.iter())
            {
                collect_clause_referenced_labels(clause, labels);
            }
        }
        Clause::Declaration(statement) => collect_declaration_referenced_labels(statement, labels),
        Clause::Expression(expression) => collect_expression_referenced_labels(expression, labels),
    }
}

/// Reports each `Justification:` entry of a `Defines:` group that no labeled
/// specification references, and each entry that lacks a `[label]` heading (which
/// can never be referenced). Every entry must justify some labeled item.
fn validate_defines_justification_usage(
    group: &DefinesGroup,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    event_log: &mut EventLog,
) {
    let Some(justification) = &group.justification else {
        return;
    };
    let mut referenced = BTreeSet::new();
    if let DefinesTarget::Declaration(statement) = &group.defines.argument {
        collect_declaration_referenced_labels(statement, &mut referenced);
    }
    if let Some(extends) = &group.extends {
        for item in &extends.arguments {
            collect_declaration_referenced_labels(&item.statement, &mut referenced);
        }
    }
    if let Some(means) = &group.means {
        for item in &means.arguments {
            collect_is_or_via_referenced_labels(item, &mut referenced);
        }
    }
    if let Some(using) = &group.using {
        for statement in &using.arguments {
            collect_declaration_referenced_labels(statement, &mut referenced);
        }
    }
    if let Some(when) = &group.when {
        for clause in &when.arguments {
            collect_clause_referenced_labels(clause, &mut referenced);
        }
    }
    if let Some(satisfies) = &group.satisfies {
        for clause in &satisfies.arguments {
            collect_clause_referenced_labels(clause, &mut referenced);
        }
    }
    for entry in &justification.arguments {
        match &entry.heading {
            Some(heading) if referenced.contains(&heading.parts.join(".")) => {}
            Some(heading) => {
                let key = heading.parts.join(".");
                emit_error(
                    event_log,
                    path,
                    locator.locate_symbol(heading.parts.last().map(String::as_str).unwrap_or("")),
                    format!(
                        "`Justification:` entry `[{key}]` is not referenced by any labeled \
                         specification; every entry must justify a labeled item"
                    ),
                );
            }
            None => emit_error(
                event_log,
                path,
                None,
                "Each `Justification:` entry must have a `[label]` heading so a labeled \
                 specification can reference it",
            ),
        }
    }
}

/// Assumes an `asserting:` clause as true. Unlike [`assume_clause`], a spec or
/// infix-spec *question* (`A \:subset?:/ B`) is taken as its underlying fact,
/// because an assertion states that the statement holds.
fn assume_asserted_clause(
    clause: &Clause,
    context: &mut TypeContext,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    registry: &SignatureRegistry,
    event_log: &mut EventLog,
) {
    if let Clause::Expression(expression) = clause
        && let Some(fact) = asserted_fact_from_expression(expression, context)
    {
        let fact = context.normalize_fact(&fact);
        context.add_fact(fact);
        return;
    }
    assume_clause(clause, context, path, locator, registry, event_log);
}

/// The fact an `asserting:` item states. Like [`fact_from_expression_in_context`]
/// but an infix-spec question (`A \:subset?:/ B`) yields its `A \:subset:/ B`
/// fact rather than being ignored as a mere predicate.
fn asserted_fact_from_expression(
    expression: &Expression,
    context: &TypeContext,
) -> Option<TypeFact> {
    if let ExpressionKind::InfixSpecStatement { left, spec, right } = &expression.kind {
        return fact_from_infix_spec_statement_in_context(left, spec, right, context);
    }
    fact_from_expression_in_context(expression, context)
}

fn declare_defines_target(target: &DefinesTarget, context: &mut TypeContext) {
    match target {
        DefinesTarget::Form(form) => declare_form_or_declaration(form, context),
        DefinesTarget::Declaration(statement) => {
            declare_declaration_statement_subjects(statement, context)
        }
    }
}

fn check_defines_target(
    defines: &DefinesSection,
    context: &TypeContext,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    registry: &SignatureRegistry,
    event_log: &mut EventLog,
) {
    match &defines.argument {
        DefinesTarget::Form(form) => {
            check_form_or_declaration(form, context, path, locator, event_log);
        }
        DefinesTarget::Declaration(statement) => {
            check_declaration_statement(statement, context, path, locator, registry, event_log);
        }
    }
    if let Some(via) = &defines.via {
        check_form_or_declaration(via, context, path, locator, event_log);
    }
}

fn check_optional_extends(
    extends: &Option<ExtendsSection>,
    context: &TypeContext,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    registry: &SignatureRegistry,
    event_log: &mut EventLog,
) {
    let Some(extends) = extends else {
        return;
    };
    for item in &extends.arguments {
        check_declaration_statement(&item.statement, context, path, locator, registry, event_log);
        if let Some(via) = &item.via {
            check_form_or_declaration(via, context, path, locator, event_log);
        }
    }
}

fn assume_defines_function_type(
    target: &DefinesTarget,
    means: &Option<DefinesMeansSection>,
    context: &mut TypeContext,
) {
    let Some(means) = means else {
        return;
    };
    if let Some(fact) = function_type_fact_from_defines_means(target, means, context) {
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

/// The header parameters a `Defines` group's `when:` may constrain.
///
/// A spec-infix heading such as `[A \:subset:/ B]` is sugar for a command whose
/// left operand is the symbol being defined, so `A` is excluded: what `A` is
/// belongs on the `Defines:` target, and `when:` says only what a *use* of the
/// command requires of the remaining operands.
fn defines_when_parameters(group: &DefinesGroup) -> WhenParameters {
    let mut parameters = header_when_parameters(&group.heading);
    if let Some(subject) = described_spec_infix_subject(&group.heading, &group.defines.argument) {
        parameters.required.remove(&subject);
        parameters.allowed.remove(&subject);
        parameters.described = Some(subject);
    }
    parameters
}

fn defines_when_parameters_from_usage(group: &DefinesGroup) -> WhenParameters {
    let mut parameters = defines_when_parameters(group);
    let described_spec_infix_subject = parameters.described.clone();
    // Components of a destructuring parameter are typed from the parameter's own
    // type, so using them in the body must not turn them into `when:`-required
    // parameters.
    let destructured_components = header_destructured_component_names(&group.heading);
    let variadic_auxiliary_names = header_variadic_parameters(&group.heading)
        .into_iter()
        .flat_map(variadic_parameter_auxiliary_names)
        .collect::<HashSet<_>>();
    for name in defines_used_names(group) {
        if described_spec_infix_subject.as_ref() == Some(&name) {
            continue;
        }
        if destructured_components.contains(&name) {
            continue;
        }
        if variadic_auxiliary_names.contains(&name) {
            continue;
        }
        if parameters.allowed.contains(&name) {
            parameters.require(name);
        }
    }
    parameters
}

fn refines_when_parameters(group: &RefinesGroup) -> WhenParameters {
    let mut parameters = header_when_parameters(&group.heading);
    if let CommandHeader::InfixSpec(header) = &group.heading
        && header.refinement.is_some()
    {
        let subject = form_or_declaration_subject_key(&header.left);
        if subject == primary_subject_key(&group.refines.argument.subject) {
            parameters.required.remove(&subject);
            parameters.allowed.remove(&subject);
            parameters.described = Some(subject);
        }
    }
    parameters
}

fn refines_when_parameters_from_usage(group: &RefinesGroup) -> WhenParameters {
    refines_when_parameters(group)
}

/// Names of every component of a named destructuring parameter `M ::= (X, *)` in
/// the header, e.g. `{X, *}`.
fn header_destructured_component_names(header: &CommandHeader) -> HashSet<String> {
    let mut components = HashSet::new();
    for form in header_parameter_forms(header) {
        if let FormOrDeclarationKind::TupleDeclaration {
            name: Some(_),
            form: tuple,
        } = &form.kind
        {
            collect_tuple_form_when_parameters(tuple, &mut components);
        }
    }
    components
}

fn described_spec_infix_subject(
    heading: &CommandHeader,
    described: &DefinesTarget,
) -> Option<String> {
    let CommandHeader::InfixSpec(header) = heading else {
        return None;
    };
    let subject = form_or_declaration_subject_key(&header.left);
    if subject == described_target_subject_key(described) {
        Some(subject)
    } else {
        None
    }
}

/// The subject name of a form or declaration — the destructuring name `H` for
/// `H ::= (X1, *_1, e1)`, or the whole key when the form has no primary name.
/// Matches [`described_target_subject_key`] so a spec-infix heading's left
/// operand and the `Defines:` argument compare on the same footing.
fn form_or_declaration_subject_key(form: &FormOrDeclaration) -> String {
    primary_form_name(form).unwrap_or_else(|| key_for_form_or_declaration(form))
}

fn defines_used_names(group: &DefinesGroup) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    collect_extends_clause_names(&group.defines, group.extends.as_ref(), &mut names);
    if let Some(means) = &group.means {
        for item in &means.arguments {
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

fn validate_defines_target_symbol_specifications(
    group: &DefinesGroup,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    event_log: &mut EventLog,
) {
    let mut covered = BTreeSet::new();
    covered.insert(described_target_subject_key(&group.defines.argument));
    collect_defines_mapping_alias_names(&group.defines.argument, &mut covered);
    collect_using_covered_symbols(&group.using, &mut covered);
    collect_valid_when_covered_symbols(&group.when, &defines_when_parameters(group), &mut covered);
    collect_means_covered_symbols(&group.means, &mut covered);
    if let DefinesTarget::Declaration(statement) = &group.defines.argument {
        collect_declaration_statement_covered_symbols(statement, &mut covered);
    }
    collect_extends_clause_covered_symbols(&group.defines, group.extends.as_ref(), &mut covered);
    let symbols = defines_target_symbols(&group.defines.argument);
    validate_target_symbol_specifications(&symbols, &covered, path, locator, event_log);
    validate_single_symbol_specification(group, &symbols, path, locator, event_log);
}

/// A function declaration alias names the same mapping as the function form,
/// so either name covers the other without requiring a second specification.
fn collect_defines_mapping_alias_names(target: &DefinesTarget, covered: &mut BTreeSet<String>) {
    if let Some((alias, mapping)) = defines_target_mapping_alias(target) {
        covered.insert(alias);
        covered.insert(mapping);
    }
}

/// The two names an aliased mapping target goes by — the alias `X` and the
/// mapping's own name `x` in `X ::= x(i_)`.
///
/// A target that names the type it extends splits the two across the
/// declaration's subject and expansion; one that does not keeps them together in
/// a single named function declaration.
fn defines_target_mapping_alias(target: &DefinesTarget) -> Option<(String, String)> {
    match target {
        DefinesTarget::Form(FormOrDeclaration {
            kind:
                FormOrDeclarationKind::FunctionDeclaration {
                    name: Some(name),
                    form,
                },
            ..
        }) => Some((name.clone(), form.name.clone())),
        DefinesTarget::Declaration(statement) => {
            let subject = is_subject_first_form(&statement.subject)?;
            // `X ::= x(i_) ::= y_` keeps both names in the subject.
            if let FormOrDeclarationKind::FunctionDeclaration {
                name: Some(name),
                form,
            } = &subject.kind
            {
                return Some((name.clone(), form.name.clone()));
            }
            // `X ::= x(i_) is …` splits the alias `X` from the mapping `x(i_)`.
            let alias = primary_form_name(subject)?;
            let expansion = is_subject_first_form(statement.expansion.as_ref()?)?;
            let FormOrDeclarationKind::FunctionDeclaration { form, .. } = &expansion.kind else {
                return None;
            };
            Some((alias, form.name.clone()))
        }
        _ => None,
    }
}

fn validate_declares_target_symbol_specifications(
    group: &DeclaresGroup,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    event_log: &mut EventLog,
) {
    let mut covered = BTreeSet::new();
    let mut assigned = HashSet::new();
    collect_using_covered_symbols(&group.using, &mut covered);
    collect_valid_when_covered_symbols(
        &group.when,
        &header_when_parameters(&group.heading),
        &mut covered,
    );
    collect_declaration_statement_covered_symbols(&group.declares.argument, &mut covered);
    collect_direct_declares_bindings(
        &group.declares.argument,
        &mut covered,
        &mut assigned,
        path,
        locator,
        event_log,
    );
    collect_expresses_bindings(
        &group.expresses,
        &mut covered,
        &mut assigned,
        path,
        locator,
        event_log,
    );
    collect_means_bindings(
        &group.means,
        &mut covered,
        &mut assigned,
        path,
        locator,
        event_log,
    );
    // A destructuring subject is determined by its components, so once `means:`
    // supplies each of them the subject itself needs no separate definition.
    if group.means.is_some()
        && let Some(expansion) = &group.declares.argument.expansion
    {
        let mut components = BTreeSet::new();
        collect_is_subject_target_symbols(expansion, &mut components);
        if !components.is_empty() && components.iter().all(|name| covered.contains(name)) {
            covered.extend(declaration_subject_keys(&group.declares.argument));
        }
    }

    // The `Declares:` value is assigned but states no type — a bare `X := {…}` with no
    // `is` and no top-level `\ty@value` build. Report that precisely (and mark the
    // targets covered so the generic "missing definition" message does not also fire).
    if group.declares.argument.definition.is_some()
        && !declaration_states_type(&group.declares.argument)
        && group.means.is_none()
    {
        for symbol in declaration_target_symbols(&group.declares.argument) {
            if covered.insert(symbol.clone()) {
                emit_error(
                    event_log,
                    path,
                    locator.locate_symbol(&symbol),
                    format!(
                        "`Declares:` target `{symbol}` must state its type: use `... is <type>` \
                         or a top-level `\\...@...` build (e.g. `\\set@{{...}}`)"
                    ),
                );
            }
        }
    }

    let symbols = declaration_target_symbols(&group.declares.argument);
    validate_declares_target_symbol_bindings(&symbols, &covered, path, locator, event_log);
}

/// Records the symbols a `means:` section accounts for.
///
/// A `:=` item defines its subject outright. A specification item (`N is \set`)
/// also counts: in an abstract declaration the symbol is deliberately left for a
/// `Realizes:`, and in a concrete one the missing value is reported precisely by
/// [`validate_concrete_means_items`] rather than as a bare missing definition.
fn collect_means_bindings(
    means: &Option<DeclaresMeansSection>,
    covered: &mut BTreeSet<String>,
    assigned: &mut HashSet<String>,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    event_log: &mut EventLog,
) {
    for statement in means_statements(means) {
        if statement.definition.is_some() {
            // A `means:` item names one symbol and gives it a value, so its
            // subject is the binding; a destructuring item also binds the parts
            // its value supplies.
            let symbols = if statement.expansion.is_some() {
                direct_definition_binding_symbols(statement)
            } else {
                declaration_subject_keys(statement)
            };
            for symbol in symbols {
                record_declares_binding(&symbol, covered, assigned, path, locator, event_log);
            }
            continue;
        }
        if statement.relation.is_some() {
            covered.extend(declaration_subject_keys(statement));
        }
    }
}

fn validate_refines_target_symbol_specifications(
    group: &RefinesGroup,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    registry: &SignatureRegistry,
    event_log: &mut EventLog,
) {
    let mut covered = BTreeSet::new();
    covered.insert(primary_subject_key(&group.refines.argument.subject));
    if let Some(expansion) = &group.refines.argument.expansion {
        collect_is_subject_covered_symbols(expansion, &mut covered);
    }
    collect_using_covered_symbols(&group.using, &mut covered);
    collect_valid_when_covered_symbols(&group.when, &refines_when_parameters(group), &mut covered);
    if let Some(extends) = &group.extends {
        collect_declaration_statement_covered_symbols(&extends.argument, &mut covered);
    }
    // A `Refines:` refines a base type that describes the same form, so any
    // symbol the base type already declares (its operator or components) is
    // inherited and need not be respecified here. For `\(associative)::binary
    // .operation:on{X}` refining `\binary.operation:on{X}`, the base's
    // `Defines: x_ * y_ is \function:...` covers `*`.
    collect_refines_base_specified_symbols(&group.heading, registry, &mut covered);
    validate_declaration_target_symbol_specifications(
        &group.refines.argument,
        &covered,
        path,
        locator,
        event_log,
    );
}

/// Adds to `covered` every symbol that the base type refined by `heading`
/// already declares — the subjects of the base type's own extended-type and
/// `means:` facts (recorded as type-extension rules) and its described
/// components.
fn collect_refines_base_specified_symbols(
    heading: &CommandHeader,
    registry: &SignatureRegistry,
    covered: &mut BTreeSet<String>,
) {
    let CommandHeader::Refined(_) = heading else {
        return;
    };
    for header_shape in shapes_for_header(heading) {
        let Some((_, base_signature)) = refined_header_base_type_fact_parts(&header_shape) else {
            continue;
        };
        for rule in &registry.extension_rules {
            if rule.subtype_signature == base_signature {
                covered.insert(rule.subject.clone());
            }
        }
        if let Some(info) = registry.type_infos.get(&base_signature) {
            for fact in &info.component_types {
                covered.insert(fact_subject(fact).to_string());
            }
        }
    }
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

/// Reports each target symbol whose type a `Defines` group states more than
/// once.
///
/// A symbol carries one type, so stating it twice is redundant at best and
/// contradictory at worst. The usual case is a `via` that already types a
/// component — `Defines: G ::= (X, *, e) is \set via X` states `X is \set` — and
/// a `means:` item that repeats it. Components the subtype clauses do not reach
/// (`*` and `e` above) still have to be typed in `means:`; that is the
/// complementary rule in [`validate_target_symbol_specifications`].
///
/// All of the group's subtype clauses count as a *single* source. `extends:`
/// exists precisely so one definition can extend several types, so two clauses
/// may legitimately name the same subject (`X is \bar via (A, B)` and
/// `X is \baz via (B, C)`) or reach the same component through different views.
///
/// `when:` and `using:` are not specification sources in this sense: `when:`
/// states what a *use* of the command requires and `using:` introduces auxiliary
/// symbols, neither of which is the definition saying what its own target is.
fn validate_single_symbol_specification(
    group: &DefinesGroup,
    symbols: &BTreeSet<String>,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    event_log: &mut EventLog,
) {
    let mut sites: Vec<(String, &'static str)> = Vec::new();

    let extends_source = if group.extends.is_some() {
        "an `extends:` clause"
    } else {
        "the `Defines:` target"
    };
    let mut extended = BTreeSet::new();
    for clause in extends_clauses(&group.defines, group.extends.as_ref()) {
        collect_is_subject_covered_symbols(&clause.statement.subject, &mut extended);
        if let Some(via) = clause.via {
            collect_form_or_declaration_target_symbols(via, &mut extended);
        }
    }
    sites.extend(extended.into_iter().map(|symbol| (symbol, extends_source)));

    if let Some(means) = &group.means {
        for item in &means.arguments {
            let mut item_symbols = BTreeSet::new();
            collect_is_or_via_covered_symbols(item, &mut item_symbols);
            sites.extend(item_symbols.into_iter().map(|symbol| (symbol, "`means:`")));
        }
    }

    let mut first: HashMap<&str, &'static str> = HashMap::new();
    for (symbol, source) in &sites {
        if !symbols.contains(symbol) {
            continue;
        }
        match first.get(symbol.as_str()) {
            None => {
                first.insert(symbol, source);
            }
            Some(previous) => emit_error(
                event_log,
                path,
                locator.locate_symbol(symbol),
                format!(
                    "Duplicate specification for target symbol `{symbol}`; it is already specified by {previous}"
                ),
            ),
        }
    }
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
                "Missing specification for target symbol `{symbol}`; specify it directly or through the type the `Defines:` target extends"
            ),
        );
    }
}

fn validate_declares_target_symbol_bindings(
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
                "Missing definition for target symbol `{symbol}`; assign it with `:=` in `Declares:` or top-level `expresses:`"
            ),
        );
    }
}

fn collect_direct_declares_bindings(
    statement: &DeclarationStatement,
    covered: &mut BTreeSet<String>,
    assigned: &mut HashSet<String>,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    event_log: &mut EventLog,
) {
    for symbol in direct_definition_binding_symbols(statement) {
        record_declares_binding(&symbol, covered, assigned, path, locator, event_log);
    }
}

fn collect_expresses_bindings(
    expresses: &Option<ExpressesSection>,
    covered: &mut BTreeSet<String>,
    assigned: &mut HashSet<String>,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    event_log: &mut EventLog,
) {
    let Some(expresses) = expresses else {
        return;
    };
    for clause in &expresses.arguments {
        let Clause::Declaration(statement) = clause else {
            continue;
        };
        if statement.definition.is_none()
            || statement.expansion.is_some()
            || statement.relation.is_some()
        {
            continue;
        }
        for symbol in declaration_subject_keys(statement) {
            record_declares_binding(&symbol, covered, assigned, path, locator, event_log);
        }
    }
}

fn record_declares_binding(
    symbol: &str,
    covered: &mut BTreeSet<String>,
    assigned: &mut HashSet<String>,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    event_log: &mut EventLog,
) {
    covered.insert(symbol.to_owned());
    if assigned.insert(symbol.to_owned()) {
        return;
    }
    emit_error(
        event_log,
        path,
        locator.locate_symbol(symbol),
        format!(
            "Duplicate definition for target symbol `{symbol}`; a symbol can have at most one `:=` in a `Declares:` item"
        ),
    );
}

fn direct_definition_binding_symbols(statement: &DeclarationStatement) -> Vec<String> {
    let (Some(expansion), Some(definition)) = (&statement.expansion, &statement.definition) else {
        return Vec::new();
    };
    let symbols = is_subject_binding_symbols(expansion);
    if symbols.is_empty() {
        return Vec::new();
    }
    if symbols.len() == 1 || expression_binding_value_count(definition) == symbols.len() {
        symbols
    } else {
        Vec::new()
    }
}

fn is_subject_binding_symbols(subject: &IsSubject) -> Vec<String> {
    let mut symbols = BTreeSet::new();
    collect_is_subject_target_symbols(subject, &mut symbols);
    symbols.into_iter().collect()
}

fn expression_binding_value_count(expression: &Expression) -> usize {
    match &expression.kind {
        ExpressionKind::Tuple(elements) => elements
            .iter()
            .map(|element| match element {
                TupleExpressionElement::Expression(expression) => {
                    expression_binding_value_count(expression)
                }
                TupleExpressionElement::Operator(_) => 1,
            })
            .sum(),
        _ => 1,
    }
}

fn collect_using_covered_symbols(using: &Option<UsingSection>, covered: &mut BTreeSet<String>) {
    if let Some(using) = using {
        for statement in &using.arguments {
            collect_declaration_statement_covered_symbols(statement, covered);
        }
    }
}

fn collect_means_covered_symbols(
    means: &Option<DefinesMeansSection>,
    covered: &mut BTreeSet<String>,
) {
    if let Some(means) = means {
        for item in &means.arguments {
            collect_is_or_via_covered_symbols(item, covered);
        }
    }
}

/// Adds the symbols the definition specifies through the types it extends: each
/// clause's subject, and every symbol of its `via` view (whose types the
/// extended type supplies).
fn collect_extends_clause_covered_symbols(
    defines: &DefinesSection,
    extends: Option<&ExtendsSection>,
    covered: &mut BTreeSet<String>,
) {
    for clause in extends_clauses(defines, extends) {
        collect_declaration_statement_covered_symbols(clause.statement, covered);
        if let Some(via) = clause.via {
            collect_form_or_declaration_target_symbols(via, covered);
        }
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
        IsOrViaItem::Have(group) => {
            for statement in have_group_declarations(group) {
                collect_declaration_statement_covered_symbols(statement, covered);
            }
        }
        IsOrViaItem::Labeled { item, .. } => {
            collect_is_or_via_covered_symbols(item, covered);
        }
    }
}

fn collect_declaration_statement_covered_symbols(
    statement: &DeclarationStatement,
    covered: &mut BTreeSet<String>,
) {
    // A definition covers (declares the type of) its target subject only when it
    // states that type explicitly: an `is`/`"op"` relation, or a top-level
    // `\ty@value` build (whose type is `\ty`). A bare definition — `X := {…}` with no
    // relation and no top-level build — states no type, so it leaves the subject
    // uncovered even when a type could otherwise be inferred (e.g. a collection
    // literal that is provably a set).
    if !declaration_states_type(statement) {
        return;
    }
    collect_is_subject_covered_symbols(&statement.subject, covered);
}

/// Whether a definition explicitly states the type of its subject: via an `is`/spec
/// relation, or a top-level `\ty@value` build.
fn declaration_states_type(statement: &DeclarationStatement) -> bool {
    statement.relation.is_some()
        || matches!(
            statement
                .definition
                .as_ref()
                .map(|definition| &definition.kind),
            Some(ExpressionKind::Build { .. })
        )
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

/// Ordered component names of a tuple form, e.g. `(X, *)` -> `["X", "*"]`.
fn tuple_form_component_names(form: &TupleForm) -> Vec<String> {
    form.elements
        .iter()
        .map(|element| match element {
            TupleFormElement::Form(form) => {
                primary_form_name(form).unwrap_or_else(|| key_for_form_or_declaration(form))
            }
            TupleFormElement::Operator(operator) => operator.text.clone(),
        })
        .collect()
}

fn tuple_form_component_shapes(form: &TupleForm) -> Vec<TargetShape> {
    form.elements
        .iter()
        .map(|element| match element {
            TupleFormElement::Form(form) => form_shape(form),
            TupleFormElement::Operator(_) => TargetShape::Operator,
        })
        .collect()
}

/// The tuple form of a destructuring form-or-declaration `Name ::= (c1, ..., cn)`.
fn form_or_declaration_tuple_form(form: &FormOrDeclaration) -> Option<&TupleForm> {
    match &form.kind {
        FormOrDeclarationKind::TupleDeclaration { form, .. } => Some(form),
        _ => None,
    }
}

fn is_subject_first_form(subject: &IsSubject) -> Option<&FormOrDeclaration> {
    match &subject.kind {
        IsSubjectKind::Forms(forms) => forms.iter().find_map(|form| match form {
            IsSubjectForm::Form(form) => Some(form),
            IsSubjectForm::PlaceholderForm(_) => None,
        }),
        IsSubjectKind::Operator(_) => None,
    }
}

/// Ordered component names of a describes target `Name ::= (c1, ..., cn)`, or an
/// empty vector when the target does not destructure a tuple.
fn defines_target_component_names(target: &DefinesTarget) -> Vec<String> {
    defines_target_tuple_form(target)
        .map(tuple_form_component_names)
        .unwrap_or_default()
}

fn defines_target_component_shapes(target: &DefinesTarget) -> Vec<TargetShape> {
    defines_target_tuple_form(target)
        .map(tuple_form_component_shapes)
        .unwrap_or_default()
}

fn defines_target_tuple_form(target: &DefinesTarget) -> Option<&TupleForm> {
    match target {
        DefinesTarget::Form(form) => form_or_declaration_tuple_form(form),
        DefinesTarget::Declaration(statement) => is_subject_first_form(&statement.subject)
            .and_then(form_or_declaration_tuple_form)
            .or_else(|| {
                statement
                    .expansion
                    .as_ref()
                    .and_then(is_subject_first_form)
                    .and_then(form_or_declaration_tuple_form)
            }),
    }
}

fn defines_target_symbols(target: &DefinesTarget) -> BTreeSet<String> {
    let mut symbols = BTreeSet::new();
    match target {
        DefinesTarget::Form(form) => {
            collect_form_or_declaration_target_symbols(form, &mut symbols);
        }
        DefinesTarget::Declaration(statement) => {
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
        FormOrDeclarationKind::MappingParameter { selector, .. } => {
            symbols.insert(selector.name().to_owned());
        }
        FormOrDeclarationKind::FunctionDeclaration { name, form } => {
            if let Some(name) = name {
                symbols.insert(name.clone());
            }
            symbols.insert(form.name.clone());
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
        Clause::Equivalently(group) => {
            for clause in &group.equivalently.arguments {
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
        Clause::Let(group) => {
            for item in &group.let_.arguments {
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
        Clause::Have(group) => collect_have_group_names(group, names),
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

/// Collects the names the definition's subtype clauses reference — each clause's
/// subject, the type it extends, and its `via` view. A `Defines:` target's
/// destructuring and defining value are excluded: they are the shape of the
/// definition itself rather than a use of a symbol declared elsewhere.
fn collect_extends_clause_names(
    defines: &DefinesSection,
    extends: Option<&ExtendsSection>,
    names: &mut BTreeSet<String>,
) {
    for clause in extends_clauses(defines, extends) {
        collect_is_subject_names(&clause.statement.subject, names);
        match &clause.statement.relation {
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
        if let Some(via) = clause.via {
            collect_form_or_declaration_names(via, names);
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
        IsOrViaItem::Have(group) => collect_have_group_names(group, names),
        IsOrViaItem::Labeled { item, .. } => collect_is_or_via_names(item, names),
    }
}

/// Collects the names referenced anywhere in a `have:` group.
fn collect_have_group_names(group: &HaveGroup, names: &mut BTreeSet<String>) {
    for clause in &group.have.arguments {
        collect_clause_names(clause, names);
    }
    for clause in &group.asserting.arguments {
        collect_clause_names(clause, names);
    }
    if let Some(section) = &group.because {
        for clause in &section.arguments {
            collect_clause_names(clause, names);
        }
    }
    if let Some(section) = &group.by {
        for expression in &section.arguments {
            collect_expression_names(expression, names);
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
        FormOrDeclarationKind::MappingParameter { owner, selector } => {
            names.insert(owner.clone());
            names.insert(selector.name().to_owned());
        }
        FormOrDeclarationKind::FunctionDeclaration { name, form } => {
            if let Some(name) = name {
                names.insert(name.clone());
            }
            names.insert(form.name.clone());
            if let Some(placeholder) = &form.magnetic_placeholder {
                names.insert(placeholder.name.clone());
            }
            for placeholder in &form.placeholders {
                names.insert(placeholder.name.clone());
            }
            if let Some(parameter) = &form.variadic_parameter {
                names.insert(parameter.name.clone());
                names.insert(parameter.index.clone());
                names.insert(parameter.length.clone());
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
            collect_set_target_names(&form.target, names);
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
        ExpressionKind::Name(name) | ExpressionKind::InferredName(name) => {
            names.insert(name.clone());
        }
        ExpressionKind::VariadicSlice(slice) => {
            names.insert(slice.name.clone());
            names.extend(variadic_slice_referenced_names(slice));
        }
        ExpressionKind::VariadicAssignment { target, value } => {
            names.insert(target.name.clone());
            names.extend(variadic_slice_referenced_names(target));
            collect_expression_names(value, names);
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
        ExpressionKind::IndexedCall(call) => {
            names.insert(call.target.clone());
            for index in &call.indices {
                collect_expression_names(index, names);
            }
        }
        ExpressionKind::Command(command) => collect_command_expression_names(command, names),
        ExpressionKind::BuiltinCommand(_) => {}
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
        ExpressionKind::Build { ty, value, .. } => {
            collect_type_expression_names(ty, names);
            collect_expression_names(value, names);
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
        ExpressionKind::SpecStatementExpr {
            subject, target, ..
        } => {
            collect_expression_names(subject, names);
            collect_expression_names(target, names);
        }
        ExpressionKind::SpecLiteral(literal) => match &literal.form {
            SpecLiteralForm::Is(ty) => collect_type_expression_names(ty, names),
            SpecLiteralForm::Spec { target, .. } => collect_expression_names(target, names),
        },
        ExpressionKind::Satisfies { subject, spec } => {
            collect_expression_names(subject, names);
            collect_expression_names(spec, names);
        }
        ExpressionKind::Mapping { lhs, rhs } => {
            collect_expression_names(lhs, names);
            collect_expression_names(rhs, names);
        }
    }
}

fn collect_type_expression_names(ty: &TypeExpression, names: &mut BTreeSet<String>) {
    match ty {
        TypeExpression::Builtin { .. } => {}
        TypeExpression::Parameter { name, .. } => {
            names.insert(name.clone());
        }
        TypeExpression::Command(command) => collect_command_expression_names(command, names),
        TypeExpression::RefinedCommand(command) => {
            collect_refined_command_expression_names(command, names);
        }
        TypeExpression::Tuple(tuple) => {
            for spec in &tuple.elements {
                collect_function_type_spec_names(spec, names);
            }
        }
        TypeExpression::Set(set) => match &set.element {
            SetTypeElement::Spec(spec) => collect_function_type_spec_names(spec, names),
            SetTypeElement::Tuple(tuple) => {
                for spec in &tuple.elements {
                    collect_function_type_spec_names(spec, names);
                }
            }
        },
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

fn collect_function_type_spec_names(spec: &FunctionTypeSpec, names: &mut BTreeSet<String>) {
    if spec.subject != "_" && spec.subject != "?" {
        names.insert(spec.subject.clone());
    }
    match &spec.kind {
        FunctionTypeSpecKind::Is(ty) => collect_type_expression_names(ty, names),
        FunctionTypeSpecKind::Spec { target, .. } => collect_expression_names(target, names),
    }
}

fn collect_set_expression_names(set: &SetExpression, names: &mut BTreeSet<String>) {
    collect_set_target_names(&set.target, names);
    for spec in &set.specs {
        collect_expression_names(spec, names);
    }
    if let Some(predicate) = &set.predicate {
        collect_set_predicate_names(predicate, names);
    }
}

fn collect_set_predicate_names(predicate: &SetPredicate, names: &mut BTreeSet<String>) {
    match predicate {
        SetPredicate::Expression(expression) => collect_expression_names(expression, names),
        SetPredicate::Definition { target, value, .. } => {
            collect_set_target_names(target, names);
            collect_expression_names(value, names);
        }
    }
}

fn collect_set_target_names(target: &SetTarget, names: &mut BTreeSet<String>) {
    match &target.kind {
        SetTargetKind::Name(name) => {
            names.insert(name.clone());
        }
        SetTargetKind::PlaceholderForm(form) => collect_placeholder_form_names(form, names),
        SetTargetKind::Expression { expression, .. } => collect_expression_names(expression, names),
        SetTargetKind::Alias { name, target } | SetTargetKind::Introduction { name, target } => {
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
    if let Some(context) = &command.context {
        for argument in &context.arguments {
            match argument {
                CommandContextArgument::Assignment { value, .. } => {
                    collect_expression_names(value, names);
                }
                CommandContextArgument::Declaration(statement) => {
                    collect_declaration_statement_names(statement, names);
                }
                CommandContextArgument::Expression(expression) => {
                    collect_expression_names(expression, names);
                }
                CommandContextArgument::Text(_) => {}
            }
        }
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
        ExpressionKind::IsType { subject, .. } => validate_when_expression_subject(
            subject,
            parameters,
            covered_parameters,
            path,
            locator,
            event_log,
        ),
        ExpressionKind::SpecStatement(statement) => validate_when_expression_subject(
            &statement.subject,
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

fn validate_when_expression_subject(
    subject: &Expression,
    parameters: &WhenParameters,
    covered_parameters: &mut HashSet<String>,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    event_log: &mut EventLog,
) {
    let subject = direct_variadic_slice(subject)
        .map(|slice| slice.name.clone())
        .or_else(|| match &subject.kind {
            ExpressionKind::SubsetCall(SubsetCall::One { target, .. })
            | ExpressionKind::SubsetCall(SubsetCall::Two { target, .. }) => Some(target.clone()),
            ExpressionKind::IndexedCall(call) => Some(call.target.clone()),
            _ => None,
        })
        .unwrap_or_else(|| key_for_expression(subject));
    validate_when_subject(
        &subject,
        parameters,
        covered_parameters,
        path,
        locator,
        event_log,
    );
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

    if parameters.described.as_deref() == Some(subject) {
        emit_error(
            event_log,
            path,
            locator.locate_symbol(subject),
            format!(
                "`when:` requirement for `{subject}` is not allowed because `{subject}` is what this definition describes; state its type on the definition's target instead"
            ),
        );
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

/// Statement-position clauses (`if:`, `then:`, `iff:`, `where:`, `such_that:`, and
/// the logical combinators) may only contain statements, not specifications. A
/// specification (`x is \real`, `A \:subset:/ B`) introduces symbols and is only
/// valid in a binding position (`exists:`, `given:`, `forAll:`, `let:`); here
/// the predicate form (`is?`, `\:...?:/`) must be used instead. (Binding
/// arguments never reach this path — they are checked directly, not as clauses.)
fn reject_specification_clause(
    clause: &Clause,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    event_log: &mut EventLog,
) {
    const IS_MESSAGE: &str = "An `is` specification introduces a symbol and is only allowed in `exists:`, `given:`, `forAll:`, or `let:`; use the statement form `is?` here";
    const INFIX_MESSAGE: &str = "An infix specification (`\\:...:/`) introduces a symbol and is only allowed in `exists:`, `given:`, `forAll:`, or `let:`; use the predicate form (`\\:...?:/`) here";

    let (message, subject) = match clause {
        // The logical combinators are position-transparent: their operands inherit
        // the enclosing statement position, so recurse into them.
        Clause::Not(group) => {
            reject_specification_clause(&group.not.argument, path, locator, event_log);
            return;
        }
        Clause::AllOf(group) => {
            reject_specification_clauses(&group.all_of.arguments, path, locator, event_log);
            return;
        }
        Clause::AnyOf(group) => {
            reject_specification_clauses(&group.any_of.arguments, path, locator, event_log);
            return;
        }
        Clause::OneOf(group) => {
            reject_specification_clauses(&group.one_of.arguments, path, locator, event_log);
            return;
        }
        Clause::Equivalently(group) => {
            reject_specification_clauses(&group.equivalently.arguments, path, locator, event_log);
            return;
        }
        Clause::Declaration(statement) => {
            let message = match &statement.relation {
                Some(DeclarationRelation::Is(_)) => IS_MESSAGE,
                Some(DeclarationRelation::InfixSpec { spec, .. }) if !spec.predicate => {
                    INFIX_MESSAGE
                }
                _ => return,
            };
            (message, primary_subject_key(&statement.subject))
        }
        Clause::Expression(expression) => match &expression.kind {
            ExpressionKind::IsType { subject, .. } => (IS_MESSAGE, key_for_expression(subject)),
            ExpressionKind::InfixSpecStatement { left, spec, .. } if !spec.predicate => {
                (INFIX_MESSAGE, key_for_expression(left))
            }
            _ => return,
        },
        // Structured clauses (if/iff/forAll/let/exists/given/piecewise) carry their own
        // binding and statement sub-sections, checked separately — stop here.
        _ => return,
    };

    emit_error(
        event_log,
        path,
        locator.locate_symbol(&subject),
        message.to_owned(),
    );
}

/// Applies [`reject_specification_clause`] to each clause in a statement-position
/// clause list.
fn reject_specification_clauses(
    clauses: &[Clause],
    path: &Path,
    locator: &mut SourceLocator<'_>,
    event_log: &mut EventLog,
) {
    for clause in clauses {
        reject_specification_clause(clause, path, locator, event_log);
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
        Clause::Expression(Expression {
            kind: ExpressionKind::BuiltinCommand(command),
            ..
        }) => {
            assume_builtin_command_expression(command, context, path, locator, registry, event_log);
        }
        Clause::AllOf(group) => {
            for clause in &group.all_of.arguments {
                assume_clause(clause, context, path, locator, registry, event_log);
            }
        }
        Clause::Equivalently(group) => {
            for clause in &group.equivalently.arguments {
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
        Clause::Equivalently(group) => {
            for clause in &group.equivalently.arguments {
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
            reject_specification_clauses(&group.then.arguments, path, locator, event_log);
            for clause in &group.then.arguments {
                check_clause(clause, &child, path, locator, registry, event_log);
            }
        }
        Clause::Let(group) => {
            let mut child = context.clone();
            for item in &group.let_.arguments {
                assume_binding_or_spec(item, &mut child, path, locator, registry, event_log);
            }
            if let Some(where_) = &group.where_ {
                for clause in &where_.arguments {
                    assume_clause(clause, &mut child, path, locator, registry, event_log);
                }
            }
            reject_specification_clauses(&group.then.arguments, path, locator, event_log);
            for clause in &group.then.arguments {
                check_clause(clause, &child, path, locator, registry, event_log);
            }
        }
        Clause::If(group) => {
            let mut child = context.clone();
            reject_specification_clauses(&group.if_.arguments, path, locator, event_log);
            for clause in &group.if_.arguments {
                assume_clause(clause, &mut child, path, locator, registry, event_log);
            }
            reject_specification_clauses(&group.then.arguments, path, locator, event_log);
            for clause in &group.then.arguments {
                check_clause(clause, &child, path, locator, registry, event_log);
            }
        }
        Clause::Iff(group) => {
            let mut child = context.clone();
            reject_specification_clauses(&group.iff.arguments, path, locator, event_log);
            for clause in &group.iff.arguments {
                assume_clause(clause, &mut child, path, locator, registry, event_log);
            }
            reject_specification_clauses(&group.then.arguments, path, locator, event_log);
            for clause in &group.then.arguments {
                check_clause(clause, &child, path, locator, registry, event_log);
            }
        }
        Clause::Piecewise(group) => {
            let mut child = context.clone();
            reject_specification_clauses(&group.if_.arguments, path, locator, event_log);
            for clause in &group.if_.arguments {
                assume_clause(clause, &mut child, path, locator, registry, event_log);
            }
            reject_specification_clauses(&group.then.arguments, path, locator, event_log);
            for clause in &group.then.arguments {
                check_clause(clause, &child, path, locator, registry, event_log);
            }
            if let Some(else_) = &group.else_ {
                reject_specification_clauses(&else_.arguments, path, locator, event_log);
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
            reject_specification_clauses(&group.then.arguments, path, locator, event_log);
            for clause in &group.then.arguments {
                check_clause(clause, &child, path, locator, registry, event_log);
            }
        }
        Clause::Have(group) => {
            check_have_group(group, context, path, locator, registry, event_log);
        }
        Clause::Declaration(statement) => {
            check_declaration_statement(statement, context, path, locator, registry, event_log);
        }
        Clause::Expression(expression) => {
            check_expression(expression, context, path, locator, registry, event_log)
        }
    }
}

fn check_builtin_command_expression(
    command: &BuiltinCommandExpression,
    context: &TypeContext,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    registry: &SignatureRegistry,
    event_log: &mut EventLog,
) {
    match format_chain(&command.chain).as_str() {
        "map" => check_variadic_map_builtin(command, context, path, locator, registry, event_log),
        "leftReduce" | "rightReduce" => {
            check_variadic_reduce_builtin(command, context, path, locator, registry, event_log)
        }
        "not" => {
            let clauses = parse_builtin_clause_arguments(
                command,
                &builtin_head_arguments(command),
                path,
                locator,
                event_log,
            );
            if clauses.len() != 1 {
                emit_builtin_command_error(
                    command,
                    path,
                    locator,
                    event_log,
                    "`\\\\not{...}` expects exactly one clause argument",
                );
            }
            for clause in &clauses {
                check_clause(clause, context, path, locator, registry, event_log);
            }
            check_builtin_tail_names(command, &[], path, locator, event_log);
        }
        "and" | "allOf" | "or" | "anyOf" | "oneOf" => {
            check_builtin_clause_list(
                command,
                &builtin_head_arguments(command),
                context,
                path,
                locator,
                registry,
                event_log,
            );
            check_builtin_tail_names(command, &[], path, locator, event_log);
        }
        "exists" | "existsUnique" => {
            let mut child = context.clone();
            assume_builtin_binding_arguments(
                command,
                &builtin_head_arguments(command),
                &mut child,
                path,
                locator,
                registry,
                event_log,
            );
            for clause in &parse_builtin_clause_arguments(
                command,
                &builtin_tail_arguments(command, "suchThat"),
                path,
                locator,
                event_log,
            ) {
                assume_clause(clause, &mut child, path, locator, registry, event_log);
            }
            check_builtin_tail_names(command, &["suchThat"], path, locator, event_log);
        }
        "forAll" | "forall" => {
            let mut child = context.clone();
            assume_builtin_binding_arguments(
                command,
                &builtin_head_arguments(command),
                &mut child,
                path,
                locator,
                registry,
                event_log,
            );
            for clause in &parse_builtin_clause_arguments(
                command,
                &builtin_tail_arguments(command, "where"),
                path,
                locator,
                event_log,
            ) {
                assume_clause(clause, &mut child, path, locator, registry, event_log);
            }
            let then_clauses = parse_builtin_clause_arguments(
                command,
                &builtin_tail_arguments(command, "then"),
                path,
                locator,
                event_log,
            );
            if then_clauses.is_empty() {
                emit_builtin_command_error(
                    command,
                    path,
                    locator,
                    event_log,
                    "`\\\\forAll{...}` requires a `:then{...}` tail",
                );
            }
            for clause in &then_clauses {
                check_clause(clause, &child, path, locator, registry, event_log);
            }
            check_builtin_tail_names(command, &["where", "then"], path, locator, event_log);
        }
        "if" => {
            let mut child = context.clone();
            for clause in &parse_builtin_clause_arguments(
                command,
                &builtin_head_arguments(command),
                path,
                locator,
                event_log,
            ) {
                assume_clause(clause, &mut child, path, locator, registry, event_log);
            }
            let then_clauses = parse_builtin_clause_arguments(
                command,
                &builtin_tail_arguments(command, "then"),
                path,
                locator,
                event_log,
            );
            for clause in &then_clauses {
                check_clause(clause, &child, path, locator, registry, event_log);
            }
            check_builtin_tail_names(command, &["then"], path, locator, event_log);
        }
        "have" => {
            let mut child = context.clone();
            for clause in &parse_builtin_clause_arguments(
                command,
                &builtin_tail_arguments(command, "iff"),
                path,
                locator,
                event_log,
            ) {
                assume_clause(clause, &mut child, path, locator, registry, event_log);
            }
            let have_clauses = parse_builtin_clause_arguments(
                command,
                &builtin_head_arguments(command),
                path,
                locator,
                event_log,
            );
            if builtin_tail_arguments(command, "iff").is_empty() {
                emit_builtin_command_error(
                    command,
                    path,
                    locator,
                    event_log,
                    "`\\\\have{...}` requires an `:iff{...}` tail",
                );
            }
            for clause in &have_clauses {
                check_clause(clause, &child, path, locator, registry, event_log);
            }
            check_builtin_tail_names(command, &["iff"], path, locator, event_log);
        }
        "given" => {
            let mut child = context.clone();
            assume_builtin_binding_arguments(
                command,
                &builtin_head_arguments(command),
                &mut child,
                path,
                locator,
                registry,
                event_log,
            );
            for clause in &parse_builtin_clause_arguments(
                command,
                &builtin_tail_arguments(command, "where"),
                path,
                locator,
                event_log,
            ) {
                assume_clause(clause, &mut child, path, locator, registry, event_log);
            }
            let then_clauses = parse_builtin_clause_arguments(
                command,
                &builtin_tail_arguments(command, "then"),
                path,
                locator,
                event_log,
            );
            if then_clauses.is_empty() {
                emit_builtin_command_error(
                    command,
                    path,
                    locator,
                    event_log,
                    "`\\\\given{...}` requires a `:then{...}` tail",
                );
            }
            for clause in &then_clauses {
                check_clause(clause, &child, path, locator, registry, event_log);
            }
            check_builtin_tail_names(command, &["where", "then"], path, locator, event_log);
        }
        "piecewise" => {
            let mut child = context.clone();
            for clause in &parse_builtin_clause_arguments(
                command,
                &builtin_tail_arguments(command, "if"),
                path,
                locator,
                event_log,
            ) {
                assume_clause(clause, &mut child, path, locator, registry, event_log);
            }
            let then_clauses = parse_builtin_clause_arguments(
                command,
                &builtin_tail_arguments(command, "then"),
                path,
                locator,
                event_log,
            );
            if then_clauses.is_empty() {
                emit_builtin_command_error(
                    command,
                    path,
                    locator,
                    event_log,
                    "`\\\\piecewise{...}` requires a `:then{...}` tail",
                );
            }
            for clause in &then_clauses {
                check_clause(clause, &child, path, locator, registry, event_log);
            }
            for clause in &parse_builtin_clause_arguments(
                command,
                &builtin_tail_arguments(command, "else"),
                path,
                locator,
                event_log,
            ) {
                check_clause(clause, context, path, locator, registry, event_log);
            }
            check_builtin_tail_names(command, &["if", "then", "else"], path, locator, event_log);
        }
        other => emit_builtin_command_error(
            command,
            path,
            locator,
            event_log,
            format!("Unknown builtin clause command `\\\\{other}`"),
        ),
    }
}

fn check_variadic_map_builtin(
    command: &BuiltinCommandExpression,
    context: &TypeContext,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    registry: &SignatureRegistry,
    event_log: &mut EventLog,
) {
    let mut slices = Vec::new();
    for text in builtin_head_arguments(command)
        .into_iter()
        .filter_map(|argument| match argument {
            BuiltinCommandArgument::Text(text) => Some(text.as_str()),
            _ => None,
        })
    {
        for part in text
            .split(',')
            .map(str::trim)
            .filter(|part| !part.is_empty())
        {
            match crate::frontend::formulation::parse_expression(part) {
                Ok(Expression {
                    kind: ExpressionKind::VariadicSlice(slice),
                    ..
                }) => slices.push(slice),
                _ => emit_builtin_command_error(
                    command,
                    path,
                    locator,
                    event_log,
                    format!("`\\map{{...}}` expects variadic slices, found `{part}`"),
                ),
            }
        }
    }
    if slices.is_empty() {
        emit_builtin_command_error(
            command,
            path,
            locator,
            event_log,
            "`\\map{...}` expects at least one variadic slice",
        );
    }
    if let Some(first) = slices.first()
        && !slices.iter().all(|slice| {
            slice.start == first.start
                && slice.index == first.index
                && slice.end == first.end
                && slice.dimensions == first.dimensions
        })
    {
        emit_builtin_command_error(
            command,
            path,
            locator,
            event_log,
            "all `\\map{...}` slices must use exactly the same start, index, and end",
        );
    }
    for slice in &slices {
        check_name(&slice.name, context, path, locator, event_log);
        if let Some(end) = &slice.end {
            check_name(end, context, path, locator, event_log);
        }
    }

    let to = builtin_tail_arguments(command, "to");
    if to.len() != 1 {
        emit_builtin_command_error(
            command,
            path,
            locator,
            event_log,
            "`\\map{...}` requires exactly one `:to{expression}` argument",
        );
    }
    let mut child = context.clone();
    for index in slices.iter().filter_map(|slice| slice.index.as_ref()) {
        child.declare_name(index.clone());
    }
    for argument in to {
        if let BuiltinCommandArgument::Text(text) = argument {
            match crate::frontend::formulation::parse_expression(text) {
                Ok(expression) => {
                    check_expression(&expression, &child, path, locator, registry, event_log)
                }
                Err(error) => emit_builtin_command_error(
                    command,
                    path,
                    locator,
                    event_log,
                    format!("invalid `\\map` result expression: {error}"),
                ),
            }
        }
    }
    check_builtin_tail_names(command, &["to"], path, locator, event_log);
}

fn check_variadic_reduce_builtin(
    command: &BuiltinCommandExpression,
    context: &TypeContext,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    _registry: &SignatureRegistry,
    event_log: &mut EventLog,
) {
    let operators = builtin_head_arguments(command);
    if operators.len() != 1 {
        emit_builtin_command_error(
            command,
            path,
            locator,
            event_log,
            "variadic reductions expect exactly one binary operator",
        );
    }
    if let Some(BuiltinCommandArgument::Text(operator)) = operators.first().copied() {
        let operator = operator
            .trim()
            .strip_prefix('`')
            .and_then(|operator| operator.strip_suffix('`'))
            .unwrap_or(operator.trim());
        check_name(operator, context, path, locator, event_log);
    }
    let on = builtin_tail_arguments(command, "on");
    if on.len() != 1 {
        emit_builtin_command_error(
            command,
            path,
            locator,
            event_log,
            "variadic reductions require exactly one `:on{slice}` argument",
        );
    }
    for argument in on {
        let BuiltinCommandArgument::Text(text) = argument else {
            continue;
        };
        match crate::frontend::formulation::parse_expression(text) {
            Ok(Expression {
                kind: ExpressionKind::VariadicSlice(slice),
                ..
            }) => {
                check_name(&slice.name, context, path, locator, event_log);
                if let Some(end) = &slice.end {
                    check_name(end, context, path, locator, event_log);
                }
            }
            _ => emit_builtin_command_error(
                command,
                path,
                locator,
                event_log,
                format!("variadic reductions expect a slice after `:on`, found `{text}`"),
            ),
        }
    }
    check_builtin_tail_names(command, &["on"], path, locator, event_log);
}

fn assume_builtin_command_expression(
    command: &BuiltinCommandExpression,
    context: &mut TypeContext,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    registry: &SignatureRegistry,
    event_log: &mut EventLog,
) {
    match format_chain(&command.chain).as_str() {
        "and" | "allOf" => {
            for clause in &parse_builtin_clause_arguments(
                command,
                &builtin_head_arguments(command),
                path,
                locator,
                event_log,
            ) {
                assume_clause(clause, context, path, locator, registry, event_log);
            }
            check_builtin_tail_names(command, &[], path, locator, event_log);
        }
        _ => check_builtin_command_expression(command, context, path, locator, registry, event_log),
    }
}

fn check_builtin_clause_list(
    command: &BuiltinCommandExpression,
    arguments: &[&BuiltinCommandArgument],
    context: &TypeContext,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    registry: &SignatureRegistry,
    event_log: &mut EventLog,
) {
    let clauses = parse_builtin_clause_arguments(command, arguments, path, locator, event_log);
    if clauses.is_empty() {
        emit_builtin_command_error(
            command,
            path,
            locator,
            event_log,
            format!(
                "`\\\\{}{}` expects at least one clause argument",
                format_chain(&command.chain),
                "{...}"
            ),
        );
    }
    for clause in &clauses {
        check_clause(clause, context, path, locator, registry, event_log);
    }
}

fn assume_builtin_binding_arguments(
    command: &BuiltinCommandExpression,
    arguments: &[&BuiltinCommandArgument],
    context: &mut TypeContext,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    registry: &SignatureRegistry,
    event_log: &mut EventLog,
) {
    if arguments.is_empty() {
        emit_builtin_command_error(
            command,
            path,
            locator,
            event_log,
            format!(
                "`\\\\{}{}` expects at least one binding argument",
                format_chain(&command.chain),
                "{...}"
            ),
        );
    }
    for argument in arguments {
        assume_builtin_binding_argument(
            command, argument, context, path, locator, registry, event_log,
        );
    }
}

fn assume_builtin_binding_argument(
    command: &BuiltinCommandExpression,
    argument: &BuiltinCommandArgument,
    context: &mut TypeContext,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    registry: &SignatureRegistry,
    event_log: &mut EventLog,
) {
    match argument {
        BuiltinCommandArgument::Text(argument) => {
            match crate::frontend::formulation::parse_refined_declaration_statement(argument) {
                Ok(statement) => {
                    assume_declaration_statement(
                        &statement, context, path, locator, registry, event_log,
                    );
                }
                Err(declaration_error) => {
                    match crate::frontend::formulation::parse_expression(argument) {
                        Ok(expression) => {
                            assume_fact_expression(
                                &expression,
                                context,
                                path,
                                locator,
                                registry,
                                event_log,
                            );
                        }
                        Err(expression_error) => emit_builtin_command_error(
                            command,
                            path,
                            locator,
                            event_log,
                            format!(
                                "Invalid builtin binding `{argument}`: {declaration_error}; {expression_error}"
                            ),
                        ),
                    }
                }
            }
        }
        BuiltinCommandArgument::Declaration(statement) => {
            assume_declaration_statement(statement, context, path, locator, registry, event_log);
        }
        BuiltinCommandArgument::Expression(expression) => {
            assume_fact_expression(expression, context, path, locator, registry, event_log);
        }
    }
}

fn parse_builtin_clause_arguments(
    command: &BuiltinCommandExpression,
    arguments: &[&BuiltinCommandArgument],
    path: &Path,
    locator: &mut SourceLocator<'_>,
    event_log: &mut EventLog,
) -> Vec<Clause> {
    arguments
        .iter()
        .filter_map(|argument| match argument {
            BuiltinCommandArgument::Text(argument) => {
                if let Ok(statement) =
                    crate::frontend::formulation::parse_ordinary_declaration_statement(argument)
                {
                    return Some(Clause::Declaration(statement));
                }

                match crate::frontend::formulation::parse_expression(argument) {
                    Ok(expression) => Some(Clause::Expression(expression)),
                    Err(error) => {
                        emit_builtin_command_error(
                            command,
                            path,
                            locator,
                            event_log,
                            format!("Invalid builtin clause argument `{argument}`: {error}"),
                        );
                        None
                    }
                }
            }
            BuiltinCommandArgument::Declaration(statement) => {
                Some(Clause::Declaration(statement.clone()))
            }
            BuiltinCommandArgument::Expression(expression) => {
                Some(Clause::Expression(expression.clone()))
            }
        })
        .collect()
}

fn builtin_head_arguments(command: &BuiltinCommandExpression) -> Vec<&BuiltinCommandArgument> {
    builtin_args_arguments(&command.head_args)
}

fn builtin_tail_arguments<'a>(
    command: &'a BuiltinCommandExpression,
    name: &str,
) -> Vec<&'a BuiltinCommandArgument> {
    command
        .tail
        .iter()
        .filter(|tail| format_chain(&tail.chain) == name)
        .flat_map(|tail| builtin_args_arguments(&tail.args))
        .collect()
}

fn builtin_args_arguments(args: &[BuiltinCommandArgs]) -> Vec<&BuiltinCommandArgument> {
    args.iter().flat_map(|args| args.arguments.iter()).collect()
}

fn check_builtin_tail_names(
    command: &BuiltinCommandExpression,
    allowed: &[&str],
    path: &Path,
    locator: &mut SourceLocator<'_>,
    event_log: &mut EventLog,
) {
    for tail in &command.tail {
        let name = format_chain(&tail.chain);
        if !allowed.iter().any(|allowed| allowed == &name) {
            emit_builtin_command_error(
                command,
                path,
                locator,
                event_log,
                format!(
                    "Unexpected tail `:{name}` for builtin command `\\\\{}`",
                    format_chain(&command.chain)
                ),
            );
        }
    }
}

fn emit_builtin_command_error(
    command: &BuiltinCommandExpression,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    event_log: &mut EventLog,
    message: impl Into<String>,
) {
    emit_error(
        event_log,
        path,
        locator.locate_symbol(&format_chain(&command.chain)),
        message,
    );
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
        Clause::Equivalently(group) => {
            for clause in &group.equivalently.arguments {
                collect_clause_assumptions(clause, context);
            }
        }
        _ => {}
    }
}

/// Checks a declaration's `:=` value. A function-literal value whose declaration
/// has an `is <type>` relation is checked with that type available, so a bare
/// parameter's spec can be inferred from it.
fn check_declaration_definition(
    statement: &DeclarationStatement,
    context: &TypeContext,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    registry: &SignatureRegistry,
    event_log: &mut EventLog,
) {
    let Some(definition) = &statement.definition else {
        return;
    };
    if matches!(definition.kind, ExpressionKind::Mapping { .. }) {
        let expected = match &statement.relation {
            Some(DeclarationRelation::Is(ty)) => Some(ty),
            _ => None,
        };
        let expected_subject = primary_subject_key(&statement.subject);
        check_mapping_expression(
            definition,
            expected,
            Some(&expected_subject),
            context,
            path,
            locator,
            registry,
            event_log,
        );
    } else {
        check_expression(definition, context, path, locator, registry, event_log);
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
    record_declaration_line_types(statement, context, registry);
    if establish_labeled_declaration_statement(
        statement, context, path, locator, registry, event_log,
    ) {
        return;
    }
    check_is_subject(&statement.subject, context, path, locator, event_log);
    if let Some(expansion) = &statement.expansion {
        check_is_subject(expansion, context, path, locator, event_log);
    }
    check_declaration_definition(statement, context, path, locator, registry, event_log);
    if let Some(relation) = &statement.relation {
        check_declaration_relation(relation, context, path, locator, registry, event_log);
    }
    check_declaration_spec_facts_supported(statement, context, path, locator, registry, event_log);
}

/// Assumes one side of a `Relation:` (`between:`/`and:`).
///
/// A declared subject introduces its symbol and facts into scope; a `#topic`
/// reference names an external documentation topic and introduces nothing.
fn assume_relation_subject(
    subject: &RelationSubject,
    context: &mut TypeContext,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    registry: &SignatureRegistry,
    event_log: &mut EventLog,
) {
    if let RelationSubject::Declaration(statement) = subject {
        assume_declaration_statement(statement, context, path, locator, registry, event_log);
    }
}

/// Introduces the inferred parameters (`X?`) that appear in a declaration
/// statement into scope, taking each one's type from the command definition its
/// argument position belongs to. Runs in the assume phase, before the relation is
/// checked, so the injected facts satisfy the subsequent requirement check and are
/// visible to every later use of the symbol.
fn declare_inferred_parameters(
    statement: &DeclarationStatement,
    context: &mut TypeContext,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    registry: &SignatureRegistry,
    event_log: &mut EventLog,
) {
    if let Some(relation) = &statement.relation {
        match relation {
            DeclarationRelation::Is(ty) => {
                declare_inferred_parameters_in_type_expression(
                    ty, context, path, locator, registry, event_log,
                );
            }
            DeclarationRelation::Spec { target, .. } => {
                declare_inferred_parameters_in_expression(
                    target, context, path, locator, registry, event_log,
                );
            }
            DeclarationRelation::InfixSpec { target, .. } => {
                declare_inferred_parameters_in_expression(
                    target, context, path, locator, registry, event_log,
                );
            }
        }
    }
    if let Some(definition) = &statement.definition {
        declare_inferred_parameters_in_expression(
            definition, context, path, locator, registry, event_log,
        );
    }
}

fn declare_inferred_parameters_in_type_expression(
    ty: &TypeExpression,
    context: &mut TypeContext,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    registry: &SignatureRegistry,
    event_log: &mut EventLog,
) {
    match ty {
        TypeExpression::Command(command) => {
            let active = active_command_expression(command, context);
            let shape = shape_for_command_expression(&active);
            let arguments = command_expression_arguments(&active);
            let signature = resolved_command_signature(&shape, registry);
            inject_inferred_parameters(
                &signature, &arguments, context, path, locator, registry, event_log,
            );
            for argument in arguments {
                declare_inferred_parameters_in_expression(
                    argument, context, path, locator, registry, event_log,
                );
            }
        }
        TypeExpression::RefinedCommand(command) => {
            let active = active_refined_command_expression(command, context);
            let shape = shape_for_refined_command_expression(&active);
            let arguments = refined_command_expression_arguments(&active);
            inject_inferred_parameters(
                &shape.signature,
                &arguments,
                context,
                path,
                locator,
                registry,
                event_log,
            );
            for argument in arguments {
                declare_inferred_parameters_in_expression(
                    argument, context, path, locator, registry, event_log,
                );
            }
        }
        TypeExpression::Tuple(tuple) => {
            for spec in &tuple.elements {
                declare_inferred_parameters_in_function_type_spec(
                    spec, context, path, locator, registry, event_log,
                );
            }
        }
        TypeExpression::Set(set) => match &set.element {
            SetTypeElement::Spec(spec) => declare_inferred_parameters_in_function_type_spec(
                spec, context, path, locator, registry, event_log,
            ),
            SetTypeElement::Tuple(tuple) => {
                for spec in &tuple.elements {
                    declare_inferred_parameters_in_function_type_spec(
                        spec, context, path, locator, registry, event_log,
                    );
                }
            }
        },
        TypeExpression::Function(function) => {
            for spec in function
                .inputs
                .iter()
                .chain(std::iter::once(&function.output))
            {
                if let FunctionTypeSpecKind::Is(ty) = &spec.kind {
                    declare_inferred_parameters_in_type_expression(
                        ty, context, path, locator, registry, event_log,
                    );
                }
            }
        }
        TypeExpression::Builtin { .. } | TypeExpression::Parameter { .. } => {}
    }
}

fn declare_inferred_parameters_in_function_type_spec(
    spec: &FunctionTypeSpec,
    context: &mut TypeContext,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    registry: &SignatureRegistry,
    event_log: &mut EventLog,
) {
    match &spec.kind {
        FunctionTypeSpecKind::Is(ty) => declare_inferred_parameters_in_type_expression(
            ty, context, path, locator, registry, event_log,
        ),
        FunctionTypeSpecKind::Spec { target, .. } => declare_inferred_parameters_in_expression(
            target, context, path, locator, registry, event_log,
        ),
    }
}

/// Recursively finds command expressions carrying inferred parameters inside a
/// general expression (a `Spec:`/`InfixSpec:` target or a definition body).
fn declare_inferred_parameters_in_expression(
    expression: &Expression,
    context: &mut TypeContext,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    registry: &SignatureRegistry,
    event_log: &mut EventLog,
) {
    match &expression.kind {
        ExpressionKind::Command(command) => {
            let active = active_command_expression(command, context);
            let shape = shape_for_command_expression(&active);
            let arguments = command_expression_arguments(&active);
            let signature = resolved_command_signature(&shape, registry);
            inject_inferred_parameters(
                &signature, &arguments, context, path, locator, registry, event_log,
            );
            for argument in arguments {
                declare_inferred_parameters_in_expression(
                    argument, context, path, locator, registry, event_log,
                );
            }
        }
        ExpressionKind::Grouped { expression, .. } | ExpressionKind::Labeled { expression, .. } => {
            declare_inferred_parameters_in_expression(
                expression, context, path, locator, registry, event_log,
            );
        }
        ExpressionKind::Build {
            value: expression,
            ty,
            ..
        } => {
            declare_inferred_parameters_in_expression(
                expression, context, path, locator, registry, event_log,
            );
            declare_inferred_parameters_in_type_expression(
                ty, context, path, locator, registry, event_log,
            );
        }
        ExpressionKind::IsType { subject, ty } => {
            declare_inferred_parameters_in_expression(
                subject, context, path, locator, registry, event_log,
            );
            declare_inferred_parameters_in_type_expression(
                ty, context, path, locator, registry, event_log,
            );
        }
        ExpressionKind::MemberOf {
            subject,
            collection,
        } => {
            declare_inferred_parameters_in_expression(
                subject, context, path, locator, registry, event_log,
            );
            declare_inferred_parameters_in_expression(
                collection, context, path, locator, registry, event_log,
            );
        }
        _ => {}
    }
}

/// Given a command's resolved `signature` and its ordered argument expressions,
/// declares every argument written as an inferred parameter (`X?`) and injects
/// the definition requirement(s) that type it. Argument i corresponds to
/// `info.parameters[i]` (the same positional correspondence the requirement check
/// relies on).
fn inject_inferred_parameters(
    signature: &str,
    arguments: &[&Expression],
    context: &mut TypeContext,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    registry: &SignatureRegistry,
    event_log: &mut EventLog,
) {
    if !arguments
        .iter()
        .any(|argument| matches!(argument.kind, ExpressionKind::InferredName(_)))
    {
        return;
    }
    let Some(info) = registry.type_infos.get(signature) else {
        return;
    };

    let actuals = arguments
        .iter()
        .map(|argument| effective_key_for_expression(argument, context, registry))
        .collect::<Vec<_>>();
    let substitutions = info
        .parameters
        .iter()
        .zip(&actuals)
        .map(|(name, actual)| (name.clone(), context.normalize_key(actual)))
        .collect::<HashMap<_, _>>();

    // Declare each inferred name first, so a requirement on one inferred parameter
    // that references another resolves once all are in scope.
    let mut introduced_parameters = Vec::new();
    for (index, argument) in arguments.iter().enumerate() {
        let ExpressionKind::InferredName(name) = &argument.kind else {
            continue;
        };
        let Some(parameter) = info.parameters.get(index) else {
            continue;
        };
        if context.has_name(name) {
            emit_error(
                event_log,
                path,
                locator.locate_symbol(name),
                format!("Inferred parameter `{name}` is already introduced"),
            );
            continue;
        }
        context.declare_name(name.clone());
        introduced_parameters.push(parameter.clone());
    }

    // Inject the requirement(s) that type each introduced inferred parameter.
    for requirement in &info.requirements {
        if info
            .hidden_parameters
            .iter()
            .any(|name| fact_mentions_name(requirement, name))
        {
            continue;
        }
        if introduced_parameters
            .iter()
            .any(|parameter| parameter == fact_subject(requirement))
        {
            context.add_fact(substitute_fact(requirement, &substitutions));
        }
    }
}

fn assume_declaration_statement(
    statement: &DeclarationStatement,
    context: &mut TypeContext,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    registry: &SignatureRegistry,
    event_log: &mut EventLog,
) {
    // An assumed declaration — a `when: x is \real` line — is never checked, so
    // like `assume_fact_expression` it needs its own type-info hook.
    record_declaration_line_types(statement, context, registry);
    let established = establish_labeled_declaration_statement(
        statement, context, path, locator, registry, event_log,
    );
    declare_inferred_parameters(statement, context, path, locator, registry, event_log);
    declare_is_subject(&statement.subject, context);
    if let Some(expansion) = &statement.expansion {
        declare_is_subject(expansion, context);
    }
    if !established {
        if let Some(relation) = &statement.relation {
            check_declaration_relation(relation, context, path, locator, registry, event_log);
        }
        check_declaration_spec_facts_supported(
            statement, context, path, locator, registry, event_log,
        );
        check_declaration_definition(statement, context, path, locator, registry, event_log);
    }
    register_declaration_collection_literal(statement, context);
    if let Some((left, right)) = declaration_substitution(statement) {
        context.add_substitution(left, right);
    }
    for fact in facts_from_declaration_statement_in_context(statement, context) {
        context.add_fact(fact);
    }
    assume_destructured_declaration_components(statement, context, registry);
}

/// Binds the components of a destructuring declaration `M ::= (X, *) is \T` (a
/// `Declares:` target or a `given:`/`using:` binding): the component types come
/// from `\T`'s stored component types, substituted onto the local names. This is
/// what lets a theorem `given: M ::= (X, *) is \magma` use `X` and `*`. Only
/// `::=` introduces symbols; `:=` requires its right-hand side to already be in
/// scope, so it is deliberately not handled here.
/// The signature whose component types a destructuring binding draws on: the
/// `is` type when the binding states one, otherwise the command its `:=` value
/// comes from.
fn destructured_value_signature(statement: &DeclarationStatement) -> Option<String> {
    if let Some(DeclarationRelation::Is(ty)) = &statement.relation {
        return key_for_type_expression(ty).map(|(_, signature)| signature);
    }
    let definition = statement.definition.as_ref()?;
    command_signature_from_key(&key_for_expression(definition))
}

/// Brings the components of a destructuring binding into scope with their types.
///
/// The type is reached either through an `is` relation — `Y ::= (X, *) is \magma`
/// — or, when the value comes from a `Declares:`/`Realizes:` command, through the
/// `:=` that produces it: `Nb ::= (N, 0, succ) := \von.neumann.naturals`. The
/// second spelling is how a declaration's value is written, so a realization's
/// components are reached the same way an abstract declaration's are.
fn assume_destructured_declaration_components(
    statement: &DeclarationStatement,
    context: &mut TypeContext,
    registry: &SignatureRegistry,
) {
    let component_names = statement
        .expansion
        .as_ref()
        .and_then(is_subject_first_form)
        .and_then(form_or_declaration_tuple_form)
        .map(tuple_form_component_names)
        .unwrap_or_default();
    if component_names.is_empty() {
        return;
    }
    let Some(signature) = destructured_value_signature(statement) else {
        return;
    };
    let Some(info) = registry.type_infos.get(&signature) else {
        return;
    };
    if info.component_types.is_empty() {
        return;
    }

    let subject = primary_subject_key(&statement.subject);
    for local in &component_names {
        context.declare_name(local.clone());
    }
    for fact in instantiate_component_type_facts(info, &subject, &component_names, context) {
        context.add_fact(fact);
    }
    context.add_destructured_components(subject, component_names);
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
    declare_inferred_parameters(statement, context, path, locator, registry, event_log);
    if let Some(relation) = &statement.relation {
        check_declaration_relation(relation, context, path, locator, registry, event_log);
    }
    check_declaration_definition(statement, context, path, locator, registry, event_log);
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
    if let Some(literal) = statement
        .definition
        .as_ref()
        .and_then(cast_expression_set_literal)
    {
        for subject in declaration_subject_keys(statement) {
            context.add_collection_literal(subject, literal.clone());
        }
    }
}

fn register_expression_collection_literal(expression: &Expression, context: &mut TypeContext) {
    match &expression.kind {
        ExpressionKind::Set(set) => {
            context.add_collection_literal(key_for_expression(expression), set.clone());
        }
        ExpressionKind::Build {
            value: expression, ..
        } => {
            if let ExpressionKind::Set(set) = &expression.kind {
                context.add_collection_literal(key_for_expression(expression), set.clone());
            }
        }
        _ => {}
    }
}

fn cast_expression_set_literal(expression: &Expression) -> Option<&SetExpression> {
    let ExpressionKind::Build {
        value: expression, ..
    } = &expression.kind
    else {
        return None;
    };
    match &expression.kind {
        ExpressionKind::Set(set) => Some(set),
        _ => None,
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
                let resolutions = implicit_refined_infix_spec_resolution(fact, context, registry);
                if !resolutions.is_empty() {
                    for resolution in resolutions {
                        check_command_requirements(
                            &resolution.base_signature,
                            &resolution.base_actuals,
                            None,
                            None,
                            context,
                            path,
                            position,
                            registry,
                            event_log,
                        );
                        check_command_requirements(
                            &resolution.refined_type_signature,
                            &resolution.refined_type_actuals,
                            None,
                            None,
                            context,
                            path,
                            position,
                            registry,
                            event_log,
                        );
                    }
                    return;
                }
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

            if !matches!(
                definition.kind,
                DefinitionKind::Defines | DefinitionKind::Refines
            ) {
                emit_error(
                    event_log,
                    path,
                    position,
                    format!(
                        "Could not validate spec fact `{}`: spec-infix signature `{}` must be defined by Defines or Refines",
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
                signature, &actuals, None, None, context, path, position, registry, event_log,
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

/// Checks a spec literal (`? is \real`, `? "in" \reals`). A spec literal is a
/// value of type `\\specification`; the only extra rule is that a `"op"` target
/// which is a command must name a value (`Declares:`), not a type (`Defines:`).
fn check_spec_literal(
    literal: &SpecLiteral,
    context: &TypeContext,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    registry: &SignatureRegistry,
    event_log: &mut EventLog,
) {
    match &literal.form {
        SpecLiteralForm::Is(ty) => {
            check_type_expression(ty, context, path, locator, registry, event_log);
        }
        SpecLiteralForm::Spec { target, .. } => {
            check_expression(target, context, path, locator, registry, event_log);
            if let ExpressionKind::Command(_) = &target.kind {
                let target_key = context.normalize_key(&key_for_expression(target));
                if let Some(signature) = command_signature_from_key(&target_key) {
                    if signature_has_kind(&signature, DefinitionKind::Defines, registry) {
                        emit_error(
                            event_log,
                            path,
                            locator.locate_symbol(&signature),
                            format!(
                                "the target of a spec operator must be a value, not the type `{signature}`"
                            ),
                        );
                    }
                }
            }
        }
    }
}

/// Checks `x satisfies spec`: the right-hand side must be provably a
/// `\\specification` (a concrete spec literal, or a variable known to be one).
fn check_satisfies(
    subject: &Expression,
    spec: &Expression,
    context: &TypeContext,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    registry: &SignatureRegistry,
    event_log: &mut EventLog,
) {
    check_expression(subject, context, path, locator, registry, event_log);
    check_expression(spec, context, path, locator, registry, event_log);

    let spec_key = context.normalize_key(&key_for_expression(spec));
    let required = TypeFact::Is {
        subject: spec_key,
        ty: BUILTIN_SPECIFICATION_SIGNATURE.to_owned(),
        signature: BUILTIN_SPECIFICATION_SIGNATURE.to_owned(),
    };
    if !prove_fact(&required, context, registry) {
        emit_error(
            event_log,
            path,
            locator.locate_symbol("satisfies"),
            "`satisfies` requires a specification on the right-hand side".to_owned(),
        );
    }
}

/// The bare parameter name of a function literal's left-hand side (placeholders are
/// already stripped of their trailing `_` at lex time).
fn mapping_parameter_name(subject: &Expression) -> Option<String> {
    match &subject.kind {
        ExpressionKind::Name(name) | ExpressionKind::InferredName(name) => Some(name.clone()),
        _ => None,
    }
}

/// The input specification of a function type, as the fact-spec used to type a
/// function literal's parameter. Handles the structural `->` type and a command
/// type (e.g. `\real.function`) that defines a function.
fn function_input_spec_from_type(
    ty: &TypeExpression,
    context: &TypeContext,
    registry: &SignatureRegistry,
) -> Option<FunctionTypeFactSpec> {
    function_input_specs_from_type(ty, context, registry)?
        .into_iter()
        .next()
}

fn function_input_specs_from_type(
    ty: &TypeExpression,
    context: &TypeContext,
    registry: &SignatureRegistry,
) -> Option<Vec<FunctionTypeFactSpec>> {
    match ty {
        TypeExpression::Function(function_type) => function_type_inputs_as_facts(function_type),
        TypeExpression::Command(command) => {
            let active = active_command_expression(command, context);
            let shape = shape_for_command_expression(&active);
            let signature = resolved_command_signature(&shape, registry);
            let info = registry.type_infos.get(&signature)?;
            info.outputs.iter().find_map(|fact| match fact {
                TypeFact::FunctionType { inputs, .. } => Some(inputs.clone()),
                _ => None,
            })
        }
        _ => None,
    }
}

fn mapping_pattern_names(expression: &Expression) -> Option<Vec<String>> {
    let mut names = Vec::new();
    collect_mapping_pattern_names(expression, &mut names).then_some(names)
}

fn collect_mapping_pattern_names(expression: &Expression, names: &mut Vec<String>) -> bool {
    match &expression.kind {
        ExpressionKind::Name(name) => {
            names.push(name.clone());
            true
        }
        ExpressionKind::Tuple(elements) => elements.iter().all(|element| match element {
            TupleExpressionElement::Expression(expression) => {
                collect_mapping_pattern_names(expression, names)
            }
            TupleExpressionElement::Operator(_) => false,
        }),
        ExpressionKind::Grouped { expression, .. }
        | ExpressionKind::Labeled { expression, .. }
        | ExpressionKind::IsType {
            subject: expression,
            ..
        }
        | ExpressionKind::IsBuiltinPredicate {
            subject: expression,
            ..
        }
        | ExpressionKind::IsPredicate {
            subject: expression,
            ..
        } => collect_mapping_pattern_names(expression, names),
        ExpressionKind::SpecStatement(statement) | ExpressionKind::SpecPredicate(statement) => {
            collect_mapping_pattern_names(&statement.subject, names)
        }
        ExpressionKind::SpecStatementExpr { subject, .. } => {
            collect_mapping_pattern_names(subject, names)
        }
        _ => false,
    }
}

fn mapping_pattern_elements(expression: &Expression) -> Option<Vec<&Expression>> {
    match &expression.kind {
        ExpressionKind::Tuple(elements) => elements
            .iter()
            .map(|element| match element {
                TupleExpressionElement::Expression(expression) => Some(expression),
                TupleExpressionElement::Operator(_) => None,
            })
            .collect(),
        ExpressionKind::Grouped { expression, .. }
        | ExpressionKind::Labeled { expression, .. }
        | ExpressionKind::IsType {
            subject: expression,
            ..
        } => mapping_pattern_elements(expression),
        _ => None,
    }
}

fn mapping_function_type_for_subject(
    subject: &str,
    context: &TypeContext,
    registry: &SignatureRegistry,
) -> Option<(Vec<FunctionTypeFactSpec>, bool)> {
    let subject = context.normalize_key(subject);
    let mut seen = HashSet::new();
    context.facts.iter().find_map(|fact| {
        mapping_function_type_from_fact(fact, &subject, context, registry, &mut seen)
    })
}

fn mapping_function_type_from_fact(
    fact: &TypeFact,
    subject: &str,
    context: &TypeContext,
    registry: &SignatureRegistry,
    seen: &mut HashSet<TypeFact>,
) -> Option<(Vec<FunctionTypeFactSpec>, bool)> {
    let fact = context.normalize_fact(fact);
    if !seen.insert(fact.clone()) || context.normalize_key(fact_subject(&fact)) != subject {
        return None;
    }
    if let TypeFact::FunctionType {
        inputs,
        variadic_tuple_input,
        ..
    } = &fact
    {
        return Some((inputs.clone(), *variadic_tuple_input));
    }

    for output in type_instance_output_facts(&fact, context, registry) {
        if let Some(function_type) =
            mapping_function_type_from_fact(&output, subject, context, registry, seen)
        {
            return Some(function_type);
        }
    }
    for extended in reduce_extension_fact(&fact, context, registry) {
        if let Some(function_type) =
            mapping_function_type_from_fact(&extended, subject, context, registry, seen)
        {
            return Some(function_type);
        }
    }
    None
}

fn type_instance_output_facts(
    fact: &TypeFact,
    context: &TypeContext,
    registry: &SignatureRegistry,
) -> Vec<TypeFact> {
    let (subject, signature, actuals) = match fact {
        TypeFact::Is {
            subject,
            ty,
            signature,
        } => {
            let Some(actuals) = actuals_for_type_key(signature, ty) else {
                return Vec::new();
            };
            (subject, signature, actuals)
        }
        TypeFact::RefinedIs {
            subject,
            ty,
            signature,
            ..
        } => {
            let Some(actuals) = actuals_for_refined_type_key(signature, ty) else {
                return Vec::new();
            };
            (subject, signature, actuals)
        }
        _ => return Vec::new(),
    };
    defined_output_facts_for_signature(signature, &actuals, subject, context, registry)
}

/// Checks a function literal `(x_ is \real) => x_ + 1`: the parameter is bound to
/// its spec's type, then the body is checked in that extended scope. `expected` is
/// the declared type (from an enclosing `is`) used to infer a bare parameter's
/// spec; without it a bare parameter is an error.
fn check_mapping_expression(
    expression: &Expression,
    expected: Option<&TypeExpression>,
    expected_subject: Option<&str>,
    context: &TypeContext,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    registry: &SignatureRegistry,
    event_log: &mut EventLog,
) {
    let ExpressionKind::Mapping { lhs, rhs } = &expression.kind else {
        return;
    };

    // Peel `(...)`/labels off the parameter binder.
    let mut binder = lhs.as_ref();
    while let ExpressionKind::Grouped { expression, .. }
    | ExpressionKind::Labeled { expression, .. } = &binder.kind
    {
        binder = expression;
    }

    if let Some(parameters) = mapping_pattern_names(binder)
        && parameters.len() > 1
        && let Some((inputs, variadic_tuple_input)) = expected
            .and_then(|ty| {
                function_input_specs_from_type(ty, context, registry).map(|inputs| (inputs, false))
            })
            .or_else(|| {
                expected_subject.and_then(|subject| {
                    mapping_function_type_for_subject(subject, context, registry)
                })
            })
    {
        let mut child = context.clone();
        for parameter in parameters {
            child.declare_name(parameter);
        }
        if variadic_tuple_input && inputs.len() == 1 {
            child.add_fact(instantiate_function_type_spec(
                &inputs[0],
                &key_for_expression(binder),
            ));
        } else if let Some(elements) = mapping_pattern_elements(binder)
            && elements.len() == inputs.len()
        {
            for (input, element) in inputs.iter().zip(elements) {
                child.add_fact(instantiate_function_type_spec(
                    input,
                    &key_for_expression(element),
                ));
            }
        } else {
            emit_error(
                event_log,
                path,
                locator.locate_symbol("=>"),
                "function literal pattern does not match the function input shape".to_owned(),
            );
        }
        let child = context_with_spec_reductions(&child, registry);
        check_expression(rhs, &child, path, locator, registry, event_log);
        return;
    }

    // In the compact binder `x_, y_, z_ is T`, the trailing type applies to
    // every parameter. This is the mapping literal spelling used by commands
    // whose overload selects one or more of those parameters.
    if let Some(parameters) = mapping_pattern_names(binder)
        && parameters.len() > 1
        && let Some(ty) = mapping_pattern_shared_type(binder)
        && let Some((ty_key, signature)) = key_for_type_expression(ty)
    {
        check_type_expression(ty, context, path, locator, registry, event_log);
        let mut child = context.clone();
        for parameter in parameters {
            child.declare_name(parameter.clone());
            child.add_fact(TypeFact::Is {
                subject: parameter,
                ty: ty_key.clone(),
                signature: signature.clone(),
            });
        }
        let child = context_with_spec_reductions(&child, registry);
        check_expression(rhs, &child, path, locator, registry, event_log);
        return;
    }

    let (parameter, explicit_spec): (Option<String>, Option<FunctionTypeFactSpec>) = match &binder
        .kind
    {
        ExpressionKind::IsType { subject, ty } => {
            check_type_expression(ty, context, path, locator, registry, event_log);
            let spec =
                key_for_type_expression(ty).map(|(ty_key, signature)| FunctionTypeFactSpec::Is {
                    ty: ty_key,
                    signature,
                });
            (mapping_parameter_name(subject), spec)
        }
        ExpressionKind::SpecStatement(statement) => (
            mapping_parameter_name(&statement.subject),
            Some(FunctionTypeFactSpec::Spec {
                operator: statement.operator.clone(),
                target: statement.name.clone(),
            }),
        ),
        ExpressionKind::SpecStatementExpr {
            subject,
            operator,
            target,
        } => {
            check_expression(target, context, path, locator, registry, event_log);
            (
                mapping_parameter_name(subject),
                Some(FunctionTypeFactSpec::Spec {
                    operator: operator.clone(),
                    target: context.normalize_key(&key_for_expression(target)),
                }),
            )
        }
        ExpressionKind::Name(name) | ExpressionKind::InferredName(name) => {
            (Some(name.clone()), None)
        }
        _ => (None, None),
    };

    let Some(parameter) = parameter else {
        emit_error(
            event_log,
            path,
            locator.locate_symbol("=>"),
            "a function literal parameter must be a name with a spec, e.g. `(x_ is ...)`"
                .to_owned(),
        );
        check_expression(rhs, context, path, locator, registry, event_log);
        return;
    };

    // Resolve the parameter's spec: explicit, else inferred from the expected type.
    let spec = explicit_spec
        .or_else(|| expected.and_then(|ty| function_input_spec_from_type(ty, context, registry)));

    let mut child = context.clone();
    child.declare_name(parameter.clone());
    match spec {
        Some(spec) => child.add_fact(instantiate_function_type_spec(&spec, &parameter)),
        None => emit_error(
            event_log,
            path,
            locator.locate_symbol(&parameter),
            format!(
                "function literal parameter `{parameter}` needs a spec (e.g. `(x_ is ...)`) unless the type is known from an `is`"
            ),
        ),
    }
    check_expression(rhs, &child, path, locator, registry, event_log);
}

fn mapping_pattern_shared_type(expression: &Expression) -> Option<&TypeExpression> {
    match &expression.kind {
        ExpressionKind::IsType { subject, ty }
            if mapping_pattern_names(subject).is_some_and(|names| names.len() > 1) =>
        {
            Some(ty)
        }
        ExpressionKind::Tuple(_) => {
            mapping_pattern_elements(expression)?
                .last()
                .and_then(|element| match &element.kind {
                    ExpressionKind::IsType { ty, .. } => Some(ty),
                    _ => None,
                })
        }
        ExpressionKind::Grouped { expression, .. } | ExpressionKind::Labeled { expression, .. } => {
            mapping_pattern_shared_type(expression)
        }
        _ => None,
    }
}

fn check_expression(
    expression: &Expression,
    context: &TypeContext,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    registry: &SignatureRegistry,
    event_log: &mut EventLog,
) {
    record_line_types(expression, context, registry);
    match &expression.kind {
        // An inferred parameter is declared into scope by the assume phase
        // (`declare_inferred_parameters`); by the time the check pass revisits it,
        // the name is in scope, so it is checked exactly like a plain `Name`. In a
        // genuinely check-only position it is not in scope and surfaces the usual
        // "Unrecognized symbol" error.
        ExpressionKind::Name(name) | ExpressionKind::InferredName(name) => {
            check_name(name, context, path, locator, event_log);
        }
        ExpressionKind::VariadicSlice(slice) => {
            check_variadic_slice_names(slice, context, path, locator, event_log);
            emit_error(
                event_log,
                path,
                locator.locate_symbol(&slice.name),
                "a variadic slice is only valid with `:=`, `=`, `!=`, `is`, `is?`, a quoted specification operator, `map`, or a reduce builtin".to_owned(),
            );
        }
        ExpressionKind::VariadicAssignment { target, value } => {
            check_variadic_slice_names(target, context, path, locator, event_log);
            check_variadic_operand(value, context, path, locator, registry, event_log);
            check_matching_variadic_slices(target, value, path, locator, event_log);
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
                if !check_provided_callable_owner_function(
                    name, arguments, context, path, locator, registry, event_log,
                ) {
                    check_disambiguated_function_call(
                        name, arguments, context, path, locator, registry, event_log,
                    );
                }
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
            if let SetTargetKind::Expression { expression, .. } = &set.target.kind {
                check_expression(expression, &child, path, locator, registry, event_log);
            }
            if let Some(predicate) = &set.predicate {
                check_set_predicate(predicate, &mut child, path, locator, registry, event_log);
            }
        }
        ExpressionKind::Grouped { expression, .. } => {
            check_expression(expression, context, path, locator, registry, event_log);
        }
        ExpressionKind::Labeled { expression, label } => {
            if !establish_labeled_expression(
                label, expression, context, path, locator, registry, event_log,
            ) {
                check_expression(expression, context, path, locator, registry, event_log);
            }
        }
        ExpressionKind::Prefix {
            operator,
            expression,
        } => {
            if let Some(call) = named_prefix_operator_desugaring(operator, expression) {
                check_expression(&call, context, path, locator, registry, event_log);
            } else {
                check_expression(expression, context, path, locator, registry, event_log);
                check_disambiguated_prefix(
                    operator, expression, context, path, locator, registry, event_log,
                );
                check_provided_prefix_operator(
                    operator, expression, context, path, locator, registry, event_log,
                );
            }
        }
        ExpressionKind::Postfix {
            expression,
            operator,
        } => {
            let call = postfix_operator_desugaring(expression, operator);
            check_expression(&call, context, path, locator, registry, event_log);
        }
        ExpressionKind::SubsetCall(subset) => {
            check_subset_call(subset, context, path, locator, registry, event_log);
        }
        ExpressionKind::IndexedCall(call) => {
            check_name(&call.target, context, path, locator, event_log);
            for index in &call.indices {
                check_expression(index, context, path, locator, registry, event_log);
                check_variadic_index_expression(
                    &call.target,
                    index,
                    context,
                    path,
                    locator,
                    registry,
                    event_log,
                );
            }
        }
        ExpressionKind::Command(command) => {
            check_command_expression(command, context, path, locator, registry, event_log);
            let active_command = active_command_expression(command, context);
            check_command_argument_expressions(
                &active_command,
                context,
                path,
                locator,
                registry,
                event_log,
            );
        }
        ExpressionKind::BuiltinCommand(command) => {
            check_builtin_command_expression(command, context, path, locator, registry, event_log);
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
            if let Some(slice) =
                direct_variadic_slice(left).or_else(|| direct_variadic_slice(right))
            {
                if !variadic_binary_operator_supported(operator) {
                    emit_error(
                        event_log,
                        path,
                        locator.locate_symbol(&slice.name),
                        "variadic slices only support the binary operators `=` and `!=`".to_owned(),
                    );
                }
                check_variadic_operand(left, context, path, locator, registry, event_log);
                check_variadic_operand(right, context, path, locator, registry, event_log);
                if let Some(left_slice) = direct_variadic_slice(left) {
                    check_matching_variadic_slices(left_slice, right, path, locator, event_log);
                }
                return;
            }
            if let Some(call) =
                binary_operator_application_desugaring(left, operator, right, context)
            {
                check_expression(&call, context, path, locator, registry, event_log);
            } else {
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
        }
        ExpressionKind::SpecStatement(statement) => {
            check_variadic_operand(
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
            check_variadic_operand(
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
        ExpressionKind::SpecStatementExpr {
            subject,
            operator,
            target,
        } => {
            check_variadic_operand(subject, context, path, locator, registry, event_log);
            check_expression(target, context, path, locator, registry, event_log);
            if let Some(fact) = fact_from_expression(expression) {
                check_spec_fact_supported(
                    &fact,
                    context,
                    path,
                    locator.locate_symbol(operator),
                    registry,
                    event_log,
                );
            }
        }
        ExpressionKind::SpecLiteral(literal) => {
            check_spec_literal(literal, context, path, locator, registry, event_log);
        }
        ExpressionKind::Satisfies { subject, spec } => {
            check_satisfies(subject, spec, context, path, locator, registry, event_log);
        }
        ExpressionKind::Mapping { .. } => {
            // No expected type here, so a bare-parameter mapping (no spec) is an error.
            check_mapping_expression(
                expression, None, None, context, path, locator, registry, event_log,
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
            check_variadic_operand(subject, context, path, locator, registry, event_log);
            check_command_predicate(command, context, path, locator, registry, event_log);
        }
        ExpressionKind::IsBuiltinPredicate { subject, ty } => {
            check_variadic_operand(subject, context, path, locator, registry, event_log);
            if direct_variadic_slice(subject).is_none() {
                check_builtin_type_predicate(
                    subject, ty, false, context, path, locator, registry, event_log,
                );
            } else {
                check_type_expression(ty, context, path, locator, registry, event_log);
            }
        }
        ExpressionKind::IsNotBuiltinPredicate { subject, ty } => {
            check_variadic_operand(subject, context, path, locator, registry, event_log);
            if direct_variadic_slice(subject).is_none() {
                check_builtin_type_predicate(
                    subject, ty, true, context, path, locator, registry, event_log,
                );
            } else {
                check_type_expression(ty, context, path, locator, registry, event_log);
            }
        }
        ExpressionKind::IsRefinedPredicate { subject, command }
        | ExpressionKind::IsNotRefinedPredicate { subject, command } => {
            check_variadic_operand(subject, context, path, locator, registry, event_log);
            check_refined_command_expression(command, context, path, locator, registry, event_log);
            let active_command = active_refined_command_expression(command, context);
            for expression in refined_command_expression_arguments(&active_command) {
                check_expression(expression, context, path, locator, registry, event_log);
            }
        }
        ExpressionKind::IsType { subject, ty } => {
            check_variadic_operand(subject, context, path, locator, registry, event_log);
            check_type_expression(ty, context, path, locator, registry, event_log);
            if direct_variadic_slice(subject).is_none() {
                check_function_call_result(
                    subject, ty, context, path, locator, registry, event_log,
                );
            }
        }
        ExpressionKind::Build { ty, value, hard } => {
            check_expression(value, context, path, locator, registry, event_log);
            check_type_expression(ty, context, path, locator, registry, event_log);
            check_build_expression(
                value, ty, *hard, context, path, locator, registry, event_log,
            );
        }
    }
}

fn direct_variadic_slice(expression: &Expression) -> Option<&VariadicSlice> {
    match &expression.kind {
        ExpressionKind::VariadicSlice(slice) => Some(slice),
        ExpressionKind::Grouped { expression, .. } | ExpressionKind::Labeled { expression, .. } => {
            direct_variadic_slice(expression)
        }
        _ => None,
    }
}

fn variadic_slice_referenced_names(slice: &VariadicSlice) -> Vec<String> {
    let mut names = slice
        .index
        .iter()
        .chain(slice.end.iter())
        .cloned()
        .collect::<Vec<_>>();
    if let Some(dimensions) = &slice.dimensions {
        for axis in [&dimensions.rows, &dimensions.columns] {
            match axis {
                VariadicSliceAxis::All => {}
                VariadicSliceAxis::Index(index) => names.push(index.clone()),
                VariadicSliceAxis::Range { start, index, end } => {
                    names.push(start.clone());
                    names.extend(index.iter().cloned());
                    names.push(end.clone());
                }
            }
        }
    }
    names
}

fn check_variadic_slice_names(
    slice: &VariadicSlice,
    context: &TypeContext,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    event_log: &mut EventLog,
) {
    check_name(&slice.name, context, path, locator, event_log);
    if let Some(dimensions) = &slice.dimensions {
        for axis in [&dimensions.rows, &dimensions.columns] {
            match axis {
                VariadicSliceAxis::All => {}
                VariadicSliceAxis::Index(index) => {
                    check_name(index, context, path, locator, event_log)
                }
                VariadicSliceAxis::Range { start, end, .. } => {
                    check_name(start, context, path, locator, event_log);
                    check_name(end, context, path, locator, event_log);
                }
            }
        }
        return;
    }
    if let Some(end) = &slice.end {
        check_name(end, context, path, locator, event_log);
    }
}

fn check_variadic_operand(
    expression: &Expression,
    context: &TypeContext,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    registry: &SignatureRegistry,
    event_log: &mut EventLog,
) {
    if let Some(slice) = direct_variadic_slice(expression) {
        check_variadic_slice_names(slice, context, path, locator, event_log);
    } else {
        check_expression(expression, context, path, locator, registry, event_log);
    }
}

fn variadic_binary_operator_supported(operator: &BinaryOperator) -> bool {
    matches!(operator, BinaryOperator::Equality(_))
        || matches!(operator, BinaryOperator::Special(operator) if operator.text == "!=")
}

fn check_matching_variadic_slices(
    left: &VariadicSlice,
    right: &Expression,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    event_log: &mut EventLog,
) {
    let Some(right) = direct_variadic_slice(right) else {
        return;
    };
    if left.start == right.start
        && left.index == right.index
        && left.end == right.end
        && left.dimensions == right.dimensions
    {
        return;
    }
    emit_error(
        event_log,
        path,
        locator.locate_symbol(&right.name),
        "paired variadic slices must use exactly the same start, index binder, and end".to_owned(),
    );
}

/// Checks `\ty@value` / `\ty@!value` — the only cast form.
fn check_build_expression(
    value: &Expression,
    ty: &TypeExpression,
    hard: bool,
    context: &TypeContext,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    registry: &SignatureRegistry,
    event_log: &mut EventLog,
) {
    if matches!(value.kind, ExpressionKind::Set(_)) {
        return;
    }
    let Some(required) = fact_from_type_assertion_in_context(value, ty, context) else {
        return;
    };
    // The soft build `\ty@value` follows subclassing and `\\coercion`; the hard build
    // `\ty@!value` additionally follows `\\encoding`.
    let succeeds = if hard {
        prove_fact_allowing_abstraction(&required, context, registry)
    } else {
        prove_fact(&required, context, registry)
    };
    if succeeds {
        return;
    }

    emit_error(
        event_log,
        path,
        cast_expression_position(value, context, locator),
        format!(
            "Could not build `{}`",
            format_build_expression(ty, value, hard)
        ),
    );
}

fn context_with_cast_expression_facts<'a>(
    expressions: impl IntoIterator<Item = &'a Expression>,
    context: &TypeContext,
) -> TypeContext {
    let mut child = context.clone();
    for expression in expressions {
        add_cast_expression_facts(expression, &mut child);
    }
    child
}

/// Clones `context` and materializes the `is`-facts implied by its specification
/// facts (`y "in" M` -> `y is \magma.element:of{M}`) so that owner-type matching
/// via `has_type_signature` sees a value's reduced type. Reductions run through
/// the existing `reduce_spec_or_member_fact`, which is cycle-guarded.
fn context_with_spec_reductions(
    context: &TypeContext,
    registry: &SignatureRegistry,
) -> TypeContext {
    let mut child = context.clone();
    let mut known = child.facts.iter().cloned().collect::<HashSet<_>>();
    let mut index = 0;
    while index < child.facts.len() {
        let fact = child.facts[index].clone();
        index += 1;
        if !matches!(fact, TypeFact::Spec { .. } | TypeFact::MemberOf { .. }) {
            continue;
        }
        let mut seen = HashSet::new();
        for reduced in reduce_spec_or_member_fact(&fact, &child, registry, &mut seen) {
            let reduced = child.normalize_fact(&reduced);
            if known.insert(reduced.clone()) {
                child.add_fact(reduced);
            }
        }
    }
    child
}

fn add_cast_expression_facts(expression: &Expression, context: &mut TypeContext) {
    match &expression.kind {
        ExpressionKind::Name(_)
        | ExpressionKind::InferredName(_)
        | ExpressionKind::VariadicSlice(_)
        | ExpressionKind::SubsetCall(_) => {}
        ExpressionKind::IndexedCall(call) => {
            for index in &call.indices {
                add_cast_expression_facts(index, context);
            }
        }
        ExpressionKind::VariadicAssignment { value, .. } => {
            add_cast_expression_facts(value, context);
        }
        ExpressionKind::FunctionCall { arguments, .. } => {
            for argument in arguments {
                add_cast_expression_facts(argument, context);
            }
        }
        ExpressionKind::FunctionNamedCall { elements, .. } => {
            for element in elements {
                add_cast_expression_facts(&element.expression, context);
            }
        }
        ExpressionKind::MemberCall {
            owner, arguments, ..
        } => {
            add_cast_expression_facts(owner, context);
            for argument in arguments {
                add_cast_expression_facts(argument, context);
            }
        }
        ExpressionKind::MemberAccess { owner, .. } => add_cast_expression_facts(owner, context),
        ExpressionKind::Tuple(elements) => {
            for element in elements {
                if let TupleExpressionElement::Expression(expression) = element {
                    add_cast_expression_facts(expression, context);
                }
            }
        }
        ExpressionKind::Set(set) => {
            for spec in &set.specs {
                add_cast_expression_facts(spec, context);
            }
            if let Some(predicate) = &set.predicate {
                add_cast_set_predicate_facts(predicate, context);
            }
        }
        ExpressionKind::Grouped { expression, .. }
        | ExpressionKind::Labeled { expression, .. }
        | ExpressionKind::Prefix { expression, .. }
        | ExpressionKind::Postfix { expression, .. } => {
            add_cast_expression_facts(expression, context)
        }
        ExpressionKind::Command(command) => {
            add_command_context_cast_facts(command.context.as_ref(), context);
            for expression in command_expression_arguments(command) {
                add_cast_expression_facts(expression, context);
            }
        }
        ExpressionKind::BuiltinCommand(command) => add_builtin_command_cast_facts(command, context),
        ExpressionKind::InfixCommand {
            left,
            command,
            right,
        } => {
            add_cast_expression_facts(left, context);
            for expression in infix_command_arguments(command) {
                add_cast_expression_facts(expression, context);
            }
            add_cast_expression_facts(right, context);
        }
        ExpressionKind::InfixSpecStatement { left, spec, right } => {
            add_cast_expression_facts(left, context);
            for expression in infix_spec_arguments(spec) {
                add_cast_expression_facts(expression, context);
            }
            add_cast_expression_facts(right, context);
        }
        ExpressionKind::Binary { left, right, .. } => {
            add_cast_expression_facts(left, context);
            add_cast_expression_facts(right, context);
        }
        ExpressionKind::SpecStatement(statement) | ExpressionKind::SpecPredicate(statement) => {
            add_cast_expression_facts(&statement.subject, context);
        }
        ExpressionKind::SpecStatementExpr {
            subject, target, ..
        } => {
            add_cast_expression_facts(subject, context);
            add_cast_expression_facts(target, context);
        }
        ExpressionKind::SpecLiteral(literal) => match &literal.form {
            SpecLiteralForm::Is(ty) => add_cast_type_expression_facts(ty, context),
            SpecLiteralForm::Spec { target, .. } => add_cast_expression_facts(target, context),
        },
        ExpressionKind::Satisfies { subject, spec } => {
            add_cast_expression_facts(subject, context);
            add_cast_expression_facts(spec, context);
        }
        ExpressionKind::Mapping { lhs, rhs } => {
            add_cast_expression_facts(lhs, context);
            add_cast_expression_facts(rhs, context);
        }
        ExpressionKind::IsPredicate { subject, command }
        | ExpressionKind::IsNotPredicate { subject, command } => {
            add_cast_expression_facts(subject, context);
            add_command_context_cast_facts(command.context.as_ref(), context);
            for expression in command_expression_arguments(command) {
                add_cast_expression_facts(expression, context);
            }
        }
        ExpressionKind::IsBuiltinPredicate { subject, ty }
        | ExpressionKind::IsNotBuiltinPredicate { subject, ty }
        | ExpressionKind::IsType { subject, ty } => {
            add_cast_expression_facts(subject, context);
            add_cast_type_expression_facts(ty, context);
        }
        ExpressionKind::IsRefinedPredicate { subject, command }
        | ExpressionKind::IsNotRefinedPredicate { subject, command } => {
            add_cast_expression_facts(subject, context);
            for expression in refined_command_expression_arguments(command) {
                add_cast_expression_facts(expression, context);
            }
        }
        ExpressionKind::Build { ty, value, .. } => {
            add_cast_expression_facts(value, context);
            add_cast_type_expression_facts(ty, context);
            register_expression_collection_literal(value, context);
            if let Some(fact) = fact_from_type_assertion_in_context(value, ty, context) {
                let normalized = context.normalize_fact(&fact);
                context.add_fact(normalized);
            }
        }
        ExpressionKind::MemberOf {
            subject,
            collection,
        } => {
            add_cast_expression_facts(subject, context);
            add_cast_expression_facts(collection, context);
        }
    }
}

fn add_cast_set_predicate_facts(predicate: &SetPredicate, context: &mut TypeContext) {
    match predicate {
        SetPredicate::Expression(expression) => add_cast_expression_facts(expression, context),
        SetPredicate::Definition { value, .. } => add_cast_expression_facts(value, context),
    }
}

fn add_cast_type_expression_facts(ty: &TypeExpression, context: &mut TypeContext) {
    match ty {
        TypeExpression::Builtin { .. } | TypeExpression::Parameter { .. } => {}
        TypeExpression::Command(command) => {
            add_command_context_cast_facts(command.context.as_ref(), context);
            for expression in command_expression_arguments(command) {
                add_cast_expression_facts(expression, context);
            }
        }
        TypeExpression::RefinedCommand(command) => {
            for expression in refined_command_expression_arguments(command) {
                add_cast_expression_facts(expression, context);
            }
        }
        TypeExpression::Tuple(tuple) => {
            for spec in &tuple.elements {
                add_cast_function_type_spec_facts(spec, context);
            }
        }
        TypeExpression::Set(set) => match &set.element {
            SetTypeElement::Spec(spec) => add_cast_function_type_spec_facts(spec, context),
            SetTypeElement::Tuple(tuple) => {
                for spec in &tuple.elements {
                    add_cast_function_type_spec_facts(spec, context);
                }
            }
        },
        TypeExpression::Function(function_type) => {
            for spec in function_type
                .inputs
                .iter()
                .chain(std::iter::once(&function_type.output))
            {
                add_cast_function_type_spec_facts(spec, context);
            }
        }
    }
}

fn add_cast_function_type_spec_facts(spec: &FunctionTypeSpec, context: &mut TypeContext) {
    match &spec.kind {
        FunctionTypeSpecKind::Is(ty) => add_cast_type_expression_facts(ty, context),
        FunctionTypeSpecKind::Spec { target, .. } => add_cast_expression_facts(target, context),
    }
}

fn add_builtin_command_cast_facts(command: &BuiltinCommandExpression, context: &mut TypeContext) {
    for argument in builtin_head_arguments(command) {
        add_builtin_command_argument_cast_facts(argument, context);
    }
    for tail in &command.tail {
        for argument in builtin_args_arguments(&tail.args) {
            add_builtin_command_argument_cast_facts(argument, context);
        }
    }
}

fn add_builtin_command_argument_cast_facts(
    argument: &BuiltinCommandArgument,
    context: &mut TypeContext,
) {
    match argument {
        BuiltinCommandArgument::Text(_) => {}
        BuiltinCommandArgument::Declaration(statement) => {
            add_declaration_statement_cast_facts(statement, context);
        }
        BuiltinCommandArgument::Expression(expression) => {
            add_cast_expression_facts(expression, context)
        }
    }
}

fn add_command_context_cast_facts(
    command_context: Option<&CommandContext>,
    context: &mut TypeContext,
) {
    let Some(command_context) = command_context else {
        return;
    };
    for argument in &command_context.arguments {
        match argument {
            CommandContextArgument::Assignment { value, .. }
            | CommandContextArgument::Expression(value) => {
                add_cast_expression_facts(value, context);
            }
            CommandContextArgument::Declaration(statement) => {
                add_declaration_statement_cast_facts(statement, context);
            }
            CommandContextArgument::Text(_) => {}
        }
    }
}

fn add_declaration_statement_cast_facts(
    statement: &DeclarationStatement,
    context: &mut TypeContext,
) {
    if let Some(definition) = &statement.definition {
        add_cast_expression_facts(definition, context);
    }
    if let Some(relation) = &statement.relation {
        add_declaration_relation_cast_facts(relation, context);
    }
}

fn add_declaration_relation_cast_facts(relation: &DeclarationRelation, context: &mut TypeContext) {
    match relation {
        DeclarationRelation::Is(ty) => add_cast_type_expression_facts(ty, context),
        DeclarationRelation::Spec { target, .. } => add_cast_expression_facts(target, context),
        DeclarationRelation::InfixSpec { spec, target } => {
            for expression in infix_spec_arguments(spec) {
                add_cast_expression_facts(expression, context);
            }
            add_cast_expression_facts(target, context);
        }
    }
}

fn cast_expression_position(
    expression: &Expression,
    context: &TypeContext,
    locator: &mut SourceLocator<'_>,
) -> Option<SourcePosition> {
    match &expression.kind {
        ExpressionKind::Name(name) => locator.locate_symbol(name),
        ExpressionKind::Command(command) => {
            let active_command = active_command_expression(command, context);
            locator.locate_reference(&shape_for_command_expression(&active_command))
        }
        ExpressionKind::Grouped { expression, .. } | ExpressionKind::Labeled { expression, .. } => {
            cast_expression_position(expression, context, locator)
        }
        _ => None,
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
        // A bare name used as a type (`x is T`) is only valid when `T` is a known
        // type — i.e. a `T is \\type` fact is provable (e.g. a `when: T is \\type`
        // type parameter). Everything else (`\real`) parses as a Command.
        TypeExpression::Parameter { name, .. } => {
            let fact = TypeFact::Is {
                subject: name.clone(),
                ty: BUILTIN_TYPE_SIGNATURE.to_owned(),
                signature: BUILTIN_TYPE_SIGNATURE.to_owned(),
            };
            if !prove_fact(&fact, context, registry) {
                emit_error(
                    event_log,
                    path,
                    locator.locate_symbol(name),
                    format!("`{name}` is not a known type"),
                );
            }
        }
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
        TypeExpression::Tuple(tuple) => {
            for spec in &tuple.elements {
                check_function_type_spec(spec, context, path, locator, registry, event_log);
            }
        }
        TypeExpression::Set(set) => match &set.element {
            SetTypeElement::Spec(spec) => {
                check_function_type_spec(spec, context, path, locator, registry, event_log)
            }
            SetTypeElement::Tuple(tuple) => {
                for spec in &tuple.elements {
                    check_function_type_spec(spec, context, path, locator, registry, event_log);
                }
            }
        },
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

fn check_set_predicate(
    predicate: &SetPredicate,
    context: &mut TypeContext,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    registry: &SignatureRegistry,
    event_log: &mut EventLog,
) {
    match predicate {
        SetPredicate::Expression(expression) => {
            check_expression(expression, context, path, locator, registry, event_log);
        }
        SetPredicate::Definition { target, value, .. } => {
            declare_set_target(target, context);
            check_expression(value, context, path, locator, registry, event_log);
            context.add_substitution(key_for_set_target(target), key_for_expression(value));
        }
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
    if spec.subject != "_" && spec.subject != "?" {
        emit_error(
            event_log,
            path,
            locator.locate_symbol(&spec.subject),
            "Function type specs must use `_` or `?` as their subject",
        );
    }

    match &spec.kind {
        FunctionTypeSpecKind::Is(ty) => {
            check_type_expression(ty, context, path, locator, registry, event_log);
        }
        FunctionTypeSpecKind::Spec { target, .. } => {
            check_expression(target, context, path, locator, registry, event_log);
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
    let requirement_context = context_with_cast_expression_facts(arguments.iter(), context);
    let function_types = function_type_facts_for_subject(name, &requirement_context, registry);
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
            &requirement_context,
            registry,
        );
        let Some(argument_subjects) = argument_subjects else {
            continue;
        };
        matched_arity = true;

        for (input, argument_subject) in inputs.iter().zip(argument_subjects) {
            let required = instantiate_function_type_spec(input, &argument_subject);
            if !prove_fact(&required, &requirement_context, registry) {
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
    let requirement_context = context_with_cast_expression_facts(arguments.iter(), context);
    let actuals = arguments
        .iter()
        .map(|argument| effective_key_for_expression(argument, &requirement_context, registry))
        .collect::<Vec<_>>();
    let position = locator.locate_symbol(name);
    check_disambiguated_expression(
        &key,
        &actuals,
        &format!("function `{name}`"),
        position,
        &requirement_context,
        path,
        locator,
        registry,
        event_log,
    );
}

/// A plain named operator is syntactic sugar for application: `x |op| y` means
/// `op(x, y)` (and `f| x` / `x |f` mean `f(x)`). When the operator is a dotted
/// member path such as `M.*` or `x.y.z`, the application tracks down through the
/// value's fields as a member call on the value the path reaches; a bare name
/// becomes a plain function call.
fn desugar_named_operator_application(
    name: &str,
    span: Span,
    arguments: Vec<Expression>,
) -> Expression {
    match name.rsplit_once('.') {
        Some((owner_path, member)) => Expression::new(
            span,
            ExpressionKind::MemberCall {
                owner: Box::new(member_path_owner_expression(owner_path, span)),
                name: member.to_owned(),
                arguments,
            },
        ),
        None => Expression::new(
            span,
            ExpressionKind::FunctionCall {
                name: name.to_owned(),
                arguments,
            },
        ),
    }
}

/// Builds the owner expression for a dotted member path: `M` becomes a name and
/// `x.y` becomes `x` with `.y` accessed, so that the final segment can be called
/// on it.
fn member_path_owner_expression(path: &str, span: Span) -> Expression {
    let mut segments = path.split('.');
    let first = segments.next().unwrap_or_default();
    let mut owner = Expression::new(span, ExpressionKind::Name(first.to_owned()));
    for segment in segments {
        owner = Expression::new(
            span,
            ExpressionKind::MemberAccess {
                owner: Box::new(owner),
                name: segment.to_owned(),
            },
        );
    }
    owner
}

/// If `operator` is a plain named operator, returns the application it desugars
/// to (`x |op| y` == `op(x, y)`). Colon-qualified named operators and the
/// built-in symbolic operators are left to their own resolution paths.
fn binary_operator_application_desugaring(
    left: &Expression,
    operator: &BinaryOperator,
    right: &Expression,
    context: &TypeContext,
) -> Option<Expression> {
    let (symbol, kind, span, named) = match operator {
        BinaryOperator::Named(operator) => {
            (operator.name.clone(), operator.kind, operator.span, true)
        }
        BinaryOperator::Equality(operator)
        | BinaryOperator::Special(operator)
        | BinaryOperator::Add(operator)
        | BinaryOperator::Subtract(operator)
        | BinaryOperator::Multiply(operator)
        | BinaryOperator::Divide(operator)
        | BinaryOperator::Power(operator) => {
            (operator.text.clone(), operator.kind, operator.span, false)
        }
    };
    if kind != NamedOperatorKind::Plain {
        return None;
    }
    // A `|op|` named operator is always application sugar. A symbolic operator
    // (`*`, `+`, ...) is sugar for `symbol(x, y)` only when the symbol names a
    // bound value in scope (e.g. a magma's operation `*`); otherwise it keeps
    // its built-in arithmetic resolution.
    if !named && !context.has_name(&symbol) {
        return None;
    }
    Some(desugar_named_operator_application(
        &symbol,
        span,
        vec![left.clone(), right.clone()],
    ))
}

/// If `operator` is a named prefix operator, returns the application it desugars
/// to (`f| x` == `f(x)`). Arithmetic prefix operators such as `-x` keep their own
/// resolution path.
fn named_prefix_operator_desugaring(
    operator: &UnaryOperator,
    expression: &Expression,
) -> Option<Expression> {
    let UnaryOperator::Named(operator) = operator else {
        return None;
    };
    Some(desugar_named_operator_application(
        &operator.text,
        operator.span,
        vec![expression.clone()],
    ))
}

/// Every postfix expression operator is a named operator (`x |f`), so it is
/// always application sugar for `f(x)`.
fn postfix_operator_desugaring(expression: &Expression, operator: &Operator) -> Expression {
    desugar_named_operator_application(&operator.text, operator.span, vec![expression.clone()])
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

        // No `Disambiguates` entry: fall back to a provided-symbol capability for
        // this operator owned by the operands' common type (e.g. `y * y` where
        // `y` is a magma element and `\magma.element` enables `x_ * y_`).
        if check_provided_binary_operator_by_operand_type(
            left, operator, right, context, path, locator, registry, event_log,
        ) {
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
    let argument_expressions = [left, right];
    let requirement_context =
        context_with_cast_expression_facts(argument_expressions.iter().copied(), context);
    let requirement_context =
        context_with_expression_result_facts(left, &requirement_context, registry);
    let requirement_context =
        context_with_expression_result_facts(right, &requirement_context, registry);
    let actuals = argument_expressions
        .iter()
        .map(|expression| effective_key_for_expression(expression, &requirement_context, registry))
        .collect::<Vec<_>>();
    let position = locator.locate_symbol(&symbol);
    check_disambiguated_expression(
        &key,
        &actuals,
        &label,
        position,
        &requirement_context,
        path,
        locator,
        registry,
        event_log,
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
    let requirement_context =
        context_with_cast_expression_facts(std::iter::once(expression), context);
    let actuals = vec![effective_key_for_expression(
        expression,
        &requirement_context,
        registry,
    )];
    let position = locator.locate_symbol(&symbol);
    check_disambiguated_expression(
        &key,
        &actuals,
        &label,
        position,
        &requirement_context,
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
    let argument_expressions = [left, right];
    let requirement_context =
        context_with_cast_expression_facts(argument_expressions.iter().copied(), context);
    let requirement_context =
        context_with_expression_result_facts(left, &requirement_context, registry);
    let requirement_context =
        context_with_expression_result_facts(right, &requirement_context, registry);
    let actuals = argument_expressions
        .iter()
        .map(|expression| effective_key_for_expression(expression, &requirement_context, registry))
        .collect::<Vec<_>>();
    let Some(rule) =
        find_provided_symbol_rule(&key, kind, &actuals, &requirement_context, registry)
    else {
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
        &requirement_context,
        path,
        locator,
        registry,
        event_log,
    );
    true
}

/// Resolves a plain binary operator through a provided-symbol capability owned by
/// the operands' common type — e.g. `y * y` where both operands are magma
/// elements and `\magma.element` enables `x_ * y_`. Both operands must have the
/// owner type (there is no colon to designate a single owner). Returns `false`
/// without emitting when no such capability applies, so the caller can report the
/// operator as unresolved.
fn check_provided_binary_operator_by_operand_type(
    left: &Expression,
    operator: &BinaryOperator,
    right: &Expression,
    context: &TypeContext,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    registry: &SignatureRegistry,
    event_log: &mut EventLog,
) -> bool {
    let (symbol, _) = binary_operator_symbol_and_kind(operator);
    let key = DisambiguationKey::BinaryOperator(symbol);
    let argument_expressions = [left, right];
    let requirement_context =
        context_with_cast_expression_facts(argument_expressions.iter().copied(), context);
    let requirement_context =
        context_with_expression_result_facts(left, &requirement_context, registry);
    let requirement_context =
        context_with_expression_result_facts(right, &requirement_context, registry);
    let requirement_context = context_with_spec_reductions(&requirement_context, registry);
    let actuals = argument_expressions
        .iter()
        .map(|expression| effective_key_for_expression(expression, &requirement_context, registry))
        .collect::<Vec<_>>();
    let Some(rule) = find_provided_symbol_rule(
        &key,
        NamedOperatorKind::BothColon,
        &actuals,
        &requirement_context,
        registry,
    ) else {
        return false;
    };

    let owner_actual = provided_symbol_owner_actual(NamedOperatorKind::BothColon, &actuals);
    check_provided_symbol_target(
        rule,
        &actuals,
        owner_actual.as_deref(),
        &requirement_context,
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
    let requirement_context =
        context_with_cast_expression_facts(std::iter::once(expression), context);
    let actuals = vec![effective_key_for_expression(
        expression,
        &requirement_context,
        registry,
    )];
    let Some(rule) = find_provided_symbol_rule(
        &key,
        NamedOperatorKind::BothColon,
        &actuals,
        &requirement_context,
        registry,
    ) else {
        return;
    };

    let owner_actual = provided_symbol_owner_actual(NamedOperatorKind::LeftColon, &actuals);
    check_provided_symbol_target(
        rule,
        &actuals,
        owner_actual.as_deref(),
        &requirement_context,
        path,
        locator,
        registry,
        event_log,
    );
}

/// Top-level component names of a tuple key like `(X,*)` -> `["X", "*"]`, or
/// `None` when the key is not a parenthesized tuple.
fn tuple_key_components(key: &str) -> Option<Vec<String>> {
    let inner = key.strip_prefix('(')?.strip_suffix(')')?;
    if inner.trim().is_empty() {
        return Some(Vec::new());
    }
    let mut components = Vec::new();
    let mut depth = 0usize;
    let mut current = String::new();
    for ch in inner.chars() {
        match ch {
            '(' | '[' | '{' => {
                depth += 1;
                current.push(ch);
            }
            ')' | ']' | '}' => {
                depth = depth.saturating_sub(1);
                current.push(ch);
            }
            ',' if depth == 0 => {
                components.push(current.trim().to_owned());
                current.clear();
            }
            other => current.push(other),
        }
    }
    components.push(current.trim().to_owned());
    Some(components)
}

/// Resolves member access/call on a destructured tuple: when `owner` reduces to a
/// tuple key `(..., name, ...)` and `name` is a bound symbol, `owner.name` is the
/// component `name` itself and `owner.name(args)` is `name(args)`. This is what
/// lets `M.*` (and `x |M.*| y` == `M.*(x, y)`) reach the `*` component of a
/// destructured `M ::= (X, *)`.
/// The bare name of an owner expression, when it is a plain name (`M`).
fn owner_name_key(owner: &Expression) -> Option<String> {
    match &owner.kind {
        ExpressionKind::Name(name) => Some(name.clone()),
        _ => None,
    }
}

fn tuple_component_access_expression(
    owner: &Expression,
    name: &str,
    arguments: &[Expression],
    owner_actual: &str,
    context: &TypeContext,
) -> Option<Expression> {
    // `name` must be a component of the owner: either the owner reduces to a
    // tuple key `(..., name, ...)` (a tuple value), or the owner is a value
    // destructured as `M ::= (..., name, ...)` (recorded component names).
    let is_tuple_value_component = tuple_key_components(&context.normalize_key(owner_actual))
        .is_some_and(|components| components.iter().any(|component| component == name));
    let is_destructured_component = owner_name_key(owner)
        .and_then(|subject| context.destructured_components_of(&subject).cloned())
        .is_some_and(|components| components.iter().any(|component| component == name));
    if (!is_tuple_value_component && !is_destructured_component) || !context.has_name(name) {
        return None;
    }
    let kind = if arguments.is_empty() {
        ExpressionKind::Name(name.to_owned())
    } else {
        ExpressionKind::FunctionCall {
            name: name.to_owned(),
            arguments: arguments.to_vec(),
        }
    };
    Some(Expression::new(owner.span, kind))
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
    let mut argument_expressions = Vec::new();
    argument_expressions.push(owner);
    argument_expressions.extend(arguments.iter());
    let requirement_context =
        context_with_cast_expression_facts(argument_expressions.iter().copied(), context);
    let requirement_context =
        context_with_expression_result_facts(owner, &requirement_context, registry);
    let requirement_context = context_with_spec_reductions(&requirement_context, registry);
    let owner_actual = effective_key_for_expression(owner, &requirement_context, registry);
    let actuals = arguments
        .iter()
        .map(|argument| effective_key_for_expression(argument, &requirement_context, registry))
        .collect::<Vec<_>>();
    let Some(rule) = find_member_provided_symbol_rule(
        &key,
        &owner_actual,
        &actuals,
        &requirement_context,
        registry,
    ) else {
        if let Some(expression) = tuple_component_access_expression(
            owner,
            name,
            arguments,
            &owner_actual,
            &requirement_context,
        ) {
            check_expression(
                &expression,
                &requirement_context,
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
        &requirement_context,
        path,
        locator,
        registry,
        event_log,
    );
}

/// Materializes the output facts of a composite member owner. Provided-symbol
/// targets are checked in a child context that binds the owner's type
/// parameters and destructured components; retaining the target's output fact
/// lets a following member resolve on expressions such as `(x * y).inv`.
fn context_with_expression_result_facts(
    expression: &Expression,
    context: &TypeContext,
    registry: &SignatureRegistry,
) -> TypeContext {
    let mut result = context.clone();
    let subject = effective_key_for_expression(expression, context, registry);
    let mut resolving = HashSet::new();
    let inferred = expression_result_facts(expression, &subject, context, registry, &mut resolving);
    for fact in inferred {
        result.add_fact(fact);
    }
    result
}

/// Materializes the inferred output facts for every expression in a command's
/// argument list.  Requirement substitution uses the expressions' effective
/// keys, so composite arguments (for example `z + z`) need their result facts
/// in the same context in which the instantiated requirements are proved.
fn context_with_expression_results<'a>(
    expressions: impl IntoIterator<Item = &'a Expression>,
    context: &TypeContext,
    registry: &SignatureRegistry,
) -> TypeContext {
    expressions
        .into_iter()
        .fold(context.clone(), |result, expression| {
            context_with_expression_result_facts(expression, &result, registry)
        })
}

fn expression_result_facts(
    expression: &Expression,
    result_subject: &str,
    context: &TypeContext,
    registry: &SignatureRegistry,
    resolving: &mut HashSet<String>,
) -> Vec<TypeFact> {
    if let ExpressionKind::Grouped { expression, .. } | ExpressionKind::Labeled { expression, .. } =
        &expression.kind
    {
        return expression_result_facts(expression, result_subject, context, registry, resolving);
    }

    let resolving_key = format!("{}=>{result_subject}", key_for_expression(expression));
    if !resolving.insert(resolving_key.clone()) {
        return Vec::new();
    }

    let facts = match &expression.kind {
        ExpressionKind::FunctionCall { name, arguments } => {
            function_call_output_facts(name, arguments, result_subject, context, registry)
        }
        ExpressionKind::MemberCall {
            owner,
            name,
            arguments,
        } => member_output_facts(
            owner,
            name,
            arguments,
            result_subject,
            context,
            registry,
            resolving,
        ),
        ExpressionKind::MemberAccess { owner, name } => member_output_facts(
            owner,
            name,
            &[],
            result_subject,
            context,
            registry,
            resolving,
        ),
        ExpressionKind::Binary {
            left,
            operator,
            right,
        } => binary_operator_output_facts(
            left,
            operator,
            right,
            result_subject,
            context,
            registry,
            resolving,
        ),
        ExpressionKind::Command(command) => {
            let key = effective_key_for_expression(expression, context, registry);
            command_declared_result_facts(
                defined_output_facts_for_key(&key, context, registry),
                &command_expression_arguments(command),
                result_subject,
                context,
                registry,
            )
        }
        ExpressionKind::InfixCommand {
            left,
            command,
            right,
        } => {
            let key = effective_key_for_expression(expression, context, registry);
            let mut arguments = vec![left.as_ref()];
            arguments.extend(infix_command_arguments(command));
            arguments.push(right.as_ref());
            command_declared_result_facts(
                defined_output_facts_for_key(&key, context, registry),
                &arguments,
                result_subject,
                context,
                registry,
            )
        }
        _ => Vec::new(),
    };

    resolving.remove(&resolving_key);
    facts
}

/// Resolves the result of a command whose declaration gives the command itself a
/// callable type.  For example, natural addition may be declared as a binary
/// operation; applying it to two naturals yields the output spec inherited from
/// that operation's function type, rather than a value whose type is itself
/// `binary.operation`.
fn command_declared_result_facts(
    direct: Vec<TypeFact>,
    arguments: &[&Expression],
    result_subject: &str,
    context: &TypeContext,
    registry: &SignatureRegistry,
) -> Vec<TypeFact> {
    let mut child = context.clone();
    for fact in &direct {
        child.add_fact(fact.clone());
    }

    let mut inferred = Vec::new();
    for fact in &direct {
        let subject = fact_subject(fact);
        for function_type in inferred_function_type_facts_for_subject(subject, &child, registry) {
            let TypeFact::FunctionType {
                inputs,
                output,
                variadic_tuple_input,
                ..
            } = function_type
            else {
                continue;
            };
            let Some(argument_subjects) = function_type_argument_subjects_from_keys(
                inputs.len(),
                variadic_tuple_input,
                &arguments
                    .iter()
                    .map(|argument| effective_key_for_expression(argument, &child, registry))
                    .collect::<Vec<_>>(),
            ) else {
                continue;
            };
            if inputs
                .iter()
                .zip(argument_subjects)
                .all(|(input, argument)| {
                    prove_fact(
                        &instantiate_function_type_spec(input, &argument),
                        &child,
                        registry,
                    )
                })
            {
                inferred.push(
                    child.normalize_fact(&instantiate_function_type_spec(&output, result_subject)),
                );
            }
        }
    }

    if inferred.is_empty() {
        rebind_result_fact_subjects(direct, result_subject, context)
    } else {
        inferred.sort_by_key(format_fact);
        inferred.dedup();
        inferred
    }
}

fn function_call_output_facts(
    name: &str,
    arguments: &[Expression],
    result_subject: &str,
    context: &TypeContext,
    registry: &SignatureRegistry,
) -> Vec<TypeFact> {
    inferred_function_type_facts_for_subject(name, context, registry)
        .into_iter()
        .filter_map(|fact| {
            let TypeFact::FunctionType {
                inputs,
                output,
                variadic_tuple_input,
                ..
            } = fact
            else {
                return None;
            };
            function_type_argument_subjects_from_keys(
                inputs.len(),
                variadic_tuple_input,
                &arguments.iter().map(key_for_expression).collect::<Vec<_>>(),
            )?;
            Some(context.normalize_fact(&instantiate_function_type_spec(&output, result_subject)))
        })
        .collect()
}

fn inferred_function_type_facts_for_subject(
    subject: &str,
    context: &TypeContext,
    registry: &SignatureRegistry,
) -> Vec<TypeFact> {
    let subject = context.normalize_key(subject);
    let mut seen = HashSet::new();
    let mut result = Vec::new();
    for fact in &context.facts {
        collect_inferred_function_type_facts(
            fact,
            &subject,
            context,
            registry,
            &mut seen,
            &mut result,
        );
    }
    result
}

fn collect_inferred_function_type_facts(
    fact: &TypeFact,
    subject: &str,
    context: &TypeContext,
    registry: &SignatureRegistry,
    seen: &mut HashSet<TypeFact>,
    result: &mut Vec<TypeFact>,
) {
    let fact = context.normalize_fact(fact);
    if !seen.insert(fact.clone()) || context.normalize_key(fact_subject(&fact)) != subject {
        return;
    }
    if matches!(fact, TypeFact::FunctionType { .. }) {
        result.push(fact.clone());
    }
    for output in type_instance_output_facts(&fact, context, registry) {
        collect_inferred_function_type_facts(&output, subject, context, registry, seen, result);
    }
    for extended in reduce_extension_fact(&fact, context, registry) {
        collect_inferred_function_type_facts(&extended, subject, context, registry, seen, result);
    }
}

fn member_output_facts(
    owner: &Expression,
    name: &str,
    arguments: &[Expression],
    result_subject: &str,
    context: &TypeContext,
    registry: &SignatureRegistry,
    resolving: &mut HashSet<String>,
) -> Vec<TypeFact> {
    let reduced_context = context_with_spec_reductions(context, registry);
    let owner_actual = effective_key_for_expression(owner, &reduced_context, registry);
    let actuals = arguments
        .iter()
        .map(|argument| effective_key_for_expression(argument, &reduced_context, registry))
        .collect::<Vec<_>>();
    let key = DisambiguationKey::Function {
        name: name.to_owned(),
        arity: arguments.len(),
    };
    if let Some(rule) =
        find_member_provided_symbol_rule(&key, &owner_actual, &actuals, &reduced_context, registry)
    {
        return provided_symbol_output_facts(
            rule,
            &actuals,
            Some(&owner_actual),
            result_subject,
            &reduced_context,
            registry,
            resolving,
        );
    }
    let Some(component) =
        tuple_component_access_expression(owner, name, arguments, &owner_actual, &reduced_context)
    else {
        return Vec::new();
    };
    expression_result_facts(
        &component,
        result_subject,
        &reduced_context,
        registry,
        resolving,
    )
}

fn binary_operator_output_facts(
    left: &Expression,
    operator: &BinaryOperator,
    right: &Expression,
    result_subject: &str,
    context: &TypeContext,
    registry: &SignatureRegistry,
    resolving: &mut HashSet<String>,
) -> Vec<TypeFact> {
    if let Some(call) = binary_operator_application_desugaring(left, operator, right, context) {
        let facts = expression_result_facts(&call, result_subject, context, registry, resolving);
        if !facts.is_empty() {
            return facts;
        }
    }

    let (symbol, kind) = binary_operator_symbol_and_kind(operator);
    let kind = provided_binary_operator_kind(&symbol, kind).unwrap_or(NamedOperatorKind::BothColon);
    let key = DisambiguationKey::BinaryOperator(symbol);
    let result_context = context_with_expression_result_facts(left, context, registry);
    let result_context = context_with_expression_result_facts(right, &result_context, registry);
    let reduced_context = context_with_spec_reductions(&result_context, registry);
    let actuals = [left, right]
        .iter()
        .map(|expression| effective_key_for_expression(expression, &reduced_context, registry))
        .collect::<Vec<_>>();
    let Some(rule) = find_provided_symbol_rule(&key, kind, &actuals, &reduced_context, registry)
    else {
        return Vec::new();
    };
    let owner_actual = provided_symbol_owner_actual(kind, &actuals);
    provided_symbol_output_facts(
        rule,
        &actuals,
        owner_actual.as_deref(),
        result_subject,
        &reduced_context,
        registry,
        resolving,
    )
}

fn provided_symbol_output_facts(
    rule: &ProvidedSymbolRule,
    actuals: &[String],
    owner_actual: Option<&str>,
    result_subject: &str,
    context: &TypeContext,
    registry: &SignatureRegistry,
    resolving: &mut HashSet<String>,
) -> Vec<TypeFact> {
    let mut child = context.clone();
    let mut directional_substitutions = HashMap::new();
    for parameter in &rule.parameters {
        child.declare_name(parameter.clone());
    }
    for (parameter, actual) in rule.parameters.iter().zip(actuals) {
        let actual = context.normalize_key(actual);
        child.add_substitution(parameter.clone(), actual.clone());
        directional_substitutions.insert(parameter.clone(), actual);
    }
    if let Some(owner_actual) = owner_actual {
        let owner_actual = context.normalize_key(owner_actual);
        bind_provided_symbol_owner_type_parameters(
            rule,
            &owner_actual,
            context,
            &mut child,
            registry,
        );
        child.declare_name(rule.owner_subject.clone());
        child.add_substitution(rule.owner_subject.clone(), owner_actual.clone());
        directional_substitutions.insert(rule.owner_subject.clone(), owner_actual.clone());
        if let Some(source_subject) = &rule.source_subject {
            child.declare_name(source_subject.clone());
            child.add_substitution(source_subject.clone(), owner_actual.clone());
            directional_substitutions.insert(source_subject.clone(), owner_actual);
        }
    }
    bind_owner_parameter_destructurings(rule, &mut child, registry);
    expression_result_facts(&rule.target, result_subject, &child, registry, resolving)
        .into_iter()
        .map(|fact| substitute_fact(&fact, &directional_substitutions))
        .collect()
}

fn rebind_result_fact_subjects(
    facts: Vec<TypeFact>,
    result_subject: &str,
    context: &TypeContext,
) -> Vec<TypeFact> {
    facts
        .into_iter()
        .map(|fact| {
            let substitutions =
                HashMap::from([(fact_subject(&fact).to_owned(), result_subject.to_owned())]);
            context.normalize_fact(&substitute_fact(&fact, &substitutions))
        })
        .collect()
}

fn check_provided_callable_owner_function(
    name: &str,
    arguments: &[Expression],
    context: &TypeContext,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    registry: &SignatureRegistry,
    event_log: &mut EventLog,
) -> bool {
    let requirement_context = context_with_cast_expression_facts(arguments.iter(), context);
    let actuals = arguments
        .iter()
        .map(|argument| effective_key_for_expression(argument, &requirement_context, registry))
        .collect::<Vec<_>>();
    let Some(rule) = find_callable_owner_provided_symbol_rule(
        name,
        arguments.len(),
        &actuals,
        &requirement_context,
        registry,
    ) else {
        return false;
    };

    check_provided_symbol_target(
        rule,
        &actuals,
        Some(name),
        &requirement_context,
        path,
        locator,
        registry,
        event_log,
    );
    true
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

fn find_callable_owner_provided_symbol_rule<'a>(
    owner_actual: &str,
    arity: usize,
    actuals: &[String],
    context: &TypeContext,
    registry: &'a SignatureRegistry,
) -> Option<&'a ProvidedSymbolRule> {
    registry.provided_symbols.iter().find(|rule| {
        matches!(
            &rule.key,
            DisambiguationKey::Function { name, arity: key_arity }
                if name == &rule.owner_subject && *key_arity == arity
        ) && rule.parameters.len() == actuals.len()
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
        bind_provided_symbol_owner_type_parameters(
            rule,
            owner_actual,
            context,
            &mut child,
            registry,
        );
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
    bind_owner_parameter_destructurings(rule, &mut child, registry);
    check_expression(&rule.target, &child, path, locator, registry, event_log);
}

/// Makes the owner type's destructuring parameters (`M ::= (X, *)` in
/// `\magma.element:of{M ::= (X, *)}`) available while a provided-symbol rule's
/// target is checked, so a target that reaches those components (`x_ |M.*| y_`)
/// resolves `M.*` to the operation component.
fn bind_owner_parameter_destructurings(
    rule: &ProvidedSymbolRule,
    context: &mut TypeContext,
    registry: &SignatureRegistry,
) {
    let Some(info) = registry.type_infos.get(&rule.owner_signature) else {
        return;
    };
    for parameter in info.parameter_destructurings.clone() {
        bind_destructured_parameter(&parameter, context, registry);
    }
}

/// Instantiates the parameters of the type that owns a provided symbol. For a
/// member declared on `\element:of{C}`, using that member on `p is
/// \element:of{D}` must substitute `C := D` throughout the capability target.
fn bind_provided_symbol_owner_type_parameters(
    rule: &ProvidedSymbolRule,
    owner_actual: &str,
    source: &TypeContext,
    target: &mut TypeContext,
    registry: &SignatureRegistry,
) {
    let Some(info) = registry.type_infos.get(&rule.owner_signature) else {
        return;
    };
    let Some(actuals) =
        type_actuals_for_signature(owner_actual, &rule.owner_signature, source, registry)
    else {
        return;
    };
    for (parameter, actual) in info.parameters.iter().zip(actuals) {
        target.declare_name(parameter.clone());
        target.add_substitution(parameter.clone(), source.normalize_key(&actual));
    }
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
        ExpressionKind::Build {
            value: expression, ..
        } => Some(effective_key_for_expression_inner(
            expression, context, registry, resolving,
        )),
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
    if let Some(rule) =
        find_callable_owner_provided_symbol_rule(name, arguments.len(), &actuals, context, registry)
    {
        return Some(effective_key_for_provided_symbol_target(
            rule,
            &actuals,
            Some(name),
            context,
            registry,
            resolving,
        ));
    }
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
    let reduced_context = context_with_spec_reductions(context, registry);
    let key = DisambiguationKey::Function {
        name: name.to_owned(),
        arity: arguments.len(),
    };
    let owner_actual =
        effective_key_for_expression_inner(owner, &reduced_context, registry, resolving);
    let actuals = effective_keys_for_expressions(arguments, &reduced_context, registry, resolving);
    if let Some(rule) =
        find_member_provided_symbol_rule(&key, &owner_actual, &actuals, &reduced_context, registry)
    {
        return Some(effective_key_for_provided_symbol_target(
            rule,
            &actuals,
            Some(&owner_actual),
            &reduced_context,
            registry,
            resolving,
        ));
    }
    if let Some(expression) =
        tuple_component_access_expression(owner, name, arguments, &owner_actual, &reduced_context)
    {
        return Some(effective_key_for_expression_inner(
            &expression,
            &reduced_context,
            registry,
            resolving,
        ));
    }
    None
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
    if let Some(call) = named_prefix_operator_desugaring(operator, expression) {
        return Some(effective_key_for_expression_inner(
            &call, context, registry, resolving,
        ));
    }

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
    let call = postfix_operator_desugaring(expression, operator);
    Some(effective_key_for_expression_inner(
        &call, context, registry, resolving,
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
    if let Some(call) = binary_operator_application_desugaring(left, operator, right, context) {
        return Some(effective_key_for_expression_inner(
            &call, context, registry, resolving,
        ));
    }

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
    if let Some(result) =
        effective_key_for_disambiguated_target(&key, &actuals, context, registry, resolving)
    {
        return Some(result);
    }

    // Fall back to a provided-symbol capability owned by the operands' common
    // type (mirrors `check_provided_binary_operator_by_operand_type`), so
    // `y * y` yields its result type when `\magma.element` enables `x_ * y_`.
    let operand_key = DisambiguationKey::BinaryOperator(symbol);
    let reduced_context = context_with_spec_reductions(context, registry);
    let rule = find_provided_symbol_rule(
        &operand_key,
        NamedOperatorKind::BothColon,
        &actuals,
        &reduced_context,
        registry,
    )?;
    let owner_actual = provided_symbol_owner_actual(NamedOperatorKind::BothColon, &actuals);
    Some(effective_key_for_provided_symbol_target(
        rule,
        &actuals,
        owner_actual.as_deref(),
        &reduced_context,
        registry,
        resolving,
    ))
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
    let mut directional_substitutions = HashMap::new();
    for parameter in &rule.parameters {
        child.declare_name(parameter.clone());
    }
    for (parameter, actual) in rule.parameters.iter().zip(actuals) {
        let actual = context.normalize_key(actual);
        child.add_substitution(parameter.clone(), actual.clone());
        directional_substitutions.insert(parameter.clone(), actual);
    }
    if let Some(owner_actual) = owner_actual {
        let owner_actual = context.normalize_key(owner_actual);
        bind_provided_symbol_owner_type_parameters(
            rule,
            &owner_actual,
            context,
            &mut child,
            registry,
        );
        child.declare_name(rule.owner_subject.clone());
        child.add_substitution(rule.owner_subject.clone(), owner_actual.clone());
        directional_substitutions.insert(rule.owner_subject.clone(), owner_actual.clone());
        if let Some(source_subject) = &rule.source_subject {
            child.declare_name(source_subject.clone());
            child.add_substitution(source_subject.clone(), owner_actual.clone());
            directional_substitutions.insert(source_subject.clone(), owner_actual);
        }
    }
    bind_owner_parameter_destructurings(rule, &mut child, registry);

    // TypeContext substitutions represent equivalence classes and normalize to
    // the lexicographically smallest representative.  That is useful for fact
    // comparison, but a capability target must retain the call-site operands:
    // `x_ + y_ :=> x_ \.plus./ y_` applied to `z + z` must yield
    // `z \.plus./ z`, not the formal `x_ \.plus./ y_`.  Reapply the bindings
    // directionally before the effective key escapes this child context.
    substitute_key(
        &effective_key_for_expression_inner(&rule.target, &child, registry, resolving),
        &directional_substitutions,
    )
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

/// The symbol a name resolves to. A backtick-stropped operator (`` `*` ``) is
/// just the operator (`*`) referred to by name, so stropping is removed for
/// symbol lookup, keying, and type resolution.
fn unstropped_name(name: &str) -> String {
    unstrop_operator_name(name).unwrap_or_else(|| name.to_owned())
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
    let requirement_context = context_with_cast_expression_facts(std::iter::once(subject), context);
    let function_types = function_type_facts_for_subject(name, &requirement_context, registry);
    let mut found_matching_arity = false;
    for function_type in &function_types {
        if function_type_matches_call_arity(function_type, function_call_arity(subject)) {
            found_matching_arity = true;
        }
        let mut seen = HashSet::new();
        if function_type_implies_required(
            function_type,
            &required,
            &requirement_context,
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
    check_command_context_arguments(
        active_command.context.as_ref(),
        context,
        path,
        locator,
        registry,
        event_log,
    );
    let shape = shape_for_command_expression(&active_command);
    let signature = resolved_command_signature(&shape, registry);
    let position = locator.locate_reference(&shape);
    let argument_expressions = command_expression_arguments(&active_command);
    let requirement_context =
        context_with_cast_expression_facts(argument_expressions.iter().copied(), context);
    let requirement_context = context_with_expression_results(
        argument_expressions.iter().copied(),
        &requirement_context,
        registry,
    );
    let actuals = argument_expressions
        .iter()
        .map(|expression| effective_key_for_expression(expression, &requirement_context, registry))
        .collect::<Vec<_>>();
    check_command_requirements(
        &signature,
        &actuals,
        Some(&shape.arg_groups),
        active_command.context.as_ref(),
        &requirement_context,
        path,
        position,
        registry,
        event_log,
    );
}

/// Checks command arguments while keeping a mapping literal's bound parameters
/// in scope for selector groups such as `:d{x_, z_}`. Those names are local to
/// the mapping literal but intentionally referenced by sibling command groups.
fn check_command_argument_expressions(
    command: &CommandExpression,
    context: &TypeContext,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    registry: &SignatureRegistry,
    event_log: &mut EventLog,
) {
    let mut argument_context = context.clone();
    if placeholder_invocation_for_command_expression(command).is_some() {
        for expression in command_expression_arguments(command) {
            if let ExpressionKind::Mapping { lhs, .. } = &expression.kind
                && let Some(parameters) = mapping_pattern_names(lhs)
            {
                for parameter in parameters {
                    argument_context.declare_name(parameter);
                }
            }
        }
    }
    for expression in command_expression_arguments(command) {
        check_expression(
            expression,
            &argument_context,
            path,
            locator,
            registry,
            event_log,
        );
    }
}

/// Returns the concrete definition signature selected for a command invocation.
/// Validation reports ambiguity and missing-reference errors; type checking uses
/// the unresolved use-site shape as a harmless fallback so it does not duplicate
/// those diagnostics.
fn resolved_command_signature(shape: &SignatureShape, registry: &SignatureRegistry) -> String {
    resolve_definition_signature(shape, registry)
        .ok()
        .flatten()
        .unwrap_or(&shape.signature)
        .to_owned()
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
    check_command_context_arguments(
        active_command.context.as_ref(),
        context,
        path,
        locator,
        registry,
        event_log,
    );
    let shape = shape_for_command_expression(&active_command);
    let signature = resolved_command_signature(&shape, registry);
    let position = locator.locate_reference(&shape);
    let argument_expressions = command_expression_arguments(&active_command);
    let requirement_context =
        context_with_cast_expression_facts(argument_expressions.iter().copied(), context);
    let requirement_context = context_with_expression_results(
        argument_expressions.iter().copied(),
        &requirement_context,
        registry,
    );
    let actuals = argument_expressions
        .iter()
        .map(|expression| effective_key_for_expression(expression, &requirement_context, registry))
        .collect::<Vec<_>>();
    if active_command.context.is_none()
        && command_type_is_nominal_without_arguments(&signature, &actuals, registry)
    {
        return;
    }
    check_command_requirements(
        &signature,
        &actuals,
        Some(&shape.arg_groups),
        active_command.context.as_ref(),
        &requirement_context,
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
    let active_command = active_command_expression(command, context);
    check_command_argument_expressions(
        &active_command,
        context,
        path,
        locator,
        registry,
        event_log,
    );
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
    let mut argument_expressions = Vec::new();
    argument_expressions.push(left);
    argument_expressions.extend(infix_command_arguments(&active_command));
    argument_expressions.push(right);
    let requirement_context =
        context_with_cast_expression_facts(argument_expressions.iter().copied(), context);
    let requirement_context = context_with_expression_results(
        argument_expressions.iter().copied(),
        &requirement_context,
        registry,
    );
    let actuals = argument_expressions
        .iter()
        .map(|expression| effective_key_for_expression(expression, &requirement_context, registry))
        .collect::<Vec<_>>();
    check_command_requirements(
        &shape.signature,
        &actuals,
        Some(&shape.arg_groups),
        None,
        &requirement_context,
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
    let argument_expressions = refined_command_expression_arguments(&active_command);
    let requirement_context =
        context_with_cast_expression_facts(argument_expressions.iter().copied(), context);
    let requirement_context = context_with_expression_results(
        argument_expressions.iter().copied(),
        &requirement_context,
        registry,
    );
    let actuals = argument_expressions
        .iter()
        .map(|expression| effective_key_for_expression(expression, &requirement_context, registry))
        .collect::<Vec<_>>();
    if command_type_is_nominal_without_arguments(&shape.signature, &actuals, registry) {
        return;
    }
    check_command_requirements(
        &shape.signature,
        &actuals,
        Some(&shape.arg_groups),
        None,
        &requirement_context,
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
    let argument_expressions = refined_command_expression_arguments(&active_command);
    let requirement_context =
        context_with_cast_expression_facts(argument_expressions.iter().copied(), context);
    let requirement_context = context_with_expression_results(
        argument_expressions.iter().copied(),
        &requirement_context,
        registry,
    );
    let actuals = argument_expressions
        .iter()
        .map(|expression| effective_key_for_expression(expression, &requirement_context, registry))
        .collect::<Vec<_>>();
    check_command_requirements(
        &shape.signature,
        &actuals,
        Some(&shape.arg_groups),
        None,
        &requirement_context,
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
    if let Some(refinement) = &mut active.refinement {
        refinement.parts = refinement
            .parts
            .iter()
            .cloned()
            .map(|mut part| {
                part.tail = active_expression_tail(&part.tail, context);
                part
            })
            .collect();
    }
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
        ExpressionKind::Name(name) | ExpressionKind::InferredName(name) => names.push(name.clone()),
        ExpressionKind::VariadicSlice(slice) => {
            names.push(slice.name.clone());
            names.extend(variadic_slice_referenced_names(slice));
        }
        ExpressionKind::VariadicAssignment { target, value } => {
            names.push(target.name.clone());
            names.extend(variadic_slice_referenced_names(target));
            collect_defined_expression_names(value, names);
        }
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
                collect_defined_set_predicate_names(predicate, names);
            }
        }
        ExpressionKind::Grouped { expression, .. }
        | ExpressionKind::Labeled { expression, .. }
        | ExpressionKind::Prefix { expression, .. }
        | ExpressionKind::Postfix { expression, .. } => {
            collect_defined_expression_names(expression, names)
        }
        ExpressionKind::SubsetCall(subset) => collect_defined_subset_call_names(subset, names),
        ExpressionKind::IndexedCall(call) => {
            names.push(call.target.clone());
            for index in &call.indices {
                collect_defined_expression_names(index, names);
            }
        }
        ExpressionKind::Command(command) => {
            for expression in command_expression_arguments(command) {
                collect_defined_expression_names(expression, names);
            }
        }
        ExpressionKind::BuiltinCommand(_) => {}
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
        ExpressionKind::SpecStatementExpr {
            subject, target, ..
        } => {
            collect_defined_expression_names(subject, names);
            collect_defined_expression_names(target, names);
        }
        ExpressionKind::SpecLiteral(literal) => match &literal.form {
            SpecLiteralForm::Is(ty) => collect_defined_type_expression_names(ty, names),
            SpecLiteralForm::Spec { target, .. } => collect_defined_expression_names(target, names),
        },
        ExpressionKind::Satisfies { subject, spec } => {
            collect_defined_expression_names(subject, names);
            collect_defined_expression_names(spec, names);
        }
        ExpressionKind::Mapping { lhs, rhs } => {
            collect_defined_expression_names(lhs, names);
            collect_defined_expression_names(rhs, names);
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
        ExpressionKind::Build { ty, value, .. } => {
            collect_defined_type_expression_names(ty, names);
            collect_defined_expression_names(value, names);
        }
    }
}

fn collect_defined_type_expression_names(ty: &TypeExpression, names: &mut Vec<String>) {
    match ty {
        TypeExpression::Builtin { .. } => {}
        TypeExpression::Parameter { name, .. } => names.push(name.clone()),
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
        TypeExpression::Tuple(tuple) => {
            for spec in &tuple.elements {
                collect_defined_function_type_spec_names(spec, names);
            }
        }
        TypeExpression::Set(set) => match &set.element {
            SetTypeElement::Spec(spec) => collect_defined_function_type_spec_names(spec, names),
            SetTypeElement::Tuple(tuple) => {
                for spec in &tuple.elements {
                    collect_defined_function_type_spec_names(spec, names);
                }
            }
        },
        TypeExpression::Function(function_type) => {
            for spec in function_type
                .inputs
                .iter()
                .chain(std::iter::once(&function_type.output))
            {
                collect_defined_function_type_spec_names(spec, names);
            }
        }
    }
}

fn collect_defined_function_type_spec_names(spec: &FunctionTypeSpec, names: &mut Vec<String>) {
    match &spec.kind {
        FunctionTypeSpecKind::Is(ty) => collect_defined_type_expression_names(ty, names),
        FunctionTypeSpecKind::Spec { target, .. } => {
            collect_defined_expression_names(target, names)
        }
    }
}

fn collect_defined_set_target_names(target: &SetTarget, names: &mut Vec<String>) {
    match &target.kind {
        SetTargetKind::Name(name) => names.push(name.clone()),
        SetTargetKind::PlaceholderForm(form) => collect_defined_placeholder_form_names(form, names),
        SetTargetKind::Expression {
            expression,
            placeholders,
        } => {
            let mut expression_names = Vec::new();
            collect_defined_expression_names(expression, &mut expression_names);
            names.extend(
                expression_names
                    .into_iter()
                    .filter(|name| !placeholders.contains(name)),
            );
        }
        SetTargetKind::Alias { name, target } | SetTargetKind::Introduction { name, target } => {
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

fn collect_defined_set_predicate_names(predicate: &SetPredicate, names: &mut Vec<String>) {
    match predicate {
        SetPredicate::Expression(expression) => collect_defined_expression_names(expression, names),
        SetPredicate::Definition { target, value, .. } => {
            collect_defined_set_target_names(target, names);
            collect_defined_expression_names(value, names);
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
            .is_some_and(|definition| definition.kind == DefinitionKind::Defines)
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
            EnablesItem::Relation(group) => {
                let mut child = context.clone();
                if let Some(when) = &group.when {
                    for item in &when.arguments {
                        assume_relation_when_item(
                            item, &mut child, path, locator, registry, event_log,
                        );
                    }
                }
                validate_relationship_declaration(
                    &group.to.argument,
                    &mut child,
                    path,
                    locator,
                    registry,
                    event_log,
                );
                if let Some(means) = &group.means {
                    check_clause(&means.argument, &child, path, locator, registry, event_log);
                }
            }
        }
    }
}

fn validate_relationship_declaration(
    declaration: &RelationshipDeclaration,
    context: &mut TypeContext,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    registry: &SignatureRegistry,
    event_log: &mut EventLog,
) {
    match declaration {
        RelationshipDeclaration::Command(command) => {
            check_command_expression(command, context, path, locator, registry, event_log);
            let active_command = active_command_expression(command, context);
            check_command_argument_expressions(
                &active_command,
                context,
                path,
                locator,
                registry,
                event_log,
            );
        }
        RelationshipDeclaration::Declaration(statement) => {
            declare_declaration_statement_subjects(statement, context);
            complete_introduced_declaration_statement(
                statement, context, path, locator, registry, event_log,
            );
        }
    }
}

fn assume_relation_when_item(
    item: &RelationWhenItem,
    context: &mut TypeContext,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    registry: &SignatureRegistry,
    event_log: &mut EventLog,
) {
    match item {
        RelationWhenItem::Declaration(statement) => {
            assume_declaration_statement(statement, context, path, locator, registry, event_log);
        }
        RelationWhenItem::HardCast(statement) => {
            validate_hard_cast_statement(statement, context, path, locator, registry, event_log);
        }
    }
}

fn validate_hard_cast_statement(
    statement: &HardCastStatement,
    context: &mut TypeContext,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    registry: &SignatureRegistry,
    event_log: &mut EventLog,
) {
    declare_is_subject(&statement.subject, context);
    if let Some(definition) = &statement.definition {
        check_expression(definition, context, path, locator, registry, event_log);
        context.add_substitution(
            primary_subject_key(&statement.subject),
            key_for_expression(definition),
        );
    }
    check_type_expression(&statement.ty, context, path, locator, registry, event_log);
    let is_statement = IsStatement {
        span: statement.span,
        subject: statement.subject.clone(),
        ty: statement.ty.clone(),
    };
    for fact in facts_from_is_statement(&is_statement) {
        context.add_fact(fact);
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
    if let Some(literal) = statement
        .definition
        .as_ref()
        .and_then(cast_expression_set_literal)
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
            owner_subject,
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
    let signature = resolved_command_signature(&shape, registry);
    let position = locator.locate_reference(&shape);
    let argument_expressions = command_expression_arguments(&active_command);
    let requirement_context =
        context_with_cast_expression_facts(argument_expressions.iter().copied(), context);
    let requirement_context = context_with_expression_results(
        argument_expressions.iter().copied(),
        &requirement_context,
        registry,
    );
    let actuals = argument_expressions
        .iter()
        .map(|expression| {
            check_expression(expression, context, path, locator, registry, event_log);
            effective_key_for_expression(expression, &requirement_context, registry)
        })
        .collect::<Vec<_>>();
    check_command_requirements(
        &signature,
        &actuals,
        Some(&shape.arg_groups),
        active_command.context.as_ref(),
        &requirement_context,
        path,
        position,
        registry,
        event_log,
    );
    check_type_expression(&requirement.ty, context, path, locator, registry, event_log);

    if !signature_has_kind(&signature, DefinitionKind::Declares, registry) {
        emit_error(
            event_log,
            path,
            position,
            format!(
                "Required definition `{}` must reference a `Declares:` entry",
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
        SpecOperatorAliasTarget::PlaceholderSpec(spec) => {
            // The subject placeholder is bound on the left of `:->` (declared into
            // `child` above); only the target name needs checking. The spec is the
            // rule's conclusion, so it is not required to be independently supported.
            check_name(&spec.name, &child, path, locator, event_log);
        }
        SpecOperatorAliasTarget::Builtin(_) => {}
    }
}

fn validate_provided_expression_alias(
    alias: &ExpressionAlias,
    context: &TypeContext,
    owner_shapes: &[HeaderShape],
    owner_subject: &str,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    registry: &SignatureRegistry,
    event_log: &mut EventLog,
) {
    // A member capability's owner must be exactly the definition's subject, so
    // that `x.inv` provides `.inv` on values of this type.
    if let ExpressionAliasLhs::Member(member) = &alias.lhs
        && context.normalize_key(&member.owner) != context.normalize_key(owner_subject)
    {
        emit_error(
            event_log,
            path,
            locator.locate_symbol(&member.owner),
            format!(
                "Member capability owner `{}` must be the described item `{owner_subject}`",
                member.owner
            ),
        );
    }

    let mut child = context.clone();
    declare_expression_alias_lhs(&alias.lhs, &mut child);
    assume_provided_expression_alias_lhs_owner_types(
        &alias.lhs,
        owner_shapes,
        owner_subject,
        &mut child,
    );
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
        ExpressionAliasLhs::Member(member) => {
            context.declare_name(member.owner.clone());
            for argument in &member.arguments {
                context.declare_name(argument.name.clone());
            }
        }
    }
}

fn assume_provided_expression_alias_lhs_owner_types(
    lhs: &ExpressionAliasLhs,
    owner_shapes: &[HeaderShape],
    owner_subject: &str,
    context: &mut TypeContext,
) {
    if matches!(lhs, ExpressionAliasLhs::Member(_)) {
        // The owner of a member capability is the definition's subject, which
        // already carries its type in the checking context; nothing to assume.
        return;
    }
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
        FormOrDeclarationKind::FunctionDeclaration { name, form } => {
            let function_name = name.as_ref().unwrap_or(&form.name);
            if context.normalize_key(function_name) == context.normalize_key(owner_subject) {
                assume_owner_type(function_name, owner_shapes, context);
            }
        }
        FormOrDeclarationKind::Name(_)
        | FormOrDeclarationKind::MappingParameter { .. }
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
        TypeExpression::Builtin { .. } | TypeExpression::Parameter { .. } => {}
        TypeExpression::Command(command) => {
            check_command_expression(command, context, path, locator, registry, event_log);
            let active_command = active_command_expression(command, context);
            check_command_argument_expressions(
                &active_command,
                context,
                path,
                locator,
                registry,
                event_log,
            );
        }
        TypeExpression::RefinedCommand(command) => {
            check_refined_command_expression(command, context, path, locator, registry, event_log);
            let active_command = active_refined_command_expression(command, context);
            for expression in refined_command_expression_arguments(&active_command) {
                check_expression(expression, context, path, locator, registry, event_log);
            }
        }
        TypeExpression::Tuple(tuple) => {
            for spec in &tuple.elements {
                check_function_type_spec(spec, context, path, locator, registry, event_log);
            }
        }
        TypeExpression::Set(set) => match &set.element {
            SetTypeElement::Spec(spec) => {
                check_function_type_spec(spec, context, path, locator, registry, event_log)
            }
            SetTypeElement::Tuple(tuple) => {
                for spec in &tuple.elements {
                    check_function_type_spec(spec, context, path, locator, registry, event_log);
                }
            }
        },
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
    actual_arg_groups: Option<&[ArgGroupShape]>,
    command_context: Option<&CommandContext>,
    context: &TypeContext,
    path: &Path,
    position: Option<SourcePosition>,
    registry: &SignatureRegistry,
    event_log: &mut EventLog,
) {
    let Some(info) = registry.type_infos.get(signature) else {
        return;
    };

    let (mut substitutions, variadic_actuals) =
        command_parameter_substitutions(info, actuals, actual_arg_groups, context);

    let mut requirement_context = context.clone();
    add_command_context_cast_facts(command_context, &mut requirement_context);
    apply_command_context_bindings(
        command_context,
        info,
        context,
        &mut requirement_context,
        &mut substitutions,
        registry,
        path,
        position,
        event_log,
    );
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
        // Solve any inferred parameters this requirement mentions (e.g. `A`/`B`
        // in `g is \function:on{A?}:to{B?}`) by unifying it against a fact already
        // known about the subject, so later requirements can use the solved value.
        if !info.inferred_parameters.is_empty() {
            let partial = substitute_fact(requirement, &substitutions);
            infer_parameters_from_requirement(
                &partial,
                &info.inferred_parameters,
                &requirement_context,
                registry,
                &mut substitutions,
            );
        }
        for instantiated in instantiate_variadic_fact(
            requirement,
            &substitutions,
            &info.variadic_parameters,
            &variadic_actuals,
        ) {
            if !prove_fact_or_literal(&instantiated, &requirement_context, registry, 0) {
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
}

/// Proves an ordinary fact, with one additional structural rule for literal
/// command arguments.  A literal does not need a nominal `literal is T` fact:
/// its components are checked against the structure declared by `T`.
fn prove_fact_or_literal(
    required: &TypeFact,
    context: &TypeContext,
    registry: &SignatureRegistry,
    depth: usize,
) -> bool {
    if prove_fact(required, context, registry) {
        return true;
    }
    if depth >= 16 {
        return false;
    }
    let Ok(expression) = crate::frontend::formulation::parse_expression(fact_subject(required))
    else {
        return false;
    };
    match &expression.kind {
        ExpressionKind::Mapping { .. } => {
            mapping_literal_establishes(&expression, required, context, registry, depth + 1)
        }
        ExpressionKind::Tuple(elements) => {
            tuple_literal_establishes(elements, required, context, registry, depth + 1)
        }
        ExpressionKind::Set(set) => {
            set_literal_establishes(set, required, context, registry, depth + 1)
        }
        _ => false,
    }
}

fn mapping_literal_establishes(
    expression: &Expression,
    required: &TypeFact,
    context: &TypeContext,
    registry: &SignatureRegistry,
    depth: usize,
) -> bool {
    let expected = if matches!(required, TypeFact::FunctionType { .. }) {
        Some(required.clone())
    } else {
        type_instance_output_facts(required, context, registry)
            .into_iter()
            .find(|fact| matches!(fact, TypeFact::FunctionType { .. }))
    };
    let Some(TypeFact::FunctionType {
        inputs,
        output,
        variadic_tuple_input,
        ..
    }) = expected
    else {
        return false;
    };
    let ExpressionKind::Mapping { lhs, rhs } = &expression.kind else {
        return false;
    };
    let Some(parameters) = mapping_pattern_names(lhs) else {
        return false;
    };
    if parameters.is_empty() {
        return false;
    }

    let mut literal_context = context.clone();
    for parameter in &parameters {
        literal_context.declare_name(parameter.clone());
    }
    if let Some(ty) = mapping_pattern_shared_type(lhs)
        && let Some((ty, signature)) = key_for_type_expression_in_context(ty, context)
    {
        for parameter in &parameters {
            literal_context.add_fact(TypeFact::Is {
                subject: parameter.clone(),
                ty: ty.clone(),
                signature: signature.clone(),
            });
        }
    } else {
        let binders = mapping_pattern_elements(lhs).unwrap_or_else(|| vec![lhs.as_ref()]);
        for binder in binders {
            if let Some(fact) = fact_from_expression_in_context(binder, &literal_context) {
                literal_context.add_fact(fact);
            }
        }
    }
    let literal_context = context_with_spec_reductions(&literal_context, registry);

    if variadic_tuple_input {
        if inputs.len() != 1 {
            return false;
        }
        for parameter in &parameters {
            let required_input = instantiate_function_type_spec(&inputs[0], parameter);
            if !prove_fact_or_literal(&required_input, &literal_context, registry, depth) {
                return false;
            }
        }
    } else {
        if inputs.len() != parameters.len() {
            return false;
        }
        for (input, parameter) in inputs.iter().zip(&parameters) {
            let required_input = instantiate_function_type_spec(input, parameter);
            if !prove_fact_or_literal(&required_input, &literal_context, registry, depth) {
                return false;
            }
        }
    }

    let literal_context = context_with_expression_result_facts(rhs, &literal_context, registry);
    let output_subject = effective_key_for_expression(rhs, &literal_context, registry);
    let required_output = instantiate_function_type_spec(&output, &output_subject);
    prove_fact_or_literal(&required_output, &literal_context, registry, depth)
}

fn tuple_literal_establishes(
    elements: &[TupleExpressionElement],
    required: &TypeFact,
    context: &TypeContext,
    registry: &SignatureRegistry,
    depth: usize,
) -> bool {
    let TypeFact::Is { ty, signature, .. } = required else {
        return false;
    };
    let Some(info) = registry.type_infos.get(signature) else {
        return false;
    };
    if info.component_shapes.len() != elements.len() {
        return false;
    }
    let element_keys = elements
        .iter()
        .map(|element| match element {
            TupleExpressionElement::Expression(expression) => key_for_expression(expression),
            TupleExpressionElement::Operator(operator) => operator.text.clone(),
        })
        .collect::<Vec<_>>();
    let mut substitutions = literal_type_parameter_substitutions(info, ty, context);
    if let Some(described) = &info.described {
        substitutions.insert(described.clone(), fact_subject(required).to_owned());
    }
    for (fact, actual) in info.component_types.iter().zip(&element_keys) {
        substitutions.insert(fact_subject(fact).to_owned(), actual.clone());
    }
    info.component_types.iter().all(|fact| {
        let instantiated = context.normalize_fact(&substitute_fact(fact, &substitutions));
        prove_fact_or_literal(&instantiated, context, registry, depth)
    })
}

fn set_literal_establishes(
    set: &SetExpression,
    required: &TypeFact,
    context: &TypeContext,
    registry: &SignatureRegistry,
    depth: usize,
) -> bool {
    let TypeFact::Is { ty, signature, .. } = required else {
        return false;
    };
    let Some(info) = registry.type_infos.get(signature) else {
        return false;
    };
    let Some(expected_target) = &info.set_element_target else {
        return false;
    };
    let mut substitutions = literal_type_parameter_substitutions(info, ty, context);
    if let Some(described) = &info.described {
        substitutions.insert(described.clone(), fact_subject(required).to_owned());
    }
    if !bind_set_target_to_key(
        expected_target,
        &key_for_set_target(&set.target),
        &mut substitutions,
        context,
    ) {
        return false;
    }

    let mut literal_context = context.clone();
    declare_set_target(&set.target, &mut literal_context);
    for spec in &set.specs {
        if let Some(fact) = fact_from_expression_in_context(spec, &literal_context) {
            literal_context.add_fact(fact);
        }
    }
    let literal_context = context_with_spec_reductions(&literal_context, registry);
    info.set_element_types.iter().all(|fact| {
        let instantiated = literal_context.normalize_fact(&substitute_fact(fact, &substitutions));
        prove_fact_or_literal(&instantiated, &literal_context, registry, depth)
    })
}

fn literal_type_parameter_substitutions(
    info: &DefinitionTypeInfo,
    ty: &str,
    context: &TypeContext,
) -> HashMap<String, String> {
    let actuals = actuals_for_type_key(&info.signature, ty).unwrap_or_default();
    info.parameters
        .iter()
        .zip(actuals)
        .map(|(parameter, actual)| (parameter.clone(), context.normalize_key(&actual)))
        .collect()
}

fn command_parameter_substitutions(
    info: &DefinitionTypeInfo,
    actuals: &[String],
    actual_arg_groups: Option<&[ArgGroupShape]>,
    context: &TypeContext,
) -> (HashMap<String, String>, HashMap<String, VariadicActual>) {
    if info.variadic_parameters.is_empty() || actual_arg_groups.is_none() {
        return (
            info.parameters
                .iter()
                .zip(actuals)
                .map(|(name, actual)| (name.clone(), context.normalize_key(actual)))
                .collect(),
            HashMap::new(),
        );
    }

    let actual_arg_groups = actual_arg_groups.expect("checked above");
    let mut variadic_shapes = HashMap::new();
    let mut variadics = info.variadic_parameters.iter();
    for (expected, actual) in info.arg_groups.iter().zip(actual_arg_groups) {
        if matches!(
            expected.count,
            ArgCount::Variadic { .. } | ArgCount::Variadic2D { .. }
        ) && let Some(parameter) = variadics.next()
        {
            match &actual.count {
                ArgCount::Exact(count) => {
                    variadic_shapes.insert(parameter.name.clone(), (*count, None));
                }
                ArgCount::Exact2D { row_lengths } => {
                    variadic_shapes.insert(
                        parameter.name.clone(),
                        (row_lengths.iter().sum(), Some(row_lengths.clone())),
                    );
                }
                _ => {}
            }
        }
    }

    let mut substitutions = HashMap::new();
    let mut grouped_actuals = HashMap::new();
    let mut actual_index = 0usize;
    for parameter in &info.parameters {
        if let Some((count, rows)) = variadic_shapes.get(parameter).cloned() {
            let end = (actual_index + count).min(actuals.len());
            let values = actuals[actual_index..end]
                .iter()
                .map(|actual| context.normalize_key(actual))
                .collect::<Vec<_>>();
            actual_index = end;
            substitutions.insert(parameter.clone(), format!("({})", values.join(",")));
            grouped_actuals.insert(parameter.clone(), VariadicActual { values, rows });
        } else if let Some(actual) = actuals.get(actual_index) {
            substitutions.insert(parameter.clone(), context.normalize_key(actual));
            actual_index += 1;
        }
    }

    for variadic in &info.variadic_parameters {
        let Some(actual) = grouped_actuals.get(&variadic.name) else {
            continue;
        };
        let values = &actual.values;
        if let Some(dimensions) = &variadic.dimensions {
            if let Some(rows) = &actual.rows {
                let row_count = rows.len();
                let column_count = rows.first().copied().unwrap_or(0);
                if let Some(length) = &dimensions.row_length {
                    substitutions.insert(length.clone(), row_count.to_string());
                }
                if let Some(length) = &dimensions.column_length {
                    substitutions.insert(length.clone(), column_count.to_string());
                }
                for (offset, value) in values.iter().enumerate() {
                    if column_count == 0 {
                        continue;
                    }
                    let row = dimensions.row_start + offset / column_count;
                    let column = dimensions.column_start + offset % column_count;
                    substitutions
                        .insert(format!("{}[{row},{column}]", variadic.name), value.clone());
                }
            }
        } else {
            if let Some(length) = &variadic.length {
                substitutions.insert(length.clone(), values.len().to_string());
                if let Some(last) = values.last() {
                    substitutions.insert(format!("{}[{length}]", variadic.name), last.clone());
                }
            }
            for (offset, value) in values.iter().enumerate() {
                let starts = if variadic.index.is_some() {
                    vec![variadic.start]
                } else {
                    vec![0, 1]
                };
                for start in starts {
                    substitutions.insert(
                        format!("{}[{}]", variadic.name, start + offset),
                        value.clone(),
                    );
                }
            }
        }
    }

    (substitutions, grouped_actuals)
}

#[derive(Clone, Debug)]
struct VariadicActual {
    values: Vec<String>,
    rows: Option<Vec<usize>>,
}

fn instantiate_variadic_fact(
    fact: &TypeFact,
    substitutions: &HashMap<String, String>,
    parameters: &[VariadicParameter],
    actuals: &HashMap<String, VariadicActual>,
) -> Vec<TypeFact> {
    let rendered = format_fact(fact);
    let referenced = parameters
        .iter()
        .filter(|parameter| {
            (parameter.dimensions.is_some() && rendered.contains(&format!("{}[", parameter.name)))
                || rendered.contains(&format!("{}...", parameter.name))
                || rendered
                    .match_indices(&format!("{}[", parameter.name))
                    .any(|(start, _)| {
                        rendered[start..]
                            .split(']')
                            .next()
                            .is_some_and(|part| part.contains("..."))
                    })
        })
        .collect::<Vec<_>>();
    if referenced.is_empty() {
        return vec![substitute_fact(fact, substitutions)];
    }

    let count = referenced
        .iter()
        .filter_map(|parameter| {
            actuals
                .get(&parameter.name)
                .map(|actual| actual.values.len())
        })
        .min()
        .unwrap_or(0);
    (0..count)
        .map(|offset| {
            let mut element_substitutions = substitutions.clone();
            for parameter in &referenced {
                let Some(value) = actuals
                    .get(&parameter.name)
                    .and_then(|actual| actual.values.get(offset))
                else {
                    continue;
                };
                add_variadic_element_substitutions(
                    &mut element_substitutions,
                    parameter,
                    offset,
                    value,
                    actuals.get(&parameter.name),
                );
            }
            substitute_fact(fact, &element_substitutions)
        })
        .collect()
}

fn add_variadic_element_substitutions(
    substitutions: &mut HashMap<String, String>,
    parameter: &VariadicParameter,
    offset: usize,
    value: &str,
    actual: Option<&VariadicActual>,
) {
    substitutions.insert(format!("{}...", parameter.name), value.to_owned());
    if let (Some(dimensions), Some(rows)) = (
        &parameter.dimensions,
        actual.and_then(|actual| actual.rows.as_ref()),
    ) {
        let columns = rows.first().copied().unwrap_or(0);
        if columns > 0 {
            let row = dimensions.row_start + offset / columns;
            let column = dimensions.column_start + offset % columns;
            substitutions.insert(
                format!("{}[{row},{column}]", parameter.name),
                value.to_owned(),
            );
            substitutions.insert(
                format!(
                    "{}[{},{}]",
                    parameter.name, dimensions.row_index, dimensions.column_index
                ),
                value.to_owned(),
            );
            substitutions.insert(
                format_two_dimensional_parameter_slice(parameter),
                value.to_owned(),
            );
            // A whole-axis selection is the dimension-independent spelling for
            // every cell.  It is especially useful when the header omits `m`
            // and `n`, and must expand element-by-element just like the fully
            // indexed range above.
            substitutions.insert(format!("{}[...,...]", parameter.name), value.to_owned());
        }
        return;
    }
    let starts = if parameter.index.is_some() {
        vec![parameter.start]
    } else {
        vec![0, 1]
    };
    for start in &starts {
        substitutions.insert(
            format!("{}[{}]", parameter.name, start + offset),
            value.to_owned(),
        );
    }
    if let Some(index) = &parameter.index {
        substitutions.insert(format!("{}[{index}]", parameter.name), value.to_owned());
    }
    if let Some(end) = &parameter.length {
        for start in starts {
            substitutions.insert(
                format!("{}[{start}...{end}]", parameter.name),
                value.to_owned(),
            );
            if let Some(index) = &parameter.index {
                substitutions.insert(
                    format!("{}[{start}...{index}...{end}]", parameter.name),
                    value.to_owned(),
                );
            }
        }
    }
}

fn format_two_dimensional_parameter_slice(parameter: &VariadicParameter) -> String {
    let dimensions = parameter
        .dimensions
        .as_ref()
        .expect("called only for a 2D parameter");
    let rows = dimensions.row_length.as_deref().unwrap_or(".");
    let columns = dimensions.column_length.as_deref().unwrap_or(".");
    format!(
        "{}[{}...{}...{},{}...{}...{}]",
        parameter.name,
        dimensions.row_start,
        dimensions.row_index,
        rows,
        dimensions.column_start,
        dimensions.column_index,
        columns
    )
}

/// Binds inferred parameters mentioned by an `is` requirement by unifying its
/// type key against the type key of a fact already known about the same subject.
/// For `p is \function:on{A}:to{B}` and a known `p is \function:on{X}:to{Y}`, this
/// binds `A := X`, `B := Y` into `substitutions`.
fn infer_parameters_from_requirement(
    requirement: &TypeFact,
    inferred: &[String],
    context: &TypeContext,
    registry: &SignatureRegistry,
    substitutions: &mut HashMap<String, String>,
) {
    let TypeFact::Is {
        subject,
        ty,
        signature,
    } = requirement
    else {
        return;
    };
    let has_unsolved = inferred
        .iter()
        .any(|name| !substitutions.contains_key(name) && key_mentions_name(ty, name));
    if !has_unsolved {
        return;
    }
    let subject_key = context.normalize_key(subject);

    // Gather the subject's `is` facts, expanding through extension rules so that,
    // e.g., `* is \binary.operation:on{X}` also yields `* is \function:on{…}` for
    // matching against `\function:on{A}:to{B}`.
    let mut queue: Vec<TypeFact> = context
        .facts
        .iter()
        .filter(|fact| {
            matches!(fact, TypeFact::Is { subject, .. } if context.normalize_key(subject) == subject_key)
        })
        .cloned()
        .collect();
    let mut seen = HashSet::new();
    while let Some(fact) = queue.pop() {
        if !seen.insert(fact.clone()) {
            continue;
        }
        if let TypeFact::Is {
            ty: fact_ty,
            signature: fact_signature,
            ..
        } = &fact
            && fact_signature == signature
            && let Some(bindings) = unify_command_type_keys(ty, fact_ty, inferred)
        {
            for (name, value) in bindings {
                substitutions
                    .entry(name)
                    .or_insert_with(|| context.normalize_key(&value));
            }
            return;
        }
        queue.extend(reduce_extension_fact(&fact, context, registry));
    }
}

/// Unifies two command-type keys (`\function:on{A}:to{B}` vs
/// `\function:on{X}:to{Y}`), binding each inferred-parameter argument to the
/// corresponding concrete argument. Returns `None` when the keys have different
/// shapes or a non-inferred argument disagrees.
fn unify_command_type_keys(
    required: &str,
    actual: &str,
    inferred: &[String],
) -> Option<Vec<(String, String)>> {
    let (base_required, args_required) = command_type_key_args(required);
    let (base_actual, args_actual) = command_type_key_args(actual);
    if base_required != base_actual || args_required.len() != args_actual.len() {
        return None;
    }
    let mut bindings = Vec::new();
    for (required_arg, actual_arg) in args_required.iter().zip(&args_actual) {
        if inferred.iter().any(|name| name == required_arg) {
            bindings.push((required_arg.clone(), actual_arg.clone()));
        } else if required_arg != actual_arg {
            return None;
        }
    }
    Some(bindings)
}

/// Splits a command-type key into its base (with `{...}` argument groups removed)
/// and the ordered contents of those groups: `\function:on{A}:to{B}` becomes
/// (`\function:on:to`, [`A`, `B`]).
fn command_type_key_args(key: &str) -> (String, Vec<String>) {
    let bytes = key.as_bytes();
    let mut base = String::new();
    let mut args = Vec::new();
    let mut index = 0;
    let mut segment_start = 0;
    while index < bytes.len() {
        if bytes[index] == b'{' {
            base.push_str(&key[segment_start..index]);
            let mut depth = 1usize;
            let mut end = index + 1;
            while end < bytes.len() {
                match bytes[end] {
                    b'{' => depth += 1,
                    b'}' => {
                        depth -= 1;
                        if depth == 0 {
                            break;
                        }
                    }
                    _ => {}
                }
                end += 1;
            }
            args.push(key[index + 1..end].to_string());
            index = end + 1;
            segment_start = index;
        } else {
            index += 1;
        }
    }
    base.push_str(&key[segment_start..]);
    (base, args)
}

fn check_command_context_arguments(
    command_context: Option<&CommandContext>,
    context: &TypeContext,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    registry: &SignatureRegistry,
    event_log: &mut EventLog,
) {
    let Some(command_context) = command_context else {
        return;
    };

    let mut local_context = context.clone();
    for argument in &command_context.arguments {
        match argument {
            CommandContextArgument::Assignment { value, .. } => {
                check_expression(value, &local_context, path, locator, registry, event_log);
            }
            CommandContextArgument::Declaration(statement) => {
                assume_declaration_statement(
                    statement,
                    &mut local_context,
                    path,
                    locator,
                    registry,
                    event_log,
                );
            }
            CommandContextArgument::Expression(expression) => {
                check_expression(
                    expression,
                    &local_context,
                    path,
                    locator,
                    registry,
                    event_log,
                );
            }
            CommandContextArgument::Text(_) => {}
        }
    }
}

fn apply_command_context_bindings(
    command_context: Option<&CommandContext>,
    info: &DefinitionTypeInfo,
    context: &TypeContext,
    requirement_context: &mut TypeContext,
    substitutions: &mut HashMap<String, String>,
    registry: &SignatureRegistry,
    path: &Path,
    position: Option<SourcePosition>,
    event_log: &mut EventLog,
) {
    let Some(command_context) = command_context else {
        return;
    };

    let parameters = command_context_parameters(command_context.kind, info);
    if parameters.is_empty() {
        emit_error(
            event_log,
            path,
            position,
            format!(
                "Command `{}` does not accept `{}`",
                info.signature,
                command_context_label(command_context.kind)
            ),
        );
    }

    for argument in &command_context.arguments {
        match argument {
            CommandContextArgument::Assignment { name, value, .. } => {
                if !parameters.iter().any(|parameter| parameter == name) {
                    emit_error(
                        event_log,
                        path,
                        position,
                        format!(
                            "Unknown `{}` parameter `{name}` for command `{}`",
                            command_context_label(command_context.kind),
                            info.signature
                        ),
                    );
                    continue;
                }
                substitutions.insert(
                    name.clone(),
                    requirement_context.normalize_key(&effective_key_for_expression(
                        value,
                        requirement_context,
                        registry,
                    )),
                );
            }
            CommandContextArgument::Declaration(statement) => {
                apply_command_context_declaration(
                    statement,
                    command_context.kind,
                    &parameters,
                    context,
                    requirement_context,
                    substitutions,
                    path,
                    position,
                    event_log,
                    info,
                );
            }
            CommandContextArgument::Expression(expression) => {
                if parameters.len() == 1 {
                    substitutions.insert(
                        parameters[0].clone(),
                        requirement_context.normalize_key(&effective_key_for_expression(
                            expression,
                            requirement_context,
                            registry,
                        )),
                    );
                }
                if let Some(fact) = fact_from_expression_in_context(expression, requirement_context)
                {
                    let normalized = requirement_context.normalize_fact(&fact);
                    requirement_context.add_fact(normalized);
                }
            }
            CommandContextArgument::Text(_) => {}
        }
    }

    for parameter in parameters {
        if !substitutions.contains_key(parameter) {
            emit_error(
                event_log,
                path,
                position,
                format!(
                    "Missing `{}` value for parameter `{parameter}` on command `{}`",
                    command_context_label(command_context.kind),
                    info.signature
                ),
            );
        }
    }
}

fn apply_command_context_declaration(
    statement: &DeclarationStatement,
    kind: CommandContextKind,
    parameters: &[String],
    context: &TypeContext,
    requirement_context: &mut TypeContext,
    substitutions: &mut HashMap<String, String>,
    path: &Path,
    position: Option<SourcePosition>,
    event_log: &mut EventLog,
    info: &DefinitionTypeInfo,
) {
    declare_is_subject(&statement.subject, requirement_context);
    if let Some(expansion) = &statement.expansion {
        declare_is_subject(expansion, requirement_context);
    }
    if let Some((left, right)) = declaration_substitution(statement) {
        requirement_context.add_substitution(left, right);
    }
    for fact in facts_from_declaration_statement_in_context(statement, requirement_context) {
        let normalized = requirement_context.normalize_fact(&fact);
        requirement_context.add_fact(normalized);
    }

    if parameters.len() != 1 {
        return;
    }
    let subjects = declaration_subject_keys(statement);
    let Some(subject) = subjects.first() else {
        return;
    };
    if subjects.len() > 1 {
        emit_error(
            event_log,
            path,
            position,
            format!(
                "Shorthand `{}` for command `{}` must identify exactly one value",
                command_context_label(kind),
                info.signature
            ),
        );
        return;
    }
    substitutions.insert(parameters[0].clone(), context.normalize_key(subject));
}

fn command_context_parameters<'a>(
    kind: CommandContextKind,
    info: &'a DefinitionTypeInfo,
) -> &'a [String] {
    match kind {
        CommandContextKind::Using => &info.using_parameters,
        CommandContextKind::Given => &info.given_parameters,
    }
}

fn command_context_label(kind: CommandContextKind) -> &'static str {
    match kind {
        CommandContextKind::Using => "#using",
        CommandContextKind::Given => "#given",
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
    let mut spec_seen = HashSet::new();
    prove_fact_threaded(required, context, registry, allow_viewable, &mut spec_seen)
}

fn prove_fact_threaded(
    required: &TypeFact,
    context: &TypeContext,
    registry: &SignatureRegistry,
    allow_viewable: bool,
    spec_seen: &mut HashSet<TypeFact>,
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

    if context.facts.iter().any(|fact| {
        fact_implies_with_options(
            fact,
            &required,
            context,
            registry,
            &mut seen,
            allow_viewable,
        )
    }) {
        return true;
    }

    // Numeric spellings are ordinary names first: an explicit fact in the
    // current scope wins. Only after local/derived facts fail do the global
    // `Specify:` categories provide their fallback type.
    if !context.has_name(fact_subject(&required))
        && let Some(fact) = numeric_literal_fact(fact_subject(&required), registry)
        && fact_implies_with_options(
            &fact,
            &required,
            context,
            registry,
            &mut seen,
            allow_viewable,
        )
    {
        return true;
    }

    // A spec requirement such as `x "in" G` is defined by the capability that
    // provides its operator (`x_ "in" G :-> x_ is \group.element:of{G}`), and that
    // definition is an equivalence. So the requirement holds when some providing
    // capability's reduction target holds — the reverse of `reduce_spec_fact`'s
    // forward materialization. `spec_seen` guards against reduction cycles.
    match &required {
        TypeFact::Spec { .. } => spec_requirement_holds_via_provider(
            &required,
            context,
            registry,
            allow_viewable,
            spec_seen,
        ),
        TypeFact::MemberOf {
            subject,
            collection,
        } => {
            let facts =
                facts_from_collection_body_membership(subject, collection, context, registry);
            !facts.is_empty()
                && facts.iter().all(|fact| {
                    prove_fact_threaded(fact, context, registry, allow_viewable, spec_seen)
                })
        }
        _ => false,
    }
}

fn numeric_literal_fact(subject: &str, registry: &SignatureRegistry) -> Option<TypeFact> {
    let specification = if is_decimal_literal(subject) {
        registry.numeric_specifications.decimal.as_ref()
    } else if subject == "0" {
        registry
            .numeric_specifications
            .zero_or_positive_int
            .as_ref()
    } else if is_positive_integer_literal(subject) {
        registry.numeric_specifications.positive_int.as_ref()
    } else if is_negative_integer_literal(subject) {
        registry.numeric_specifications.int.as_ref()
    } else {
        None
    }?;
    Some(TypeFact::Is {
        subject: subject.to_owned(),
        ty: specification.ty.clone(),
        signature: specification.signature.clone(),
    })
}

fn is_decimal_literal(value: &str) -> bool {
    let value = value.strip_prefix('-').unwrap_or(value);
    let Some((whole, fractional)) = value.split_once('.') else {
        return false;
    };
    !whole.is_empty()
        && !fractional.is_empty()
        && whole.chars().all(|ch| ch.is_ascii_digit())
        && fractional.chars().all(|ch| ch.is_ascii_digit())
}

fn is_positive_integer_literal(value: &str) -> bool {
    !value.is_empty()
        && value != "0"
        && value.chars().all(|ch| ch.is_ascii_digit())
        && value.chars().any(|ch| ch != '0')
}

fn is_negative_integer_literal(value: &str) -> bool {
    value
        .strip_prefix('-')
        .is_some_and(is_positive_integer_literal)
}

/// Whether a spec requirement holds because a capability that provides its
/// operator reduces it to facts that themselves hold. Each providing capability
/// is an independent way to satisfy the spec (so the rules are tried
/// disjunctively), while all of a single capability's target facts must hold.
fn spec_requirement_holds_via_provider(
    required: &TypeFact,
    context: &TypeContext,
    registry: &SignatureRegistry,
    allow_viewable: bool,
    spec_seen: &mut HashSet<TypeFact>,
) -> bool {
    let TypeFact::Spec {
        subject,
        operator,
        target,
    } = required
    else {
        return false;
    };
    if !spec_seen.insert(required.clone()) {
        return false;
    }
    for rule in &registry.spec_rules {
        if &rule.operator != operator {
            continue;
        }
        if !has_type_signature(target, &rule.owner_signature, context, registry) {
            continue;
        }
        if rule.source_requires_literal
            && !collection_is_literal_or_has_body(target, context, registry)
        {
            continue;
        }
        let targets = spec_rule_direct_targets(rule, subject, target, context);
        if !targets.is_empty()
            && targets
                .iter()
                .all(|fact| prove_fact_threaded(fact, context, registry, allow_viewable, spec_seen))
        {
            return true;
        }
    }
    false
}

/// The facts a spec rule reduces `subject "op" target` to, substituted but not
/// recursively reduced. Mirrors the per-rule branch of [`reduce_spec_fact`].
fn spec_rule_direct_targets(
    rule: &SpecOperatorRule,
    subject: &str,
    target: &str,
    context: &TypeContext,
) -> Vec<TypeFact> {
    let mut substitutions = HashMap::from([
        (rule.placeholder.clone(), subject.to_owned()),
        (rule.target.clone(), target.to_owned()),
    ]);
    if let Some(source_subject) = &rule.source_subject {
        substitutions.insert(source_subject.clone(), target.to_owned());
    }

    let mut result = Vec::new();
    match &rule.target_alias {
        SpecOperatorAliasTarget::Builtin(_) => {}
        SpecOperatorAliasTarget::IsOrSpec(target_alias) => {
            for next in facts_from_is_or_spec(target_alias) {
                result.push(context.normalize_fact(&substitute_fact(&next, &substitutions)));
            }
        }
        SpecOperatorAliasTarget::MemberOf(target_alias) => {
            if rule.source_subject.is_none() {
                return result;
            }
            if let Some(next) = fact_from_expression(target_alias) {
                result.push(context.normalize_fact(&substitute_fact(&next, &substitutions)));
            }
        }
        SpecOperatorAliasTarget::PlaceholderSpec(target_alias) => {
            if let Some(subject) = placeholder_pattern_name(&target_alias.placeholder_form) {
                let next = TypeFact::Spec {
                    subject,
                    operator: target_alias.operator.clone(),
                    target: target_alias.name.clone(),
                };
                result.push(context.normalize_fact(&substitute_fact(&next, &substitutions)));
            }
        }
    }
    result
}

fn prove_fact_allowing_abstraction(
    required: &TypeFact,
    context: &TypeContext,
    registry: &SignatureRegistry,
) -> bool {
    if prove_fact(required, context, registry) {
        return true;
    }

    let required = context.normalize_fact(required);
    let mut seen = HashSet::new();
    if defined_output_facts_for_key(fact_subject(&required), context, registry)
        .iter()
        .any(|fact| {
            fact_implies_allowing_abstraction(fact, &required, context, registry, &mut seen)
        })
    {
        return true;
    }

    context.facts.iter().any(|fact| {
        fact_implies_allowing_abstraction(fact, &required, context, registry, &mut seen)
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
        .is_some_and(|signature| signature_has_kind(signature, DefinitionKind::Defines, registry))
        || infix_command_signatures_from_key(key)
            .iter()
            .any(|signature| signature_has_kind(signature, DefinitionKind::Defines, registry))
}

fn key_is_statement(key: &str, registry: &SignatureRegistry) -> bool {
    key_is_builtin_clause_command(key)
        || key_contains_top_level(key, " is? ")
        || key_contains_top_level(key, " is_not? ")
        || key_contains_top_level(key, " = ")
        || key_contains_top_level(key, " != ")
        || key_contains_top_level_quoted_spec(key, true)
        || key_contains_top_level_infix_spec(key, true)
        || key_is_states_command_reference(key, registry)
}

fn key_is_builtin_clause_command(key: &str) -> bool {
    [
        "\\\\not",
        "\\\\and",
        "\\\\allOf",
        "\\\\or",
        "\\\\anyOf",
        "\\\\oneOf",
        "\\\\exists",
        "\\\\existsUnique",
        "\\\\forAll",
        "\\\\forall",
        "\\\\if",
        "\\\\have",
        "\\\\given",
        "\\\\piecewise",
    ]
    .iter()
    .any(|prefix| {
        key == *prefix
            || key
                .strip_prefix(prefix)
                .is_some_and(|rest| rest.starts_with('{') || rest.starts_with(':'))
    })
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
    let key = strip_command_context_key_suffix(key);
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

fn strip_command_context_key_suffix(key: &str) -> &str {
    let using_index = find_top_level_key_substring(key, "#using");
    let given_index = find_top_level_key_substring(key, "#given");
    match (using_index, given_index) {
        (Some(left), Some(right)) => &key[..left.min(right)],
        (Some(index), None) | (None, Some(index)) => &key[..index],
        (None, None) => key,
    }
}

fn find_top_level_key_substring(key: &str, needle: &str) -> Option<usize> {
    let mut search_start = 0;
    while search_start < key.len() {
        let Some(relative_start) = key[search_start..].find(needle) else {
            return None;
        };
        let start = search_start + relative_start;
        if key_is_top_level_at(key, start) {
            return Some(start);
        }
        search_start = start + needle.len();
    }
    None
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

    if equivalence_fact_implies_required(&fact, required, context, registry) {
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

fn fact_implies_allowing_abstraction(
    fact: &TypeFact,
    required: &TypeFact,
    context: &TypeContext,
    registry: &SignatureRegistry,
    seen: &mut HashSet<TypeFact>,
) -> bool {
    let fact = context.normalize_fact(fact);
    if fact_implies_with_options(
        &fact,
        required,
        context,
        registry,
        &mut HashSet::new(),
        true,
    ) {
        return true;
    }
    if !seen.insert(fact.clone()) {
        return false;
    }

    if abstraction_fact_implies_required(&fact, required, context, registry, seen) {
        return true;
    }

    for extended in reduce_extension_fact(&fact, context, registry) {
        if fact_implies_allowing_abstraction(&extended, required, context, registry, seen) {
            return true;
        }
    }

    for reduced in reduce_refined_fact(&fact, context, registry) {
        if fact_implies_allowing_abstraction(&reduced, required, context, registry, seen) {
            return true;
        }
    }

    if matches!(fact, TypeFact::Spec { .. } | TypeFact::MemberOf { .. }) {
        let mut spec_seen = HashSet::new();
        for reduced in reduce_spec_or_member_fact(&fact, context, registry, &mut spec_seen) {
            if fact_implies_allowing_abstraction(&reduced, required, context, registry, seen) {
                return true;
            }
        }
    }

    false
}

fn abstraction_fact_implies_required(
    fact: &TypeFact,
    required: &TypeFact,
    context: &TypeContext,
    registry: &SignatureRegistry,
    seen: &mut HashSet<TypeFact>,
) -> bool {
    let TypeFact::Is {
        subject,
        ty,
        signature,
    } = fact
    else {
        return false;
    };
    let actuals = actuals_for_type_key(signature, ty).unwrap_or_default();

    registry
        .abstraction_rules
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
            let abstracted = context.normalize_fact(&substitute_fact(&rule.target, &substitutions));
            fact_implies_with_options(&abstracted, required, context, registry, seen, true)
        })
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

/// A value known to be one member of an equivalence class satisfies a requirement
/// that it be any other member of the same class, as long as the target member's
/// header parameters are all pinned — to matching actuals — by the known member.
/// This is the interchangeability declared by a top-level `Equivalent:` item.
fn equivalence_fact_implies_required(
    fact: &TypeFact,
    required: &TypeFact,
    context: &TypeContext,
    registry: &SignatureRegistry,
) -> bool {
    if registry.equivalence_classes.is_empty() {
        return false;
    }
    // Both facts must speak about the same value.
    if context.normalize_key(fact_subject(fact)) != context.normalize_key(fact_subject(required)) {
        return false;
    }
    let Some((from_signature, from_actuals)) = command_fact_signature_and_actuals(fact) else {
        return false;
    };
    let Some((to_signature, to_actuals)) = command_fact_signature_and_actuals(required) else {
        return false;
    };
    if from_signature == to_signature {
        return false; // identical commands are settled by the direct equality check
    }

    registry.equivalence_classes.iter().any(|class| {
        let (Some(from), Some(to)) = (class.member(&from_signature), class.member(&to_signature))
        else {
            return false;
        };
        if from.params.len() != from_actuals.len() || to.params.len() != to_actuals.len() {
            return false;
        }
        // Bind each header parameter to the actual the known member supplies, then
        // require the target member's actuals to agree under that binding. A target
        // parameter the known member does not mention leaves the target actual
        // unpinned, so the reduction is (soundly) not available.
        let bindings: HashMap<&str, String> = from
            .params
            .iter()
            .map(String::as_str)
            .zip(
                from_actuals
                    .iter()
                    .map(|actual| context.normalize_key(actual)),
            )
            .collect();
        to.params.iter().zip(&to_actuals).all(|(param, actual)| {
            bindings
                .get(param.as_str())
                .is_some_and(|bound| *bound == context.normalize_key(actual))
        })
    })
}

/// Whether two distinct command signatures are members of a common equivalence
/// class (declared by some top-level `Equivalent:` item).
fn signatures_are_equivalent(left: &str, right: &str, registry: &SignatureRegistry) -> bool {
    left != right
        && registry
            .equivalence_classes
            .iter()
            .any(|class| class.member(left).is_some() && class.member(right).is_some())
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
    let implicit_refinement = implicit_refined_infix_spec_resolution(fact, context, registry);
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
    let mut result = implicit_refinement
        .iter()
        .flat_map(|resolution| {
            [
                resolution.base_fact.clone(),
                resolution.refined_type_fact.clone(),
            ]
        })
        .collect::<Vec<_>>();
    result.extend(
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
            }),
    );
    result
}

#[derive(Clone, Debug)]
struct ImplicitRefinedInfixSpecResolution {
    base_signature: String,
    base_actuals: Vec<String>,
    refined_type_signature: String,
    refined_type_actuals: Vec<String>,
    base_fact: TypeFact,
    refined_type_fact: TypeFact,
}

/// Resolves an undeclared refined spec-infix fact through the type extended by
/// its declared base operator. For example, if `\:subset:/` extends `\set` and
/// `\nonempty::set` is a declared refinement, then
/// `A \:nonempty::subset:/ B` implicitly denotes both `A \:subset:/ B` and
/// `A is \nonempty::set`.
fn implicit_refined_infix_spec_resolution(
    fact: &TypeFact,
    context: &TypeContext,
    registry: &SignatureRegistry,
) -> Vec<ImplicitRefinedInfixSpecResolution> {
    let TypeFact::InfixSpec {
        subject,
        signature,
        args,
        target,
    } = fact
    else {
        return Vec::new();
    };
    if registry.definitions.contains_key(signature) {
        return Vec::new();
    }

    let Some((refinement_prefix, base_signature)) = split_refined_infix_spec_signature(signature)
    else {
        return Vec::new();
    };
    let Some(base_definition) = registry.definitions.get(&base_signature) else {
        return Vec::new();
    };
    if base_definition.kind != DefinitionKind::Defines {
        return Vec::new();
    }
    let Some(base_info) = registry.type_infos.get(&base_signature) else {
        return Vec::new();
    };

    let mut full_actuals = Vec::with_capacity(args.len() + 2);
    full_actuals.push(context.normalize_key(subject));
    full_actuals.extend(args.iter().map(|arg| context.normalize_key(arg)));
    full_actuals.push(context.normalize_key(target));
    let Some(refinement_actual_count) = full_actuals.len().checked_sub(base_info.parameters.len())
    else {
        return Vec::new();
    };
    if refinement_actual_count > args.len() {
        return Vec::new();
    }

    let refinement_actuals = full_actuals[1..1 + refinement_actual_count].to_vec();
    let mut base_actuals = Vec::with_capacity(base_info.parameters.len());
    base_actuals.push(full_actuals[0].clone());
    base_actuals.extend_from_slice(&full_actuals[1 + refinement_actual_count..]);
    let base_fact = TypeFact::InfixSpec {
        subject: subject.clone(),
        signature: base_signature.clone(),
        args: args[refinement_actual_count..].to_vec(),
        target: target.clone(),
    };

    registry
        .extension_rules
        .iter()
        .filter(|rule| rule.subtype_signature == base_signature)
        .filter_map(|rule| {
            let mut substitutions = base_info
                .parameters
                .iter()
                .zip(&base_actuals)
                .map(|(name, actual)| (name.clone(), actual.clone()))
                .collect::<HashMap<_, _>>();
            substitutions.insert(rule.subject.clone(), context.normalize_key(subject));
            let extended = context.normalize_fact(&substitute_fact(&rule.target, &substitutions));
            let TypeFact::Is {
                ty: base_ty,
                signature: base_type_signature,
                ..
            } = extended
            else {
                return None;
            };

            let refined_type_signature = format!(
                "\\{}::{}",
                refinement_prefix,
                base_type_signature.strip_prefix('\\')?
            );
            let refined_definition = registry.definitions.get(&refined_type_signature)?;
            if refined_definition.kind != DefinitionKind::Refines {
                return None;
            }
            let refined_info = registry.type_infos.get(&refined_type_signature)?;
            let mut refined_type_actuals = refinement_actuals.clone();
            refined_type_actuals.extend(actuals_for_type_key(&base_type_signature, &base_ty)?);
            if refined_type_actuals.len() != refined_info.parameters.len() {
                return None;
            }
            let refined_substitutions = refined_info
                .parameters
                .iter()
                .zip(&refined_type_actuals)
                .map(|(name, actual)| (name.clone(), actual.clone()))
                .collect::<HashMap<_, _>>();
            let refined_ty = context.normalize_key(&substitute_key(
                &refined_info.type_key,
                &refined_substitutions,
            ));

            Some(ImplicitRefinedInfixSpecResolution {
                base_signature: base_signature.clone(),
                base_actuals: base_actuals.clone(),
                refined_type_signature: refined_type_signature.clone(),
                refined_type_actuals,
                base_fact: base_fact.clone(),
                refined_type_fact: TypeFact::RefinedIs {
                    subject: subject.clone(),
                    ty: refined_ty,
                    signature: refined_type_signature,
                    base_ty,
                    base_signature: base_type_signature,
                },
            })
        })
        .collect()
}

fn split_refined_infix_spec_signature(signature: &str) -> Option<(String, String)> {
    let body = signature.strip_prefix("\\:")?.strip_suffix(":/")?;
    let segments = split_refined_key(&format!("\\{body}"))?;
    let (base, refinements) = segments.split_last()?;
    if refinements.is_empty() {
        return None;
    }
    Some((refinements.join("::"), format!("\\:{base}:/")))
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

        if rule.source_requires_literal
            && !collection_is_literal_or_has_body(target, context, registry)
        {
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
            SpecOperatorAliasTarget::PlaceholderSpec(target_alias) => {
                // `x_ "in" A :-> x_ "in" B`: the conclusion is a spec on the bound
                // placeholder. Substitution then rewrites the placeholder to the
                // triggering fact's subject (and the left target to its target).
                let Some(subject) = placeholder_pattern_name(&target_alias.placeholder_form) else {
                    continue;
                };
                let next = TypeFact::Spec {
                    subject,
                    operator: target_alias.operator.clone(),
                    target: target_alias.name.clone(),
                };
                let next = substitute_fact(&next, &substitutions);
                let next = context.normalize_fact(&next);
                result.push(next.clone());
                if matches!(next, TypeFact::Spec { .. } | TypeFact::MemberOf { .. }) {
                    result.extend(reduce_spec_or_member_fact(&next, context, registry, seen));
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

    // Membership in a use of a set-defining command (`y "in" \set:where{? is \real}`,
    // `y "in" \set:of{\real}`): instantiate the stored body with the call's actual
    // arguments and derive the member's element facts. Gated on a registered body,
    // so abstract set variables and body-less collection types stay opaque below.
    let body_facts = facts_from_collection_body_membership(subject, collection, context, registry);
    if !body_facts.is_empty() {
        return body_facts;
    }

    if collection_has_registered_collection_type(collection, context, registry) {
        return vec![opaque_type_fact(subject)];
    }

    Vec::new()
}

fn facts_from_collection_body_membership(
    subject: &str,
    collection: &str,
    context: &TypeContext,
    registry: &SignatureRegistry,
) -> Vec<TypeFact> {
    let Some((signature, actuals)) =
        collection_body_signature_and_actuals(collection, context, registry)
    else {
        return Vec::new();
    };
    let Some(body) = registry.collection_bodies.get(&signature) else {
        return Vec::new();
    };
    let Some(info) = registry.type_infos.get(&signature) else {
        return Vec::new();
    };

    let actual_arg_groups = actual_arg_groups_for_key(collection);
    let (key_substitutions, _) =
        command_parameter_substitutions(info, &actuals, actual_arg_groups.as_deref(), context);
    let mut substitutions = HashMap::new();
    for (parameter, actual) in key_substitutions {
        if let Ok(expression) = crate::frontend::formulation::parse_expression(&actual) {
            substitutions.insert(parameter, expression);
        }
    }

    let instantiated = substitute_set_expression(body, &substitutions);
    facts_from_collection_literal_membership(subject, &instantiated, context)
}

fn collection_is_literal_or_has_body(
    collection: &str,
    context: &TypeContext,
    registry: &SignatureRegistry,
) -> bool {
    if context.collection_literal(collection).is_some() {
        return true;
    }
    collection_body_signature_and_actuals(collection, context, registry).is_some()
}

fn collection_body_signature_and_actuals(
    collection: &str,
    context: &TypeContext,
    registry: &SignatureRegistry,
) -> Option<(String, Vec<String>)> {
    let collection = context.normalize_key(collection);
    // Try infix first: the ordinary command-key parser can otherwise consume an
    // entire infix expression as though it were one command signature.
    infix_command_signature_and_actuals_from_key(&collection)
        .into_iter()
        .chain(command_signature_and_actuals_from_key(&collection))
        .find(|(signature, _)| registry.collection_bodies.contains_key(signature))
}

fn facts_from_collection_literal_membership(
    subject: &str,
    literal: &SetExpression,
    context: &TypeContext,
) -> Vec<TypeFact> {
    // Bind a produced tuple/function pattern component-wise. For example,
    // `(n, m) "in" { (a_, b_) : a_ "in" A; b_ "in" B }` establishes
    // `n "in" A` and `m "in" B`, not merely a fact about the tuple as a whole.
    let mut substitutions = HashMap::new();
    if !bind_set_target_to_key(&literal.target, subject, &mut substitutions, context) {
        return Vec::new();
    }

    let mut child = context.clone();
    declare_set_target(&literal.target, &mut child);

    let mut result = Vec::new();
    for spec in &literal.specs {
        let Some(fact) = fact_from_expression_in_context(spec, &child) else {
            continue;
        };
        child.add_fact(fact.clone());
        if substitutions
            .keys()
            .any(|name| key_mentions_name(fact_subject(&fact), name))
        {
            result.push(child.normalize_fact(&substitute_fact(&fact, &substitutions)));
        }
    }
    result
}

fn bind_set_target_to_key(
    target: &SetTarget,
    actual: &str,
    substitutions: &mut HashMap<String, String>,
    context: &TypeContext,
) -> bool {
    match &target.kind {
        SetTargetKind::Name(name) => bind_cast_name_to_key(name, actual, substitutions, context),
        SetTargetKind::PlaceholderForm(form) => match &form.kind {
            PlaceholderFormKind::Placeholder(placeholder) => {
                bind_cast_name_to_key(&placeholder.name, actual, substitutions, context)
            }
            PlaceholderFormKind::Function {
                placeholder,
                arguments,
            } => {
                let Some((actual_name, actual_arguments)) = function_call_parts_from_key(actual)
                else {
                    return false;
                };
                arguments.len() == actual_arguments.len()
                    && bind_cast_name_to_key(
                        &placeholder.name,
                        &actual_name,
                        substitutions,
                        context,
                    )
                    && arguments
                        .iter()
                        .zip(actual_arguments)
                        .all(|(argument, actual_argument)| {
                            bind_cast_name_to_key(
                                &argument.name,
                                &actual_argument,
                                substitutions,
                                context,
                            )
                        })
            }
        },
        SetTargetKind::Expression { expression, .. } => bind_cast_name_to_key(
            &key_for_expression(expression),
            actual,
            substitutions,
            context,
        ),
        SetTargetKind::Alias { name, target } | SetTargetKind::Introduction { name, target } => {
            bind_cast_name_to_key(name, actual, substitutions, context)
                && bind_set_target_to_key(target, actual, substitutions, context)
        }
        SetTargetKind::Function { name, arguments } => {
            let Some((actual_name, actual_arguments)) = function_call_parts_from_key(actual) else {
                return false;
            };
            arguments.len() == actual_arguments.len()
                && bind_cast_name_to_key(name, &actual_name, substitutions, context)
                && arguments
                    .iter()
                    .zip(actual_arguments)
                    .all(|(argument, actual_argument)| {
                        bind_set_target_to_key(argument, &actual_argument, substitutions, context)
                    })
        }
        SetTargetKind::Tuple(elements) => {
            let Some(actual_arguments) = tuple_arguments_from_key(actual) else {
                return false;
            };
            elements.len() == actual_arguments.len()
                && elements
                    .iter()
                    .zip(actual_arguments)
                    .all(|(element, actual_argument)| match element {
                        SetTargetElement::Target(target) => {
                            bind_set_target_to_key(target, &actual_argument, substitutions, context)
                        }
                        SetTargetElement::Operator(operator) => {
                            context.normalize_key(&operator.text)
                                == context.normalize_key(&actual_argument)
                        }
                    })
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

    let actual_arg_groups = actual_arg_groups_for_key(key);
    let (mut base_substitutions, variadic_actuals) =
        command_parameter_substitutions(info, actuals, actual_arg_groups.as_deref(), context);
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
        .flat_map(|output| {
            let mut substitutions = base_substitutions.clone();
            substitutions.insert(fact_subject(output).to_owned(), key.to_owned());
            instantiate_variadic_fact(
                output,
                &substitutions,
                &info.variadic_parameters,
                &variadic_actuals,
            )
            .into_iter()
            .map(|fact| output_context.normalize_fact(&fact))
        })
        .collect()
}

fn actual_arg_groups_for_key(key: &str) -> Option<Vec<ArgGroupShape>> {
    let expression = crate::frontend::formulation::parse_expression(key).ok()?;
    match expression.kind {
        ExpressionKind::Command(command) => Some(shape_for_command_expression(&command).arg_groups),
        ExpressionKind::InfixCommand { command, .. } => {
            Some(shape_for_infix_command(&command).arg_groups)
        }
        ExpressionKind::InfixSpecStatement { spec, .. } => {
            Some(shape_for_infix_spec(&spec).arg_groups)
        }
        _ => None,
    }
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

/// Returns the concrete type arguments through which `subject` has `signature`.
/// This follows the same requirement/extension/refinement paths used by
/// [`has_type_signature`], but retains the matching type's arguments so a
/// provided member can instantiate references to its owner's type parameters.
fn type_actuals_for_signature(
    subject: &str,
    signature: &str,
    context: &TypeContext,
    registry: &SignatureRegistry,
) -> Option<Vec<String>> {
    let subject = context.normalize_key(subject);
    let mut seen = HashSet::new();
    context
        .facts
        .iter()
        .find_map(|fact| {
            fact_type_actuals_for_signature(fact, &subject, signature, context, registry, &mut seen)
        })
        .or_else(|| {
            defined_output_facts_for_key(&subject, context, registry)
                .iter()
                .find_map(|fact| {
                    fact_type_actuals_for_signature(
                        fact, &subject, signature, context, registry, &mut seen,
                    )
                })
        })
}

fn fact_type_actuals_for_signature(
    fact: &TypeFact,
    subject: &str,
    signature: &str,
    context: &TypeContext,
    registry: &SignatureRegistry,
    seen: &mut HashSet<TypeFact>,
) -> Option<Vec<String>> {
    let fact = context.normalize_fact(fact);
    if !seen.insert(fact.clone()) {
        return None;
    }

    match &fact {
        TypeFact::Is {
            subject: fact_subject,
            ty,
            signature: fact_signature,
        } if fact_subject == subject && fact_signature == signature => {
            return actuals_for_type_key(signature, ty);
        }
        TypeFact::RefinedIs {
            subject: fact_subject,
            ty,
            signature: fact_signature,
            ..
        } if fact_subject == subject && fact_signature == signature => {
            return actuals_for_refined_type_key(signature, ty);
        }
        _ => {}
    }

    command_requirement_facts(&fact, context, registry)
        .iter()
        .find_map(|next| {
            fact_type_actuals_for_signature(next, subject, signature, context, registry, seen)
        })
        .or_else(|| {
            reduce_extension_fact(&fact, context, registry)
                .iter()
                .find_map(|next| {
                    fact_type_actuals_for_signature(
                        next, subject, signature, context, registry, seen,
                    )
                })
        })
        .or_else(|| {
            reduce_refined_fact(&fact, context, registry)
                .iter()
                .find_map(|next| {
                    fact_type_actuals_for_signature(
                        next, subject, signature, context, registry, seen,
                    )
                })
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

    // A value known to be one member of an equivalence class has the type
    // signature of every other member of the class (they are interchangeable, and
    // provide the same capabilities per the `Equivalent:` validation). This lets a
    // provided symbol or spec operator resolve through the class.
    if let TypeFact::Is {
        subject: fact_subject,
        signature: fact_signature,
        ..
    } = &fact
        && fact_subject == subject
        && signatures_are_equivalent(fact_signature, signature, registry)
    {
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
    /// The configured numeric type required by each variadic parameter's
    /// indices. A one-based parameter uses `Specify:positiveInt:is`, while a
    /// zero-based parameter uses `Specify:zeroOrPositiveInt:is`.
    variadic_index_types: HashMap<String, NumericTypeSpecification>,
    active_disambiguations: Vec<DisambiguationKey>,
    defer_unresolved_provided_symbols: bool,
    /// Maps a destructured value (`M` from `M ::= (X, *)`) to its component names
    /// in tuple order, so member access `M.*` can resolve to the `*` component.
    destructured_components: HashMap<String, Vec<String>>,
    /// Maps a `Justification:` entry label (its `[label]` heading, dot-joined) to
    /// its `have:`/`asserting:` group. A labeled specification `(.x.)[:label:]`
    /// whose label is present here is established via the referenced group.
    justifications: Rc<HashMap<String, HaveGroup>>,
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

    fn add_destructured_components(&mut self, subject: String, components: Vec<String>) {
        self.destructured_components.insert(subject, components);
    }

    fn destructured_components_of(&self, subject: &str) -> Option<&Vec<String>> {
        self.destructured_components.get(subject).or_else(|| {
            self.destructured_components
                .get(&self.normalize_key(subject))
        })
    }

    fn declare_name(&mut self, name: impl Into<String>) {
        self.symbols.insert(name.into());
    }

    fn has_name(&self, name: &str) -> bool {
        self.symbols.contains(name) || self.symbols.contains(&unstropped_name(name))
    }

    fn set_justifications(&mut self, justifications: HashMap<String, HaveGroup>) {
        self.justifications = Rc::new(justifications);
    }

    fn justification(&self, label: &str) -> Option<&HaveGroup> {
        self.justifications.get(label)
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
        || is_decimal_literal(name)
        || is_negative_integer_literal(name)
}

fn declare_header_symbols(
    header: &CommandHeader,
    context: &mut TypeContext,
    registry: &SignatureRegistry,
) {
    for form in header_forms(header) {
        declare_form_or_declaration(form, context);
    }
    for variadic in header_variadic_parameters(header) {
        context.declare_name(variadic.name.clone());
        for name in variadic_parameter_auxiliary_names(variadic) {
            context.declare_name(name);
        }
        let start = variadic
            .dimensions
            .as_ref()
            .map(|dimensions| dimensions.row_start)
            .unwrap_or_else(|| {
                if variadic.index.is_none() {
                    1
                } else {
                    variadic.start
                }
            });
        let specification = if start == 0 {
            registry
                .numeric_specifications
                .zero_or_positive_int
                .as_ref()
        } else {
            registry.numeric_specifications.positive_int.as_ref()
        };
        let Some(specification) = specification.cloned() else {
            continue;
        };
        context
            .variadic_index_types
            .insert(variadic.name.clone(), specification.clone());

        let mut index_names = variadic
            .index
            .iter()
            .chain(variadic.length.iter())
            .cloned()
            .collect::<Vec<_>>();
        if let Some(dimensions) = &variadic.dimensions {
            index_names.push(dimensions.row_index.clone());
            index_names.push(dimensions.column_index.clone());
            index_names.extend(dimensions.row_length.iter().cloned());
            index_names.extend(dimensions.column_length.iter().cloned());
        }
        for name in index_names {
            context.add_fact(TypeFact::Is {
                subject: name,
                ty: specification.ty.clone(),
                signature: specification.signature.clone(),
            });
        }
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
        FormOrDeclarationKind::MappingParameter { owner, selector } => {
            check_name(owner, context, path, locator, event_log);
            check_name(selector.name(), context, path, locator, event_log);
        }
        FormOrDeclarationKind::FunctionDeclaration { name, form } => {
            if let Some(name) = name {
                check_name(name, context, path, locator, event_log);
            }
            check_name(&form.name, context, path, locator, event_log);
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
                check_set_target(&form.target, context, path, locator, event_log);
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

fn check_set_target(
    target: &SetTarget,
    context: &TypeContext,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    event_log: &mut EventLog,
) {
    match &target.kind {
        SetTargetKind::Name(name) => check_name(name, context, path, locator, event_log),
        SetTargetKind::PlaceholderForm(form) => {
            check_placeholder_form(form, context, path, locator, event_log)
        }
        // Expression targets are checked after their collection specifications
        // have introduced the local placeholders and their assumptions.
        SetTargetKind::Expression { .. } => {}
        SetTargetKind::Alias { name, target } | SetTargetKind::Introduction { name, target } => {
            check_name(name, context, path, locator, event_log);
            check_set_target(target, context, path, locator, event_log);
        }
        SetTargetKind::Function { name, arguments } => {
            check_name(name, context, path, locator, event_log);
            for argument in arguments {
                check_set_target(argument, context, path, locator, event_log);
            }
        }
        SetTargetKind::Tuple(elements) => {
            for element in elements {
                if let SetTargetElement::Target(target) = element {
                    check_set_target(target, context, path, locator, event_log);
                }
            }
        }
    }
}

fn check_subset_call(
    subset: &SubsetCall,
    context: &TypeContext,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    registry: &SignatureRegistry,
    event_log: &mut EventLog,
) {
    match subset {
        SubsetCall::One { target, first, .. } => {
            check_name(target, context, path, locator, event_log);
            check_name(first, context, path, locator, event_log);
            check_variadic_index_name(target, first, context, path, locator, registry, event_log);
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
            check_variadic_index_name(target, first, context, path, locator, registry, event_log);
            check_variadic_index_name(target, second, context, path, locator, registry, event_log);
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

fn check_variadic_index_name(
    target: &str,
    index: &str,
    context: &TypeContext,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    registry: &SignatureRegistry,
    event_log: &mut EventLog,
) {
    let expression = Expression::new(Span::default(), ExpressionKind::Name(index.to_owned()));
    check_variadic_index_expression(
        target,
        &expression,
        context,
        path,
        locator,
        registry,
        event_log,
    );
}

fn check_variadic_index_expression(
    target: &str,
    index: &Expression,
    context: &TypeContext,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    registry: &SignatureRegistry,
    event_log: &mut EventLog,
) {
    let Some(specification) = context.variadic_index_types.get(target) else {
        return;
    };
    let child = context_with_expression_result_facts(index, context, registry);
    let subject = effective_key_for_expression(index, &child, registry);
    let required = TypeFact::Is {
        subject: subject.clone(),
        ty: specification.ty.clone(),
        signature: specification.signature.clone(),
    };
    if prove_fact(&required, &child, registry) {
        return;
    }
    emit_error(
        event_log,
        path,
        locator.locate_symbol(target),
        format!(
            "Could not establish index requirement `{subject} is {}` for variadic parameter `{target}`",
            specification.ty
        ),
    );
}

fn assume_fact_expression(
    expression: &Expression,
    context: &mut TypeContext,
    path: &Path,
    locator: &mut SourceLocator<'_>,
    registry: &SignatureRegistry,
    event_log: &mut EventLog,
) {
    // An assumed clause — a `when:` or `given:` line — is never checked as an
    // expression, so it needs its own hook to reach a type-info pass.
    record_line_types(expression, context, registry);
    match &expression.kind {
        ExpressionKind::IsType { subject, ty } => {
            check_type_expression(ty, context, path, locator, registry, event_log);
            declare_names_from_expression(subject, context);
        }
        ExpressionKind::Build { value, ty, hard } => {
            check_type_expression(ty, context, path, locator, registry, event_log);
            declare_names_from_expression(value, context);
            register_expression_collection_literal(value, context);
            check_build_expression(
                value, ty, *hard, context, path, locator, registry, event_log,
            );
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
        // An operator subject (`x_ * y_`) introduces the operator symbol (`*`) as
        // a named value, so that a use like `a * b` resolves as the application
        // `*(a, b)` rather than an unresolved built-in operator.
        IsSubjectKind::Operator(operator) => context.declare_name(operator.text.clone()),
    }
}

fn declare_form_or_declaration(form: &FormOrDeclaration, context: &mut TypeContext) {
    match &form.kind {
        FormOrDeclarationKind::Name(name) => context.declare_name(name.clone()),
        FormOrDeclarationKind::MappingParameter { selector, .. } => {
            context.declare_name(selector.name().to_owned());
        }
        FormOrDeclarationKind::FunctionDeclaration { name, form } => {
            if let Some(name) = name {
                context.declare_name(name.clone());
                context.add_substitution(name.clone(), form.name.clone());
            }
            context.declare_name(form.name.clone());
            if let Some(placeholder) = &form.magnetic_placeholder {
                context.declare_name(placeholder.name.clone());
            }
            for placeholder in &form.placeholders {
                context.declare_name(placeholder.name.clone());
            }
            if let Some(parameter) = &form.variadic_parameter {
                context.declare_name(parameter.name.clone());
                context.declare_name(parameter.index.clone());
                context.declare_name(parameter.length.clone());
            }
        }
        FormOrDeclarationKind::TupleDeclaration { name, form } => {
            if let Some(name) = name {
                context.declare_name(name.clone());
            }
            for element in &form.elements {
                match element {
                    TupleFormElement::Form(form) => declare_form_or_declaration(form, context),
                    // An operator component (e.g. the `*` in `M ::= (X, *)`) is a
                    // named symbol too, so that `x * y` can resolve as `*(x, y)`.
                    TupleFormElement::Operator(operator) => {
                        context.declare_name(operator.text.clone());
                    }
                }
            }
        }
        FormOrDeclarationKind::SetDeclaration { name, form } => {
            if let Some(name) = name {
                context.declare_name(name.clone());
            }
            declare_set_target(&form.target, context);
        }
        // An operator form (`x_ * y_`, `neg| x_`, `x_ !`) introduces both its
        // placeholders and the operator symbol itself as named values, so a use
        // like `a * b` resolves as the application `*(a, b)`.
        FormOrDeclarationKind::InfixOperator {
            left,
            operator,
            right,
        } => {
            context.declare_name(left.name.clone());
            context.declare_name(operator.text.clone());
            context.declare_name(right.name.clone());
        }
        FormOrDeclarationKind::PrefixOperator {
            operator,
            placeholder,
        }
        | FormOrDeclarationKind::PostfixOperator {
            placeholder,
            operator,
        } => {
            context.declare_name(operator.text.clone());
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
        SetTargetKind::Expression { placeholders, .. } => {
            for name in placeholders {
                context.declare_name(name.clone());
            }
        }
        SetTargetKind::Alias { name, target } | SetTargetKind::Introduction { name, target } => {
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
        ExpressionKind::Name(name) | ExpressionKind::InferredName(name) => {
            context.declare_name(name.clone())
        }
        ExpressionKind::VariadicSlice(slice) => {
            context.declare_name(slice.name.clone());
            for name in variadic_slice_referenced_names(slice) {
                context.declare_name(name);
            }
        }
        ExpressionKind::VariadicAssignment { target, value } => {
            context.declare_name(target.name.clone());
            for name in variadic_slice_referenced_names(target) {
                context.declare_name(name);
            }
            declare_names_from_expression(value, context);
        }
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
                declare_names_from_set_predicate(predicate, context);
            }
        }
        ExpressionKind::Grouped { expression, .. }
        | ExpressionKind::Labeled { expression, .. }
        | ExpressionKind::Prefix { expression, .. }
        | ExpressionKind::Postfix { expression, .. } => {
            declare_names_from_expression(expression, context);
        }
        ExpressionKind::SubsetCall(subset) => declare_subset_call_names(subset, context),
        ExpressionKind::IndexedCall(call) => {
            context.declare_name(call.target.clone());
            for index in &call.indices {
                declare_names_from_expression(index, context);
            }
        }
        ExpressionKind::Command(command) => {
            let active_command = active_command_expression(command, context);
            for expression in command_expression_arguments(&active_command) {
                declare_names_from_expression(expression, context);
            }
        }
        ExpressionKind::BuiltinCommand(_) => {}
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
        ExpressionKind::SpecStatementExpr {
            subject, target, ..
        } => {
            declare_names_from_expression(subject, context);
            declare_names_from_expression(target, context);
        }
        ExpressionKind::SpecLiteral(literal) => match &literal.form {
            SpecLiteralForm::Is(ty) => declare_names_from_type_expression(ty, context),
            SpecLiteralForm::Spec { target, .. } => declare_names_from_expression(target, context),
        },
        ExpressionKind::Satisfies { subject, spec } => {
            declare_names_from_expression(subject, context);
            declare_names_from_expression(spec, context);
        }
        ExpressionKind::Mapping { lhs, rhs } => {
            declare_names_from_expression(lhs, context);
            declare_names_from_expression(rhs, context);
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
        ExpressionKind::Build { ty, value, .. } => {
            declare_names_from_type_expression(ty, context);
            declare_names_from_expression(value, context);
        }
    }
}

fn declare_names_from_type_expression(ty: &TypeExpression, context: &mut TypeContext) {
    match ty {
        TypeExpression::Builtin { .. } | TypeExpression::Parameter { .. } => {}
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
        TypeExpression::Tuple(tuple) => {
            for spec in &tuple.elements {
                declare_names_from_function_type_spec(spec, context);
            }
        }
        TypeExpression::Set(set) => match &set.element {
            SetTypeElement::Spec(spec) => declare_names_from_function_type_spec(spec, context),
            SetTypeElement::Tuple(tuple) => {
                for spec in &tuple.elements {
                    declare_names_from_function_type_spec(spec, context);
                }
            }
        },
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

fn declare_names_from_set_predicate(predicate: &SetPredicate, context: &mut TypeContext) {
    match predicate {
        SetPredicate::Expression(expression) => declare_names_from_expression(expression, context),
        SetPredicate::Definition { target, value, .. } => {
            declare_set_target(target, context);
            declare_names_from_expression(value, context);
        }
    }
}

fn declare_names_from_function_type_spec(spec: &FunctionTypeSpec, context: &mut TypeContext) {
    match &spec.kind {
        FunctionTypeSpecKind::Is(ty) => declare_names_from_type_expression(ty, context),
        FunctionTypeSpecKind::Spec { target, .. } => declare_names_from_expression(target, context),
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

/// Rewrites an expression, replacing each `Name`/`InferredName` leaf (and each
/// bare-name type `TypeExpression::Parameter`) whose name is a key of
/// `substitutions` with the mapped expression. Used to instantiate a command's
/// set-definition body with its actual arguments at a membership use site.
fn substitute_expression(
    expression: &Expression,
    substitutions: &HashMap<String, Expression>,
) -> Expression {
    let sub = |expr: &Expression| substitute_expression(expr, substitutions);
    let boxed = |expr: &Expression| Box::new(substitute_expression(expr, substitutions));
    let kind = match &expression.kind {
        ExpressionKind::Name(name) | ExpressionKind::InferredName(name) => {
            if let Some(replacement) = substitutions.get(name) {
                return replacement.clone();
            }
            expression.kind.clone()
        }
        ExpressionKind::FunctionCall { name, arguments } => match substitutions.get(name) {
            // A replaced callee only makes sense if it is itself a name.
            Some(Expression {
                kind: ExpressionKind::Name(new_name),
                ..
            }) => ExpressionKind::FunctionCall {
                name: new_name.clone(),
                arguments: arguments.iter().map(sub).collect(),
            },
            _ => ExpressionKind::FunctionCall {
                name: name.clone(),
                arguments: arguments.iter().map(sub).collect(),
            },
        },
        ExpressionKind::MemberCall {
            owner,
            name,
            arguments,
        } => ExpressionKind::MemberCall {
            owner: boxed(owner),
            name: name.clone(),
            arguments: arguments.iter().map(sub).collect(),
        },
        ExpressionKind::MemberAccess { owner, name } => ExpressionKind::MemberAccess {
            owner: boxed(owner),
            name: name.clone(),
        },
        ExpressionKind::Tuple(elements) => ExpressionKind::Tuple(
            elements
                .iter()
                .map(|element| match element {
                    TupleExpressionElement::Expression(expr) => {
                        TupleExpressionElement::Expression(sub(expr))
                    }
                    other => other.clone(),
                })
                .collect(),
        ),
        ExpressionKind::Set(set) => {
            ExpressionKind::Set(substitute_set_expression(set, substitutions))
        }
        ExpressionKind::Grouped {
            expression,
            dot_delimited,
        } => ExpressionKind::Grouped {
            expression: boxed(expression),
            dot_delimited: *dot_delimited,
        },
        ExpressionKind::Labeled { expression, label } => ExpressionKind::Labeled {
            expression: boxed(expression),
            label: label.clone(),
        },
        ExpressionKind::Command(command) => {
            ExpressionKind::Command(substitute_command_expression(command, substitutions))
        }
        ExpressionKind::InfixCommand {
            left,
            command,
            right,
        } => ExpressionKind::InfixCommand {
            left: boxed(left),
            command: command.clone(),
            right: boxed(right),
        },
        ExpressionKind::Prefix {
            operator,
            expression,
        } => ExpressionKind::Prefix {
            operator: operator.clone(),
            expression: boxed(expression),
        },
        ExpressionKind::Postfix {
            expression,
            operator,
        } => ExpressionKind::Postfix {
            expression: boxed(expression),
            operator: operator.clone(),
        },
        ExpressionKind::Binary {
            left,
            operator,
            right,
        } => ExpressionKind::Binary {
            left: boxed(left),
            operator: operator.clone(),
            right: boxed(right),
        },
        ExpressionKind::SpecStatement(statement) => {
            // A collection-body parameter can be instantiated by a compound
            // collection expression, which requires the expression-target form.
            if let Some(target) = substitutions.get(&statement.name) {
                ExpressionKind::SpecStatementExpr {
                    subject: boxed(&statement.subject),
                    operator: statement.operator.clone(),
                    target: Box::new(target.clone()),
                }
            } else {
                ExpressionKind::SpecStatement(SpecStatement {
                    span: statement.span,
                    subject: boxed(&statement.subject),
                    operator: statement.operator.clone(),
                    name: statement.name.clone(),
                })
            }
        }
        ExpressionKind::SpecStatementExpr {
            subject,
            operator,
            target,
        } => ExpressionKind::SpecStatementExpr {
            subject: boxed(subject),
            operator: operator.clone(),
            target: boxed(target),
        },
        ExpressionKind::SpecLiteral(literal) => ExpressionKind::SpecLiteral(SpecLiteral {
            span: literal.span,
            form: match &literal.form {
                SpecLiteralForm::Is(ty) => {
                    SpecLiteralForm::Is(substitute_type_expression(ty, substitutions))
                }
                SpecLiteralForm::Spec { operator, target } => SpecLiteralForm::Spec {
                    operator: operator.clone(),
                    target: boxed(target),
                },
            },
        }),
        ExpressionKind::Satisfies { subject, spec } => ExpressionKind::Satisfies {
            subject: boxed(subject),
            spec: boxed(spec),
        },
        ExpressionKind::Mapping { lhs, rhs } => ExpressionKind::Mapping {
            lhs: boxed(lhs),
            rhs: boxed(rhs),
        },
        ExpressionKind::IsType { subject, ty } => ExpressionKind::IsType {
            subject: boxed(subject),
            ty: substitute_type_expression(ty, substitutions),
        },
        ExpressionKind::IsBuiltinPredicate { subject, ty } => ExpressionKind::IsBuiltinPredicate {
            subject: boxed(subject),
            ty: substitute_type_expression(ty, substitutions),
        },
        ExpressionKind::IsNotBuiltinPredicate { subject, ty } => {
            ExpressionKind::IsNotBuiltinPredicate {
                subject: boxed(subject),
                ty: substitute_type_expression(ty, substitutions),
            }
        }
        ExpressionKind::Build { value, ty, hard } => ExpressionKind::Build {
            value: boxed(value),
            ty: substitute_type_expression(ty, substitutions),
            hard: *hard,
        },
        ExpressionKind::MemberOf {
            subject,
            collection,
        } => ExpressionKind::MemberOf {
            subject: boxed(subject),
            collection: boxed(collection),
        },
        // Predicate/command-heavy and leaf nodes not expected to carry substitutable
        // parameters inside a set-definition body are left unchanged.
        other => other.clone(),
    };
    Expression::new(expression.span, kind)
}

fn substitute_set_expression(
    set: &SetExpression,
    substitutions: &HashMap<String, Expression>,
) -> SetExpression {
    let target = match &set.target.kind {
        SetTargetKind::Expression {
            expression,
            placeholders,
        } => {
            let mut target_substitutions = substitutions.clone();
            for placeholder in placeholders {
                target_substitutions.remove(placeholder);
            }
            SetTarget::new(
                set.target.span,
                SetTargetKind::Expression {
                    expression: Box::new(substitute_expression(expression, &target_substitutions)),
                    placeholders: placeholders.clone(),
                },
            )
        }
        _ => set.target.clone(),
    };
    SetExpression {
        span: set.span,
        target,
        specs: set
            .specs
            .iter()
            .map(|spec| substitute_expression(spec, substitutions))
            .collect(),
        predicate: set.predicate.as_ref().map(|predicate| match predicate {
            SetPredicate::Expression(expr) => {
                SetPredicate::Expression(Box::new(substitute_expression(expr, substitutions)))
            }
            other => other.clone(),
        }),
    }
}

fn substitute_command_expression(
    command: &CommandExpression,
    substitutions: &HashMap<String, Expression>,
) -> CommandExpression {
    let mut result = command.clone();
    for group in &mut result.head_args {
        for expr in &mut group.expressions {
            *expr = substitute_expression(expr, substitutions);
        }
    }
    for part in &mut result.tail {
        for group in &mut part.args {
            for expr in &mut group.expressions {
                *expr = substitute_expression(expr, substitutions);
            }
        }
    }
    for group in &mut result.paren_args {
        for expr in &mut group.expressions {
            *expr = substitute_expression(expr, substitutions);
        }
    }
    result
}

/// Substitutes into a type position. A bare-name type `Parameter(T)` whose name is
/// mapped is replaced by the actual argument reinterpreted as a type (a command
/// like `\real` becomes `TypeExpression::Command`).
fn substitute_type_expression(
    ty: &TypeExpression,
    substitutions: &HashMap<String, Expression>,
) -> TypeExpression {
    match ty {
        TypeExpression::Parameter { span, name } => match substitutions.get(name) {
            Some(replacement) => expression_as_type_expression(replacement).unwrap_or_else(|| {
                TypeExpression::Parameter {
                    span: *span,
                    name: name.clone(),
                }
            }),
            None => ty.clone(),
        },
        TypeExpression::Command(command) => {
            TypeExpression::Command(substitute_command_expression(command, substitutions))
        }
        TypeExpression::Tuple(tuple) => TypeExpression::Tuple(TupleType {
            span: tuple.span,
            elements: tuple
                .elements
                .iter()
                .map(|spec| substitute_function_type_spec_expression(spec, substitutions))
                .collect(),
        }),
        TypeExpression::Set(set) => TypeExpression::Set(SetType {
            span: set.span,
            element: match &set.element {
                SetTypeElement::Spec(spec) => SetTypeElement::Spec(
                    substitute_function_type_spec_expression(spec, substitutions),
                ),
                SetTypeElement::Tuple(tuple) => SetTypeElement::Tuple(TupleType {
                    span: tuple.span,
                    elements: tuple
                        .elements
                        .iter()
                        .map(|spec| substitute_function_type_spec_expression(spec, substitutions))
                        .collect(),
                }),
            },
        }),
        TypeExpression::Function(function) => TypeExpression::Function(FunctionType {
            span: function.span,
            inputs: function
                .inputs
                .iter()
                .map(|spec| substitute_function_type_spec_expression(spec, substitutions))
                .collect(),
            output: substitute_function_type_spec_expression(&function.output, substitutions),
            notation: function.notation,
        }),
        other => other.clone(),
    }
}

fn substitute_function_type_spec_expression(
    spec: &FunctionTypeSpec,
    substitutions: &HashMap<String, Expression>,
) -> FunctionTypeSpec {
    let mut result = spec.clone();
    result.kind = match &spec.kind {
        FunctionTypeSpecKind::Is(ty) => {
            FunctionTypeSpecKind::Is(Box::new(substitute_type_expression(ty, substitutions)))
        }
        FunctionTypeSpecKind::Spec { operator, target } => FunctionTypeSpecKind::Spec {
            operator: operator.clone(),
            target: Box::new(substitute_expression(target, substitutions)),
        },
    };
    result
}

/// Reinterprets an expression used as a type (e.g. the `\real` argument bound to a
/// `\\type` parameter) as a `TypeExpression`.
fn expression_as_type_expression(expression: &Expression) -> Option<TypeExpression> {
    match &expression.kind {
        ExpressionKind::Command(command) => Some(TypeExpression::Command(command.clone())),
        ExpressionKind::Name(name) | ExpressionKind::InferredName(name) => {
            Some(TypeExpression::Parameter {
                span: expression.span,
                name: name.clone(),
            })
        }
        ExpressionKind::Grouped { expression, .. } | ExpressionKind::Labeled { expression, .. } => {
            expression_as_type_expression(expression)
        }
        _ => None,
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
                && replacement.is_none_or(|(length, _)| name.len() > length)
            {
                replacement = Some((name.len(), value.as_str()));
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

fn facts_from_is_or_via_item_in_context(
    item: &IsOrViaItem,
    context: &TypeContext,
) -> Vec<TypeFact> {
    match item {
        IsOrViaItem::IsVia(statement) => facts_from_is_statement(&statement.is_statement),
        IsOrViaItem::Declaration(statement) => {
            facts_from_declaration_statement_in_context(statement, context)
        }
        IsOrViaItem::Have(group) => have_group_declarations(group)
            .flat_map(|statement| facts_from_declaration_statement_in_context(statement, context))
            .collect(),
        IsOrViaItem::Labeled { item, .. } => facts_from_is_or_via_item_in_context(item, context),
    }
}

/// The declaration statements a `have:` group's `have:` clauses stand for — the
/// specification the group provides. Non-declaration `have:` clauses (bare
/// statements or expressions) contribute no typing facts.
fn have_group_declarations(group: &HaveGroup) -> impl Iterator<Item = &DeclarationStatement> {
    group
        .have
        .arguments
        .iter()
        .filter_map(|clause| match clause {
            Clause::Declaration(statement) => Some(statement),
            _ => None,
        })
}

#[derive(Clone, Debug)]
struct DescribedFunctionTarget {
    subject: String,
    inputs: Vec<String>,
    output: String,
    variadic_tuple_input: bool,
}

fn function_type_fact_from_defines_means(
    target: &DefinesTarget,
    means: &DefinesMeansSection,
    context: &TypeContext,
) -> Option<TypeFact> {
    let target = described_function_target(target)?;
    for item in &means.arguments {
        for fact in facts_from_is_or_via_item_in_context(item, context) {
            if let TypeFact::FunctionType {
                subject,
                inputs,
                output,
                ..
            } = fact
                && context.normalize_key(&subject) == context.normalize_key(&target.subject)
                && inputs.len() == target.inputs.len()
            {
                return Some(TypeFact::FunctionType {
                    subject: target.subject,
                    inputs,
                    output,
                    variadic_tuple_input: target.variadic_tuple_input,
                });
            }
        }
    }

    let specs = function_type_specs_from_defines_means(means, context);
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

fn described_function_target(target: &DefinesTarget) -> Option<DescribedFunctionTarget> {
    let DefinesTarget::Declaration(statement) = target else {
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

fn function_type_specs_from_defines_means(
    means: &DefinesMeansSection,
    context: &TypeContext,
) -> HashMap<String, FunctionTypeFactSpec> {
    let mut specs = HashMap::new();
    for item in &means.arguments {
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
        return facts_from_declaration_cast_definition(statement);
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
        return facts_from_declaration_cast_definition_in_context(statement, context);
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

fn facts_from_declaration_cast_definition(statement: &DeclarationStatement) -> Vec<TypeFact> {
    let Some(Expression {
        kind: ExpressionKind::Build { ty, .. },
        ..
    }) = &statement.definition
    else {
        return Vec::new();
    };
    declaration_subject_keys(statement)
        .into_iter()
        .filter_map(|subject| fact_from_type_key_assertion_without_context(subject, ty))
        .collect()
}

fn facts_from_declaration_cast_definition_in_context(
    statement: &DeclarationStatement,
    context: &TypeContext,
) -> Vec<TypeFact> {
    let Some(Expression {
        kind: ExpressionKind::Build { ty, .. },
        ..
    }) = &statement.definition
    else {
        return Vec::new();
    };
    declaration_subject_keys(statement)
        .into_iter()
        .filter_map(|subject| fact_from_type_key_assertion(subject, ty, context))
        .collect()
}

fn facts_from_declaration_is(
    statement: &DeclarationStatement,
    ty: &TypeExpression,
) -> Vec<TypeFact> {
    if let Some(facts) = literal_type_facts_from_is_subject(&statement.subject, ty) {
        return facts;
    }

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
    if let Some(facts) = literal_type_facts_from_is_subject(&statement.subject, ty) {
        return facts
            .iter()
            .map(|fact| context.normalize_fact(fact))
            .collect();
    }

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
    if let Some(facts) = literal_type_facts_from_is_subject(&statement.subject, &statement.ty) {
        return facts;
    }

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

#[derive(Clone, Debug)]
enum LiteralTypeSubject {
    Leaf(String),
    Tuple(Vec<LiteralTypeSubject>),
    Set(Box<LiteralTypeSubject>),
}

fn literal_type_facts_from_is_subject(
    subject: &IsSubject,
    ty: &TypeExpression,
) -> Option<Vec<TypeFact>> {
    if !matches!(ty, TypeExpression::Tuple(_) | TypeExpression::Set(_)) {
        return None;
    }
    let pattern = literal_type_subject(subject)?;
    literal_type_facts(&pattern, ty)
}

fn literal_type_subject(subject: &IsSubject) -> Option<LiteralTypeSubject> {
    match &subject.kind {
        IsSubjectKind::Forms(forms) if forms.len() == 1 => match &forms[0] {
            IsSubjectForm::Form(form) => Some(literal_type_subject_from_form(form)),
            IsSubjectForm::PlaceholderForm(form) => {
                Some(LiteralTypeSubject::Leaf(key_for_placeholder_form(form)))
            }
        },
        IsSubjectKind::Operator(operator) => Some(LiteralTypeSubject::Leaf(operator.text.clone())),
        _ => None,
    }
}

fn literal_type_subject_from_form(form: &FormOrDeclaration) -> LiteralTypeSubject {
    match &form.kind {
        FormOrDeclarationKind::TupleDeclaration { form, .. } => LiteralTypeSubject::Tuple(
            form.elements
                .iter()
                .map(|element| match element {
                    TupleFormElement::Form(form) => literal_type_subject_from_form(form),
                    TupleFormElement::Operator(operator) => {
                        LiteralTypeSubject::Leaf(operator.text.clone())
                    }
                })
                .collect(),
        ),
        FormOrDeclarationKind::SetDeclaration { form, .. } => {
            LiteralTypeSubject::Set(Box::new(literal_type_subject_from_set_target(&form.target)))
        }
        _ => LiteralTypeSubject::Leaf(
            primary_form_name(form).unwrap_or_else(|| key_for_form_or_declaration(form)),
        ),
    }
}

fn literal_type_subject_from_set_target(target: &SetTarget) -> LiteralTypeSubject {
    match &target.kind {
        SetTargetKind::Name(name) => LiteralTypeSubject::Leaf(name.clone()),
        SetTargetKind::PlaceholderForm(form) => {
            LiteralTypeSubject::Leaf(key_for_placeholder_form(form))
        }
        SetTargetKind::Expression { .. } => LiteralTypeSubject::Leaf(key_for_set_target(target)),
        SetTargetKind::Alias { target, .. } | SetTargetKind::Introduction { target, .. } => {
            literal_type_subject_from_set_target(target)
        }
        SetTargetKind::Function { .. } => LiteralTypeSubject::Leaf(key_for_set_target(target)),
        SetTargetKind::Tuple(elements) => LiteralTypeSubject::Tuple(
            elements
                .iter()
                .map(|element| match element {
                    SetTargetElement::Target(target) => {
                        literal_type_subject_from_set_target(target)
                    }
                    SetTargetElement::Operator(operator) => {
                        LiteralTypeSubject::Leaf(operator.text.clone())
                    }
                })
                .collect(),
        ),
    }
}

fn literal_type_facts(subject: &LiteralTypeSubject, ty: &TypeExpression) -> Option<Vec<TypeFact>> {
    match (subject, ty) {
        (LiteralTypeSubject::Tuple(subjects), TypeExpression::Tuple(tuple))
            if subjects.len() == tuple.elements.len() =>
        {
            let mut facts = Vec::new();
            for (subject, spec) in subjects.iter().zip(&tuple.elements) {
                facts.extend(literal_type_spec_facts(subject, spec)?);
            }
            Some(facts)
        }
        (LiteralTypeSubject::Set(subject), TypeExpression::Set(set)) => match &set.element {
            SetTypeElement::Spec(spec) => literal_type_spec_facts(subject, spec),
            SetTypeElement::Tuple(tuple) => {
                let LiteralTypeSubject::Tuple(subjects) = subject.as_ref() else {
                    return None;
                };
                if subjects.len() != tuple.elements.len() {
                    return None;
                }
                let mut facts = Vec::new();
                for (subject, spec) in subjects.iter().zip(&tuple.elements) {
                    facts.extend(literal_type_spec_facts(subject, spec)?);
                }
                Some(facts)
            }
        },
        _ => None,
    }
}

fn literal_type_spec_facts(
    subject: &LiteralTypeSubject,
    spec: &FunctionTypeSpec,
) -> Option<Vec<TypeFact>> {
    if let FunctionTypeSpecKind::Is(ty) = &spec.kind
        && matches!(
            ty.as_ref(),
            TypeExpression::Tuple(_) | TypeExpression::Set(_)
        )
    {
        return literal_type_facts(subject, ty);
    }
    let LiteralTypeSubject::Leaf(subject) = subject else {
        return None;
    };
    let spec = function_type_spec_as_fact(spec)?;
    Some(vec![instantiate_function_type_spec(&spec, subject)])
}

fn fact_from_expression(expression: &Expression) -> Option<TypeFact> {
    match &expression.kind {
        ExpressionKind::IsType { subject, ty } => fact_from_type_assertion(subject, ty),
        ExpressionKind::SpecStatement(statement) => Some(TypeFact::Spec {
            subject: key_for_expression(&statement.subject),
            operator: statement.operator.clone(),
            target: statement.name.clone(),
        }),
        ExpressionKind::SpecStatementExpr {
            subject,
            operator,
            target,
        } => Some(TypeFact::Spec {
            subject: key_for_expression(subject),
            operator: operator.clone(),
            target: key_for_expression(target),
        }),
        ExpressionKind::Satisfies { subject, spec } => fact_from_satisfies(subject, spec),
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
        ExpressionKind::Build {
            value: expression,
            ty,
            ..
        } => fact_from_type_assertion(expression, ty),
        _ => None,
    }
}

/// The specification a `satisfies` expression's right-hand side denotes, if it is
/// a concrete spec literal, with its `?` placeholder replaced by `subject`.
/// Returns `None` for an abstract spec (e.g. a `\\specification` parameter), which
/// stays inert until instantiated.
fn fact_from_satisfies(subject: &Expression, spec: &Expression) -> Option<TypeFact> {
    let literal = spec_literal_of(spec)?;
    let base = fact_from_spec_literal(literal)?;
    let substitutions = HashMap::from([("?".to_owned(), key_for_expression(subject))]);
    Some(substitute_fact(&base, &substitutions))
}

/// Peels `Grouped`/`Labeled` wrappers to find a spec literal.
fn spec_literal_of(expression: &Expression) -> Option<&SpecLiteral> {
    match &expression.kind {
        ExpressionKind::SpecLiteral(literal) => Some(literal),
        ExpressionKind::Grouped { expression, .. } | ExpressionKind::Labeled { expression, .. } => {
            spec_literal_of(expression)
        }
        _ => None,
    }
}

/// The fact a spec literal asserts about its `?` placeholder (subject key `"?"`).
fn fact_from_spec_literal(literal: &SpecLiteral) -> Option<TypeFact> {
    match &literal.form {
        SpecLiteralForm::Is(ty) => {
            let (ty_key, signature) = key_for_type_expression(ty)?;
            Some(TypeFact::Is {
                subject: "?".to_owned(),
                ty: ty_key,
                signature,
            })
        }
        SpecLiteralForm::Spec { operator, target } => Some(TypeFact::Spec {
            subject: "?".to_owned(),
            operator: operator.clone(),
            target: key_for_expression(target),
        }),
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
        ExpressionKind::SpecStatementExpr {
            subject,
            operator,
            target,
        } => Some(TypeFact::Spec {
            subject: context.normalize_key(&key_for_expression(subject)),
            operator: operator.clone(),
            target: context.normalize_key(&key_for_expression(target)),
        }),
        ExpressionKind::Satisfies { subject, spec } => {
            fact_from_satisfies(subject, spec).map(|fact| context.normalize_fact(&fact))
        }
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
        ExpressionKind::Build {
            value: expression,
            ty,
            ..
        } => fact_from_type_assertion_in_context(expression, ty, context),
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

fn fact_from_type_key_assertion_without_context(
    subject: String,
    ty: &TypeExpression,
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
        return Some(refined_fact_from_command(subject, command));
    }

    let (ty, signature) = key_for_type_expression(ty)?;
    Some(TypeFact::Is {
        subject,
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
            target: key_for_expression(target),
        }),
    }
}

#[derive(Clone, Debug, Default)]
struct WhenParameters {
    required: HashSet<String>,
    allowed: HashSet<String>,
    /// The symbol the definition describes, when the heading also makes it a
    /// parameter — the left operand of a spec-infix heading. `when:` states what
    /// a *use* of the command requires of its operands, so the described symbol
    /// belongs on the `Defines:` target instead and is rejected here.
    described: Option<String>,
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
            if command.left_placeholder {
                allow_optional_form_when_parameter(command.left.as_ref(), parameters);
            } else {
                require_optional_form_when_parameter(command.left.as_ref(), parameters);
            }
            collect_curly_heading_parameters(&command.head_args, parameters);
            collect_tail_parameters(&command.tail, parameters);
            if command.right_placeholder {
                allow_optional_form_when_parameter(command.right.as_ref(), parameters);
            } else {
                require_optional_form_when_parameter(command.right.as_ref(), parameters);
            }
        }
        CommandHeader::InfixSpec(spec) => {
            require_form_when_parameter(&spec.left, parameters);
            if let Some(refinement) = &spec.refinement {
                for part in &refinement.parts {
                    collect_tail_parameters(&part.tail, parameters);
                }
            }
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
    for group in groups {
        if let Some(variadic) = &group.variadic {
            parameters.require(variadic.name.clone());
            for name in variadic.index.iter().chain(variadic.length.iter()) {
                parameters.allow(name.clone());
            }
            if let Some(dimensions) = &variadic.dimensions {
                for name in [
                    Some(&dimensions.row_index),
                    Some(&dimensions.column_index),
                    dimensions.row_length.as_ref(),
                    dimensions.column_length.as_ref(),
                ]
                .into_iter()
                .flatten()
                {
                    parameters.allow(name.clone());
                }
            }
        }
        for form in &group.forms {
            require_form_when_parameter(form, parameters);
        }
    }
}

fn header_variadic_parameters(header: &CommandHeader) -> Vec<&VariadicParameter> {
    fn collect<'a>(groups: &'a [CurlyHeadingArgs], out: &mut Vec<&'a VariadicParameter>) {
        out.extend(groups.iter().filter_map(|group| group.variadic.as_ref()));
    }
    fn collect_tail<'a>(parts: &'a [CommandHeaderTailPart], out: &mut Vec<&'a VariadicParameter>) {
        for part in parts {
            collect(&part.args, out);
        }
    }

    let mut result = Vec::new();
    match header {
        CommandHeader::Command(command) => {
            collect(&command.head_args, &mut result);
            collect_tail(&command.tail, &mut result);
        }
        CommandHeader::Infix(command) => {
            collect(&command.head_args, &mut result);
            collect_tail(&command.tail, &mut result);
        }
        CommandHeader::InfixSpec(spec) => {
            if let Some(refinement) = &spec.refinement {
                for part in &refinement.parts {
                    collect_tail(&part.tail, &mut result);
                }
            }
            collect(&spec.head_args, &mut result);
            collect_tail(&spec.tail, &mut result);
        }
        CommandHeader::Refined(command) => {
            for part in &command.parts {
                collect_tail(&part.tail, &mut result);
            }
            collect(&command.head_args, &mut result);
            collect_tail(&command.tail, &mut result);
        }
    }
    result
}

fn variadic_parameter_auxiliary_names(parameter: &VariadicParameter) -> Vec<String> {
    let mut names = parameter
        .index
        .iter()
        .chain(parameter.length.iter())
        .cloned()
        .collect::<Vec<_>>();
    if let Some(dimensions) = &parameter.dimensions {
        names.push(dimensions.row_index.clone());
        names.push(dimensions.column_index.clone());
        names.extend(dimensions.row_length.iter().cloned());
        names.extend(dimensions.column_length.iter().cloned());
    }
    names
}

/// Every parameter form appearing in a command header (curly-brace groups, tail
/// parts, and infix left/right operands).
fn header_parameter_forms(header: &CommandHeader) -> Vec<&FormOrDeclaration> {
    let mut forms = Vec::new();
    match header {
        CommandHeader::Command(command) => {
            collect_curly_parameter_forms(&command.head_args, &mut forms);
            collect_tail_parameter_forms(&command.tail, &mut forms);
        }
        CommandHeader::Infix(command) => {
            if let Some(left) = command.left.as_ref() {
                forms.push(left);
            }
            collect_curly_parameter_forms(&command.head_args, &mut forms);
            collect_tail_parameter_forms(&command.tail, &mut forms);
            if let Some(right) = command.right.as_ref() {
                forms.push(right);
            }
        }
        CommandHeader::InfixSpec(spec) => {
            forms.push(&spec.left);
            if let Some(refinement) = &spec.refinement {
                for part in &refinement.parts {
                    collect_tail_parameter_forms(&part.tail, &mut forms);
                }
            }
            collect_curly_parameter_forms(&spec.head_args, &mut forms);
            collect_tail_parameter_forms(&spec.tail, &mut forms);
            forms.push(&spec.right);
        }
        CommandHeader::Refined(command) => {
            for part in &command.parts {
                collect_tail_parameter_forms(&part.tail, &mut forms);
            }
            collect_curly_parameter_forms(&command.head_args, &mut forms);
            collect_tail_parameter_forms(&command.tail, &mut forms);
        }
    }
    forms
}

fn collect_curly_parameter_forms<'a>(
    groups: &'a [CurlyHeadingArgs],
    out: &mut Vec<&'a FormOrDeclaration>,
) {
    for group in groups {
        for form in &group.forms {
            out.push(form);
        }
    }
}

fn collect_tail_parameter_forms<'a>(
    parts: &'a [CommandHeaderTailPart],
    out: &mut Vec<&'a FormOrDeclaration>,
) {
    for part in parts {
        for group in &part.args {
            for form in &group.forms {
                out.push(form);
            }
        }
    }
}

/// The type signature declared for `name` by an `is` fact currently in context,
/// e.g. `\magma` for a parameter `M` with `M is \magma`.
fn declared_type_signature(name: &str, context: &TypeContext) -> Option<String> {
    context.facts.iter().find_map(|fact| match fact {
        TypeFact::Is {
            subject, signature, ..
        } if subject == name => Some(signature.clone()),
        _ => None,
    })
}

/// Binds the components of a destructuring parameter `M ::= (X, *)` by copying
/// the component type facts from `M`'s type (looked up in the registry) with the
/// type's own subject and component names substituted by the local ones. This is
/// what lets `\magma.element:of{M ::= (X, *)}` know `X is \set` and
/// `* is \binary.operation:on{M}` without a separate `when:` entry.
fn assume_destructured_parameter_components(
    header: &CommandHeader,
    context: &mut TypeContext,
    registry: &SignatureRegistry,
) {
    for parameter in destructured_parameters(header, context) {
        bind_destructured_parameter(&parameter, context, registry);
    }
}

/// Records a destructured parameter's components in `context`: declares each
/// component name, adds its type facts (resolved lazily from the parameter's
/// type), and remembers the component list so member access (`M.*`) can reach
/// them.
fn bind_destructured_parameter(
    parameter: &DestructuredParameter,
    context: &mut TypeContext,
    registry: &SignatureRegistry,
) {
    context.declare_name(parameter.name.clone());
    for component in &parameter.components {
        context.declare_name(component.clone());
    }
    for fact in destructured_parameter_component_facts(parameter, context, registry) {
        context.add_fact(fact);
    }
    context.add_destructured_components(parameter.name.clone(), parameter.components.clone());
}

/// Positionally maps the parameter's type's component types onto the local
/// component names (substituting that type's subject by the parameter name), so
/// `{M ::= (X, *)}` with `M is \magma` gives `X is \set`, `* is \binary.operation:on{M}`.
fn destructured_parameter_component_facts(
    parameter: &DestructuredParameter,
    context: &TypeContext,
    registry: &SignatureRegistry,
) -> Vec<TypeFact> {
    let Some(info) = registry.type_infos.get(&parameter.type_signature) else {
        return Vec::new();
    };
    if info.component_types.is_empty() {
        return Vec::new();
    }
    instantiate_component_type_facts(info, &parameter.name, &parameter.components, context)
}

fn instantiate_component_type_facts(
    info: &DefinitionTypeInfo,
    subject: &str,
    component_names: &[String],
    context: &TypeContext,
) -> Vec<TypeFact> {
    let mut substitutions = HashMap::new();
    if let Some(described) = &info.described {
        substitutions.insert(described.clone(), subject.to_owned());
    }
    for (index, fact) in info.component_types.iter().enumerate() {
        if let Some(local) = component_names.get(index) {
            substitutions.insert(fact_subject(fact).to_owned(), local.clone());
        }
    }
    info.component_types
        .iter()
        .map(|fact| context.normalize_fact(&substitute_fact(fact, &substitutions)))
        .collect()
}

/// The destructuring parameters of a header (`{M ::= (X, *)}`), each recorded with
/// its component names and the signature of its declared type. Order-independent:
/// component types are resolved later from that type (see
/// `destructured_parameter_component_facts`).
fn destructured_parameters(
    header: &CommandHeader,
    context: &TypeContext,
) -> Vec<DestructuredParameter> {
    let mut result = Vec::new();
    for form in header_parameter_forms(header) {
        let FormOrDeclarationKind::TupleDeclaration {
            name: Some(name),
            form: tuple,
        } = &form.kind
        else {
            continue;
        };
        let Some(type_signature) = declared_type_signature(name, context) else {
            continue;
        };
        result.push(DestructuredParameter {
            name: name.clone(),
            components: tuple_form_component_names(tuple),
            type_signature,
        });
    }
    result
}

fn collect_tail_parameters(parts: &[CommandHeaderTailPart], parameters: &mut WhenParameters) {
    for part in parts {
        for group in &part.args {
            if let Some(variadic) = &group.variadic {
                if part.optional {
                    parameters.allow(variadic.name.clone());
                } else {
                    parameters.require(variadic.name.clone());
                }
                for name in variadic_parameter_auxiliary_names(variadic) {
                    parameters.allow(name);
                }
            }
        }
    }
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

fn allow_optional_form_when_parameter(
    form: Option<&FormOrDeclaration>,
    parameters: &mut WhenParameters,
) {
    if let Some(form) = form {
        allow_form_when_parameter(form, parameters);
    }
}

fn require_form_when_parameter(form: &FormOrDeclaration, parameters: &mut WhenParameters) {
    // A mapping-parameter selector (`f.x_`, `f.u?_`, or a variadic subset) is
    // bound by the associated mapping form in the header. It remains an allowed
    // `when:` subject for an optional refinement, but it does not need a
    // separate type requirement of its own.
    if matches!(form.kind, FormOrDeclarationKind::MappingParameter { .. }) {
        allow_form_when_parameter(form, parameters);
        return;
    }
    // A named destructuring parameter `M ::= (X, *)` requires only `M`; its
    // components are typed from `M`'s type, so they are allowed (a `when:` entry
    // may still refine them) but not independently required.
    if let FormOrDeclarationKind::TupleDeclaration {
        name: Some(name),
        form: tuple,
    } = &form.kind
    {
        parameters.require(name.clone());
        let mut components = HashSet::new();
        collect_tuple_form_when_parameters(tuple, &mut components);
        for component in components {
            parameters.allow(component);
        }
        return;
    }
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
        TypeExpression::Parameter { name, .. } => Some((name.clone(), name.clone())),
        TypeExpression::RefinedCommand(_)
        | TypeExpression::Tuple(_)
        | TypeExpression::Set(_)
        | TypeExpression::Function(_) => None,
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
        TypeExpression::Parameter { name, .. } => Some((name.clone(), name.clone())),
        TypeExpression::RefinedCommand(_)
        | TypeExpression::Tuple(_)
        | TypeExpression::Set(_)
        | TypeExpression::Function(_) => None,
    }
}

fn key_for_expression(expression: &Expression) -> String {
    match &expression.kind {
        ExpressionKind::Name(name) | ExpressionKind::InferredName(name) => unstropped_name(name),
        ExpressionKind::VariadicSlice(slice) => key_for_variadic_slice(slice),
        ExpressionKind::VariadicAssignment { target, value } => format!(
            "{} := {}",
            key_for_variadic_slice(target),
            key_for_expression(value)
        ),
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
        ExpressionKind::SubsetCall(subset) => key_for_subset_call(subset),
        ExpressionKind::IndexedCall(call) => format!(
            "{}[{}]",
            call.target,
            call.indices
                .iter()
                .map(key_for_expression)
                .collect::<Vec<_>>()
                .join(",")
        ),
        ExpressionKind::Command(command) => key_for_command_expression(command),
        ExpressionKind::BuiltinCommand(command) => key_for_builtin_command_expression(command),
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
        ExpressionKind::SpecStatementExpr {
            subject,
            operator,
            target,
        } => format!(
            "{}\"{}\"{}",
            key_for_expression(subject),
            operator,
            key_for_expression(target)
        ),
        ExpressionKind::SpecLiteral(literal) => match &literal.form {
            SpecLiteralForm::Is(ty) => format!(
                "? is {}",
                key_for_type_expression(ty)
                    .map(|(key, _)| key)
                    .unwrap_or_else(|| key_for_non_command_type_expression(ty))
            ),
            SpecLiteralForm::Spec { operator, target } => {
                format!("?\"{}\"{}", operator, key_for_expression(target))
            }
        },
        ExpressionKind::Satisfies { subject, spec } => format!(
            "{} satisfies {}",
            key_for_expression(subject),
            key_for_expression(spec)
        ),
        ExpressionKind::Mapping { lhs, rhs } => {
            format!("{} => {}", key_for_expression(lhs), key_for_expression(rhs))
        }
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
        ExpressionKind::Build { ty, value, hard } => format_build_expression(ty, value, *hard),
    }
}

fn key_for_variadic_slice(slice: &VariadicSlice) -> String {
    if let Some(dimensions) = &slice.dimensions {
        return format!(
            "{}[{},{}]",
            slice.name,
            key_for_variadic_slice_axis(&dimensions.rows),
            key_for_variadic_slice_axis(&dimensions.columns)
        );
    }
    let Some(start) = slice.start else {
        return format!("{}...", slice.name);
    };
    match (&slice.index, &slice.end) {
        (Some(index), Some(end)) => {
            format!("{}[{start}...{index}...{end}]", slice.name)
        }
        (None, Some(end)) => format!("{}[{start}...{end}]", slice.name),
        _ => format!("{}...", slice.name),
    }
}

fn key_for_variadic_slice_axis(axis: &VariadicSliceAxis) -> String {
    match axis {
        VariadicSliceAxis::All => "...".to_owned(),
        VariadicSliceAxis::Index(index) => index.clone(),
        VariadicSliceAxis::Range { start, index, end } => match index {
            Some(index) => format!("{start}...{index}...{end}"),
            None => format!("{start}...{end}"),
        },
    }
}

fn key_for_subset_call(subset: &SubsetCall) -> String {
    match subset {
        SubsetCall::One { target, first, .. } => format!("{target}[{first}]"),
        SubsetCall::Two {
            target,
            first,
            second,
            ..
        } => format!("{target}[{first},{second}]"),
        SubsetCall::Nested {
            target,
            outer,
            inner_target,
            ..
        } => format!("{target}[{outer}[{inner_target}]]"),
    }
}

fn format_build_expression(ty: &TypeExpression, value: &Expression, hard: bool) -> String {
    format!(
        "{}{}{}",
        key_for_type_expression(ty)
            .map(|(key, _)| key)
            .unwrap_or_else(|| key_for_non_command_type_expression(ty)),
        if hard { "@!" } else { "@" },
        key_for_expression(value)
    )
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
        TypeExpression::Parameter { name, .. } => name.clone(),
        TypeExpression::Builtin { chain, .. } => format!("\\\\{}", format_chain(chain)),
        TypeExpression::Command(command) => key_for_command_expression(command),
        TypeExpression::RefinedCommand(command) => key_for_refined_command_expression(command),
        TypeExpression::Tuple(tuple) => format!(
            "({})",
            tuple
                .elements
                .iter()
                .map(key_for_function_type_spec)
                .collect::<Vec<_>>()
                .join(",")
        ),
        TypeExpression::Set(set) => format!(
            "{{{}:...}}",
            match &set.element {
                SetTypeElement::Spec(spec) => key_for_function_type_spec(spec),
                SetTypeElement::Tuple(tuple) => format!(
                    "({})",
                    tuple
                        .elements
                        .iter()
                        .map(key_for_function_type_spec)
                        .collect::<Vec<_>>()
                        .join(",")
                ),
            }
        ),
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

fn key_for_function_type_spec(spec: &FunctionTypeSpec) -> String {
    function_type_spec_as_fact(spec)
        .as_ref()
        .map(format_function_type_spec)
        .unwrap_or_else(|| "?".to_owned())
}

fn key_for_named_expression_lhs(lhs: &FunctionNamedExpressionElementLhs) -> String {
    format!("{lhs:?}")
}

fn key_for_form_or_declaration(form: &FormOrDeclaration) -> String {
    match &form.kind {
        FormOrDeclarationKind::Name(name) => name.clone(),
        FormOrDeclarationKind::MappingParameter { owner, selector } => {
            format!("{owner}.{}", selector.name())
        }
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
                .chain(
                    form.variadic_parameter
                        .iter()
                        .map(|parameter| parameter.name.clone()),
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
            let set = format!("{{{}}}", key_for_set_target(&form.target));
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
        | FormOrDeclarationKind::MappingParameter { .. }
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
        ExpressionAliasLhs::Form(form) => disambiguation_key_and_parameters(form)
            .map(|(key, parameters)| (normalize_placeholder_operator_key(key), parameters)),
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
        // A member capability `x.f(a_)` is keyed by the member name and its
        // argument arity; the owner (`x`, the subject) is bound separately as the
        // rule's `owner_subject`, so it is not a parameter here.
        ExpressionAliasLhs::Member(member) => {
            let parameters = member
                .arguments
                .iter()
                .map(|argument| argument.name.clone())
                .collect::<Vec<_>>();
            Some((
                DisambiguationKey::Function {
                    name: member.member.clone(),
                    arity: parameters.len(),
                },
                parameters,
            ))
        }
    }
}

fn normalize_placeholder_operator_key(key: DisambiguationKey) -> DisambiguationKey {
    match key {
        DisambiguationKey::BinaryOperator(operator)
            if operator.starts_with('[') && operator.ends_with(']') =>
        {
            DisambiguationKey::BinaryOperator(operator[1..operator.len() - 1].to_owned())
        }
        key => key,
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
        .chain(
            form.variadic_parameter
                .iter()
                .map(|parameter| parameter.name.clone()),
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
        SetTargetKind::Expression { expression, .. } => key_for_expression(expression),
        SetTargetKind::Alias { name, target } => {
            format!("{name}:={}", key_for_set_target(target))
        }
        SetTargetKind::Introduction { name, target } => {
            format!("{name}::={}", key_for_set_target(target))
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
        key.push_str(&key_for_set_predicate(predicate));
    }
    key
}

fn key_for_set_predicate(predicate: &SetPredicate) -> String {
    match predicate {
        SetPredicate::Expression(expression) => key_for_expression(expression),
        SetPredicate::Definition { target, value, .. } => {
            format!(
                "{}:={}",
                key_for_set_target(target),
                key_for_expression(value)
            )
        }
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
    if let Some(context) = &command.context {
        key.push_str(&key_for_command_context(context));
    }
    key
}

fn key_for_command_context(context: &CommandContext) -> String {
    let label = match context.kind {
        CommandContextKind::Using => "#using",
        CommandContextKind::Given => "#given",
    };
    format!(
        "{label}{{{}}}",
        context
            .arguments
            .iter()
            .map(key_for_command_context_argument)
            .collect::<Vec<_>>()
            .join(";")
    )
}

fn key_for_command_context_argument(argument: &CommandContextArgument) -> String {
    match argument {
        CommandContextArgument::Assignment { name, value, .. } => {
            format!("{name}:={}", key_for_expression(value))
        }
        CommandContextArgument::Declaration(statement) => key_for_declaration_statement(statement),
        CommandContextArgument::Expression(expression) => key_for_expression(expression),
        CommandContextArgument::Text(text) => text.clone(),
    }
}

fn key_for_builtin_command_expression(command: &BuiltinCommandExpression) -> String {
    let mut key = format!("\\\\{}", format_chain(&command.chain));
    append_builtin_args(&mut key, &command.head_args);
    for tail in &command.tail {
        key.push(':');
        key.push_str(&format_chain(&tail.chain));
        append_builtin_args(&mut key, &tail.args);
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
    if let Some(refinement) = &spec.refinement {
        let command = RefinedCommandExpression {
            span: spec.span,
            prefix_chain: refinement.prefix_chain.clone(),
            parts: refinement.parts.clone(),
            refined_tail: RefinedTail::Chain(spec.chain.clone()),
            head_args: spec.head_args.clone(),
            tail: spec.tail.clone(),
            paren_args: Vec::new(),
        };
        let refined_key = key_for_refined_command_expression(&command);
        return format!(
            "\\:{}{}",
            refined_key.strip_prefix('\\').unwrap_or(&refined_key),
            if spec.predicate { "?:/" } else { ":/" }
        );
    }

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
        let values = args
            .expressions
            .iter()
            .map(key_for_expression)
            .collect::<Vec<_>>();
        if let Some(rows) = &args.rows {
            let mut offset = 0usize;
            let mut rendered_rows = Vec::new();
            for count in rows {
                let end = (offset + count).min(values.len());
                rendered_rows.push(values[offset..end].join(","));
                offset = end;
            }
            key.push_str(&rendered_rows.join(";"));
        } else {
            key.push_str(&values.join(","));
        }
        key.push('}');
    }
}

fn append_builtin_args(key: &mut String, groups: &[BuiltinCommandArgs]) {
    for args in groups {
        key.push('{');
        key.push_str(
            &args
                .arguments
                .iter()
                .map(key_for_builtin_command_argument)
                .collect::<Vec<_>>()
                .join(";"),
        );
        key.push('}');
    }
}

fn key_for_builtin_command_argument(argument: &BuiltinCommandArgument) -> String {
    match argument {
        BuiltinCommandArgument::Text(text) => text.clone(),
        BuiltinCommandArgument::Expression(expression) => key_for_expression(expression),
        BuiltinCommandArgument::Declaration(statement) => key_for_declaration_statement(statement),
    }
}

fn key_for_declaration_statement(statement: &DeclarationStatement) -> String {
    let mut key = declaration_subject_keys(statement).join(", ");
    if let Some(expansion) = &statement.expansion {
        key.push_str(" ::= ");
        key.push_str(&primary_subject_key(expansion));
    }
    if let Some(definition) = &statement.definition {
        key.push_str(" := ");
        key.push_str(&key_for_expression(definition));
    }
    if let Some(relation) = &statement.relation {
        match relation {
            DeclarationRelation::Is(ty) => {
                key.push_str(" is ");
                key.push_str(
                    &key_for_type_expression(ty)
                        .map(|(key, _)| key)
                        .unwrap_or_else(|| key_for_non_command_type_expression(ty)),
                );
            }
            DeclarationRelation::Spec { operator, target } => {
                key.push_str(&format!(" \"{operator}\" {}", key_for_expression(target)));
            }
            DeclarationRelation::InfixSpec { spec, target } => {
                key.push_str(&key_for_infix_spec(spec));
                key.push_str(&key_for_expression(target));
            }
        }
    }
    key
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
    spec.refinement
        .iter()
        .flat_map(|refinement| refinement.parts.iter())
        .flat_map(|part| part.tail.iter())
        .flat_map(|tail| tail.args.iter())
        .flat_map(|args| args.expressions.iter())
        .chain(
            spec.head_args
                .iter()
                .flat_map(|args| args.expressions.iter()),
        )
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
    let ty = strip_command_context_key_suffix(ty);

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
            ',' | ';' if paren_depth == 0 && brace_depth == 0 && bracket_depth == 0 => {
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
        FormOrDeclarationKind::MappingParameter { selector, .. } => {
            Some(selector.name().to_owned())
        }
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

fn described_target_subject_key(target: &DefinesTarget) -> String {
    match target {
        DefinesTarget::Form(form) => {
            primary_form_name(form).unwrap_or_else(|| key_for_form_or_declaration(form))
        }
        DefinesTarget::Declaration(statement) => primary_subject_key(&statement.subject),
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

#[cfg(test)]
mod desugar_tests {
    use super::*;
    use crate::frontend::formulation::parser::parse_expression;

    fn operand(name: &str) -> Expression {
        Expression::new(Span::new(0, 0), ExpressionKind::Name(name.to_owned()))
    }

    #[test]
    fn named_operator_desugars_to_application() {
        // A bare name becomes a function call: `x |op| y` == `op(x, y)`.
        let call = desugar_named_operator_application(
            "op",
            Span::new(0, 0),
            vec![operand("x"), operand("y")],
        );
        match call.kind {
            ExpressionKind::FunctionCall { name, arguments } => {
                assert_eq!(name, "op");
                assert_eq!(arguments.len(), 2);
            }
            other => panic!("expected function call, got {other:?}"),
        }

        // A dotted path becomes a member call that tracks down through the value's
        // fields: `x |M.*| y` == `M.*(x, y)`.
        let call = desugar_named_operator_application(
            "M.*",
            Span::new(0, 0),
            vec![operand("x"), operand("y")],
        );
        match call.kind {
            ExpressionKind::MemberCall {
                owner,
                name,
                arguments,
            } => {
                assert!(matches!(owner.kind, ExpressionKind::Name(ref n) if n == "M"));
                assert_eq!(name, "*");
                assert_eq!(arguments.len(), 2);
            }
            other => panic!("expected member call, got {other:?}"),
        }

        // A longer path nests member accesses: `x |a.b.c| y` == `a.b.c(x, y)`.
        let call = desugar_named_operator_application(
            "a.b.c",
            Span::new(0, 0),
            vec![operand("x"), operand("y")],
        );
        match call.kind {
            ExpressionKind::MemberCall { owner, name, .. } => {
                assert_eq!(name, "c");
                assert!(matches!(owner.kind, ExpressionKind::MemberAccess { .. }));
            }
            other => panic!("expected member call, got {other:?}"),
        }
    }

    #[test]
    fn named_prefix_and_postfix_desugar_to_calls() {
        let expr = parse_expression("f| x").expect("prefix parses");
        let ExpressionKind::Prefix {
            operator,
            expression,
        } = &expr.kind
        else {
            panic!("expected prefix, got {:?}", expr.kind);
        };
        let call = named_prefix_operator_desugaring(operator, expression).expect("named prefix");
        assert!(matches!(
            call.kind,
            ExpressionKind::FunctionCall { ref name, ref arguments }
                if name == "f" && arguments.len() == 1
        ));

        let expr = parse_expression("x |f").expect("postfix parses");
        let ExpressionKind::Postfix {
            expression,
            operator,
        } = &expr.kind
        else {
            panic!("expected postfix, got {:?}", expr.kind);
        };
        let call = postfix_operator_desugaring(expression, operator);
        assert!(matches!(
            call.kind,
            ExpressionKind::FunctionCall { ref name, ref arguments }
                if name == "f" && arguments.len() == 1
        ));
    }
}

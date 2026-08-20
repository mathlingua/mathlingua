use super::*;

pub(in crate::backend::semantic) fn walk_optional_is_or_specs(
    section: &Option<UsingSection>,
    visit: &mut impl FnMut(&SignatureShape),
) {
    if let Some(section) = section {
        for statement in &section.arguments {
            walk_declaration_statement(statement, visit);
        }
    }
}

pub(in crate::backend::semantic) fn walk_optional_clauses<T>(
    section: &Option<T>,
    visit: &mut impl FnMut(&SignatureShape),
) where
    T: ClauseSection,
{
    if let Some(section) = section {
        for clause in section.clauses() {
            walk_clause(clause, visit);
        }
    }
}

pub(in crate::backend::semantic) trait ClauseSection {
    /// Returns the clauses contained by the section.
    fn clauses(&self) -> &[Clause];
}

impl ClauseSection for WhenSection {
    /// Returns the clauses in a `when:` section.
    fn clauses(&self) -> &[Clause] {
        &self.arguments
    }
}

impl ClauseSection for SatisfiesSection {
    /// Returns the clauses in a `satisfies:` section.
    fn clauses(&self) -> &[Clause] {
        &self.arguments
    }
}

pub(in crate::backend::semantic) fn walk_optional_enables(
    section: &Option<EnablesSection>,
    visit: &mut impl FnMut(&SignatureShape),
) {
    if let Some(section) = section {
        for item in &section.arguments {
            match item {
                EnablesItem::Capability(group) => {
                    walk_alias_kind(&group.capability.argument, visit)
                }
                EnablesItem::FromCapability(group) => {
                    walk_declaration_statement(&group.from.argument, visit);
                    walk_alias_kind(&group.capability.argument, visit);
                }
                EnablesItem::FromAs(group) => {
                    walk_declaration_statement(&group.from.argument, visit);
                    walk_expression(&group.as_.argument.left, visit);
                    walk_expression(&group.as_.argument.right, visit);
                }
                EnablesItem::Relation(group) => {
                    walk_relationship_declaration(&group.to.argument, visit);
                    if let Some(when) = &group.when {
                        for item in &when.arguments {
                            walk_relation_when_item(item, visit);
                        }
                    }
                    if let Some(specifies) = &group.specifies {
                        walk_clause(&specifies.argument, visit);
                    }
                }
            }
        }
    }
}

fn walk_relationship_command(command: &CommandExpression, visit: &mut impl FnMut(&SignatureShape)) {
    let shape = shape_for_command_expression(command);
    visit(&shape);
    walk_command_expression_arguments(command, visit);
}

fn walk_relationship_declaration(
    declaration: &RelationshipDeclaration,
    visit: &mut impl FnMut(&SignatureShape),
) {
    match declaration {
        RelationshipDeclaration::Command(command) => walk_relationship_command(command, visit),
        RelationshipDeclaration::Declaration(statement) => {
            walk_declaration_statement(statement, visit)
        }
    }
}

fn walk_relation_when_item(item: &RelationWhenItem, visit: &mut impl FnMut(&SignatureShape)) {
    match item {
        RelationWhenItem::Declaration(statement) => walk_declaration_statement(statement, visit),
        RelationWhenItem::HardCast(statement) => walk_hard_cast_statement(statement, visit),
    }
}

pub(in crate::backend::semantic) fn walk_optional_requires(
    section: &Option<RequiresSection>,
    visit: &mut impl FnMut(&SignatureShape),
) {
    if let Some(section) = section {
        for item in &section.arguments {
            match item {
                RequiresItem::Capability(group) => {
                    walk_alias_kind(&group.capability.argument, visit)
                }
                RequiresItem::Definition(group) => {
                    let shape = shape_for_command_expression(&group.definition.argument.command);
                    visit(&shape);
                    walk_command_expression_arguments(&group.definition.argument.command, visit);
                    walk_type_expression(&group.definition.argument.ty, visit);
                }
            }
        }
    }
}

pub(in crate::backend::semantic) fn walk_optional_aliases(
    section: &Option<AliasesSection>,
    visit: &mut impl FnMut(&SignatureShape),
) {
    if let Some(section) = section {
        for item in &section.arguments {
            match item {
                AliasItem::Alias(group) => walk_alias_kind(&group.alias.argument, visit),
            }
        }
    }
}

pub(in crate::backend::semantic) fn walk_alias_kind(
    kind: &AliasKind,
    visit: &mut impl FnMut(&SignatureShape),
) {
    match kind {
        AliasKind::Expression(alias) => {
            match &alias.lhs {
                ExpressionAliasLhs::Form(form) => {
                    walk_form_or_declaration(form, visit);
                }
                ExpressionAliasLhs::Command(command) => {
                    let shape = shape_for_command_header_node(command);
                    visit(&shape);
                }
                ExpressionAliasLhs::InfixCommand(command) => {
                    let shape = shape_for_infix_command_header(command);
                    visit(&shape);
                }
                // A member LHS (`x.inv`, `x.f(a_)`) names no command, so there is
                // nothing to reference-validate on the left.
                ExpressionAliasLhs::Member(_) => {}
            }
            // The reduction target (right of `:=>`) can reference commands too, so
            // reference-validate it like any other expression.
            walk_expression(&alias.expression, visit);
        }
        AliasKind::SpecOperator(alias) => walk_spec_operator_alias_target(&alias.target, visit),
    }
}

/// Walks the reduction target of a spec-operator capability (`x_ "in" X :-> …`)
/// so that command references in the target — for example the `\group.element:of`
/// in `x_ is \group.element:of{G}` — are reference-validated like any other use.
fn walk_spec_operator_alias_target(
    target: &SpecOperatorAliasTarget,
    visit: &mut impl FnMut(&SignatureShape),
) {
    match target {
        SpecOperatorAliasTarget::IsOrSpec(is_or_spec) => match is_or_spec.as_ref() {
            IsOrSpec::Is(statement) => walk_is_statement(statement, visit),
            IsOrSpec::Spec(_) => {}
        },
        SpecOperatorAliasTarget::MemberOf(expression) => walk_expression(expression, visit),
        SpecOperatorAliasTarget::PlaceholderSpec(_) | SpecOperatorAliasTarget::Builtin(_) => {}
    }
}

pub(in crate::backend::semantic) fn walk_specify_item(
    item: &SpecifyItem,
    visit: &mut impl FnMut(&SignatureShape),
) {
    let group = match item {
        SpecifyItem::Decimal(group)
        | SpecifyItem::ZeroOrPositiveInt(group)
        | SpecifyItem::PositiveInt(group)
        | SpecifyItem::Int(group) => group,
    };
    walk_type_expression(&group.is_.argument, visit);
}

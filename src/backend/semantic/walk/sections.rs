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
                EnablesItem::Viewable(group) => {
                    walk_declaration_statement(&group.as_.argument, visit);
                    if let Some(states) = &group.states {
                        walk_clause(&states.argument, visit);
                    }
                }
                EnablesItem::Connection(group) => {
                    if let Some(using) = &group.using {
                        for statement in &using.arguments {
                            walk_declaration_statement(statement, visit);
                        }
                    }
                }
            }
        }
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
        AliasKind::Expression(alias) => match &alias.lhs {
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
        },
        AliasKind::SpecOperator(_) => {}
    }
}

pub(in crate::backend::semantic) fn walk_specify_item(
    item: &SpecifyItem,
    visit: &mut impl FnMut(&SignatureShape),
) {
    match item {
        SpecifyItem::PositiveInt(group) => walk_open_text_clauses(&group.is_, visit),
        SpecifyItem::NegativeInt(group) => walk_open_text_clauses(&group.is_, visit),
        SpecifyItem::Zero(group) => walk_open_text_clauses(&group.is_, visit),
        SpecifyItem::PositiveDecimal(group) => walk_open_text_clauses(&group.is_, visit),
        SpecifyItem::NegativeDecimal(group) => walk_open_text_clauses(&group.is_, visit),
    }
}

pub(in crate::backend::semantic) fn walk_open_text_clauses(
    _section: &IsSection,
    _visit: &mut impl FnMut(&SignatureShape),
) {
}

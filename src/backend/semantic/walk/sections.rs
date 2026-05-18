/// Traverses an optional `using:` section containing `is` or `spec` entries.
fn walk_optional_is_or_specs(
    section: &Option<UsingSection>,
    visit: &mut impl FnMut(&SignatureShape),
) {
    if let Some(section) = section {
        for spec in &section.arguments {
            walk_is_or_spec(spec, visit);
        }
    }
}

/// Traverses any optional section that stores a list of logical clauses.
fn walk_optional_clauses<T>(section: &Option<T>, visit: &mut impl FnMut(&SignatureShape))
where
    T: ClauseSection,
{
    if let Some(section) = section {
        for clause in section.clauses() {
            walk_clause(clause, visit);
        }
    }
}

/// Adapter trait for structural sections whose payload is a slice of clauses.
trait ClauseSection {
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

/// Traverses references that appear inside an optional `provides:` section.
///
/// Symbols can contain aliases that reference forms or commands, and connection
/// groups can introduce `using:` requirements that must also be validated.
fn walk_optional_provides(
    section: &Option<ProvidesSection>,
    visit: &mut impl FnMut(&SignatureShape),
) {
    if let Some(section) = section {
        for item in &section.arguments {
            match item {
                ProvidesItem::Symbol(group) => walk_alias_kind(&group.symbol.argument, visit),
                ProvidesItem::Connection(group) => {
                    if let Some(using) = &group.using {
                        for spec in &using.arguments {
                            walk_is_or_spec(spec, visit);
                        }
                    }
                }
            }
        }
    }
}

/// Traverses references that appear inside an optional `aliases:` section.
fn walk_optional_aliases(
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

/// Traverses the definition side of an alias declaration.
///
/// Expression aliases may introduce forms or command headers that reference
/// existing signatures.  Spec-operator aliases are currently pure declarations
/// for this checker and do not contain command references.
fn walk_alias_kind(kind: &AliasKind, visit: &mut impl FnMut(&SignatureShape)) {
    match kind {
        AliasKind::Expression(alias) => match &alias.lhs {
            crate::frontend::formulation::ast::ExpressionAliasLhs::Form(form) => {
                walk_form_or_declaration(form, visit);
            }
            crate::frontend::formulation::ast::ExpressionAliasLhs::Command(command) => {
                let shape = shape_for_command_header_node(command);
                visit(&shape);
            }
            crate::frontend::formulation::ast::ExpressionAliasLhs::InfixCommand(command) => {
                let shape = shape_for_infix_command_header(command);
                visit(&shape);
            }
        },
        AliasKind::SpecOperator(_) => {}
    }
}

/// Traverses one `Specify` item for command references.
fn walk_specify_item(item: &SpecifyItem, visit: &mut impl FnMut(&SignatureShape)) {
    match item {
        SpecifyItem::PositiveInt(group) => walk_open_text_clauses(&group.is_, visit),
        SpecifyItem::NegativeInt(group) => walk_open_text_clauses(&group.is_, visit),
        SpecifyItem::Zero(group) => walk_open_text_clauses(&group.is_, visit),
        SpecifyItem::PositiveDecimal(group) => walk_open_text_clauses(&group.is_, visit),
        SpecifyItem::NegativeDecimal(group) => walk_open_text_clauses(&group.is_, visit),
    }
}

/// Placeholder traversal for `is:` sections that currently contain open text.
///
/// Open text is not parsed into formulation AST nodes yet, so there are no
/// semantic command references to validate here.
fn walk_open_text_clauses(_section: &IsSection, _visit: &mut impl FnMut(&SignatureShape)) {}


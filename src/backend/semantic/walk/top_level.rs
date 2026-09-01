use super::*;

pub(in crate::backend::semantic) fn walk_top_level_item(
    item: &TopLevelItem,
    visit: &mut impl FnMut(&SignatureShape),
) {
    match item {
        TopLevelItem::Disambiguates(group) => {
            for branch in &group.branches {
                for clause in &branch.when.arguments {
                    walk_clause(clause, visit);
                }
                walk_expression(&branch.to.argument, visit);
            }
            if let Some(else_) = &group.else_ {
                walk_expression(&else_.argument, visit);
            }
            walk_optional_aliases(&group.aliases, visit);
            walk_optional_justification(&group.justification, visit);
        }
        TopLevelItem::Declares(group) => {
            walk_declares_target(&group.declares.argument, visit);
            if let Some(via) = &group.declares.via {
                walk_form_or_declaration(via, visit);
            }
            walk_optional_is_or_specs(&group.using, visit);
            walk_optional_clauses(&group.when, visit);
            if let Some(section) = &group.extends {
                for item in &section.arguments {
                    walk_declaration_statement(&item.statement, visit);
                    if let Some(via) = &item.via {
                        walk_form_or_declaration(via, visit);
                    }
                }
            }
            if let Some(section) = &group.specifies {
                for item in &section.arguments {
                    walk_is_or_via_item(item, visit);
                }
            }
            walk_optional_clauses(&group.satisfies, visit);
            walk_optional_requires(&group.requires, visit);
            walk_optional_enables(&group.enables, visit);
            walk_optional_aliases(&group.aliases, visit);
            walk_optional_justification(&group.justification, visit);
        }
        TopLevelItem::Defines(group) => {
            walk_declaration_statement(&group.defines.argument, visit);
            walk_optional_is_or_specs(&group.using, visit);
            walk_optional_clauses(&group.when, visit);
            walk_optional_specifies(&group.specifies, visit);
            if let Some(section) = &group.expresses {
                for clause in &section.arguments {
                    walk_clause(clause, visit);
                }
            }
            walk_optional_requires(&group.requires, visit);
            walk_optional_enables(&group.enables, visit);
            walk_optional_aliases(&group.aliases, visit);
            walk_optional_justification(&group.justification, visit);
        }
        TopLevelItem::Realizes(group) => {
            walk_declaration_statement(&group.realizes.argument, visit);
            walk_optional_is_or_specs(&group.using, visit);
            walk_optional_clauses(&group.when, visit);
            walk_optional_specifies(&group.specifies, visit);
            if let Some(section) = &group.expresses {
                for clause in &section.arguments {
                    walk_clause(clause, visit);
                }
            }
            walk_optional_requires(&group.requires, visit);
            walk_optional_enables(&group.enables, visit);
            walk_optional_aliases(&group.aliases, visit);
            walk_optional_justification(&group.justification, visit);
        }
        TopLevelItem::Refines(group) => {
            walk_declaration_statement(&group.refines.argument, visit);
            walk_optional_is_or_specs(&group.using, visit);
            walk_optional_clauses(&group.when, visit);
            if let Some(section) = &group.specifies
                && !declaration_has_dynamic_refined_tail(&section.argument)
            {
                walk_declaration_statement(&section.argument, visit);
            }
            walk_optional_clauses(&group.satisfies, visit);
            walk_optional_requires(&group.requires, visit);
            walk_optional_enables(&group.enables, visit);
            walk_optional_aliases(&group.aliases, visit);
            walk_optional_justification(&group.justification, visit);
        }
        TopLevelItem::States(group) => {
            walk_optional_is_or_specs(&group.using, visit);
            walk_optional_clauses(&group.when, visit);
            for clause in &group.that.arguments {
                walk_clause(clause, visit);
            }
            walk_optional_requires(&group.requires, visit);
            walk_optional_enables(&group.enables, visit);
            walk_optional_aliases(&group.aliases, visit);
            walk_optional_justification(&group.justification, visit);
        }
        TopLevelItem::Axiom(group) => {
            walk_theorem_like(
                group.given.as_ref(),
                group.where_.as_ref(),
                &group.then,
                group.iff.as_ref(),
                visit,
            );
            walk_optional_aliases(&group.aliases, visit);
            walk_optional_justification(&group.justification, visit);
        }
        TopLevelItem::Theorem(group) => {
            walk_theorem_like(
                group.given.as_ref(),
                group.where_.as_ref(),
                &group.then,
                group.iff.as_ref(),
                visit,
            );
            walk_optional_aliases(&group.aliases, visit);
            walk_optional_justification(&group.justification, visit);
        }
        TopLevelItem::Conjecture(group) => {
            walk_theorem_like(
                group.given.as_ref(),
                group.where_.as_ref(),
                &group.then,
                group.iff.as_ref(),
                visit,
            );
            walk_optional_aliases(&group.aliases, visit);
            walk_optional_justification(&group.justification, visit);
        }
        TopLevelItem::Example(group) => {
            for item in &group.example.arguments {
                if let ExampleItem::Clause(clause) = item {
                    walk_clause(clause, visit);
                }
            }
            walk_optional_aliases(&group.aliases, visit);
            walk_optional_justification(&group.justification, visit);
        }
        TopLevelItem::Specify(group) => {
            for item in &group.specify.arguments {
                walk_specify_item(item, visit);
            }
        }
        TopLevelItem::Relation(group) => {
            walk_optional_is_or_specs(&group.using, visit);
            let (first, second) = group.endpoints.subjects();
            walk_relation_subject(first, visit);
            walk_relation_subject(second, visit);
            walk_optional_clauses(&group.when, visit);
            if let Some(RelationSpecifies::Statement(clause)) =
                group.specifies.as_ref().map(|m| &m.argument)
            {
                walk_clause(clause, visit);
            }
            walk_optional_aliases(&group.aliases, visit);
            walk_optional_justification(&group.justification, visit);
        }
        TopLevelItem::Equivalent(group) => {
            walk_optional_is_or_specs(&group.using, visit);
            walk_optional_clauses(&group.when, visit);
            for expression in &group.to.arguments {
                walk_expression(expression, visit);
            }
            walk_optional_justification(&group.justification, visit);
        }
        TopLevelItem::Title(_)
        | TopLevelItem::SectionTitle(_)
        | TopLevelItem::SubsectionTitle(_)
        | TopLevelItem::Text(_)
        | TopLevelItem::Writing(_)
        | TopLevelItem::Person(_)
        | TopLevelItem::Resource(_)
        | TopLevelItem::Topic(_)
        // `Text*` placeholders are opaque prose: no command references to validate.
        | TopLevelItem::TextItem(_) => {}
    }
}

/// Walks the items of a `Defines:`/`Realizes:` `specifies:` section.
fn walk_optional_specifies(
    specifies: &Option<DefinesSpecifiesSection>,
    visit: &mut impl FnMut(&SignatureShape),
) {
    if let Some(section) = specifies {
        for item in &section.arguments {
            walk_is_or_via_item(item, visit);
        }
    }
}

/// Walks the `have:`/`asserting:` groups of an optional `Justification:` section so
/// their command references are reference-validated like any other clause content.
fn walk_optional_justification(
    justification: &Option<JustificationSection>,
    visit: &mut impl FnMut(&SignatureShape),
) {
    if let Some(section) = justification {
        for group in &section.arguments {
            walk_have_group(group, visit);
        }
    }
}

/// Walks one endpoint of a `Relation:`. Only a declared subject introduces
/// symbols; a quoted reference (a topic or a definition signature) names an
/// external target and contributes none.
fn walk_relation_subject(subject: &RelationSubject, visit: &mut impl FnMut(&SignatureShape)) {
    match subject {
        RelationSubject::Declaration(statement) => walk_declaration_statement(statement, visit),
        RelationSubject::Reference(_) => {}
    }
}

fn walk_declares_target(target: &DeclaresTarget, visit: &mut impl FnMut(&SignatureShape)) {
    match target {
        DeclaresTarget::Form(form) => walk_form_or_declaration(form, visit),
        DeclaresTarget::Declaration(statement) => walk_declaration_statement(statement, visit),
    }
}

fn declaration_has_dynamic_refined_tail(statement: &DeclarationStatement) -> bool {
    matches!(
        &statement.relation,
        Some(DeclarationRelation::Is(TypeExpression::RefinedCommand(command)))
            if matches!(command.refined_tail, RefinedTail::Name { .. })
    )
}

pub(in crate::backend::semantic) fn walk_theorem_like(
    given: Option<&GivenSection>,
    where_: Option<&WhereSection>,
    then: &ThenSection,
    iff: Option<&IffSection>,
    visit: &mut impl FnMut(&SignatureShape),
) {
    if let Some(section) = given {
        for statement in &section.arguments {
            walk_declaration_statement(statement, visit);
        }
    }
    if let Some(section) = where_ {
        for clause in &section.arguments {
            walk_clause(clause, visit);
        }
    }
    for clause in &then.arguments {
        walk_clause(clause, visit);
    }
    if let Some(section) = iff {
        for clause in &section.arguments {
            walk_clause(clause, visit);
        }
    }
}

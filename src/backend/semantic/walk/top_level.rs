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
        }
        TopLevelItem::Describes(group) => {
            walk_describes_target(&group.describes.argument, visit);
            walk_optional_is_or_specs(&group.using, visit);
            walk_optional_clauses(&group.when, visit);
            if let Some(section) = &group.extends {
                walk_is_or_via_item(&section.argument, visit);
            }
            if let Some(section) = &group.specifies {
                for item in &section.arguments {
                    walk_is_or_via_item(item, visit);
                }
            }
            walk_optional_clauses(&group.satisfies, visit);
            walk_optional_provides(&group.provides, visit);
            walk_optional_aliases(&group.aliases, visit);
        }
        TopLevelItem::Defines(group) => {
            walk_declaration_statement(&group.defines.argument, visit);
            walk_optional_is_or_specs(&group.using, visit);
            walk_optional_clauses(&group.when, visit);
            if let Some(section) = &group.expresses {
                walk_clause(&section.argument, visit);
            }
            walk_optional_provides(&group.provides, visit);
            walk_optional_aliases(&group.aliases, visit);
        }
        TopLevelItem::Refines(group) => {
            walk_declaration_statement(&group.refines.argument, visit);
            walk_optional_is_or_specs(&group.using, visit);
            walk_optional_clauses(&group.when, visit);
            if let Some(section) = &group.extends
                && !declaration_has_dynamic_refined_tail(&section.argument)
            {
                walk_declaration_statement(&section.argument, visit);
            }
            walk_optional_clauses(&group.satisfies, visit);
            walk_optional_provides(&group.provides, visit);
            walk_optional_aliases(&group.aliases, visit);
        }
        TopLevelItem::States(group) => {
            walk_optional_is_or_specs(&group.using, visit);
            walk_optional_clauses(&group.when, visit);
            for clause in &group.that.arguments {
                walk_clause(clause, visit);
            }
            walk_optional_provides(&group.provides, visit);
            walk_optional_aliases(&group.aliases, visit);
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
        }
        TopLevelItem::Corollary(group) => {
            walk_theorem_like(
                group.given.as_ref(),
                group.where_.as_ref(),
                &group.then,
                group.iff.as_ref(),
                visit,
            );
            walk_optional_aliases(&group.aliases, visit);
        }
        TopLevelItem::Lemma(group) => {
            walk_theorem_like(
                group.given.as_ref(),
                group.where_.as_ref(),
                &group.then,
                group.iff.as_ref(),
                visit,
            );
            walk_optional_aliases(&group.aliases, visit);
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
        }
        TopLevelItem::Specify(group) => {
            for item in &group.specify.arguments {
                walk_specify_item(item, visit);
            }
        }
        TopLevelItem::Title(_)
        | TopLevelItem::SectionTitle(_)
        | TopLevelItem::SubsectionTitle(_)
        | TopLevelItem::Text(_)
        | TopLevelItem::Person(_)
        | TopLevelItem::Resource(_) => {}
    }
}

fn walk_describes_target(target: &DescribesTarget, visit: &mut impl FnMut(&SignatureShape)) {
    match target {
        DescribesTarget::Form(form) => walk_form_or_declaration(form, visit),
        DescribesTarget::Declaration(statement) => walk_declaration_statement(statement, visit),
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

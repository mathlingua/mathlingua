use super::*;

/// Traverses a logical clause tree for command references.
///
/// Clause traversal is recursive because quantifiers, conditionals, and grouped
/// logical constructs can contain nested clauses in multiple sections.
pub(in crate::backend::semantic) fn walk_clause(
    clause: &Clause,
    visit: &mut impl FnMut(&SignatureShape),
) {
    match clause {
        Clause::Not(group) => walk_clause(&group.not.argument, visit),
        Clause::AllOf(group) => {
            for clause in &group.all_of.arguments {
                walk_clause(clause, visit);
            }
        }
        Clause::AnyOf(group) => {
            for clause in &group.any_of.arguments {
                walk_clause(clause, visit);
            }
        }
        Clause::OneOf(group) => {
            for clause in &group.one_of.arguments {
                walk_clause(clause, visit);
            }
        }
        Clause::Exists(group) => {
            walk_is_or_spec(&group.exists.argument, visit);
            for clause in &group.such_that.arguments {
                walk_clause(clause, visit);
            }
        }
        Clause::ExistsUnique(group) => {
            walk_is_or_spec(&group.exists_unique.argument, visit);
            for clause in &group.such_that.arguments {
                walk_clause(clause, visit);
            }
        }
        Clause::ForAll(group) => {
            walk_is_or_spec(&group.for_all.argument, visit);
            if let Some(section) = &group.where_ {
                for clause in &section.arguments {
                    walk_clause(clause, visit);
                }
            }
            for clause in &group.then.arguments {
                walk_clause(clause, visit);
            }
        }
        Clause::If(group) => {
            for clause in &group.if_.arguments {
                walk_clause(clause, visit);
            }
            for clause in &group.then.arguments {
                walk_clause(clause, visit);
            }
        }
        Clause::Iff(group) => {
            for clause in &group.iff.arguments {
                walk_clause(clause, visit);
            }
            for clause in &group.then.arguments {
                walk_clause(clause, visit);
            }
        }
        Clause::Piecewise(group) => {
            for clause in &group.if_.arguments {
                walk_clause(clause, visit);
            }
            for clause in &group.then.arguments {
                walk_clause(clause, visit);
            }
            if let Some(section) = &group.else_ {
                for clause in &section.arguments {
                    walk_clause(clause, visit);
                }
            }
        }
        Clause::Given(group) => {
            walk_is_or_refined_spec(&group.given.argument, visit);
            if let Some(section) = &group.where_ {
                for clause in &section.arguments {
                    walk_clause(clause, visit);
                }
            }
            for clause in &group.then.arguments {
                walk_clause(clause, visit);
            }
        }
        Clause::IsOrSpec(spec) => walk_is_or_spec(spec, visit),
        Clause::Expression(expression) => walk_expression(expression, visit),
    }
}

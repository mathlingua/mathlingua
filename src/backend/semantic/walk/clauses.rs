use super::*;

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
        Clause::Equivalently(group) => {
            for clause in &group.equivalently.arguments {
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
            for item in &group.exists.arguments {
                walk_binding_or_spec(item, visit);
            }
            if let Some(section) = &group.such_that {
                for clause in &section.arguments {
                    walk_clause(clause, visit);
                }
            }
        }
        Clause::ExistsUnique(group) => {
            for item in &group.exists_unique.arguments {
                walk_binding_or_spec(item, visit);
            }
            if let Some(section) = &group.such_that {
                for clause in &section.arguments {
                    walk_clause(clause, visit);
                }
            }
        }
        Clause::ForAll(group) => {
            for item in &group.for_all.arguments {
                walk_binding_or_spec(item, visit);
            }
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
            for statement in &group.given.arguments {
                walk_declaration_statement(statement, visit);
            }
            if let Some(section) = &group.where_ {
                for clause in &section.arguments {
                    walk_clause(clause, visit);
                }
            }
            for clause in &group.then.arguments {
                walk_clause(clause, visit);
            }
        }
        Clause::Declaration(statement) => walk_declaration_statement(statement, visit),
        Clause::Expression(expression) => walk_expression(expression, visit),
    }
}

fn walk_binding_or_spec(item: &BindingOrSpec, visit: &mut impl FnMut(&SignatureShape)) {
    match item {
        BindingOrSpec::Declaration(statement) => walk_declaration_statement(statement, visit),
    }
}

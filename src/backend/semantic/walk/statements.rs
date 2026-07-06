use super::*;

pub(in crate::backend::semantic) fn walk_is_or_via_item(
    item: &IsOrViaItem,
    visit: &mut impl FnMut(&SignatureShape),
) {
    match item {
        IsOrViaItem::IsVia(statement) => {
            walk_is_statement(&statement.is_statement, visit);
            walk_form_or_declaration(&statement.via, visit);
        }
        IsOrViaItem::Declaration(statement) => walk_declaration_statement(statement, visit),
    }
}

pub(in crate::backend::semantic) fn walk_declaration_statement(
    statement: &DeclarationStatement,
    visit: &mut impl FnMut(&SignatureShape),
) {
    walk_is_subject(&statement.subject, visit);
    if let Some(expansion) = &statement.expansion {
        walk_is_subject(expansion, visit);
    }
    if let Some(definition) = &statement.definition {
        walk_expression(definition, visit);
    }
    match &statement.relation {
        Some(DeclarationRelation::Is(ty)) => walk_type_expression(ty, visit),
        Some(DeclarationRelation::InfixSpec { spec, target }) => {
            let shape = shape_for_infix_spec(spec);
            visit(&shape);
            walk_infix_spec_arguments(spec, visit);
            walk_expression(target, visit);
        }
        Some(DeclarationRelation::Spec { target, .. }) => walk_expression(target, visit),
        None => {}
    }
}

pub(in crate::backend::semantic) fn walk_is_statement(
    statement: &IsStatement,
    visit: &mut impl FnMut(&SignatureShape),
) {
    walk_is_subject(&statement.subject, visit);
    walk_type_expression(&statement.ty, visit);
}

pub(in crate::backend::semantic) fn walk_is_subject(
    subject: &IsSubject,
    visit: &mut impl FnMut(&SignatureShape),
) {
    match &subject.kind {
        IsSubjectKind::Forms(forms) => {
            for form in forms {
                match form {
                    IsSubjectForm::Form(form) => walk_form_or_declaration(form, visit),
                    IsSubjectForm::PlaceholderForm(form) => walk_placeholder_form(form, visit),
                }
            }
        }
        IsSubjectKind::Operator(_) => {}
    }
}

pub(in crate::backend::semantic) fn walk_type_expression(
    ty: &TypeExpression,
    visit: &mut impl FnMut(&SignatureShape),
) {
    match ty {
        TypeExpression::Builtin { .. } => {}
        TypeExpression::Command(command) => {
            let shape = shape_for_command_expression(command);
            visit(&shape);
            walk_command_expression_arguments(command, visit);
        }
        TypeExpression::RefinedCommand(command) => {
            let shape = shape_for_refined_command_expression(command);
            visit(&shape);
            walk_refined_command_expression_arguments(command, visit);
        }
        TypeExpression::Function(function_type) => {
            for spec in function_type
                .inputs
                .iter()
                .chain(std::iter::once(&function_type.output))
            {
                walk_function_type_spec(spec, visit);
            }
        }
        TypeExpression::Coercion { ty, literal, .. } => {
            walk_type_expression(ty, visit);
            walk_set_target(&literal.target, visit);
            for spec in &literal.specs {
                walk_expression(spec, visit);
            }
            if let Some(predicate) = &literal.predicate {
                walk_set_predicate(predicate, visit);
            }
        }
    }
}

fn walk_set_target(target: &SetTarget, visit: &mut impl FnMut(&SignatureShape)) {
    match &target.kind {
        SetTargetKind::Name(_) | SetTargetKind::PlaceholderForm(_) => {}
        SetTargetKind::Alias { target, .. } | SetTargetKind::Introduction { target, .. } => {
            walk_set_target(target, visit)
        }
        SetTargetKind::Function { arguments, .. } => {
            for argument in arguments {
                walk_set_target(argument, visit);
            }
        }
        SetTargetKind::Tuple(elements) => {
            for element in elements {
                if let SetTargetElement::Target(target) = element {
                    walk_set_target(target, visit);
                }
            }
        }
    }
}

fn walk_set_predicate(predicate: &SetPredicate, visit: &mut impl FnMut(&SignatureShape)) {
    match predicate {
        SetPredicate::Expression(expression) => walk_expression(expression, visit),
        SetPredicate::Definition { target, value, .. } => {
            walk_set_target(target, visit);
            walk_expression(value, visit);
        }
    }
}

fn walk_function_type_spec(spec: &FunctionTypeSpec, visit: &mut impl FnMut(&SignatureShape)) {
    match &spec.kind {
        FunctionTypeSpecKind::Is(ty) => walk_type_expression(ty, visit),
        FunctionTypeSpecKind::Spec { .. } => {}
    }
}

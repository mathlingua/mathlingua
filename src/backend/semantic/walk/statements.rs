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

pub(in crate::backend::semantic) fn walk_hard_cast_statement(
    statement: &HardCastStatement,
    visit: &mut impl FnMut(&SignatureShape),
) {
    walk_is_subject(&statement.subject, visit);
    if let Some(definition) = &statement.definition {
        walk_expression(definition, visit);
    }
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
        TypeExpression::Builtin { .. } | TypeExpression::Parameter { .. } => {}
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
    }
}

fn walk_function_type_spec(spec: &FunctionTypeSpec, visit: &mut impl FnMut(&SignatureShape)) {
    match &spec.kind {
        FunctionTypeSpecKind::Is(ty) => walk_type_expression(ty, visit),
        FunctionTypeSpecKind::Spec { .. } => {}
    }
}

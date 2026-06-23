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
        IsOrViaItem::IsOrSpec(spec) => walk_is_or_spec(spec, visit),
    }
}

pub(in crate::backend::semantic) fn walk_is_or_spec(
    spec: &IsOrSpec,
    visit: &mut impl FnMut(&SignatureShape),
) {
    match spec {
        IsOrSpec::Is(statement) => walk_is_statement(statement, visit),
        IsOrSpec::Spec(statement) => walk_spec_subject(&statement.subject, visit),
    }
}

pub(in crate::backend::semantic) fn walk_is_or_refined_spec(
    spec: &IsOrRefinedStatementSpec,
    visit: &mut impl FnMut(&SignatureShape),
) {
    match spec {
        IsOrRefinedStatementSpec::Is(statement) => walk_is_statement(statement, visit),
        IsOrRefinedStatementSpec::Spec(statement) => walk_spec_subject(&statement.subject, visit),
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

pub(in crate::backend::semantic) fn walk_spec_subject(
    subject: &SpecSubject,
    visit: &mut impl FnMut(&SignatureShape),
) {
    match &subject.kind {
        SpecSubjectKind::Form(form) => walk_form_or_declaration(form, visit),
        SpecSubjectKind::Operator(_) => {}
    }
}

pub(in crate::backend::semantic) fn walk_type_expression(
    ty: &TypeExpression,
    visit: &mut impl FnMut(&SignatureShape),
) {
    match ty {
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

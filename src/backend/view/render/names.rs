use super::*;

pub(super) fn primary_declaration_statement_name(
    statement: &DeclarationStatement,
) -> Option<String> {
    primary_is_subject_name(&statement.subject)
}

pub(super) fn primary_is_subject_name(subject: &IsSubject) -> Option<String> {
    match &subject.kind {
        IsSubjectKind::Forms(forms) => forms.iter().find_map(|form| match form {
            IsSubjectForm::Form(form) => primary_form_name(form),
            IsSubjectForm::PlaceholderForm(form) => primary_placeholder_form_name(form),
        }),
        IsSubjectKind::Operator(_) => None,
    }
}

pub(super) fn primary_form_name(form: &FormOrDeclaration) -> Option<String> {
    match &form.kind {
        FormOrDeclarationKind::Name(name) => Some(name.clone()),
        FormOrDeclarationKind::FunctionDeclaration { name, form } => {
            name.clone().or_else(|| Some(form.name.clone()))
        }
        FormOrDeclarationKind::TupleDeclaration { name, .. }
        | FormOrDeclarationKind::SetDeclaration { name, .. } => name.clone(),
        FormOrDeclarationKind::InfixOperator { .. }
        | FormOrDeclarationKind::PrefixOperator { .. }
        | FormOrDeclarationKind::PostfixOperator { .. } => None,
    }
}

pub(super) fn primary_placeholder_form_name(form: &PlaceholderForm) -> Option<String> {
    match &form.kind {
        PlaceholderFormKind::Placeholder(placeholder) => Some(placeholder.name.clone()),
        PlaceholderFormKind::Function { placeholder, .. } => Some(placeholder.name.clone()),
    }
}

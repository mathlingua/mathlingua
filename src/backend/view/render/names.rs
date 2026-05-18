/// Extracts the primary subject name from a `Defines:`-compatible specification.
fn primary_is_or_spec_name(spec: &IsOrSpec) -> Option<String> {
    match spec {
        IsOrSpec::Is(statement) => primary_is_statement_name(statement),
        IsOrSpec::Spec(statement) => primary_spec_subject_name(&statement.subject),
    }
}

/// Extracts the primary subject name from a `Refines:`-compatible specification.
fn primary_is_or_refined_spec_name(spec: &IsOrRefinedStatementSpec) -> Option<String> {
    match spec {
        IsOrRefinedStatementSpec::Is(statement) => primary_is_statement_name(statement),
        IsOrRefinedStatementSpec::Spec(statement) => primary_spec_subject_name(&statement.subject),
    }
}

/// Extracts the primary subject name from an `is` statement.
fn primary_is_statement_name(statement: &IsStatement) -> Option<String> {
    primary_is_subject_name(&statement.subject)
}

/// Extracts the first usable name from an `is` subject.
fn primary_is_subject_name(subject: &IsSubject) -> Option<String> {
    match &subject.kind {
        IsSubjectKind::Forms(forms) => forms.iter().find_map(|form| match form {
            IsSubjectForm::Form(form) => primary_form_name(form),
            IsSubjectForm::PlaceholderForm(form) => primary_placeholder_form_name(form),
        }),
        IsSubjectKind::Operator(_) => None,
    }
}

/// Extracts the primary name from a spec subject.
fn primary_spec_subject_name(subject: &SpecSubject) -> Option<String> {
    match &subject.kind {
        SpecSubjectKind::Form(form) => primary_form_name(form),
        SpecSubjectKind::Operator(_) => None,
    }
}

/// Extracts the primary bindable name from a form or declaration.
///
/// The result is used as a substitution key for `written:` templates and group
/// heading rendering.
fn primary_form_name(form: &FormOrDeclaration) -> Option<String> {
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

/// Extracts the primary name from a placeholder form.
fn primary_placeholder_form_name(
    form: &crate::frontend::formulation::ast::PlaceholderForm,
) -> Option<String> {
    match &form.kind {
        crate::frontend::formulation::ast::PlaceholderFormKind::Placeholder(placeholder) => {
            Some(placeholder.name.clone())
        }
        crate::frontend::formulation::ast::PlaceholderFormKind::Function {
            placeholder, ..
        } => Some(placeholder.name.clone()),
    }
}


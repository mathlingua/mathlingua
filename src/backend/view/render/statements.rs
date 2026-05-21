//! Rendering for `is` statements and related specification forms.

use super::*;

/// Renders an `is` statement or refined-capable specification as LaTeX.
pub(super) fn render_is_or_refined_spec(
    spec: &IsOrRefinedStatementSpec,
    registry: &RenderRegistry,
) -> String {
    match spec {
        IsOrRefinedStatementSpec::Is(statement) => render_is_statement(statement, registry),
        IsOrRefinedStatementSpec::Spec(statement) => format!(
            "{} {} {}",
            render_spec_subject(&statement.subject),
            render_quoted_operator(&statement.operator),
            escape_math_identifier(&statement.name)
        ),
    }
}

/// Renders a parsed `is` statement, preserving subject-aware written templates.
pub(super) fn render_is_statement(statement: &IsStatement, registry: &RenderRegistry) -> String {
    let subject_latex = render_is_subject(&statement.subject);
    match &statement.ty {
        TypeExpression::Command(command) => {
            render_is_command_with_subject_latex(subject_latex, command, registry)
        }
        TypeExpression::RefinedCommand(command) => {
            render_is_refined_command_with_subject_latex(subject_latex, command, registry)
        }
    }
}

/// Renders the subject side of an `is` statement.
pub(super) fn render_is_subject(subject: &IsSubject) -> String {
    match &subject.kind {
        IsSubjectKind::Forms(forms) => forms
            .iter()
            .map(|form| match form {
                IsSubjectForm::Form(form) => render_form_or_declaration(form),
                IsSubjectForm::PlaceholderForm(form) => render_placeholder_form(form),
            })
            .collect::<Vec<_>>()
            .join(", "),
        IsSubjectKind::Operator(operator) => render_operator_text(&operator.text),
    }
}

/// Renders the subject side of a specification statement.
pub(super) fn render_spec_subject(subject: &SpecSubject) -> String {
    match &subject.kind {
        SpecSubjectKind::Form(form) => render_form_or_declaration(form),
        SpecSubjectKind::Operator(operator) => render_operator_text(&operator.text),
    }
}

/// Renders a form or declaration as math-mode LaTeX.
///
/// Definitions can use these forms both as the thing being described and as
/// command parameters.  Placeholder suffixes are hidden to produce readable math
/// while preserving enough structure for substitutions.
pub(super) fn render_form_or_declaration(form: &FormOrDeclaration) -> String {
    match &form.kind {
        FormOrDeclarationKind::Name(name) => escape_math_identifier(name),
        FormOrDeclarationKind::FunctionDeclaration { name, form } => {
            let name = name.as_ref().unwrap_or(&form.name);
            let args = form
                .magnetic_placeholder
                .iter()
                .map(|placeholder| render_form_placeholder_name(&placeholder.name))
                .chain(
                    form.placeholders
                        .iter()
                        .map(|placeholder| render_form_placeholder_name(&placeholder.name)),
                )
                .collect::<Vec<_>>()
                .join(", ");
            if args.is_empty() {
                escape_math_identifier(name)
            } else {
                format!("{}({})", escape_math_identifier(name), args)
            }
        }
        FormOrDeclarationKind::TupleDeclaration { name, form } => {
            let rendered = form
                .elements
                .iter()
                .map(|element| match element {
                    TupleFormElement::Form(form) => render_form_or_declaration(form),
                    TupleFormElement::Operator(operator) => render_operator_text(&operator.text),
                })
                .collect::<Vec<_>>()
                .join(", ");
            match name {
                Some(name) => format!(
                    "{} := \\left({rendered}\\right)",
                    escape_math_identifier(name)
                ),
                None => format!("\\left({rendered}\\right)"),
            }
        }
        FormOrDeclarationKind::SetDeclaration { name, form } => {
            let rendered = format!(
                "\\left\\{{{}\\right\\}}",
                render_placeholder_form(&form.placeholder_form)
            );
            match name {
                Some(name) => format!("{} := {rendered}", escape_math_identifier(name)),
                None => rendered,
            }
        }
        FormOrDeclarationKind::InfixOperator {
            left,
            operator,
            right,
        } => format!(
            "{} {} {}",
            render_form_placeholder_name(&left.name),
            render_operator_text(&operator.text),
            render_form_placeholder_name(&right.name)
        ),
        FormOrDeclarationKind::PrefixOperator {
            operator,
            placeholder,
        } => format!(
            "{}{}",
            render_operator_text(&operator.text),
            render_form_placeholder_name(&placeholder.name)
        ),
        FormOrDeclarationKind::PostfixOperator {
            placeholder,
            operator,
        } => format!(
            "{}{}",
            render_form_placeholder_name(&placeholder.name),
            render_operator_text(&operator.text)
        ),
    }
}

/// Renders an `is` relationship whose type is a normal command expression.
pub(super) fn render_is_command(
    subject: &Expression,
    command: &CommandExpression,
    registry: &RenderRegistry,
) -> String {
    let subject_latex = render_expression(subject, registry);
    render_is_command_with_subject_latex(subject_latex, command, registry)
}

/// Renders an `is` relationship whose type is a refined command expression.
pub(super) fn render_is_refined_command(
    subject: &Expression,
    command: &RefinedCommandExpression,
    registry: &RenderRegistry,
) -> String {
    let subject_latex = render_expression(subject, registry);
    render_is_refined_command_with_subject_latex(subject_latex, command, registry)
}

/// Renders a normal command type after the subject has already been rendered.
///
/// If the command has a `written:` template that includes the subject placeholder,
/// the written template replaces the whole `subject is type` phrase.  Otherwise
/// the subject is kept and the command is rendered after `is`.
pub(super) fn render_is_command_with_subject_latex(
    subject_latex: String,
    command: &CommandExpression,
    registry: &RenderRegistry,
) -> String {
    let signature = command_expression_signature(command);
    let Some(render) = registry.commands.get(&signature) else {
        return format!(
            "{} \\textrm{{ is }} {}",
            subject_latex,
            render_command_expression(command, registry)
        );
    };

    let substitutions =
        command_substitutions(command, render, Some(subject_latex.clone()), registry);

    if let Some(written) = &render.written {
        let includes_subject = render
            .subject_variable
            .as_ref()
            .is_some_and(|name| template_contains_placeholder(written, name));
        let rendered = substitute_math_template(written, &substitutions);
        if includes_subject {
            rendered
        } else {
            format!("{subject_latex} \\textrm{{ is }} {rendered}")
        }
    } else {
        format!(
            "{} \\textrm{{ is }} {}",
            subject_latex,
            render_called_template(&render.called, &substitutions)
        )
    }
}

/// Renders a refined command type after the subject has already been rendered.
pub(super) fn render_is_refined_command_with_subject_latex(
    subject_latex: String,
    command: &RefinedCommandExpression,
    registry: &RenderRegistry,
) -> String {
    format!(
        "{} \\textrm{{ is }} {}",
        subject_latex,
        render_refined_command_called(command, registry)
    )
}

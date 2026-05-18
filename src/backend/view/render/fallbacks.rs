use super::*;

/// Renders an unknown command chain in a readable escaped fallback form.
pub(super) fn render_command_like(chain: &Chain, _registry: &RenderRegistry) -> String {
    format!("\\backslash{}", escape_latex_math(&format_chain(chain)))
}

/// Renders a binary operator as LaTeX.
pub(super) fn render_binary_operator(operator: &BinaryOperator) -> String {
    match operator {
        BinaryOperator::Equality(operator)
        | BinaryOperator::Special(operator)
        | BinaryOperator::Add(operator)
        | BinaryOperator::Subtract(operator)
        | BinaryOperator::Multiply(operator)
        | BinaryOperator::Divide(operator)
        | BinaryOperator::Power(operator) => render_operator_text(&operator.text),
        BinaryOperator::Named(operator) => format!("\\textrm{{ {} }}", operator.name),
    }
}

/// Renders an operator token that is already part of parsed math syntax.
pub(super) fn render_operator_text(operator: &str) -> String {
    match operator {
        "*" => "\\ast".to_string(),
        _ => escape_latex_math(operator),
    }
}

/// Renders a quoted operator as a temporary LaTeX command.
///
/// This is a bridge until full type-checking resolves quoted operators
/// semantically.  For example, `"in"` renders as `\in`.
pub(super) fn render_quoted_operator(operator: &str) -> String {
    // Temporary rendering until full type-checking can resolve quoted operators semantically.
    format!("\\{}", escape_latex_command_name(operator))
}

/// Renders subset/index-call syntax as bracketed LaTeX.
pub(super) fn render_subset_call(call: &crate::frontend::formulation::ast::SubsetCall) -> String {
    match call {
        crate::frontend::formulation::ast::SubsetCall::One { target, first, .. } => {
            format!(
                "{}[{}]",
                escape_math_identifier(target),
                escape_math_identifier(first)
            )
        }
        crate::frontend::formulation::ast::SubsetCall::Two {
            target,
            first,
            second,
            ..
        } => format!(
            "{}[{}, {}]",
            escape_math_identifier(target),
            escape_math_identifier(first),
            escape_math_identifier(second)
        ),
        crate::frontend::formulation::ast::SubsetCall::Nested {
            target,
            outer,
            inner_target,
            ..
        } => format!(
            "{}[{}[{}]]",
            escape_math_identifier(target),
            escape_math_identifier(outer),
            escape_math_identifier(inner_target)
        ),
    }
}

/// Renders a placeholder form while hiding placeholder suffix markers.
pub(super) fn render_placeholder_form(
    form: &crate::frontend::formulation::ast::PlaceholderForm,
) -> String {
    match &form.kind {
        crate::frontend::formulation::ast::PlaceholderFormKind::Placeholder(placeholder) => {
            render_form_placeholder_name(&placeholder.name)
        }
        crate::frontend::formulation::ast::PlaceholderFormKind::Function {
            placeholder,
            arguments,
        } => {
            let arguments = arguments
                .iter()
                .map(|argument| render_form_placeholder_name(&argument.name))
                .collect::<Vec<_>>()
                .join(", ");
            format!(
                "{}({arguments})",
                render_form_placeholder_name(&placeholder.name)
            )
        }
    }
}

/// Renders a form placeholder name, trimming trailing placeholder underscores.
pub(super) fn render_form_placeholder_name(name: &str) -> String {
    let trimmed = name.trim_end_matches('_');
    if trimmed.is_empty() {
        escape_math_identifier(name)
    } else {
        escape_math_identifier(trimmed)
    }
}

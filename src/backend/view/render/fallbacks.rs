use super::*;

pub(super) fn render_command_like(chain: &Chain, _registry: &RenderRegistry) -> String {
    format!("\\backslash{}", escape_latex_math(&format_chain(chain)))
}

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

pub(super) fn render_operator_text(operator: &str) -> String {
    match operator {
        "*" => "\\ast".to_string(),
        "!=" => "\\ne".to_string(),
        _ => escape_latex_math(operator),
    }
}

pub(super) fn render_quoted_operator(operator: &str) -> String {
    // Temporary rendering until full type-checking can resolve quoted operators semantically.
    format!("\\{}", escape_latex_command_name(operator))
}

pub(super) fn render_subset_call(call: &SubsetCall, registry: &RenderRegistry) -> String {
    match call {
        SubsetCall::One { target, first, .. } => {
            format!(
                "{}[{}]",
                escape_math_identifier(target, registry),
                escape_math_identifier(first, registry)
            )
        }
        SubsetCall::Two {
            target,
            first,
            second,
            ..
        } => format!(
            "{}[{}, {}]",
            escape_math_identifier(target, registry),
            escape_math_identifier(first, registry),
            escape_math_identifier(second, registry)
        ),
        SubsetCall::Nested {
            target,
            outer,
            inner_target,
            ..
        } => format!(
            "{}[{}[{}]]",
            escape_math_identifier(target, registry),
            escape_math_identifier(outer, registry),
            escape_math_identifier(inner_target, registry)
        ),
    }
}

pub(super) fn render_placeholder_form(form: &PlaceholderForm, registry: &RenderRegistry) -> String {
    match &form.kind {
        PlaceholderFormKind::Placeholder(placeholder) => {
            render_form_placeholder_name(&placeholder.name, registry)
        }
        PlaceholderFormKind::Function {
            placeholder,
            arguments,
        } => {
            let arguments = arguments
                .iter()
                .map(|argument| render_form_placeholder_name(&argument.name, registry))
                .collect::<Vec<_>>()
                .join(", ");
            format!(
                "{}({arguments})",
                render_form_placeholder_name(&placeholder.name, registry)
            )
        }
    }
}

pub(super) fn render_form_placeholder_name(name: &str, registry: &RenderRegistry) -> String {
    let trimmed = name.trim_end_matches('_');
    if trimmed.is_empty() {
        escape_math_identifier(name, registry)
    } else {
        escape_math_identifier(trimmed, registry)
    }
}

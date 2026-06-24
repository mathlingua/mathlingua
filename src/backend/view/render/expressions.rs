use super::*;

pub(super) fn render_expression(expression: &Expression, registry: &RenderRegistry) -> String {
    match &expression.kind {
        ExpressionKind::Name(name) => escape_math_identifier(name),
        ExpressionKind::FunctionCall { name, arguments } => {
            let args = arguments
                .iter()
                .map(|argument| render_expression(argument, registry))
                .collect::<Vec<_>>()
                .join(", ");
            format!("{}({})", escape_math_identifier(name), args)
        }
        ExpressionKind::FunctionNamedCall { name, elements } => {
            let args = elements
                .iter()
                .map(|element| render_expression(&element.expression, registry))
                .collect::<Vec<_>>()
                .join(", ");
            format!("{}({})", escape_math_identifier(name), args)
        }
        ExpressionKind::Tuple(elements) => {
            let values = elements
                .iter()
                .map(|element| match element {
                    TupleExpressionElement::Expression(expression) => {
                        render_expression(expression, registry)
                    }
                    TupleExpressionElement::Operator(operator) => {
                        render_operator_text(&operator.text)
                    }
                })
                .collect::<Vec<_>>()
                .join(", ");
            format!("\\left({values}\\right)")
        }
        ExpressionKind::Set(set) => render_set_expression(set, registry),
        ExpressionKind::Grouped { expression, .. } => {
            format!("\\left({}\\right)", render_expression(expression, registry))
        }
        ExpressionKind::Labeled { expression, .. } => render_expression(expression, registry),
        ExpressionKind::SubsetCall(call) => render_subset_call(call),
        ExpressionKind::Command(command) => render_command_expression(command, registry),
        ExpressionKind::InfixCommand {
            left,
            command,
            right,
        } => format!(
            "{} {} {}",
            render_expression(left, registry),
            render_command_like(&command.chain, registry),
            render_expression(right, registry)
        ),
        ExpressionKind::Prefix {
            operator,
            expression,
        } => match operator {
            UnaryOperator::Arithmetic(operator) => format!(
                "{}{}",
                render_operator_text(&operator.text),
                render_expression(expression, registry)
            ),
        },
        ExpressionKind::Binary {
            left,
            operator,
            right,
        } => format!(
            "{} {} {}",
            render_expression(left, registry),
            render_binary_operator(operator),
            render_expression(right, registry)
        ),
        ExpressionKind::SpecStatement(statement) => render_spec_statement(statement, registry),
        ExpressionKind::IsPredicate { subject, command } => format!(
            "{} \\textrm{{ is }} {}",
            render_expression(subject, registry),
            render_predicate_command_expression(command, registry)
        ),
        ExpressionKind::IsNotPredicate { subject, command } => format!(
            "{} \\textrm{{ is not }} {}",
            render_expression(subject, registry),
            render_predicate_command_expression(command, registry)
        ),
        ExpressionKind::IsType { subject, ty } => match ty {
            TypeExpression::Command(command) => render_is_command(subject, command, registry),
            TypeExpression::RefinedCommand(command) => {
                render_is_refined_command(subject, command, registry)
            }
            TypeExpression::Function(function_type) => format!(
                "{} \\textrm{{ is }} {}",
                render_expression(subject, registry),
                render_function_type(function_type, registry)
            ),
        },
    }
}

pub(super) fn render_set_expression(set: &SetExpression, registry: &RenderRegistry) -> String {
    let target = render_placeholder_form(&set.target);
    let spec = render_expression(&set.spec, registry);

    match &set.predicate {
        Some(predicate) => format!(
            "\\left\\{{ {target} \\: : \\: {spec} \\: | \\: {} \\right\\}}",
            render_expression(predicate, registry)
        ),
        None => format!("\\left\\{{ {target} \\: : \\: {spec} \\right\\}}"),
    }
}

pub(super) fn render_spec_statement(
    statement: &SpecStatement,
    registry: &RenderRegistry,
) -> String {
    format!(
        "{} {} {}",
        render_expression(&statement.subject, registry),
        render_quoted_operator(&statement.operator),
        escape_math_identifier(&statement.name)
    )
}

pub(super) fn render_simple_set_spec_latex(
    text: &str,
    registry: &RenderRegistry,
) -> Option<String> {
    let trimmed = text.trim();
    let inner = trimmed.strip_prefix('{')?.strip_suffix('}')?;
    let (head, predicate) = match split_once_top_level(inner, '|') {
        Some((head, predicate)) => (head, Some(predicate)),
        None => (inner, None),
    };
    let (target, spec) = split_once_top_level(head, ':')?;
    let target = target.trim();
    let spec = spec.trim();
    if target.is_empty() || spec.is_empty() {
        return None;
    }

    let target = render_latex_fragment(target, registry);
    let spec = render_latex_fragment(spec, registry);

    match predicate.map(str::trim).filter(|value| !value.is_empty()) {
        Some(predicate) => Some(format!(
            "\\left\\{{ {target} \\: : \\: {spec} \\: | \\: {} \\right\\}}",
            render_latex_fragment(predicate, registry)
        )),
        None => Some(format!("\\left\\{{ {target} \\: : \\: {spec} \\right\\}}")),
    }
}

pub(super) fn render_latex_fragment(text: &str, registry: &RenderRegistry) -> String {
    render_parsed_formulation_latex(text, registry)
        .unwrap_or_else(|| escape_latex_math(text.trim()))
}

pub(super) fn split_once_top_level(input: &str, delimiter: char) -> Option<(&str, &str)> {
    let mut paren_depth = 0usize;
    let mut brace_depth = 0usize;
    let mut bracket_depth = 0usize;
    let mut in_quote = false;

    for (index, ch) in input.char_indices() {
        match ch {
            '"' => in_quote = !in_quote,
            '(' if !in_quote => paren_depth += 1,
            ')' if !in_quote => paren_depth = paren_depth.saturating_sub(1),
            '{' if !in_quote => brace_depth += 1,
            '}' if !in_quote => brace_depth = brace_depth.saturating_sub(1),
            '[' if !in_quote => bracket_depth += 1,
            ']' if !in_quote => bracket_depth = bracket_depth.saturating_sub(1),
            _ if ch == delimiter
                && !in_quote
                && paren_depth == 0
                && brace_depth == 0
                && bracket_depth == 0 =>
            {
                let after_delimiter = index + ch.len_utf8();
                return Some((&input[..index], &input[after_delimiter..]));
            }
            _ => {}
        }
    }

    None
}

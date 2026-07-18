use super::*;

pub(super) fn render_expression(expression: &Expression, registry: &RenderRegistry) -> String {
    match &expression.kind {
        ExpressionKind::Name(name) => escape_math_identifier(name, registry),
        // The `?` on an inferred parameter is authoring-only; render the bare name.
        ExpressionKind::InferredName(name) => escape_math_identifier(name, registry),
        ExpressionKind::FunctionCall { name, arguments } => {
            if let Some(rendered) = render_provided_function_call(name, arguments, registry) {
                return rendered;
            }
            let args = arguments
                .iter()
                .map(|argument| render_expression(argument, registry))
                .collect::<Vec<_>>()
                .join(", ");
            format!("{}({})", escape_math_identifier(name, registry), args)
        }
        ExpressionKind::FunctionNamedCall { name, elements } => {
            let args = elements
                .iter()
                .map(|element| render_expression(&element.expression, registry))
                .collect::<Vec<_>>()
                .join(", ");
            format!("{}({})", escape_math_identifier(name, registry), args)
        }
        ExpressionKind::MemberCall {
            owner,
            name,
            arguments,
        } => {
            let args = arguments
                .iter()
                .map(|argument| render_expression(argument, registry))
                .collect::<Vec<_>>()
                .join(", ");
            format!(
                "{}.{}({})",
                render_expression(owner, registry),
                escape_math_identifier(name, registry),
                args
            )
        }
        ExpressionKind::MemberAccess { owner, name } => format!(
            "{}.{}",
            render_expression(owner, registry),
            escape_math_identifier(name, registry)
        ),
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
        ExpressionKind::Grouped {
            expression,
            dot_delimited,
        } => {
            let rendered = render_expression(expression, registry);
            if *dot_delimited {
                rendered
            } else {
                format!("\\left({rendered}\\right)")
            }
        }
        ExpressionKind::Labeled { expression, .. } => render_expression(expression, registry),
        ExpressionKind::SubsetCall(call) => render_subset_call(call, registry),
        ExpressionKind::Command(command) => render_command_expression(command, registry),
        ExpressionKind::BuiltinCommand(command) => {
            render_builtin_command_expression(command, registry)
        }
        ExpressionKind::InfixCommand {
            left,
            command,
            right,
        } => render_infix_command_expression(left, command, right, registry),
        ExpressionKind::InfixSpecStatement { left, spec, right } => {
            render_infix_spec_expression(left, spec, right, registry)
        }
        ExpressionKind::Prefix {
            operator,
            expression,
        } => match operator {
            UnaryOperator::Arithmetic(operator) | UnaryOperator::Named(operator) => format!(
                "{}{}",
                render_operator_text(&operator.text),
                render_expression(expression, registry)
            ),
        },
        ExpressionKind::Postfix {
            expression,
            operator,
        } => format!(
            "{}{}",
            render_expression(expression, registry),
            render_operator_text(&operator.text)
        ),
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
        ExpressionKind::SpecStatement(statement) | ExpressionKind::SpecPredicate(statement) => {
            render_spec_statement(statement, registry)
        }
        ExpressionKind::MemberOf {
            subject,
            collection,
        } => format!(
            "{} \\textrm{{ member of }} {}",
            render_expression(subject, registry),
            render_expression(collection, registry)
        ),
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
        ExpressionKind::IsBuiltinPredicate { subject, ty } => format!(
            "{} \\textrm{{ is }} {}",
            render_expression(subject, registry),
            render_type_expression(ty, registry)
        ),
        ExpressionKind::IsNotBuiltinPredicate { subject, ty } => format!(
            "{} \\textrm{{ is not }} {}",
            render_expression(subject, registry),
            render_type_expression(ty, registry)
        ),
        ExpressionKind::IsRefinedPredicate { subject, command } => format!(
            "{} \\textrm{{ is }} {}",
            render_expression(subject, registry),
            render_refined_command_called(command, registry)
        ),
        ExpressionKind::IsNotRefinedPredicate { subject, command } => format!(
            "{} \\textrm{{ is not }} {}",
            render_expression(subject, registry),
            render_refined_command_called(command, registry)
        ),
        ExpressionKind::IsType { subject, ty } => match ty {
            TypeExpression::Builtin { chain, .. } => format!(
                "{} \\textrm{{ is }} {}",
                render_expression(subject, registry),
                render_builtin_type_chain(chain)
            ),
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
        ExpressionKind::Cast {
            expression,
            ty,
            hard,
        } => format!(
            "{} \\textrm{{ {} }} {}",
            render_expression(expression, registry),
            if *hard { "as!" } else { "as" },
            render_type_expression(ty, registry)
        ),
    }
}

fn render_builtin_command_expression(
    command: &BuiltinCommandExpression,
    registry: &RenderRegistry,
) -> String {
    let head = render_builtin_arguments(&command.head_args, registry);
    let tail = |name: &str| {
        command
            .tail
            .iter()
            .filter(|tail| format_chain(&tail.chain) == name)
            .flat_map(|tail| render_builtin_arguments(&tail.args, registry))
            .collect::<Vec<_>>()
    };

    match format_chain(&command.chain).as_str() {
        "not" => format!(
            "\\neg {}",
            head.first()
                .cloned()
                .unwrap_or_else(|| "\\cdots".to_owned())
        ),
        "and" | "allOf" => head.join(" \\textrm{ and } "),
        "or" | "anyOf" => head.join(" \\textrm{ or } "),
        "oneOf" => format!("\\textrm{{one of }} {}", head.join(", ")),
        "exists" => render_builtin_quantifier("\\exists", &head, &tail("suchThat")),
        "existsUnique" => render_builtin_quantifier("\\exists!", &head, &tail("suchThat")),
        "forAll" | "forall" => {
            let where_ = tail("where");
            let then = tail("then");
            let mut rendered = render_builtin_quantifier("\\forall", &head, &where_);
            if !then.is_empty() {
                rendered.push_str(" \\textrm{ then } ");
                rendered.push_str(&then.join(" \\textrm{ and } "));
            }
            rendered
        }
        "if" => {
            let then = tail("then");
            let mut rendered = format!("\\textrm{{if }} {}", head.join(" \\textrm{ and } "));
            if !then.is_empty() {
                rendered.push_str(" \\textrm{ then } ");
                rendered.push_str(&then.join(" \\textrm{ and } "));
            }
            rendered
        }
        "have" => {
            let iff = tail("iff");
            format!(
                "{} \\Longleftrightarrow {}",
                head.join(" \\textrm{ and } "),
                iff.join(" \\textrm{ and } ")
            )
        }
        "given" => {
            let where_ = tail("where");
            let then = tail("then");
            let mut rendered = format!("\\textrm{{given }} {}", head.join("; "));
            if !where_.is_empty() {
                rendered.push_str(" \\textrm{ where } ");
                rendered.push_str(&where_.join(" \\textrm{ and } "));
            }
            if !then.is_empty() {
                rendered.push_str(" \\textrm{ then } ");
                rendered.push_str(&then.join(" \\textrm{ and } "));
            }
            rendered
        }
        "piecewise" => {
            let if_ = tail("if");
            let then = tail("then");
            let else_ = tail("else");
            let mut rendered = head.join(" \\textrm{ and } ");
            if !if_.is_empty() {
                if !rendered.is_empty() {
                    rendered.push(' ');
                }
                rendered.push_str("\\textrm{if } ");
                rendered.push_str(&if_.join(" \\textrm{ and } "));
            }
            if !then.is_empty() {
                rendered.push_str(" \\textrm{ then } ");
                rendered.push_str(&then.join(" \\textrm{ and } "));
            }
            if !else_.is_empty() {
                rendered.push_str(" \\textrm{ else } ");
                rendered.push_str(&else_.join(" \\textrm{ and } "));
            }
            rendered
        }
        other => format!(
            "\\operatorname{{{}}}\\left({}\\right)",
            escape_latex_math(other),
            head.join("; ")
        ),
    }
}

fn render_builtin_quantifier(symbol: &str, head: &[String], such_that: &[String]) -> String {
    let mut rendered = format!("{symbol} {}", head.join("; "));
    if !such_that.is_empty() {
        rendered.push_str(" \\textrm{ such that } ");
        rendered.push_str(&such_that.join(" \\textrm{ and } "));
    }
    rendered
}

fn render_builtin_arguments(
    groups: &[BuiltinCommandArgs],
    registry: &RenderRegistry,
) -> Vec<String> {
    groups
        .iter()
        .flat_map(|group| group.arguments.iter())
        .map(|argument| render_builtin_argument(argument, registry))
        .collect()
}

fn render_builtin_argument(argument: &BuiltinCommandArgument, registry: &RenderRegistry) -> String {
    match argument {
        BuiltinCommandArgument::Text(argument) => {
            if let Ok(statement) =
                crate::frontend::formulation::parse_refined_declaration_statement(argument)
            {
                return render_declaration_statement(&statement, registry);
            }
            if let Ok(expression) = crate::frontend::formulation::parse_expression(argument) {
                return render_expression(&expression, registry);
            }
            format!("\\textrm{{{}}}", escape_latex_text(argument.trim()))
        }
        BuiltinCommandArgument::Declaration(statement) => {
            render_declaration_statement(statement, registry)
        }
        BuiltinCommandArgument::Expression(expression) => render_expression(expression, registry),
    }
}

fn render_provided_function_call(
    name: &str,
    arguments: &[Expression],
    registry: &RenderRegistry,
) -> Option<String> {
    let render = registry.provided_calls.iter().find(|render| {
        render.function_name == name && render.parameters.len() == arguments.len()
    })?;
    let mut substitutions = HashMap::new();
    substitutions.insert(
        render.owner_subject.clone(),
        escape_math_identifier(name, registry),
    );
    for (parameter, argument) in render.parameters.iter().zip(arguments) {
        let rendered_argument = render_expression(argument, registry);
        substitutions.insert(parameter.clone(), rendered_argument.clone());
        if !parameter.ends_with('_') {
            substitutions.insert(format!("{parameter}_"), rendered_argument);
        }
    }
    Some(substitute_math_template(&render.written, &substitutions))
}

pub(super) fn render_infix_spec_like(spec: &InfixSpec, registry: &RenderRegistry) -> String {
    render_command_like(&spec.chain, registry)
}

pub(super) fn render_set_expression(set: &SetExpression, registry: &RenderRegistry) -> String {
    let target = render_set_target(&set.target, registry);
    let spec = set
        .specs
        .iter()
        .map(|spec| render_expression(spec, registry))
        .collect::<Vec<_>>()
        .join(", ");

    match &set.predicate {
        Some(predicate) => format!(
            "\\left\\{{ {target} \\: : \\: {spec} \\: | \\: {} \\right\\}}",
            render_set_predicate(predicate, registry)
        ),
        None => format!("\\left\\{{ {target} \\: : \\: {spec} \\right\\}}"),
    }
}

pub(super) fn render_set_target(target: &SetTarget, registry: &RenderRegistry) -> String {
    match &target.kind {
        SetTargetKind::Name(name) => escape_math_identifier(name, registry),
        SetTargetKind::PlaceholderForm(form) => render_placeholder_form(form, registry),
        SetTargetKind::Alias { name, target } => {
            format!(
                "{} := {}",
                escape_math_identifier(name, registry),
                render_set_target(target, registry)
            )
        }
        SetTargetKind::Introduction { name, target } => {
            format!(
                "{} ::= {}",
                escape_math_identifier(name, registry),
                render_set_target(target, registry)
            )
        }
        SetTargetKind::Function { name, arguments } => {
            let arguments = arguments
                .iter()
                .map(|target| render_set_target(target, registry))
                .collect::<Vec<_>>()
                .join(", ");
            format!("{}({arguments})", escape_math_identifier(name, registry))
        }
        SetTargetKind::Tuple(elements) => {
            let elements = elements
                .iter()
                .map(|element| match element {
                    SetTargetElement::Target(target) => render_set_target(target, registry),
                    SetTargetElement::Operator(operator) => render_operator_text(&operator.text),
                })
                .collect::<Vec<_>>()
                .join(", ");
            format!("\\left({elements}\\right)")
        }
    }
}

fn render_set_predicate(predicate: &SetPredicate, registry: &RenderRegistry) -> String {
    match predicate {
        SetPredicate::Expression(expression) => render_expression(expression, registry),
        SetPredicate::Definition { target, value, .. } => format!(
            "{} := {}",
            render_set_target(target, registry),
            render_expression(value, registry)
        ),
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
        escape_math_identifier(&statement.name, registry)
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

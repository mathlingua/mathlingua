use super::*;

pub(super) fn render_expression(expression: &Expression, registry: &RenderRegistry) -> String {
    match &expression.kind {
        ExpressionKind::Name(name) => escape_math_identifier(name, registry),
        // The `?` on an inferred parameter is authoring-only; render the bare name.
        ExpressionKind::InferredName(name) => escape_math_identifier(name, registry),
        ExpressionKind::VariadicSlice(slice) => render_variadic_slice(slice, registry),
        ExpressionKind::VariadicAssignment { target, value } => render_variadic_relation(
            VariadicRenderOperand::Slice(render_variadic_slice_parts(target, registry)),
            ":=",
            render_variadic_operand(value, registry),
        )
        .expect("a variadic assignment always has a slice operand"),
        ExpressionKind::FunctionCall { name, arguments } => {
            if let Some(rendered) = render_documented_mapping_call(name, arguments, registry) {
                return rendered;
            }
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
            let direct = matches!(owner.kind, ExpressionKind::Command(_));
            if !direct
                && let Some(rendered) = render_provided_member(owner, name, arguments, registry)
            {
                return rendered;
            }
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
        ExpressionKind::MemberAccess { owner, name } => {
            let direct = matches!(owner.kind, ExpressionKind::Command(_));
            if !direct && let Some(rendered) = render_provided_member(owner, name, &[], registry) {
                return rendered;
            }
            format!(
                "{}.{}",
                render_expression(owner, registry),
                escape_math_identifier(name, registry)
            )
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
        ExpressionKind::IndexedCall(call) => format!(
            "{}[{}]",
            escape_math_identifier(&call.target, registry),
            call.indices
                .iter()
                .map(|index| render_expression(index, registry))
                .collect::<Vec<_>>()
                .join(", ")
        ),
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
        } => render_expression_relation(left, &render_binary_operator(operator), right, registry),
        ExpressionKind::SpecStatement(statement) | ExpressionKind::SpecPredicate(statement) => {
            render_spec_statement(statement, registry)
        }
        ExpressionKind::SpecStatementExpr {
            subject,
            operator,
            target,
        } => render_subject_relation(
            subject,
            &render_quoted_operator(operator),
            &render_expression(target, registry),
            registry,
        ),
        ExpressionKind::SpecLiteral(literal) => match &literal.form {
            SpecLiteralForm::Is(ty) => {
                format!(
                    "\\cdot \\textrm{{ is }} {}",
                    render_type_expression(ty, registry)
                )
            }
            SpecLiteralForm::Spec { operator, target } => format!(
                "\\cdot {} {}",
                render_quoted_operator(operator),
                render_expression(target, registry)
            ),
        },
        ExpressionKind::Satisfies { subject, spec } => format!(
            "{} \\textrm{{ satisfies }} {}",
            render_expression(subject, registry),
            render_expression(spec, registry)
        ),
        ExpressionKind::Mapping { lhs, rhs } => format!(
            "{} \\mapsto {}",
            render_expression(lhs, registry),
            render_expression(rhs, registry)
        ),
        ExpressionKind::MemberOf {
            subject,
            collection,
        } => format!(
            "{} \\textrm{{ member of }} {}",
            render_expression(subject, registry),
            render_expression(collection, registry)
        ),
        ExpressionKind::IsPredicate { subject, command } => render_subject_relation(
            subject,
            "\\textrm{ is }",
            &render_predicate_command_expression(command, registry),
            registry,
        ),
        ExpressionKind::IsNotPredicate { subject, command } => render_subject_relation(
            subject,
            "\\textrm{ is not }",
            &render_predicate_command_expression(command, registry),
            registry,
        ),
        ExpressionKind::IsBuiltinPredicate { subject, ty } => render_subject_relation(
            subject,
            "\\textrm{ is }",
            &render_type_expression(ty, registry),
            registry,
        ),
        ExpressionKind::IsNotBuiltinPredicate { subject, ty } => render_subject_relation(
            subject,
            "\\textrm{ is not }",
            &render_type_expression(ty, registry),
            registry,
        ),
        ExpressionKind::IsRefinedPredicate { subject, command } => render_subject_relation(
            subject,
            "\\textrm{ is }",
            &render_refined_command_called(command, registry),
            registry,
        ),
        ExpressionKind::IsNotRefinedPredicate { subject, command } => render_subject_relation(
            subject,
            "\\textrm{ is not }",
            &render_refined_command_called(command, registry),
            registry,
        ),
        ExpressionKind::IsType { subject, ty } => match ty {
            TypeExpression::Builtin { chain, .. } => render_subject_relation(
                subject,
                "\\textrm{ is }",
                &render_builtin_type_chain(chain),
                registry,
            ),
            TypeExpression::Command(command) => direct_variadic_slice(subject)
                .map(|slice| {
                    render_variadic_slice_with(slice, registry, true, |subject_latex| {
                        render_is_command_with_subject_latex(
                            subject_latex.to_owned(),
                            command,
                            registry,
                        )
                    })
                })
                .unwrap_or_else(|| render_is_command(subject, command, registry)),
            TypeExpression::RefinedCommand(command) => direct_variadic_slice(subject)
                .map(|slice| {
                    render_variadic_slice_with(slice, registry, true, |subject_latex| {
                        render_is_refined_command_with_subject_latex(
                            subject_latex.to_owned(),
                            command,
                            registry,
                        )
                    })
                })
                .unwrap_or_else(|| render_is_refined_command(subject, command, registry)),
            TypeExpression::Tuple(_) | TypeExpression::Set(_) => render_subject_relation(
                subject,
                "\\textrm{ is }",
                &render_type_expression(ty, registry),
                registry,
            ),
            TypeExpression::Function(function_type) => render_subject_relation(
                subject,
                "\\textrm{ is }",
                &render_function_type(function_type, registry),
                registry,
            ),
            TypeExpression::Parameter { name, .. } => render_subject_relation(
                subject,
                "\\textrm{ is }",
                &escape_math_identifier(name, registry),
                registry,
            ),
        },
        ExpressionKind::Build { ty, value } => format!(
            "{}{}{}",
            render_type_expression(ty, registry),
            // The build marker is authoring syntax, not mathematical
            // notation. Use an explicit LaTeX space so it remains visible
            // between ordinary math atoms such as `set` and a set literal.
            "\\,",
            render_expression(value, registry)
        ),
    }
}

fn render_documented_mapping_call(
    name: &str,
    arguments: &[Expression],
    registry: &RenderRegistry,
) -> Option<String> {
    let render = registry.mapping_writing.iter().find(|render| {
        render.function_name == name && render.parameters.len() == arguments.len()
    })?;
    let is_mapping_form = arguments
        .iter()
        .zip(&render.parameters)
        .all(|(argument, parameter)| {
            matches!(
                &argument.kind,
                ExpressionKind::Name(argument_name)
                    if argument_name == parameter
                        && argument.span.end.saturating_sub(argument.span.start)
                            == argument_name.len() + if render.magnetic { 2 } else { 1 }
            )
        });
    let template = if is_mapping_form {
        render.mapping_written.as_deref()?
    } else {
        render.invocation_written.as_deref()?
    };
    let mut substitutions = HashMap::new();
    insert_parameter_substitution(
        &mut substitutions,
        &render.function_name,
        escape_math_identifier(name, registry),
    );
    for (parameter, argument) in render.parameters.iter().zip(arguments) {
        insert_parameter_substitution(
            &mut substitutions,
            parameter,
            render_expression(argument, registry),
        );
    }
    Some(substitute_math_template(template, &substitutions))
}

#[derive(Clone)]
enum VariadicRenderPart {
    Element(String),
    Ellipsis,
    RowBreak,
}

enum VariadicRenderOperand {
    Scalar(String),
    Slice(Vec<VariadicRenderPart>),
}

fn direct_variadic_slice(expression: &Expression) -> Option<&VariadicSlice> {
    match &expression.kind {
        ExpressionKind::VariadicSlice(slice) => Some(slice),
        ExpressionKind::Grouped { expression, .. } | ExpressionKind::Labeled { expression, .. } => {
            direct_variadic_slice(expression)
        }
        _ => None,
    }
}

fn render_variadic_operand(
    expression: &Expression,
    registry: &RenderRegistry,
) -> VariadicRenderOperand {
    match direct_variadic_slice(expression) {
        Some(slice) => VariadicRenderOperand::Slice(render_variadic_slice_parts(slice, registry)),
        None => VariadicRenderOperand::Scalar(render_expression(expression, registry)),
    }
}

fn render_variadic_relation(
    left: VariadicRenderOperand,
    relation: &str,
    right: VariadicRenderOperand,
) -> Option<String> {
    let count = match (&left, &right) {
        (VariadicRenderOperand::Slice(left), VariadicRenderOperand::Slice(right)) => {
            if left.len() != right.len() {
                return None;
            }
            left.len()
        }
        (VariadicRenderOperand::Slice(parts), _) | (_, VariadicRenderOperand::Slice(parts)) => {
            parts.len()
        }
        _ => return None,
    };

    let mut rendered = Vec::with_capacity(count);
    for index in 0..count {
        let is_row_break = matches!(
            &left,
            VariadicRenderOperand::Slice(parts)
                if matches!(parts[index], VariadicRenderPart::RowBreak)
        ) || matches!(
            &right,
            VariadicRenderOperand::Slice(parts)
                if matches!(parts[index], VariadicRenderPart::RowBreak)
        );
        if is_row_break {
            rendered.push(VariadicRenderPart::RowBreak);
            continue;
        }
        let left_part = match &left {
            VariadicRenderOperand::Scalar(value) => Some(value.as_str()),
            VariadicRenderOperand::Slice(parts) => match &parts[index] {
                VariadicRenderPart::Element(value) => Some(value.as_str()),
                VariadicRenderPart::Ellipsis | VariadicRenderPart::RowBreak => None,
            },
        };
        let right_part = match &right {
            VariadicRenderOperand::Scalar(value) => Some(value.as_str()),
            VariadicRenderOperand::Slice(parts) => match &parts[index] {
                VariadicRenderPart::Element(value) => Some(value.as_str()),
                VariadicRenderPart::Ellipsis | VariadicRenderPart::RowBreak => None,
            },
        };

        match (left_part, right_part) {
            (Some(left), Some(right)) => rendered.push(VariadicRenderPart::Element(format!(
                "{left} {relation} {right}"
            ))),
            _ => rendered.push(VariadicRenderPart::Ellipsis),
        }
    }

    Some(join_variadic_render_parts(rendered, true))
}

fn render_expression_relation(
    left: &Expression,
    relation: &str,
    right: &Expression,
    registry: &RenderRegistry,
) -> String {
    render_variadic_relation(
        render_variadic_operand(left, registry),
        relation,
        render_variadic_operand(right, registry),
    )
    .unwrap_or_else(|| {
        format!(
            "{} {relation} {}",
            render_expression(left, registry),
            render_expression(right, registry)
        )
    })
}

fn render_subject_relation(
    subject: &Expression,
    relation: &str,
    target: &str,
    registry: &RenderRegistry,
) -> String {
    direct_variadic_slice(subject)
        .map(|slice| {
            render_variadic_slice_with(slice, registry, true, |element| {
                format!("{element} {relation} {target}")
            })
        })
        .unwrap_or_else(|| {
            format!(
                "{} {relation} {target}",
                render_expression(subject, registry)
            )
        })
}

fn render_variadic_slice_with(
    slice: &VariadicSlice,
    registry: &RenderRegistry,
    space_around_ellipsis: bool,
    render_element: impl Fn(&str) -> String,
) -> String {
    let rendered = render_variadic_slice_parts(slice, registry)
        .into_iter()
        .map(|part| match part {
            VariadicRenderPart::Element(element) => {
                VariadicRenderPart::Element(render_element(&element))
            }
            VariadicRenderPart::Ellipsis => VariadicRenderPart::Ellipsis,
            VariadicRenderPart::RowBreak => VariadicRenderPart::RowBreak,
        })
        .collect();
    join_variadic_render_parts(rendered, space_around_ellipsis)
}

fn join_variadic_render_parts(
    parts: Vec<VariadicRenderPart>,
    space_around_ellipsis: bool,
) -> String {
    let mut rendered = String::new();
    let mut previous_was_ellipsis = false;
    let mut previous_was_row_break = false;
    for part in parts {
        if matches!(part, VariadicRenderPart::RowBreak) {
            rendered.push_str("; \\; ");
            previous_was_ellipsis = false;
            previous_was_row_break = true;
            continue;
        }
        let is_ellipsis = matches!(part, VariadicRenderPart::Ellipsis);
        if !rendered.is_empty() && !previous_was_row_break {
            if space_around_ellipsis && (previous_was_ellipsis || is_ellipsis) {
                rendered.push_str(", \\; ");
            } else {
                rendered.push_str(", ");
            }
        }
        match part {
            VariadicRenderPart::Element(element) => rendered.push_str(&element),
            VariadicRenderPart::Ellipsis => rendered.push_str("\\ldots"),
            VariadicRenderPart::RowBreak => unreachable!(),
        }
        previous_was_ellipsis = is_ellipsis;
        previous_was_row_break = false;
    }
    rendered
}

fn render_variadic_slice_parts(
    slice: &VariadicSlice,
    registry: &RenderRegistry,
) -> Vec<VariadicRenderPart> {
    if let Some(dimensions) = &slice.dimensions {
        return render_two_dimensional_variadic_slice_parts(slice, dimensions, registry);
    }
    let name = escape_math_identifier(&slice.name, registry);
    let Some(start) = slice.start else {
        return vec![
            VariadicRenderPart::Element(format!("{name}_{{1}}")),
            VariadicRenderPart::Ellipsis,
            VariadicRenderPart::Element(format!("{name}_{{.}}")),
        ];
    };
    let end = slice.end.as_deref().unwrap_or("n");
    let mut parts = vec![
        VariadicRenderPart::Element(format!("{name}_{{{start}}}")),
        VariadicRenderPart::Ellipsis,
    ];
    if let Some(index) = slice.index.as_deref() {
        parts.push(VariadicRenderPart::Element(format!(
            "{name}_{{{}}}",
            escape_math_identifier(index, registry)
        )));
        parts.push(VariadicRenderPart::Ellipsis);
    }
    parts.push(VariadicRenderPart::Element(format!(
        "{name}_{{{}}}",
        escape_math_identifier(end, registry)
    )));
    parts
}

fn render_two_dimensional_variadic_slice_parts(
    slice: &VariadicSlice,
    dimensions: &VariadicSliceDimensions,
    registry: &RenderRegistry,
) -> Vec<VariadicRenderPart> {
    let name = escape_math_identifier(&slice.name, registry);
    let rows = render_variadic_axis_parts(&dimensions.rows, registry);
    let columns = render_variadic_axis_parts(&dimensions.columns, registry);
    let mut result = Vec::new();
    for (row_offset, row) in rows.iter().enumerate() {
        if row_offset > 0 {
            result.push(VariadicRenderPart::RowBreak);
        }
        match row {
            VariadicRenderPart::Ellipsis => result.push(VariadicRenderPart::Ellipsis),
            VariadicRenderPart::Element(row) => {
                for column in &columns {
                    match column {
                        VariadicRenderPart::Element(column) => result.push(
                            VariadicRenderPart::Element(format!("{name}_{{{row},{column}}}")),
                        ),
                        VariadicRenderPart::Ellipsis => result.push(VariadicRenderPart::Ellipsis),
                        VariadicRenderPart::RowBreak => unreachable!(),
                    }
                }
            }
            VariadicRenderPart::RowBreak => unreachable!(),
        }
    }
    result
}

fn render_variadic_axis_parts(
    axis: &VariadicSliceAxis,
    registry: &RenderRegistry,
) -> Vec<VariadicRenderPart> {
    let element =
        |value: &str| VariadicRenderPart::Element(escape_math_identifier(value, registry));
    match axis {
        VariadicSliceAxis::All => vec![element("1"), VariadicRenderPart::Ellipsis, element(".")],
        VariadicSliceAxis::Index(index) => vec![element(index)],
        VariadicSliceAxis::Range { start, index, end } => {
            let mut parts = vec![element(start), VariadicRenderPart::Ellipsis];
            if let Some(index) = index {
                parts.push(element(index));
                parts.push(VariadicRenderPart::Ellipsis);
            }
            parts.push(element(end));
            parts
        }
    }
}

pub(super) fn render_variadic_slice(slice: &VariadicSlice, registry: &RenderRegistry) -> String {
    render_variadic_slice_with(slice, registry, false, str::to_owned)
}

pub(super) fn render_variadic_parameter(
    parameter: &VariadicParameter,
    registry: &RenderRegistry,
) -> String {
    if let Some(dimensions) = &parameter.dimensions {
        return render_variadic_slice(
            &VariadicSlice {
                span: parameter.span,
                name: parameter.name.clone(),
                start: None,
                index: None,
                end: None,
                dimensions: Some(VariadicSliceDimensions {
                    rows: VariadicSliceAxis::Range {
                        start: dimensions.row_start.to_string(),
                        index: Some(dimensions.row_index.clone()),
                        end: dimensions
                            .row_length
                            .clone()
                            .unwrap_or_else(|| ".".to_owned()),
                    },
                    columns: VariadicSliceAxis::Range {
                        start: dimensions.column_start.to_string(),
                        index: Some(dimensions.column_index.clone()),
                        end: dimensions
                            .column_length
                            .clone()
                            .unwrap_or_else(|| ".".to_owned()),
                    },
                }),
            },
            registry,
        );
    }
    render_variadic_slice(
        &VariadicSlice {
            span: parameter.span,
            name: parameter.name.clone(),
            start: Some(if parameter.index.is_some() {
                parameter.start
            } else {
                1
            }),
            index: parameter.index.clone(),
            end: Some(parameter.length.clone().unwrap_or_else(|| ".".to_owned())),
            dimensions: None,
        },
        registry,
    )
}

/// Returns the symbolic rows used when a 2D header parameter is substituted
/// into its own documented template. Unlike a concrete invocation, the
/// symbolic expansion has one-cell ellipsis rows between its indexed rows.
pub(super) fn render_variadic_parameter_matrix_elements(
    parameter: &VariadicParameter,
    registry: &RenderRegistry,
) -> Option<Vec<Vec<String>>> {
    let dimensions = parameter.dimensions.as_ref()?;
    let slice = VariadicSlice {
        span: parameter.span,
        name: parameter.name.clone(),
        start: None,
        index: None,
        end: None,
        dimensions: Some(VariadicSliceDimensions {
            rows: VariadicSliceAxis::Range {
                start: dimensions.row_start.to_string(),
                index: Some(dimensions.row_index.clone()),
                end: dimensions
                    .row_length
                    .clone()
                    .unwrap_or_else(|| ".".to_owned()),
            },
            columns: VariadicSliceAxis::Range {
                start: dimensions.column_start.to_string(),
                index: Some(dimensions.column_index.clone()),
                end: dimensions
                    .column_length
                    .clone()
                    .unwrap_or_else(|| ".".to_owned()),
            },
        }),
    };

    let mut rows = vec![Vec::new()];
    for part in render_variadic_slice_parts(&slice, registry) {
        match part {
            VariadicRenderPart::Element(value) => rows.last_mut()?.push(value),
            VariadicRenderPart::Ellipsis => rows.last_mut()?.push("\\ldots".to_owned()),
            VariadicRenderPart::RowBreak => rows.push(Vec::new()),
        }
    }
    Some(rows)
}

pub(super) fn render_variadic_parameter_elements(
    parameter: &VariadicParameter,
    registry: &RenderRegistry,
) -> Vec<String> {
    if let Some(dimensions) = &parameter.dimensions {
        let name = escape_math_identifier(&parameter.name, registry);
        let rows = dimensions.row_length.as_deref().unwrap_or(".");
        let columns = dimensions.column_length.as_deref().unwrap_or(".");
        return vec![format!(
            "{name}_{{{},{}}}, \\ldots, {name}_{{{},{}}}",
            dimensions.row_start,
            dimensions.column_start,
            escape_math_identifier(rows, registry),
            escape_math_identifier(columns, registry)
        )];
    }
    let name = escape_math_identifier(&parameter.name, registry);
    let start = if parameter.index.is_some() {
        parameter.start
    } else {
        1
    };
    let mut elements = vec![format!("{name}_{{{start}}}"), "\\ldots".to_owned()];
    if let Some(index) = &parameter.index {
        elements.push(format!(
            "{name}_{{{}}}",
            escape_math_identifier(index, registry)
        ));
        elements.push("\\ldots".to_owned());
    }
    let end = parameter.length.as_deref().unwrap_or(".");
    elements.push(format!(
        "{name}_{{{}}}",
        escape_math_identifier(end, registry)
    ));
    elements
}

fn render_builtin_command_expression(
    command: &BuiltinCommandExpression,
    registry: &RenderRegistry,
) -> String {
    let command_name = format_chain(&command.chain);
    if command_name == "map" {
        return render_variadic_map(command, registry)
            .unwrap_or_else(|| "\\operatorname{map}".to_string());
    }
    if matches!(command_name.as_str(), "leftReduce" | "rightReduce") {
        return render_variadic_reduce(command, registry)
            .unwrap_or_else(|| format!("\\operatorname{{{command_name}}}"));
    }

    let head = render_builtin_arguments(&command.head_args, registry);
    let tail = |name: &str| {
        command
            .tail
            .iter()
            .filter(|tail| format_chain(&tail.chain) == name)
            .flat_map(|tail| render_builtin_arguments(&tail.args, registry))
            .collect::<Vec<_>>()
    };

    match command_name.as_str() {
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
        "let" => {
            let where_ = tail("where");
            let then = tail("then");
            let mut rendered = format!("\\textrm{{let }} {}", head.join("; "));
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

fn render_variadic_map(
    command: &BuiltinCommandExpression,
    registry: &RenderRegistry,
) -> Option<String> {
    let slices = builtin_argument_texts(&command.head_args)
        .flat_map(|text| text.split(','))
        .map(str::trim)
        .filter(|text| !text.is_empty())
        .map(crate::frontend::formulation::parse_expression)
        .map(|expression| match expression.ok()?.kind {
            ExpressionKind::VariadicSlice(slice) => Some(slice),
            _ => None,
        })
        .collect::<Option<Vec<_>>>()?;
    let first = slices.first()?;
    if !slices.iter().all(|slice| {
        slice.start == first.start && slice.index == first.index && slice.end == first.end
    }) {
        return None;
    }

    let to = command
        .tail
        .iter()
        .find(|part| format_chain(&part.chain) == "to")?;
    let body_text = builtin_argument_texts(&to.args).next()?;
    let body = crate::frontend::formulation::parse_expression(body_text).ok()?;
    let start = first.start?;
    let end = first.end.as_deref()?;

    let at = |index: &str| render_variadic_map_body(&body, &slices, index, registry);
    let mut rendered = vec![at(&start.to_string())];
    rendered.push(at(&(start + 1).to_string()));
    rendered.push("\\ldots".to_string());
    rendered.push(at(end));
    Some(rendered.join(", "))
}

fn render_variadic_map_body(
    body: &Expression,
    slices: &[VariadicSlice],
    replacement_index: &str,
    registry: &RenderRegistry,
) -> String {
    let mut rendered = render_expression(body, registry);
    for slice in slices {
        let Some(index) = &slice.index else {
            continue;
        };
        let name = escape_math_identifier(&slice.name, registry);
        let needle = format!("{name}[{}]", escape_math_identifier(index, registry));
        let replacement = format!(
            "{name}_{{{}}}",
            escape_math_identifier(replacement_index, registry)
        );
        rendered = rendered.replace(&needle, &replacement);
    }
    rendered
}

fn render_variadic_reduce(
    command: &BuiltinCommandExpression,
    registry: &RenderRegistry,
) -> Option<String> {
    let operator = builtin_argument_texts(&command.head_args).next()?.trim();
    let operator = operator
        .strip_prefix('`')
        .and_then(|operator| operator.strip_suffix('`'))
        .unwrap_or(operator);
    let operator = render_operator_text(operator);
    let on = command
        .tail
        .iter()
        .find(|part| format_chain(&part.chain) == "on")?;
    let slice = builtin_argument_texts(&on.args)
        .next()
        .and_then(|text| crate::frontend::formulation::parse_expression(text).ok())?;
    let ExpressionKind::VariadicSlice(slice) = slice.kind else {
        return None;
    };
    let name = escape_math_identifier(&slice.name, registry);
    let start = slice.start?;
    let end = escape_math_identifier(slice.end.as_deref()?, registry);
    let mut terms = vec![format!("{name}_{{{start}}}"), "\\ldots".to_string()];
    if let Some(index) = &slice.index {
        terms.push(format!(
            "{name}_{{{}}}",
            escape_math_identifier(index, registry)
        ));
        terms.push("\\ldots".to_string());
    }
    terms.push(format!("{name}_{{{end}}}"));
    Some(terms.join(&format!(" {operator} ")))
}

fn builtin_argument_texts(groups: &[BuiltinCommandArgs]) -> impl Iterator<Item = &str> {
    groups
        .iter()
        .flat_map(|group| group.arguments.iter())
        .filter_map(|argument| match argument {
            BuiltinCommandArgument::Text(text) => Some(text.as_str()),
            BuiltinCommandArgument::Declaration(_) | BuiltinCommandArgument::Expression(_) => None,
        })
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
    insert_parameter_substitution(
        &mut substitutions,
        &render.owner_subject,
        escape_math_identifier(name, registry),
    );
    for (parameter, argument) in render.parameters.iter().zip(arguments) {
        insert_parameter_substitution(
            &mut substitutions,
            parameter,
            render_expression(argument, registry),
        );
    }
    Some(substitute_math_template(&render.written, &substitutions))
}

fn render_provided_member(
    owner: &Expression,
    name: &str,
    arguments: &[Expression],
    registry: &RenderRegistry,
) -> Option<String> {
    let render = registry
        .provided_members
        .iter()
        .find(|render| render.member_name == name && render.parameters.len() == arguments.len())?;
    let mut substitutions = HashMap::new();
    insert_parameter_substitution(
        &mut substitutions,
        &render.owner_subject,
        render_expression(owner, registry),
    );
    for (parameter, argument) in render.parameters.iter().zip(arguments) {
        insert_parameter_substitution(
            &mut substitutions,
            parameter,
            render_expression(argument, registry),
        );
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
        .join(";\\, ");

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
        SetTargetKind::Expression { expression, .. } => render_expression(expression, registry),
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
    let operator = render_quoted_operator(&statement.operator);
    let target = escape_math_identifier(&statement.name, registry);
    render_subject_relation(&statement.subject, &operator, &target, registry)
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

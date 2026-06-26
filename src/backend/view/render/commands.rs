use super::*;

pub(super) fn render_command_expression(
    command: &CommandExpression,
    registry: &RenderRegistry,
) -> String {
    let signature = command_expression_signature(command);
    let Some(render) = registry.commands.get(&signature) else {
        return render_command_like(&command.chain, registry);
    };
    let substitutions = command_substitutions(command, render, None, registry);

    match &render.written {
        Some(written) => substitute_math_template(written, &substitutions),
        None => render.render_called(&substitutions),
    }
}

pub(super) fn render_predicate_command_expression(
    command: &CommandExpression,
    registry: &RenderRegistry,
) -> String {
    let signature = command_expression_signature(command);
    let Some(render) = registry.commands.get(&signature) else {
        return render_command_like(&command.chain, registry);
    };
    let substitutions = command_substitutions(command, render, None, registry);

    if let Some(written) = &render.written {
        let includes_subject = render
            .subject_variable
            .as_ref()
            .is_some_and(|name| template_contains_placeholder(written, name));
        if !includes_subject {
            return substitute_math_template(written, &substitutions);
        }
    }

    render.render_called(&substitutions)
}

pub(super) fn render_infix_command_expression(
    left: &Expression,
    command: &InfixCommand,
    right: &Expression,
    registry: &RenderRegistry,
) -> String {
    let signature = infix_command_signature(command);
    let Some(render) = registry.commands.get(&signature) else {
        return format!(
            "{} {} {}",
            render_expression(left, registry),
            render_command_like(&command.chain, registry),
            render_expression(right, registry)
        );
    };
    let substitutions = infix_command_substitutions(left, command, right, render, registry);

    match &render.written {
        Some(written) => substitute_math_template(written, &substitutions),
        None => render.render_called(&substitutions),
    }
}

pub(super) fn render_infix_spec_expression(
    left: &Expression,
    spec: &InfixSpec,
    right: &Expression,
    registry: &RenderRegistry,
) -> String {
    let signature = infix_spec_signature(spec);
    let Some(render) = registry.commands.get(&signature) else {
        return format!(
            "{} {} {}",
            render_expression(left, registry),
            render_infix_spec_like(spec, registry),
            render_expression(right, registry)
        );
    };
    let substitutions = infix_spec_substitutions(left, spec, right, render, registry);

    match &render.written {
        Some(written) => substitute_math_template(written, &substitutions),
        None => render.render_called(&substitutions),
    }
}

pub(super) fn render_refined_command_called(
    command: &RefinedCommandExpression,
    registry: &RenderRegistry,
) -> String {
    let called = refined_command_called_template(command, registry);
    called.latex
}

#[derive(Clone, Debug)]
pub(super) struct CalledTemplate {
    pub(super) latex: String,
}

pub(super) fn type_expression_called_template(
    ty: &TypeExpression,
    registry: &RenderRegistry,
) -> Option<CalledTemplate> {
    match ty {
        TypeExpression::Builtin { .. } => None,
        TypeExpression::Command(command) => command_called_template(command, registry),
        TypeExpression::RefinedCommand(command) => {
            Some(refined_command_called_template(command, registry))
        }
        TypeExpression::Function(_) => None,
    }
}

pub(super) fn command_called_template(
    command: &CommandExpression,
    registry: &RenderRegistry,
) -> Option<CalledTemplate> {
    let render = registry
        .commands
        .get(&command_expression_signature(command))?;
    let substitutions = command_substitutions(command, render, None, registry);
    Some(CalledTemplate {
        latex: render.render_called(&substitutions),
    })
}

pub(super) fn refined_command_called_template(
    command: &RefinedCommandExpression,
    registry: &RenderRegistry,
) -> CalledTemplate {
    let mut refinement_templates = Vec::new();

    let base_signature = refined_command_base_signature(command);

    for part in &command.parts {
        let signature = refined_command_part_signature(command, part);
        if let Some(render) = registry.commands.get(&signature) {
            let substitutions = command_substitutions_for_names(
                &render.parameters,
                refined_command_part_argument_values(command, part, registry),
            );
            refinement_templates.push(render.render_called(&substitutions));
        } else {
            refinement_templates.push(render_called_template(
                &format_chain(&part.chain),
                &HashMap::new(),
            ));
        }
    }

    let base_latex = if let Some(render) = registry.commands.get(&base_signature) {
        let substitutions = command_substitutions_for_names(
            &render.parameters,
            refined_command_base_argument_values(command, registry),
        );
        render.render_called(&substitutions)
    } else {
        render_called_template(
            &refined_tail_signature(&command.refined_tail),
            &HashMap::new(),
        )
    };

    let latex = if refinement_templates.is_empty() {
        base_latex
    } else {
        join_called_latex_parts(vec![refinement_templates.join("\\textrm{, }"), base_latex])
    };

    CalledTemplate { latex }
}

pub(super) fn refines_target_type(statement: &DeclarationStatement) -> Option<&TypeExpression> {
    match &statement.relation {
        Some(DeclarationRelation::Is(ty)) => Some(ty),
        _ => None,
    }
}

pub(super) fn command_substitutions(
    command: &CommandExpression,
    render: &CommandRender,
    subject_latex: Option<String>,
    registry: &RenderRegistry,
) -> HashMap<String, String> {
    let mut substitutions = HashMap::new();

    if let (Some(name), Some(value)) = (&render.subject_variable, subject_latex) {
        substitutions.insert(name.clone(), value);
    }

    for (name, value) in render
        .parameters
        .iter()
        .zip(command_argument_values(command, registry))
    {
        substitutions.insert(name.clone(), value);
    }

    substitutions
}

pub(super) fn infix_command_substitutions(
    left: &Expression,
    command: &InfixCommand,
    right: &Expression,
    render: &CommandRender,
    registry: &RenderRegistry,
) -> HashMap<String, String> {
    command_substitutions_for_names(
        &render.parameters,
        infix_argument_values(left, &command.head_args, &command.tail, right, registry),
    )
}

pub(super) fn infix_spec_substitutions(
    left: &Expression,
    spec: &InfixSpec,
    right: &Expression,
    render: &CommandRender,
    registry: &RenderRegistry,
) -> HashMap<String, String> {
    command_substitutions_for_names(
        &render.parameters,
        infix_argument_values(left, &spec.head_args, &spec.tail, right, registry),
    )
}

pub(super) fn command_substitutions_for_names(
    names: &[String],
    values: Vec<String>,
) -> HashMap<String, String> {
    names.iter().cloned().zip(values).collect()
}

pub(super) fn command_header_substitutions(header: &CommandHeader) -> HashMap<String, String> {
    let mut substitutions = HashMap::new();

    for form in command_header_forms(header) {
        if let Some(name) = primary_form_name(form) {
            substitutions.insert(name, render_form_or_declaration(form));
        }
    }

    substitutions
}

pub(super) fn command_header_forms(header: &CommandHeader) -> Vec<&FormOrDeclaration> {
    match header {
        CommandHeader::Command(header) => simple_command_header_forms(header),
        CommandHeader::Infix(header) => infix_command_header_forms(header),
        CommandHeader::InfixSpec(header) => infix_spec_header_forms(header),
        CommandHeader::Refined(header) => refined_command_header_forms(header),
    }
}

pub(super) fn simple_command_header_forms(header: &CommandHeaderNode) -> Vec<&FormOrDeclaration> {
    let mut forms = Vec::new();
    forms.extend(header.head_args.iter().flat_map(|args| args.forms.iter()));
    forms.extend(
        header
            .tail
            .iter()
            .flat_map(|part| part.args.iter())
            .flat_map(|args| args.forms.iter()),
    );
    forms.extend(header.paren_args.iter().flat_map(|args| args.forms.iter()));
    forms
}

pub(super) fn infix_command_header_forms(header: &InfixCommandHeader) -> Vec<&FormOrDeclaration> {
    let mut forms = Vec::new();
    if let Some(left) = &header.left {
        forms.push(left);
    }
    forms.extend(header.head_args.iter().flat_map(|args| args.forms.iter()));
    forms.extend(
        header
            .tail
            .iter()
            .flat_map(|part| part.args.iter())
            .flat_map(|args| args.forms.iter()),
    );
    if let Some(right) = &header.right {
        forms.push(right);
    }
    forms
}

pub(super) fn infix_spec_header_forms(header: &InfixSpecHeader) -> Vec<&FormOrDeclaration> {
    let mut forms = Vec::new();
    forms.push(&header.left);
    forms.extend(header.head_args.iter().flat_map(|args| args.forms.iter()));
    forms.extend(
        header
            .tail
            .iter()
            .flat_map(|part| part.args.iter())
            .flat_map(|args| args.forms.iter()),
    );
    forms.push(&header.right);
    forms
}

pub(super) fn refined_command_header_forms(
    header: &RefinedCommandHeader,
) -> Vec<&FormOrDeclaration> {
    let mut forms = Vec::new();
    forms.extend(header.head_args.iter().flat_map(|args| args.forms.iter()));
    forms.extend(
        header
            .tail
            .iter()
            .flat_map(|part| part.args.iter())
            .flat_map(|args| args.forms.iter()),
    );
    forms.extend(header.paren_args.iter().flat_map(|args| args.forms.iter()));
    forms.extend(
        header
            .parts
            .iter()
            .flat_map(|part| part.tail.iter())
            .flat_map(|tail_part| tail_part.args.iter())
            .flat_map(|args| args.forms.iter()),
    );
    forms
}

pub(super) fn refined_command_base_argument_values(
    command: &RefinedCommandExpression,
    registry: &RenderRegistry,
) -> Vec<String> {
    command
        .head_args
        .iter()
        .flat_map(|args| args.expressions.iter())
        .chain(
            command
                .tail
                .iter()
                .flat_map(|part| part.args.iter())
                .flat_map(|args| args.expressions.iter()),
        )
        .chain(
            command
                .paren_args
                .iter()
                .flat_map(|args| args.expressions.iter()),
        )
        .map(|expression| render_expression(expression, registry))
        .collect()
}

pub(super) fn refined_command_part_argument_values(
    command: &RefinedCommandExpression,
    part: &RefinedExpressionPart,
    registry: &RenderRegistry,
) -> Vec<String> {
    refined_command_base_argument_values(command, registry)
        .into_iter()
        .chain(expression_tail_argument_values(&part.tail, registry))
        .collect()
}

pub(super) fn command_argument_values(
    command: &CommandExpression,
    registry: &RenderRegistry,
) -> Vec<String> {
    command
        .head_args
        .iter()
        .flat_map(|args| args.expressions.iter())
        .chain(
            command
                .tail
                .iter()
                .flat_map(|part| part.args.iter())
                .flat_map(|args| args.expressions.iter()),
        )
        .chain(
            command
                .paren_args
                .iter()
                .flat_map(|args| args.expressions.iter()),
        )
        .map(|expression| render_expression(expression, registry))
        .collect()
}

pub(super) fn infix_argument_values(
    left: &Expression,
    head_args: &[CurlyExpressionArgs],
    tail: &[CommandExpressionTailPart],
    right: &Expression,
    registry: &RenderRegistry,
) -> Vec<String> {
    std::iter::once(render_expression(left, registry))
        .chain(
            head_args
                .iter()
                .flat_map(|args| args.expressions.iter())
                .map(|expression| render_expression(expression, registry)),
        )
        .chain(expression_tail_argument_values(tail, registry))
        .chain(std::iter::once(render_expression(right, registry)))
        .collect()
}

pub(super) fn expression_tail_argument_values(
    tail: &[CommandExpressionTailPart],
    registry: &RenderRegistry,
) -> Vec<String> {
    tail.iter()
        .flat_map(|part| part.args.iter())
        .flat_map(|args| args.expressions.iter())
        .map(|expression| render_expression(expression, registry))
        .collect()
}

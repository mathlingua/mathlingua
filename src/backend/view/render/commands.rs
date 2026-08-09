use super::*;

pub(super) fn render_command_expression(
    command: &CommandExpression,
    registry: &RenderRegistry,
) -> String {
    let signature = command_expression_signature(command);
    let Some(render) = registry.commands.get(&signature) else {
        return append_command_context_suffix(
            render_command_like(&command.chain, registry),
            command.context.as_ref(),
            registry,
        );
    };
    let substitutions = command_substitutions(command, render, None, registry);

    let latex = match render.effective_written(&substitutions) {
        Some(written) => substitute_math_template(written, &substitutions),
        None => render.render_called(&substitutions),
    };

    append_command_context_suffix(
        command_reference_latex(&signature, latex, registry),
        command.context.as_ref(),
        registry,
    )
}

pub(super) fn render_predicate_command_expression(
    command: &CommandExpression,
    registry: &RenderRegistry,
) -> String {
    let signature = command_expression_signature(command);
    let Some(render) = registry.commands.get(&signature) else {
        return append_command_context_suffix(
            render_command_like(&command.chain, registry),
            command.context.as_ref(),
            registry,
        );
    };
    let substitutions = command_substitutions(command, render, None, registry);

    if let Some(written) = render.effective_written(&substitutions) {
        let includes_subject = render
            .subject_variable
            .as_ref()
            .is_some_and(|name| template_contains_placeholder(written, name));
        if !includes_subject {
            return append_command_context_suffix(
                command_reference_latex(
                    &signature,
                    substitute_math_template(written, &substitutions),
                    registry,
                ),
                command.context.as_ref(),
                registry,
            );
        }
    }

    append_command_context_suffix(
        command_reference_latex(&signature, render.render_called(&substitutions), registry),
        command.context.as_ref(),
        registry,
    )
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

    let latex = match render.effective_written(&substitutions) {
        Some(written) => substitute_math_template(written, &substitutions),
        None => render.render_called(&substitutions),
    };

    command_reference_latex(&signature, latex, registry)
}

pub(super) fn render_infix_spec_expression(
    left: &Expression,
    spec: &InfixSpec,
    right: &Expression,
    registry: &RenderRegistry,
) -> String {
    let signature = infix_spec_signature(spec);
    if let Some(render) = registry.commands.get(&signature) {
        let substitutions = infix_spec_substitutions(left, spec, right, render, registry);
        if let Some(written) = render.effective_written(&substitutions) {
            return command_reference_latex(
                &signature,
                substitute_math_template(written, &substitutions),
                registry,
            );
        }
    }

    if let Some(refinement) = &spec.refinement {
        let mut base = spec.clone();
        base.refinement = None;
        let base_latex = render_infix_spec_expression(left, &base, right, registry);
        let refinements = refinement
            .parts
            .iter()
            .map(|part| render_infix_spec_refinement_part(spec, part, left, right, registry))
            .collect();
        return append_refinement_suffix(base_latex, refinements);
    }

    if let Some(render) = registry.commands.get(&signature) {
        let substitutions = infix_spec_substitutions(left, spec, right, render, registry);
        return command_reference_latex(&signature, render.render_called(&substitutions), registry);
    }

    format!(
        "{} {} {}",
        render_expression(left, registry),
        render_infix_spec_like(spec, registry),
        render_expression(right, registry)
    )
}

fn render_infix_spec_refinement_part(
    spec: &InfixSpec,
    part: &RefinedExpressionPart,
    left: &Expression,
    right: &Expression,
    registry: &RenderRegistry,
) -> String {
    let mut single = spec.clone();
    if let Some(refinement) = &mut single.refinement {
        refinement.parts = vec![part.clone()];
    }
    let signature = infix_spec_signature(&single);
    if let Some(render) = registry.commands.get(&signature) {
        let substitutions = infix_spec_substitutions(left, &single, right, render, registry);
        return command_reference_latex(&signature, render.render_called(&substitutions), registry);
    }

    let mut type_prefix = "\\".to_owned();
    if let Some(prefix) = spec
        .refinement
        .as_ref()
        .and_then(|refinement| refinement.prefix_chain.as_ref())
    {
        type_prefix.push_str(&format_chain(prefix));
        type_prefix.push_str("::");
    }
    type_prefix.push_str(&format_chain(&part.chain));
    add_expression_tail_signature(&mut type_prefix, &part.tail);
    type_prefix.push_str("::");
    if let Some((refined_signature, render)) = registry
        .commands
        .iter()
        .find(|(candidate, _)| candidate.starts_with(&type_prefix))
    {
        return command_reference_latex(
            refined_signature,
            render.render_called(&HashMap::new()),
            registry,
        );
    }

    render_called_template(&format_chain(&part.chain), &HashMap::new())
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

#[derive(Clone, Debug)]
pub(super) struct TypeTemplate {
    pub(super) latex: String,
    pub(super) includes_subject: bool,
}

pub(super) fn command_type_template(
    command: &CommandExpression,
    subject_latex: Option<String>,
    registry: &RenderRegistry,
) -> Option<TypeTemplate> {
    let signature = command_expression_signature(command);
    let render = registry.commands.get(&signature)?;
    let substitutions = command_substitutions(command, render, subject_latex.clone(), registry);
    let written_includes_subject = render
        .effective_written(&substitutions)
        .is_some_and(|written| written_contains_subject_placeholder(written, render));
    let use_written = render
        .effective_written(&substitutions)
        .filter(|_| subject_latex.is_some() || !written_includes_subject);
    let latex = match use_written {
        Some(written) => substitute_math_template(written, &substitutions),
        None => render.render_called(&substitutions),
    };

    Some(TypeTemplate {
        latex: append_command_context_suffix(
            command_reference_latex(&signature, latex, registry),
            command.context.as_ref(),
            registry,
        ),
        includes_subject: subject_latex.is_some() && written_includes_subject,
    })
}

fn written_contains_subject_placeholder(written: &str, render: &CommandRender) -> bool {
    render
        .subject_variable
        .as_ref()
        .is_some_and(|name| template_contains_placeholder(written, name))
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
            refinement_templates.push(command_reference_latex(
                &signature,
                render.render_called(&substitutions),
                registry,
            ));
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
        command_reference_latex(
            &base_signature,
            render.render_called(&substitutions),
            registry,
        )
    } else {
        render_called_template(
            &refined_tail_signature(&command.refined_tail),
            &HashMap::new(),
        )
    };

    let latex = prepend_refinement_prefix(base_latex, refinement_templates);

    CalledTemplate { latex }
}

pub(super) fn append_refinement_suffix(base_latex: String, refinements: Vec<String>) -> String {
    let Some(list) = refinement_list(&refinements) else {
        return base_latex;
    };
    format!("{base_latex}\\textrm{{ }}\\left({list}\\right)")
}

pub(super) fn prepend_refinement_prefix(base_latex: String, refinements: Vec<String>) -> String {
    let Some(list) = refinement_list(&refinements) else {
        return base_latex;
    };
    if refinements.len() == 1 {
        format!("{list}\\textrm{{ }}{base_latex}")
    } else {
        format!("\\left({list}\\right)\\textrm{{ }}{base_latex}")
    }
}

fn refinement_list(refinements: &[String]) -> Option<String> {
    let list = match refinements {
        [] => return None,
        [only] => only.clone(),
        [left, right] => format!("{left}\\textrm{{ and }}{right}"),
        _ => {
            let (last, head) = refinements.split_last().expect("refinements are nonempty");
            format!("{}\\textrm{{, and }}{}", head.join("\\textrm{, }"), last)
        }
    };
    Some(list)
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

    substitutions.extend(render_parameter_substitutions(
        render,
        command_argument_values(command, registry),
        command_argument_group_values(command, registry),
    ));

    if let Some(context) = &command.context {
        for argument in &context.arguments {
            if let CommandContextArgument::Assignment { name, value, .. } = argument {
                substitutions.insert(name.clone(), render_expression(value, registry));
            }
        }
    }

    substitutions
}

fn append_command_context_suffix(
    latex: String,
    context: Option<&CommandContext>,
    registry: &RenderRegistry,
) -> String {
    let Some(context) = context else {
        return latex;
    };

    let arguments = context
        .arguments
        .iter()
        .map(|argument| render_command_context_argument(argument, registry))
        .filter(|argument| !argument.is_empty())
        .collect::<Vec<_>>();

    if arguments.is_empty() {
        return latex;
    }

    let label = match context.kind {
        CommandContextKind::Using => "using",
        CommandContextKind::Given => "given",
    };

    format!("{latex} \\textrm{{ {label} }} {}", arguments.join("; "))
}

fn render_command_context_argument(
    argument: &CommandContextArgument,
    registry: &RenderRegistry,
) -> String {
    match argument {
        CommandContextArgument::Assignment { value, .. } => render_expression(value, registry),
        CommandContextArgument::Declaration(statement) => {
            render_declaration_statement(statement, registry)
        }
        CommandContextArgument::Expression(expression) => render_expression(expression, registry),
        CommandContextArgument::Text(text) => {
            format!("\\textrm{{{}}}", escape_latex_text(text.trim()))
        }
    }
}

pub(super) fn infix_command_substitutions(
    left: &Expression,
    command: &InfixCommand,
    right: &Expression,
    render: &CommandRender,
    registry: &RenderRegistry,
) -> HashMap<String, String> {
    render_parameter_substitutions(
        render,
        infix_argument_values(left, &command.head_args, &command.tail, right, registry),
        expression_curly_group_values(&command.head_args, &command.tail, registry),
    )
}

fn render_parameter_substitutions(
    render: &CommandRender,
    values: Vec<String>,
    groups: Vec<Vec<String>>,
) -> HashMap<String, String> {
    let variadic_values = render
        .variadic_parameters
        .iter()
        .filter_map(|(parameter, group_index)| {
            groups
                .get(*group_index)
                .cloned()
                .map(|values| (parameter.name.clone(), (parameter, values)))
        })
        .collect::<HashMap<_, _>>();

    let mut substitutions = HashMap::new();
    let mut value_index = 0usize;
    for name in &render.parameters {
        if let Some((parameter, variadic)) = variadic_values.get(name) {
            value_index += variadic.len();
            substitutions.insert(name.clone(), variadic.join(", "));
            insert_variadic_substitution(&mut substitutions, name, variadic);
            if let Some(length) = &parameter.length {
                substitutions.insert(length.clone(), variadic.len().to_string());
                if let Some(last) = variadic.last() {
                    substitutions.insert(format!("{}[{length}]", parameter.name), last.clone());
                }
            }
            for (offset, value) in variadic.iter().enumerate() {
                let starts = if parameter.index.is_some() {
                    vec![parameter.start]
                } else {
                    vec![0, 1]
                };
                for start in starts {
                    substitutions.insert(
                        format!("{}[{}]", parameter.name, start + offset),
                        value.clone(),
                    );
                }
            }
        } else if let Some(value) = values.get(value_index) {
            substitutions.insert(name.clone(), value.clone());
            value_index += 1;
        }
    }
    substitutions
}

pub(super) fn infix_spec_substitutions(
    left: &Expression,
    spec: &InfixSpec,
    right: &Expression,
    render: &CommandRender,
    registry: &RenderRegistry,
) -> HashMap<String, String> {
    let refinement_values = spec
        .refinement
        .iter()
        .flat_map(|refinement| refinement.parts.iter())
        .flat_map(|part| part.tail.iter())
        .flat_map(|tail| tail.args.iter())
        .flat_map(|args| args.expressions.iter())
        .map(|expression| render_expression(expression, registry));
    let values = std::iter::once(render_expression(left, registry))
        .chain(refinement_values)
        .chain(
            spec.head_args
                .iter()
                .flat_map(|args| args.expressions.iter())
                .map(|expression| render_expression(expression, registry)),
        )
        .chain(expression_tail_argument_values(&spec.tail, registry))
        .chain(std::iter::once(render_expression(right, registry)))
        .collect();
    let mut groups = spec
        .refinement
        .iter()
        .flat_map(|refinement| refinement.parts.iter())
        .flat_map(|part| expression_curly_group_values(&[], &part.tail, registry))
        .collect::<Vec<_>>();
    groups.extend(expression_curly_group_values(
        &spec.head_args,
        &spec.tail,
        registry,
    ));
    render_parameter_substitutions(render, values, groups)
}

pub(super) fn command_substitutions_for_names(
    names: &[String],
    values: Vec<String>,
) -> HashMap<String, String> {
    names.iter().cloned().zip(values).collect()
}

pub(super) fn command_header_substitutions(
    header: &CommandHeader,
    registry: &RenderRegistry,
) -> HashMap<String, String> {
    let mut substitutions = HashMap::new();

    for form in command_header_forms(header) {
        if let Some(name) = primary_form_name(form) {
            substitutions.insert(name, render_form_or_declaration_head(form, registry));
        }
    }
    for parameter in command_header_variadic_parameters(header) {
        substitutions.insert(
            parameter.name.clone(),
            render_variadic_parameter(parameter, registry),
        );
        insert_variadic_substitution(
            &mut substitutions,
            &parameter.name,
            &render_variadic_parameter_elements(parameter, registry),
        );
    }

    substitutions
}

fn command_header_variadic_parameters(header: &CommandHeader) -> Vec<&VariadicParameter> {
    fn collect<'a>(groups: &'a [CurlyHeadingArgs], out: &mut Vec<&'a VariadicParameter>) {
        out.extend(groups.iter().filter_map(|group| group.variadic.as_ref()));
    }
    fn collect_tail<'a>(parts: &'a [CommandHeaderTailPart], out: &mut Vec<&'a VariadicParameter>) {
        for part in parts {
            collect(&part.args, out);
        }
    }

    let mut result = Vec::new();
    match header {
        CommandHeader::Command(command) => {
            collect(&command.head_args, &mut result);
            collect_tail(&command.tail, &mut result);
        }
        CommandHeader::Infix(command) => {
            collect(&command.head_args, &mut result);
            collect_tail(&command.tail, &mut result);
        }
        CommandHeader::InfixSpec(spec) => {
            if let Some(refinement) = &spec.refinement {
                for part in &refinement.parts {
                    collect_tail(&part.tail, &mut result);
                }
            }
            collect(&spec.head_args, &mut result);
            collect_tail(&spec.tail, &mut result);
        }
        CommandHeader::Refined(command) => {
            for part in &command.parts {
                collect_tail(&part.tail, &mut result);
            }
            collect(&command.head_args, &mut result);
            collect_tail(&command.tail, &mut result);
        }
    }
    result
}

/// Whether a header parameter is a *named destructuring* — `H ::= (X', *', e')`,
/// `g ::= f(x_)`, or `S ::= {x_ : …}` — as opposed to a plain name or an
/// anonymous form.
fn is_named_destructuring(form: &FormOrDeclaration) -> bool {
    matches!(
        &form.kind,
        FormOrDeclarationKind::FunctionDeclaration { name: Some(_), .. }
            | FormOrDeclarationKind::TupleDeclaration { name: Some(_), .. }
            | FormOrDeclarationKind::SetDeclaration { name: Some(_), .. }
    )
}

/// Renders a header parameter for use *inside a title*: a named destructuring
/// collapses to just its name (`H`) so the title stays readable — the full
/// destructuring is surfaced separately by [`command_header_parameter_destructurings`].
/// Every other form renders in full.
fn render_form_or_declaration_head(form: &FormOrDeclaration, registry: &RenderRegistry) -> String {
    match &form.kind {
        FormOrDeclarationKind::FunctionDeclaration {
            name: Some(name), ..
        }
        | FormOrDeclarationKind::TupleDeclaration {
            name: Some(name), ..
        }
        | FormOrDeclarationKind::SetDeclaration {
            name: Some(name), ..
        } => escape_math_identifier(name, registry),
        _ => render_form_or_declaration(form, registry),
    }
}

/// The full `name ::= …` rendering of each named-destructuring header parameter,
/// in header order — shown as lines beneath a card title so the title itself can
/// use the plain names.
pub(super) fn command_header_parameter_destructurings(
    header: &CommandHeader,
    registry: &RenderRegistry,
) -> Vec<String> {
    command_header_forms(header)
        .into_iter()
        .filter(|form| is_named_destructuring(form))
        .map(|form| render_form_or_declaration(form, registry))
        .collect()
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
    if let Some(refinement) = &header.refinement {
        forms.extend(
            refinement
                .parts
                .iter()
                .flat_map(|part| part.tail.iter())
                .flat_map(|part| part.args.iter())
                .flat_map(|args| args.forms.iter()),
        );
    }
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

fn command_argument_group_values(
    command: &CommandExpression,
    registry: &RenderRegistry,
) -> Vec<Vec<String>> {
    let mut groups = expression_curly_group_values(&command.head_args, &command.tail, registry);
    groups.extend(command.paren_args.iter().map(|args| {
        args.expressions
            .iter()
            .map(|expression| render_expression(expression, registry))
            .collect()
    }));
    groups
}

fn expression_curly_group_values(
    head: &[CurlyExpressionArgs],
    tail: &[CommandExpressionTailPart],
    registry: &RenderRegistry,
) -> Vec<Vec<String>> {
    head.iter()
        .chain(tail.iter().flat_map(|part| part.args.iter()))
        .map(|args| {
            args.expressions
                .iter()
                .map(|expression| render_expression(expression, registry))
                .collect()
        })
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

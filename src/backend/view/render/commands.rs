use super::*;

/// Renders a standalone normal command expression.
///
/// Written templates are preferred for standalone command expressions; otherwise
/// the plain-text `called:` template is rendered in text mode.
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
        None => render_called_template(&render.called, &substitutions),
    }
}

/// Renders a command expression used as an `is`/`is not` predicate.
///
/// Predicate rendering deliberately falls back to `called:` when a `written:`
/// template contains the subject placeholder, because predicate syntax supplies
/// the subject separately.
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

    render_called_template(&render.called, &substitutions)
}

/// Renders the called-text form of a refined command expression.
pub(super) fn render_refined_command_called(
    command: &RefinedCommandExpression,
    registry: &RenderRegistry,
) -> String {
    let called = refined_command_called_template(command, registry);
    render_called_template(&called.template, &called.substitutions)
}

/// Unrendered called-template text paired with all substitutions needed for it.
#[derive(Clone, Debug)]
pub(super) struct CalledTemplate {
    /// Plain-LaTeX-mode template, usually assembled from `called:` text.
    pub(super) template: String,
    /// Placeholder substitutions rendered as math-mode LaTeX.
    pub(super) substitutions: HashMap<String, String>,
}

/// Extracts a called-template representation from any type expression.
pub(super) fn type_expression_called_template(
    ty: &TypeExpression,
    registry: &RenderRegistry,
) -> Option<CalledTemplate> {
    match ty {
        TypeExpression::Command(command) => command_called_template(command, registry),
        TypeExpression::RefinedCommand(command) => {
            Some(refined_command_called_template(command, registry))
        }
    }
}

/// Extracts a called-template representation from a normal command expression.
pub(super) fn command_called_template(
    command: &CommandExpression,
    registry: &RenderRegistry,
) -> Option<CalledTemplate> {
    let render = registry
        .commands
        .get(&command_expression_signature(command))?;
    Some(CalledTemplate {
        template: render.called.clone(),
        substitutions: command_substitutions(command, render, None, registry),
    })
}

/// Builds the called-template representation for a refined command expression.
///
/// The resulting template concatenates refinement labels with the base command's
/// called text and gathers substitutions from both the refinement parts and the
/// base command.
pub(super) fn refined_command_called_template(
    command: &RefinedCommandExpression,
    registry: &RenderRegistry,
) -> CalledTemplate {
    let mut refinement_templates = Vec::new();
    let mut substitutions = HashMap::new();

    let base_signature = refined_command_base_signature(command);
    if let Some(render) = registry.commands.get(&base_signature) {
        substitutions.extend(command_substitutions_for_names(
            &render.parameters,
            refined_command_base_argument_values(command, registry),
        ));
    }

    for part in &command.parts {
        let signature = refined_command_part_signature(command, part);
        if let Some(render) = registry.commands.get(&signature) {
            refinement_templates.push(render.called.clone());
            substitutions.extend(command_substitutions_for_names(
                &render.parameters,
                refined_command_part_argument_values(command, part, registry),
            ));
        } else {
            refinement_templates.push(format_chain(&part.chain));
        }
    }

    let base_template = if let Some(render) = registry.commands.get(&base_signature) {
        render.called.clone()
    } else {
        refined_tail_signature(&command.refined_tail)
    };

    let template = if refinement_templates.is_empty() {
        base_template
    } else {
        format!("{} {}", refinement_templates.join(", "), base_template)
    };

    CalledTemplate {
        template,
        substitutions,
    }
}

/// Returns the type expression targeted by a `Refines:` statement.
///
/// `Refines:` can also parse as a spec statement; those do not identify a type
/// command and therefore cannot enrich a card heading.
pub(super) fn refines_target_type(spec: &IsOrRefinedStatementSpec) -> Option<&TypeExpression> {
    match spec {
        IsOrRefinedStatementSpec::Is(statement) => Some(&statement.ty),
        IsOrRefinedStatementSpec::Spec(_) => None,
    }
}

/// Builds placeholder substitutions for a normal command expression.
///
/// The optional subject is inserted under the defining subject variable so
/// templates like `f? : A? \to B?` can render complete statements.
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

/// Builds a substitution map from explicit parameter names and rendered values.
pub(super) fn command_substitutions_for_names(
    names: &[String],
    values: Vec<String>,
) -> HashMap<String, String> {
    names.iter().cloned().zip(values).collect()
}

/// Builds placeholder substitutions from forms supplied in a command header.
///
/// Heading substitutions are used for group card titles where the source heading
/// itself provides concrete parameter names such as `{A}` and `{B}`.
pub(super) fn command_header_substitutions(header: &CommandHeader) -> HashMap<String, String> {
    let mut substitutions = HashMap::new();

    for form in command_header_forms(header) {
        if let Some(name) = primary_form_name(form) {
            substitutions.insert(name, render_form_or_declaration(form));
        }
    }

    substitutions
}

/// Collects all forms that appear in any kind of command header.
pub(super) fn command_header_forms(header: &CommandHeader) -> Vec<&FormOrDeclaration> {
    match header {
        CommandHeader::Command(header) => simple_command_header_forms(header),
        CommandHeader::Infix(header) => infix_command_header_forms(header),
        CommandHeader::Refined(header) => refined_command_header_forms(header),
    }
}

/// Collects forms from a normal prefix command header.
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

/// Collects forms from an infix command header.
pub(super) fn infix_command_header_forms(header: &InfixCommandHeader) -> Vec<&FormOrDeclaration> {
    let mut forms = Vec::new();
    forms.extend(header.head_args.iter().flat_map(|args| args.forms.iter()));
    forms.extend(
        header
            .tail
            .iter()
            .flat_map(|part| part.args.iter())
            .flat_map(|args| args.forms.iter()),
    );
    forms
}

/// Collects forms from a refined command header.
///
/// Refined headers can place arguments on the base command, the base tail, and
/// refinement parts, so all of those locations are included.
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

/// Renders argument values that belong to the base command in a refined expression.
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

/// Renders argument values available to a single refined-command part.
///
/// A refinement part can refer both to the base command arguments and to any
/// arguments attached directly to that part.
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

/// Renders all argument values supplied to a normal command expression.
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

/// Renders all argument values attached to command tail segments.
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

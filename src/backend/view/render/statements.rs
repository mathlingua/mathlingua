use super::*;

pub(super) fn render_declaration_statement(
    statement: &DeclarationStatement,
    registry: &RenderRegistry,
) -> String {
    let mut rendered = render_is_subject(&statement.subject, registry);
    if let Some(expansion) = &statement.expansion {
        rendered.push_str(" ::= ");
        rendered.push_str(&render_is_subject(expansion, registry));
    }
    if let Some(definition) = &statement.definition {
        rendered.push_str(" := ");
        rendered.push_str(&render_expression(definition, registry));
    }
    if let Some(relation) = &statement.relation {
        rendered.push(' ');
        rendered.push_str(&render_declaration_relation(relation, registry));
    }
    rendered
}

pub(super) fn render_is_via_statement(
    statement: &IsViaStatement,
    registry: &RenderRegistry,
) -> String {
    format!(
        "{} \\textrm{{ is }} {} \\textrm{{ via }} {}",
        render_is_subject(&statement.is_statement.subject, registry),
        render_type_expression(&statement.is_statement.ty, registry),
        render_form_or_declaration(&statement.via, registry)
    )
}

fn render_declaration_relation(
    relation: &DeclarationRelation,
    registry: &RenderRegistry,
) -> String {
    match relation {
        DeclarationRelation::Is(ty) => {
            format!("\\textrm{{ is }} {}", render_type_expression(ty, registry))
        }
        DeclarationRelation::Spec { operator, target } => {
            format!(
                "{} {}",
                render_quoted_operator(operator),
                render_expression(target, registry)
            )
        }
        DeclarationRelation::InfixSpec { spec, target } => {
            format!(
                "{} {}",
                render_command_like(&spec.chain, registry),
                render_expression(target, registry)
            )
        }
    }
}

pub(super) fn render_is_subject(subject: &IsSubject, registry: &RenderRegistry) -> String {
    match &subject.kind {
        IsSubjectKind::Forms(forms) => forms
            .iter()
            .map(|form| match form {
                IsSubjectForm::Form(form) => render_form_or_declaration(form, registry),
                IsSubjectForm::PlaceholderForm(form) => render_placeholder_form(form, registry),
            })
            .collect::<Vec<_>>()
            .join(", "),
        IsSubjectKind::Operator(operator) => render_operator_text(&operator.text),
    }
}

pub(super) fn render_form_or_declaration(
    form: &FormOrDeclaration,
    registry: &RenderRegistry,
) -> String {
    match &form.kind {
        FormOrDeclarationKind::Name(name) => escape_math_identifier(name, registry),
        FormOrDeclarationKind::FunctionDeclaration { name, form } => {
            let name = name.as_ref().unwrap_or(&form.name);
            let args =
                form.magnetic_placeholder
                    .iter()
                    .map(|placeholder| render_form_placeholder_name(&placeholder.name, registry))
                    .chain(form.placeholders.iter().map(|placeholder| {
                        render_form_placeholder_name(&placeholder.name, registry)
                    }))
                    .collect::<Vec<_>>()
                    .join(", ");
            if args.is_empty() {
                escape_math_identifier(name, registry)
            } else {
                format!("{}({})", escape_math_identifier(name, registry), args)
            }
        }
        FormOrDeclarationKind::TupleDeclaration { name, form } => {
            let rendered = form
                .elements
                .iter()
                .map(|element| match element {
                    TupleFormElement::Form(form) => render_form_or_declaration(form, registry),
                    TupleFormElement::Operator(operator) => render_operator_text(&operator.text),
                })
                .collect::<Vec<_>>()
                .join(", ");
            match name {
                Some(name) => format!(
                    "{} ::= \\left({rendered}\\right)",
                    escape_math_identifier(name, registry)
                ),
                None => format!("\\left({rendered}\\right)"),
            }
        }
        FormOrDeclarationKind::SetDeclaration { name, form } => {
            let placeholder = render_placeholder_form(&form.placeholder_form, registry);
            let rendered = if form.has_condition_placeholder {
                format!("\\left\\{{{} \\: : \\: \\ldots\\right\\}}", placeholder)
            } else {
                format!("\\left\\{{{}\\right\\}}", placeholder)
            };
            match name {
                Some(name) => format!("{} ::= {rendered}", escape_math_identifier(name, registry)),
                None => rendered,
            }
        }
        FormOrDeclarationKind::InfixOperator {
            left,
            operator,
            right,
        } => format!(
            "{} {} {}",
            render_form_placeholder_name(&left.name, registry),
            render_operator_text(&operator.text),
            render_form_placeholder_name(&right.name, registry)
        ),
        FormOrDeclarationKind::PrefixOperator {
            operator,
            placeholder,
        } => format!(
            "{}{}",
            render_operator_text(&operator.text),
            render_form_placeholder_name(&placeholder.name, registry)
        ),
        FormOrDeclarationKind::PostfixOperator {
            placeholder,
            operator,
        } => format!(
            "{}{}",
            render_form_placeholder_name(&placeholder.name, registry),
            render_operator_text(&operator.text)
        ),
    }
}

pub(super) fn render_is_command(
    subject: &Expression,
    command: &CommandExpression,
    registry: &RenderRegistry,
) -> String {
    let subject_latex = render_expression(subject, registry);
    render_is_command_with_subject_latex(subject_latex, command, registry)
}

pub(super) fn render_is_refined_command(
    subject: &Expression,
    command: &RefinedCommandExpression,
    registry: &RenderRegistry,
) -> String {
    let subject_latex = render_expression(subject, registry);
    render_is_refined_command_with_subject_latex(subject_latex, command, registry)
}

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

    format!(
        "{} \\textrm{{ is }} {}",
        subject_latex,
        command_reference_latex(&signature, render.render_called(&substitutions), registry)
    )
}

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

pub(super) fn render_function_type(
    function_type: &FunctionType,
    registry: &RenderRegistry,
) -> String {
    format!(
        "\\left({}\\right) \\Rightarrow \\left({}\\right)",
        function_type
            .inputs
            .iter()
            .map(|spec| render_function_type_spec(spec, registry))
            .collect::<Vec<_>>()
            .join(", "),
        render_function_type_spec(&function_type.output, registry)
    )
}

fn render_function_type_spec(spec: &FunctionTypeSpec, registry: &RenderRegistry) -> String {
    match &spec.kind {
        FunctionTypeSpecKind::Is(ty) => {
            format!(
                "\\_ \\textrm{{ is }} {}",
                render_type_expression(ty, registry)
            )
        }
        FunctionTypeSpecKind::Spec { operator, target } => format!(
            "\\_ {} {}",
            render_quoted_operator(operator),
            escape_math_identifier(target, registry)
        ),
    }
}

pub(super) fn render_type_expression(ty: &TypeExpression, registry: &RenderRegistry) -> String {
    match ty {
        TypeExpression::Builtin { chain, .. } => render_builtin_type_chain(chain),
        TypeExpression::Command(command) => command_called_template(command, registry)
            .map(|called| called.latex)
            .unwrap_or_else(|| render_command_expression(command, registry)),
        TypeExpression::RefinedCommand(command) => render_refined_command_called(command, registry),
        TypeExpression::Function(function_type) => render_function_type(function_type, registry),
        TypeExpression::Coercion { ty, literal, .. } => format!(
            "{}@{}",
            render_type_expression(ty, registry),
            render_set_expression(literal, registry)
        ),
    }
}

pub(super) fn render_builtin_type_chain(chain: &Chain) -> String {
    format!("\\textrm{{{}}}", escape_latex_text(&format_chain(chain)))
}

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
    let subject = render_is_subject(&statement.is_statement.subject, registry);
    let ty = render_type_expression_with_subject(
        &statement.is_statement.ty,
        Some(subject.clone()),
        registry,
    );
    if ty.includes_subject {
        format!(
            "{} \\textrm{{ via }} {}",
            ty.latex,
            render_form_or_declaration(&statement.via, registry)
        )
    } else {
        format!(
            "{} \\textrm{{ is }} {} \\textrm{{ via }} {}",
            subject,
            ty.latex,
            render_form_or_declaration(&statement.via, registry)
        )
    }
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
        FormOrDeclarationKind::MappingParameter { owner, selector } => format!(
            "{}.{}",
            escape_math_identifier(owner, registry),
            escape_math_identifier(selector.name(), registry)
        ),
        FormOrDeclarationKind::FunctionDeclaration { name, form } => {
            let name = name.as_ref().unwrap_or(&form.name);
            if let Some(rendered) = render_documented_mapping_form(name, form, registry) {
                return rendered;
            }
            let args =
                form.magnetic_placeholder
                    .iter()
                    .map(|placeholder| render_form_placeholder_name(&placeholder.name, registry))
                    .chain(form.placeholders.iter().map(|placeholder| {
                        render_form_placeholder_name(&placeholder.name, registry)
                    }))
                    .chain(form.variadic_parameter.iter().map(|parameter| {
                        let suffix = if parameter.tuple { "__" } else { "_" };
                        format!(
                            "{}{}[{} := {}\\ldots {}]",
                            escape_math_identifier(&parameter.name, registry),
                            suffix,
                            escape_math_identifier(&parameter.index, registry),
                            parameter.start,
                            escape_math_identifier(&parameter.length, registry)
                        )
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
            let placeholder = render_set_target(&form.target, registry);
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

fn render_documented_mapping_form(
    name: &str,
    form: &FunctionForm,
    registry: &RenderRegistry,
) -> Option<String> {
    let parameters = form
        .magnetic_placeholder
        .iter()
        .map(|placeholder| placeholder.name.clone())
        .chain(
            form.placeholders
                .iter()
                .map(|placeholder| placeholder.name.clone()),
        )
        .collect::<Vec<_>>();
    let render = registry.mapping_writing.iter().find(|render| {
        render.function_name == name
            && render.parameters == parameters
            && render.magnetic == form.magnetic_placeholder.is_some()
            && render.mapping_written.is_some()
    })?;
    let mut substitutions = HashMap::new();
    substitutions.insert(
        render.function_name.clone(),
        escape_math_identifier(name, registry),
    );
    for parameter in &render.parameters {
        let rendered_parameter = escape_math_identifier(parameter, registry);
        substitutions.insert(parameter.clone(), rendered_parameter.clone());
        if !parameter.ends_with('_') {
            substitutions.insert(format!("{parameter}_"), rendered_parameter);
        }
    }
    Some(substitute_math_template(
        render.mapping_written.as_deref()?,
        &substitutions,
    ))
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
    let ty = command_type_template(command, Some(subject_latex.clone()), registry).unwrap_or_else(
        || TypeTemplate {
            latex: render.render_called(&substitutions),
            includes_subject: false,
        },
    );
    if ty.includes_subject {
        return ty.latex;
    }

    format!("{} \\textrm{{ is }} {}", subject_latex, ty.latex)
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
        "\\left({}\\right) \\to \\left({}\\right)",
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
            render_is_type_with_subject("\\cdot".to_string(), ty, registry)
        }
        FunctionTypeSpecKind::Spec { operator, target } => format!(
            "\\cdot {} {}",
            render_quoted_operator(operator),
            render_expression(target, registry)
        ),
    }
}

fn render_tuple_type(tuple: &TupleType, registry: &RenderRegistry) -> String {
    format!(
        "\\left({}\\right)",
        tuple
            .elements
            .iter()
            .map(|spec| render_function_type_spec(spec, registry))
            .collect::<Vec<_>>()
            .join(", ")
    )
}

fn render_set_type(set: &SetType, registry: &RenderRegistry) -> String {
    let element = match &set.element {
        SetTypeElement::Spec(spec) => render_function_type_spec(spec, registry),
        SetTypeElement::Tuple(tuple) => render_tuple_type(tuple, registry),
    };
    format!("\\left\\{{{} \\: : \\: \\ldots\\right\\}}", element)
}

#[derive(Clone, Debug)]
struct RenderedTypeExpression {
    latex: String,
    includes_subject: bool,
}

pub(super) fn render_type_expression(ty: &TypeExpression, registry: &RenderRegistry) -> String {
    render_type_expression_with_subject(ty, None, registry).latex
}

fn render_is_type_with_subject(
    subject_latex: String,
    ty: &TypeExpression,
    registry: &RenderRegistry,
) -> String {
    let rendered = render_type_expression_with_subject(ty, Some(subject_latex.clone()), registry);
    if rendered.includes_subject {
        rendered.latex
    } else {
        format!("{} \\textrm{{ is }} {}", subject_latex, rendered.latex)
    }
}

fn render_type_expression_with_subject(
    ty: &TypeExpression,
    subject_latex: Option<String>,
    registry: &RenderRegistry,
) -> RenderedTypeExpression {
    match ty {
        TypeExpression::Builtin { chain, .. } => RenderedTypeExpression {
            latex: render_builtin_type_chain(chain),
            includes_subject: false,
        },
        TypeExpression::Command(command) => command_type_template(command, subject_latex, registry)
            .map(|ty| RenderedTypeExpression {
                latex: ty.latex,
                includes_subject: ty.includes_subject,
            })
            .unwrap_or_else(|| RenderedTypeExpression {
                latex: render_command_expression(command, registry),
                includes_subject: false,
            }),
        TypeExpression::RefinedCommand(command) => RenderedTypeExpression {
            latex: render_refined_command_called(command, registry),
            includes_subject: false,
        },
        TypeExpression::Tuple(tuple) => RenderedTypeExpression {
            latex: render_tuple_type(tuple, registry),
            includes_subject: false,
        },
        TypeExpression::Set(set) => RenderedTypeExpression {
            latex: render_set_type(set, registry),
            includes_subject: false,
        },
        TypeExpression::Function(function_type) => RenderedTypeExpression {
            latex: render_function_type(function_type, registry),
            includes_subject: false,
        },
        TypeExpression::Parameter { name, .. } => RenderedTypeExpression {
            latex: escape_math_identifier(name, registry),
            includes_subject: false,
        },
    }
}

pub(super) fn render_builtin_type_chain(chain: &Chain) -> String {
    format!("\\textrm{{{}}}", escape_latex_text(&format_chain(chain)))
}

use std::collections::HashMap;

use crate::backend::semantic::ParsedSourceFile;
use crate::frontend::formulation::ast::{
    BinaryOperator, Chain, ChainPart, CommandExpression, CommandExpressionTailPart, CommandHeader,
    CommandHeaderNode, Expression, ExpressionKind, FormOrDeclaration, FormOrDeclarationKind,
    InfixCommandHeader, IsOrRefinedStatementSpec, IsOrSpec, IsStatement, IsSubject, IsSubjectForm,
    IsSubjectKind, RefinedCommandExpression, RefinedCommandHeader, RefinedExpressionPart,
    RefinedTail, SetExpression, SpecStatement, SpecSubject, SpecSubjectKind,
    TupleExpressionElement, TupleFormElement, TypeExpression, UnaryOperator,
};
use crate::frontend::formulation::{
    parse_command_header, parse_expression, parse_form_or_declaration,
    parse_is_or_refined_statement_spec,
};
use crate::frontend::structural::ast::*;

#[derive(Clone, Debug, Default)]
pub(super) struct RenderRegistry {
    commands: HashMap<String, CommandRender>,
}

#[derive(Clone, Debug)]
struct CommandRender {
    subject_variable: Option<String>,
    parameters: Vec<String>,
    called: String,
    written: Option<String>,
}

pub(super) fn build_render_registry(files: &[ParsedSourceFile]) -> RenderRegistry {
    let mut registry = RenderRegistry::default();

    for file in files {
        for item in &file.document.items {
            let Some(entry) = render_entry(item) else {
                continue;
            };
            registry.commands.insert(entry.signature, entry.render);
        }
    }

    registry
}

pub(super) fn render_formulation_latex(text: &str, registry: &RenderRegistry) -> Option<String> {
    render_parsed_formulation_latex(text, registry)
        .or_else(|| render_simple_set_spec_latex(text, registry))
}

pub(super) fn render_group_heading_latex(
    kind: &str,
    heading: Option<&str>,
    registry: &RenderRegistry,
) -> Option<String> {
    if !matches!(kind, "Defines" | "Describes" | "Refines") {
        return None;
    }

    let header = parse_command_header(heading?).ok()?;
    let render = registry.commands.get(&command_header_signature(&header))?;
    let substitutions = command_header_substitutions(&header, registry);

    Some(render_called_template(
        &capitalize_first_plain_text_word(&render.called),
        &substitutions,
    ))
}

fn capitalize_first_plain_text_word(template: &str) -> String {
    let mut result = String::with_capacity(template.len());
    let mut in_math = false;
    let mut changed = false;

    for ch in template.chars() {
        if ch == '$' {
            in_math = !in_math;
        }

        if !changed && !in_math && ch.is_ascii_alphabetic() {
            result.push(ch.to_ascii_uppercase());
            changed = true;
        } else {
            result.push(ch);
        }
    }

    result
}

fn render_parsed_formulation_latex(text: &str, registry: &RenderRegistry) -> Option<String> {
    if let Ok(expression) = parse_expression(text) {
        return Some(render_expression(&expression, registry));
    }

    if let Ok(spec) = parse_is_or_refined_statement_spec(text) {
        return Some(render_is_or_refined_spec(&spec, registry));
    }

    parse_form_or_declaration(text)
        .ok()
        .map(|form| render_form_or_declaration(&form, registry))
}

struct RenderEntry {
    signature: String,
    render: CommandRender,
}

fn render_entry(item: &TopLevelItem) -> Option<RenderEntry> {
    match item {
        TopLevelItem::Describes(group) => render_entry_from_parts(
            command_header_signature(&group.heading),
            command_header_parameters(&group.heading),
            primary_form_name(&group.describes.argument),
            group.documented.as_ref(),
        ),
        TopLevelItem::Defines(group) => render_entry_from_parts(
            command_header_signature(&group.heading),
            command_header_parameters(&group.heading),
            primary_is_or_spec_name(&group.defines.argument),
            group.documented.as_ref(),
        ),
        TopLevelItem::Refines(group) => render_entry_from_parts(
            command_header_signature(&group.heading),
            command_header_parameters(&group.heading),
            primary_is_or_refined_spec_name(&group.refines.argument),
            group.documented.as_ref(),
        ),
        _ => None,
    }
}

fn render_entry_from_parts(
    signature: String,
    parameters: Vec<String>,
    subject_variable: Option<String>,
    documented: Option<&DocumentedSection>,
) -> Option<RenderEntry> {
    let documented = documented?;
    let called = documented.arguments.iter().find_map(|item| match item {
        DocumentedItem::Called(group) => Some(group),
        _ => None,
    })?;

    Some(RenderEntry {
        signature,
        render: CommandRender {
            subject_variable,
            parameters,
            called: join_called_text(&called.called),
            written: called.written.as_ref().map(join_written_text),
        },
    })
}

fn join_called_text(section: &CalledSection) -> String {
    section
        .arguments
        .iter()
        .map(|text| text.0.as_str())
        .collect::<Vec<_>>()
        .join(" ")
}

fn join_written_text(section: &WrittenSection) -> String {
    section
        .arguments
        .iter()
        .map(|text| text.0.as_str())
        .collect::<Vec<_>>()
        .join(" ")
}

fn render_expression(expression: &Expression, registry: &RenderRegistry) -> String {
    match &expression.kind {
        ExpressionKind::Name(name) => escape_math_identifier(name),
        ExpressionKind::FunctionCall { name, arguments } => {
            let args = arguments
                .iter()
                .map(|argument| render_expression(argument, registry))
                .collect::<Vec<_>>()
                .join(", ");
            format!("{}\\left({}\\right)", escape_math_identifier(name), args)
        }
        ExpressionKind::FunctionNamedCall { name, elements } => {
            let args = elements
                .iter()
                .map(|element| render_expression(&element.expression, registry))
                .collect::<Vec<_>>()
                .join(", ");
            format!("{}\\left({}\\right)", escape_math_identifier(name), args)
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
            render_command_expression(command, registry)
        ),
        ExpressionKind::IsNotPredicate { subject, command } => format!(
            "{} \\textrm{{ is not }} {}",
            render_expression(subject, registry),
            render_command_expression(command, registry)
        ),
        ExpressionKind::IsType { subject, ty } => match ty {
            TypeExpression::Command(command) => render_is_command(subject, command, registry),
            TypeExpression::RefinedCommand(command) => {
                render_is_refined_command(subject, command, registry)
            }
        },
    }
}

fn render_set_expression(set: &SetExpression, registry: &RenderRegistry) -> String {
    let target = render_placeholder_form(&set.target);
    let spec = render_spec_statement(&set.spec, registry);

    match &set.predicate {
        Some(predicate) => format!(
            "\\left\\{{ {target} \\: : \\: {spec} \\: | \\: {} \\right\\}}",
            render_expression(predicate, registry)
        ),
        None => format!("\\left\\{{ {target} \\: : \\: {spec} \\right\\}}"),
    }
}

fn render_spec_statement(statement: &SpecStatement, registry: &RenderRegistry) -> String {
    format!(
        "{} {} {}",
        render_expression(&statement.subject, registry),
        render_quoted_operator(&statement.operator),
        escape_math_identifier(&statement.name)
    )
}

fn render_simple_set_spec_latex(text: &str, registry: &RenderRegistry) -> Option<String> {
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

fn render_latex_fragment(text: &str, registry: &RenderRegistry) -> String {
    render_parsed_formulation_latex(text, registry)
        .unwrap_or_else(|| escape_latex_math(text.trim()))
}

fn split_once_top_level(input: &str, delimiter: char) -> Option<(&str, &str)> {
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

fn render_is_or_refined_spec(spec: &IsOrRefinedStatementSpec, registry: &RenderRegistry) -> String {
    match spec {
        IsOrRefinedStatementSpec::Is(statement) => render_is_statement(statement, registry),
        IsOrRefinedStatementSpec::Spec(statement) => format!(
            "{} {} {}",
            render_spec_subject(&statement.subject, registry),
            render_quoted_operator(&statement.operator),
            escape_math_identifier(&statement.name)
        ),
    }
}

fn render_is_statement(statement: &IsStatement, registry: &RenderRegistry) -> String {
    let subject_latex = render_is_subject(&statement.subject, registry);
    match &statement.ty {
        TypeExpression::Command(command) => {
            render_is_command_with_subject_latex(subject_latex, command, registry)
        }
        TypeExpression::RefinedCommand(command) => {
            render_is_refined_command_with_subject_latex(subject_latex, command, registry)
        }
    }
}

fn render_is_subject(subject: &IsSubject, registry: &RenderRegistry) -> String {
    match &subject.kind {
        IsSubjectKind::Forms(forms) => forms
            .iter()
            .map(|form| match form {
                IsSubjectForm::Form(form) => render_form_or_declaration(form, registry),
                IsSubjectForm::PlaceholderForm(form) => render_placeholder_form(form),
            })
            .collect::<Vec<_>>()
            .join(", "),
        IsSubjectKind::Operator(operator) => render_operator_text(&operator.text),
    }
}

fn render_spec_subject(subject: &SpecSubject, registry: &RenderRegistry) -> String {
    match &subject.kind {
        SpecSubjectKind::Form(form) => render_form_or_declaration(form, registry),
        SpecSubjectKind::Operator(operator) => render_operator_text(&operator.text),
    }
}

fn render_form_or_declaration(form: &FormOrDeclaration, registry: &RenderRegistry) -> String {
    match &form.kind {
        FormOrDeclarationKind::Name(name) => escape_math_identifier(name),
        FormOrDeclarationKind::FunctionDeclaration { name, form } => {
            let name = name.as_ref().unwrap_or(&form.name);
            let args = form
                .magnetic_placeholder
                .iter()
                .map(|placeholder| render_form_placeholder_name(&placeholder.name))
                .chain(
                    form.placeholders
                        .iter()
                        .map(|placeholder| render_form_placeholder_name(&placeholder.name)),
                )
                .collect::<Vec<_>>()
                .join(", ");
            if args.is_empty() {
                escape_math_identifier(name)
            } else {
                format!("{}\\left({}\\right)", escape_math_identifier(name), args)
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
                    "{} := \\left({rendered}\\right)",
                    escape_math_identifier(name)
                ),
                None => format!("\\left({rendered}\\right)"),
            }
        }
        FormOrDeclarationKind::SetDeclaration { name, form } => {
            let rendered = format!(
                "\\left\\{{{}\\right\\}}",
                render_placeholder_form(&form.placeholder_form)
            );
            match name {
                Some(name) => format!("{} := {rendered}", escape_math_identifier(name)),
                None => rendered,
            }
        }
        FormOrDeclarationKind::InfixOperator {
            left,
            operator,
            right,
        } => format!(
            "{} {} {}",
            render_form_placeholder_name(&left.name),
            render_operator_text(&operator.text),
            render_form_placeholder_name(&right.name)
        ),
        FormOrDeclarationKind::PrefixOperator {
            operator,
            placeholder,
        } => format!(
            "{}{}",
            render_operator_text(&operator.text),
            render_form_placeholder_name(&placeholder.name)
        ),
        FormOrDeclarationKind::PostfixOperator {
            placeholder,
            operator,
        } => format!(
            "{}{}",
            render_form_placeholder_name(&placeholder.name),
            render_operator_text(&operator.text)
        ),
    }
}

fn render_is_command(
    subject: &Expression,
    command: &CommandExpression,
    registry: &RenderRegistry,
) -> String {
    let subject_latex = render_expression(subject, registry);
    render_is_command_with_subject_latex(subject_latex, command, registry)
}

fn render_is_refined_command(
    subject: &Expression,
    command: &RefinedCommandExpression,
    registry: &RenderRegistry,
) -> String {
    let subject_latex = render_expression(subject, registry);
    render_is_refined_command_with_subject_latex(subject_latex, command, registry)
}

fn render_is_command_with_subject_latex(
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

    if let Some(written) = &render.written {
        let includes_subject = render
            .subject_variable
            .as_ref()
            .is_some_and(|name| template_contains_placeholder(written, name));
        let rendered = substitute_math_template(written, &substitutions);
        if includes_subject {
            rendered
        } else {
            format!("{subject_latex} \\textrm{{ is }} {rendered}")
        }
    } else {
        format!(
            "{} \\textrm{{ is }} {}",
            subject_latex,
            render_called_template(&render.called, &substitutions)
        )
    }
}

fn render_is_refined_command_with_subject_latex(
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

fn render_command_expression(command: &CommandExpression, registry: &RenderRegistry) -> String {
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

fn render_refined_command_called(
    command: &RefinedCommandExpression,
    registry: &RenderRegistry,
) -> String {
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

    render_called_template(&template, &substitutions)
}

fn command_substitutions(
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

fn command_substitutions_for_names(
    names: &[String],
    values: Vec<String>,
) -> HashMap<String, String> {
    names.iter().cloned().zip(values).collect()
}

fn command_header_substitutions(
    header: &CommandHeader,
    registry: &RenderRegistry,
) -> HashMap<String, String> {
    let mut substitutions = HashMap::new();

    for form in command_header_forms(header) {
        if let Some(name) = primary_form_name(form) {
            substitutions.insert(name, render_form_or_declaration(form, registry));
        }
    }

    substitutions
}

fn command_header_forms(header: &CommandHeader) -> Vec<&FormOrDeclaration> {
    match header {
        CommandHeader::Command(header) => simple_command_header_forms(header),
        CommandHeader::Infix(header) => infix_command_header_forms(header),
        CommandHeader::Refined(header) => refined_command_header_forms(header),
    }
}

fn simple_command_header_forms(header: &CommandHeaderNode) -> Vec<&FormOrDeclaration> {
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

fn infix_command_header_forms(header: &InfixCommandHeader) -> Vec<&FormOrDeclaration> {
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

fn refined_command_header_forms(header: &RefinedCommandHeader) -> Vec<&FormOrDeclaration> {
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

fn refined_command_base_argument_values(
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

fn refined_command_part_argument_values(
    command: &RefinedCommandExpression,
    part: &RefinedExpressionPart,
    registry: &RenderRegistry,
) -> Vec<String> {
    refined_command_base_argument_values(command, registry)
        .into_iter()
        .chain(expression_tail_argument_values(&part.tail, registry))
        .collect()
}

fn command_argument_values(command: &CommandExpression, registry: &RenderRegistry) -> Vec<String> {
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

fn expression_tail_argument_values(
    tail: &[CommandExpressionTailPart],
    registry: &RenderRegistry,
) -> Vec<String> {
    tail.iter()
        .flat_map(|part| part.args.iter())
        .flat_map(|args| args.expressions.iter())
        .map(|expression| render_expression(expression, registry))
        .collect()
}

fn render_command_like(chain: &Chain, _registry: &RenderRegistry) -> String {
    format!("\\backslash{}", escape_latex_math(&format_chain(chain)))
}

fn render_binary_operator(operator: &BinaryOperator) -> String {
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

fn render_operator_text(operator: &str) -> String {
    match operator {
        "*" => "\\ast".to_string(),
        _ => escape_latex_math(operator),
    }
}

fn render_quoted_operator(operator: &str) -> String {
    // Temporary rendering until full type-checking can resolve quoted operators semantically.
    format!("\\{}", escape_latex_command_name(operator))
}

fn render_subset_call(call: &crate::frontend::formulation::ast::SubsetCall) -> String {
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

fn render_placeholder_form(form: &crate::frontend::formulation::ast::PlaceholderForm) -> String {
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
                "{}\\left({arguments}\\right)",
                render_form_placeholder_name(&placeholder.name)
            )
        }
    }
}

fn render_form_placeholder_name(name: &str) -> String {
    let trimmed = name.trim_end_matches('_');
    if trimmed.is_empty() {
        escape_math_identifier(name)
    } else {
        escape_math_identifier(trimmed)
    }
}

fn render_called_template(template: &str, substitutions: &HashMap<String, String>) -> String {
    let mut result = String::new();
    let mut in_math = false;

    for segment in template.split('$') {
        if in_math {
            result.push_str(&substitute_math_template(segment, substitutions));
        } else if !segment.is_empty() {
            result.push_str(&format!("\\textrm{{{}}}", escape_latex_text(segment)));
        }
        in_math = !in_math;
    }

    result
}

fn substitute_math_template(template: &str, substitutions: &HashMap<String, String>) -> String {
    let mut result = String::new();
    let chars = template.chars().collect::<Vec<_>>();
    let mut index = 0;

    while index < chars.len() {
        if is_placeholder_start(chars[index]) {
            let start = index;
            index += 1;
            while index < chars.len() && is_placeholder_continue(chars[index]) {
                index += 1;
            }
            if index < chars.len() && chars[index] == '?' {
                let name = chars[start..index].iter().collect::<String>();
                if let Some(value) = substitutions.get(&name) {
                    result.push_str(value);
                    index += 1;
                    continue;
                }
            }
            result.extend(chars[start..index].iter());
        } else {
            result.push(chars[index]);
            index += 1;
        }
    }

    result
}

fn template_contains_placeholder(template: &str, name: &str) -> bool {
    let needle = format!("{name}?");
    template.contains(&needle)
}

fn is_placeholder_start(ch: char) -> bool {
    ch.is_ascii_alphabetic() || ch == '_'
}

fn is_placeholder_continue(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_' || ch == '.'
}

fn primary_is_or_spec_name(spec: &IsOrSpec) -> Option<String> {
    match spec {
        IsOrSpec::Is(statement) => primary_is_statement_name(statement),
        IsOrSpec::Spec(statement) => primary_spec_subject_name(&statement.subject),
    }
}

fn primary_is_or_refined_spec_name(spec: &IsOrRefinedStatementSpec) -> Option<String> {
    match spec {
        IsOrRefinedStatementSpec::Is(statement) => primary_is_statement_name(statement),
        IsOrRefinedStatementSpec::Spec(statement) => primary_spec_subject_name(&statement.subject),
    }
}

fn primary_is_statement_name(statement: &IsStatement) -> Option<String> {
    primary_is_subject_name(&statement.subject)
}

fn primary_is_subject_name(subject: &IsSubject) -> Option<String> {
    match &subject.kind {
        IsSubjectKind::Forms(forms) => forms.iter().find_map(|form| match form {
            IsSubjectForm::Form(form) => primary_form_name(form),
            IsSubjectForm::PlaceholderForm(form) => primary_placeholder_form_name(form),
        }),
        IsSubjectKind::Operator(_) => None,
    }
}

fn primary_spec_subject_name(subject: &SpecSubject) -> Option<String> {
    match &subject.kind {
        SpecSubjectKind::Form(form) => primary_form_name(form),
        SpecSubjectKind::Operator(_) => None,
    }
}

fn primary_form_name(form: &FormOrDeclaration) -> Option<String> {
    match &form.kind {
        FormOrDeclarationKind::Name(name) => Some(name.clone()),
        FormOrDeclarationKind::FunctionDeclaration { name, form } => {
            name.clone().or_else(|| Some(form.name.clone()))
        }
        FormOrDeclarationKind::TupleDeclaration { name, .. }
        | FormOrDeclarationKind::SetDeclaration { name, .. } => name.clone(),
        FormOrDeclarationKind::InfixOperator { .. }
        | FormOrDeclarationKind::PrefixOperator { .. }
        | FormOrDeclarationKind::PostfixOperator { .. } => None,
    }
}

fn primary_placeholder_form_name(
    form: &crate::frontend::formulation::ast::PlaceholderForm,
) -> Option<String> {
    match &form.kind {
        crate::frontend::formulation::ast::PlaceholderFormKind::Placeholder(placeholder) => {
            Some(placeholder.name.clone())
        }
        crate::frontend::formulation::ast::PlaceholderFormKind::Function {
            placeholder, ..
        } => Some(placeholder.name.clone()),
    }
}

fn command_header_signature(header: &crate::frontend::formulation::ast::CommandHeader) -> String {
    match header {
        crate::frontend::formulation::ast::CommandHeader::Command(command) => {
            let mut signature = format!("\\{}", format_chain(&command.chain));
            add_header_tail_signature(&mut signature, &command.tail);
            signature
        }
        crate::frontend::formulation::ast::CommandHeader::Infix(command) => {
            format!("\\:{}:/", format_chain(&command.chain))
        }
        crate::frontend::formulation::ast::CommandHeader::Refined(command) => {
            refined_command_header_signature(command)
        }
    }
}

fn command_header_parameters(
    header: &crate::frontend::formulation::ast::CommandHeader,
) -> Vec<String> {
    command_header_forms(header)
        .into_iter()
        .filter_map(primary_form_name)
        .collect()
}

fn command_expression_signature(command: &CommandExpression) -> String {
    let mut signature = format!("\\{}", format_chain(&command.chain));
    add_expression_tail_signature(&mut signature, &command.tail);
    signature
}

fn refined_command_header_signature(command: &RefinedCommandHeader) -> String {
    let mut signature = "\\".to_string();
    if let Some(prefix) = &command.prefix_chain {
        signature.push_str(&format_chain(prefix));
        signature.push_str("::");
    }
    for (index, part) in command.parts.iter().enumerate() {
        if index > 0 {
            signature.push_str("::");
        }
        signature.push_str(&format_chain(&part.chain));
        add_header_tail_signature(&mut signature, &part.tail);
    }
    signature.push_str("::");
    signature.push_str(&refined_tail_signature(&command.refined_tail));
    add_header_tail_signature(&mut signature, &command.tail);
    signature
}

fn refined_command_base_signature(command: &RefinedCommandExpression) -> String {
    let mut signature = format!("\\{}", refined_tail_signature(&command.refined_tail));
    add_expression_tail_signature(&mut signature, &command.tail);
    signature
}

fn refined_command_part_signature(
    command: &RefinedCommandExpression,
    part: &RefinedExpressionPart,
) -> String {
    let mut signature = "\\".to_string();
    if let Some(prefix) = &command.prefix_chain {
        signature.push_str(&format_chain(prefix));
        signature.push_str("::");
    }
    signature.push_str(&format_chain(&part.chain));
    add_expression_tail_signature(&mut signature, &part.tail);
    signature.push_str("::");
    signature.push_str(&refined_tail_signature(&command.refined_tail));
    add_expression_tail_signature(&mut signature, &command.tail);
    signature
}

fn add_header_tail_signature(
    signature: &mut String,
    tail: &[crate::frontend::formulation::ast::CommandHeaderTailPart],
) {
    for part in tail {
        signature.push(':');
        signature.push_str(&format_chain(&part.chain));
    }
}

fn add_expression_tail_signature(signature: &mut String, tail: &[CommandExpressionTailPart]) {
    for part in tail {
        signature.push(':');
        signature.push_str(&format_chain(&part.chain));
    }
}

fn refined_tail_signature(tail: &RefinedTail) -> String {
    match tail {
        RefinedTail::Chain(chain) => format_chain(chain),
        RefinedTail::Name { name, .. } => name.clone(),
    }
}

fn format_chain(chain: &Chain) -> String {
    chain
        .parts
        .iter()
        .map(|part| match part {
            ChainPart::Name(name) => name.clone(),
            ChainPart::Alias(name) => format!("${name}"),
            ChainPart::Operator(operator) => operator.clone(),
        })
        .collect::<Vec<_>>()
        .join(".")
}

fn escape_math_identifier(value: &str) -> String {
    escape_latex_math(value)
}

fn escape_latex_math(value: &str) -> String {
    value
        .replace('\\', "\\backslash ")
        .replace('_', "\\_")
        .replace('{', "\\{")
        .replace('}', "\\}")
}

fn escape_latex_command_name(value: &str) -> String {
    value
        .chars()
        .filter(|ch| ch.is_ascii_alphabetic())
        .collect()
}

fn escape_latex_text(value: &str) -> String {
    value
        .replace('\\', "\\textbackslash{}")
        .replace('{', "\\{")
        .replace('}', "\\}")
}

#[cfg(test)]
mod tests {
    use super::{build_render_registry, render_formulation_latex, render_group_heading_latex};
    use crate::backend::semantic::ParsedSourceFile;
    use crate::events::EventLog;
    use crate::frontend::structural::parse_document;
    use std::path::PathBuf;

    fn registry_for(source: &str) -> super::RenderRegistry {
        let mut event_log = EventLog::new();
        let document = parse_document(source, &mut event_log);
        assert!(event_log.events().is_empty());
        build_render_registry(&[ParsedSourceFile {
            path: PathBuf::from("test.mlg"),
            source: source.to_string(),
            document,
        }])
    }

    #[test]
    fn renders_written_templates_with_subject_and_argument_substitutions() {
        let registry = registry_for(
            r#"[\function:on{A}:to{B}]
Describes: f(x__)
Documented:
. called: "function on $A?$ to $B?$"
  written:
  . "f? \: : \: A? \rightarrow B?"
"#,
        );

        assert_eq!(
            render_formulation_latex(r#"g is \function:on{X}:to{Y}"#, &registry),
            Some(r#"g \: : \: X \rightarrow Y"#.to_string())
        );
    }

    #[test]
    fn renders_called_templates_as_text_when_written_is_missing() {
        let registry = registry_for(
            r#"[\group]
Describes: G := (X, *, e)
Documented:
. called: "group"
"#,
        );

        assert_eq!(
            render_formulation_latex(r#"G is \group"#, &registry),
            Some(r#"G \textrm{ is } \textrm{group}"#.to_string())
        );
    }

    #[test]
    fn renders_comma_separated_is_or_spec_subjects_with_called_commands() {
        let registry = registry_for(
            r#"[\set]
Describes: X
Documented:
. called: "set"
"#,
        );

        assert_eq!(
            render_formulation_latex(r#"A, B is \set"#, &registry),
            Some(r#"A, B \textrm{ is } \textrm{set}"#.to_string())
        );
    }

    #[test]
    fn renders_quoted_operators_as_temporary_latex_commands() {
        let registry = registry_for("");

        assert_eq!(
            render_formulation_latex(r#"x "in" X"#, &registry),
            Some(r#"x \in X"#.to_string())
        );
        assert_eq!(
            render_formulation_latex(r#"a "to" B"#, &registry),
            Some(r#"a \to B"#.to_string())
        );
    }

    #[test]
    fn renders_called_templates_with_math_substitutions() {
        let registry = registry_for(
            r#"[\field:over{V}]
Describes: F
Documented:
. called: "field over $V?$"
"#,
        );

        assert_eq!(
            render_formulation_latex(r#"G is \field:over{X}"#, &registry),
            Some(r#"G \textrm{ is } \textrm{field over }X"#.to_string())
        );
    }

    #[test]
    fn renders_refined_command_types_from_called_templates() {
        let registry = registry_for(
            r#"[\function:on{A}:to{B}]
Describes: f(x__)
Documented:
. called: "function on $A?$ to $B?$"
  written:
  . "f? \: : \: A? \rightarrow B?"

[\(bounded)::function:on{A}:to{B}]
Refines: f(x__) is \function:on{A}:to{B}
Documented:
. called: "bounded"
  written:
  . "\operatorname{Bounded}"

[\(continuous)::function:on{A}:to{B}]
Refines: f(x__) is \function:on{A}:to{B}
Documented:
. called: "continuous"
  written:
  . "\operatorname{Continuous}"
"#,
        );

        assert_eq!(
            render_formulation_latex(r#"g is \function:on{X}:to{Y}"#, &registry),
            Some(r#"g \: : \: X \rightarrow Y"#.to_string())
        );
        assert_eq!(
            render_formulation_latex(
                r#"g is \(bounded, continuous)::function:on{X}:to{Y}"#,
                &registry
            ),
            Some(
                r#"g \textrm{ is } \textrm{bounded, continuous function on }X\textrm{ to }Y"#
                    .to_string()
            )
        );
    }

    #[test]
    fn renders_definition_group_headings_from_called_text() {
        let registry = registry_for(
            r#"[\function:on{A}:to{B}]
Describes: f(x__)
Documented:
. called: "function on $A?$ to $B?$"
"#,
        );

        assert_eq!(
            render_group_heading_latex("Describes", Some(r#"\function:on{A}:to{B}"#), &registry),
            Some(r#"\textrm{Function on }A\textrm{ to }B"#.to_string())
        );
    }

    #[test]
    fn capitalizes_definition_group_heading_called_text() {
        let registry = registry_for(
            r#"[\set]
Describes: X
Documented:
. called: "set"
"#,
        );

        assert_eq!(
            render_group_heading_latex("Describes", Some(r#"\set"#), &registry),
            Some(r#"\textrm{Set}"#.to_string())
        );
        assert_eq!(
            render_formulation_latex(r#"X is \set"#, &registry),
            Some(r#"X \textrm{ is } \textrm{set}"#.to_string())
        );
    }

    #[test]
    fn renders_function_forms_with_placeholder_suffixes_hidden() {
        let registry = registry_for("");

        assert_eq!(
            render_formulation_latex("f(x_)", &registry),
            Some(r#"f\left(x\right)"#.to_string())
        );
        assert_eq!(
            render_formulation_latex("f(x__)", &registry),
            Some(r#"f\left(x\right)"#.to_string())
        );
    }

    #[test]
    fn renders_tuple_declarations_with_operator_symbols() {
        let registry = registry_for("");

        assert_eq!(
            render_formulation_latex("G := (X, *, e)", &registry),
            Some(r#"G := \left(X, \ast, e\right)"#.to_string())
        );
    }

    #[test]
    fn renders_set_builder_specs() {
        let registry = registry_for("");

        assert_eq!(
            render_formulation_latex("{x : y | z}", &registry),
            Some(r#"\left\{ x \: : \: y \: | \: z \right\}"#.to_string())
        );
        assert_eq!(
            render_formulation_latex(r#"{x "in" X : f_(a_, b_) | z}"#, &registry),
            Some(r#"\left\{ f\left(a, b\right) \: : \: x \in X \: | \: z \right\}"#.to_string())
        );
    }
}

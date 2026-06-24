use super::*;

#[derive(Clone, Debug, Default)]
pub(in crate::backend::view) struct RenderRegistry {
    /// Map from canonical command signature to the rendering data for that command.
    pub(super) commands: HashMap<String, CommandRender>,
}

#[derive(Clone, Debug)]
pub(super) struct CommandRender {
    pub(super) subject_variable: Option<String>,
    pub(super) parameters: Vec<String>,
    pub(super) called: String,
    pub(super) written: Option<String>,
}

pub(in crate::backend::view) fn build_render_registry(
    files: &[ParsedSourceFile],
) -> RenderRegistry {
    let mut registry = RenderRegistry::default();

    for file in files {
        for item in &file.document.items {
            for entry in render_entries(item) {
                registry.commands.insert(entry.signature, entry.render);
            }
        }
    }

    registry
}

pub(in crate::backend::view) fn render_formulation_latex(
    text: &str,
    registry: &RenderRegistry,
) -> Option<String> {
    render_parsed_formulation_latex(text, registry)
        .or_else(|| render_simple_set_spec_latex(text, registry))
}

pub(in crate::backend::view) fn render_group_heading_latex(
    kind: &str,
    heading: Option<&str>,
    primary_inline_argument: Option<&str>,
    registry: &RenderRegistry,
) -> Option<String> {
    if !matches!(kind, "Defines" | "Describes" | "Refines") {
        return None;
    }

    let header = parse_command_header(heading?).ok()?;
    let render = registry.commands.get(&command_header_signature(&header))?;

    if kind == "Refines"
        && let Some(latex) =
            render_refines_group_heading_latex(&header, primary_inline_argument, render, registry)
    {
        return Some(latex);
    }

    let substitutions = command_header_substitutions(&header);

    Some(render_called_template(&render.called, &substitutions))
}

pub(super) fn render_refines_group_heading_latex(
    header: &CommandHeader,
    refines_argument: Option<&str>,
    refinement_render: &CommandRender,
    registry: &RenderRegistry,
) -> Option<String> {
    let statement = parse_refined_declaration_statement(refines_argument?).ok()?;
    let target = refines_target_type(&statement)?;
    let target_called = type_expression_called_template(target, registry)?;
    let mut substitutions = target_called.substitutions;
    substitutions.extend(command_header_substitutions(header));
    let template = format!("{} {}", refinement_render.called, target_called.template);

    Some(render_called_template(&template, &substitutions))
}

pub(super) fn render_parsed_formulation_latex(
    text: &str,
    registry: &RenderRegistry,
) -> Option<String> {
    if let Ok(expression) = parse_expression(text) {
        return Some(render_expression(&expression, registry));
    }

    if let Ok(statement) = parse_refined_declaration_statement(text) {
        return Some(render_declaration_statement(&statement, registry));
    }

    parse_form_or_declaration(text)
        .ok()
        .map(|form| render_form_or_declaration(&form))
}

pub(super) struct RenderEntry {
    /// Canonical command signature used as the registry key.
    signature: String,
    /// Rendering metadata stored for that signature.
    render: CommandRender,
}

pub(super) fn render_entries(item: &TopLevelItem) -> Vec<RenderEntry> {
    match item {
        TopLevelItem::Describes(group) => render_entries_from_parts(
            command_header_signatures(&group.heading),
            primary_form_name(&group.describes.argument),
            group.documented.as_ref(),
        ),
        TopLevelItem::Defines(group) => render_entries_from_parts(
            command_header_signatures(&group.heading),
            primary_declaration_statement_name(&group.defines.argument),
            group.documented.as_ref(),
        ),
        TopLevelItem::Refines(group) => render_entries_from_parts(
            command_header_signatures(&group.heading),
            primary_declaration_statement_name(&group.refines.argument),
            group.documented.as_ref(),
        ),
        _ => Vec::new(),
    }
}

pub(super) fn render_entries_from_parts(
    signatures: Vec<CommandHeaderSignature>,
    subject_variable: Option<String>,
    documented: Option<&DocumentedSection>,
) -> Vec<RenderEntry> {
    let Some(documented) = documented else {
        return Vec::new();
    };
    let Some(called) = documented.arguments.iter().find_map(|item| match item {
        DocumentedItem::Called(group) => Some(group),
        _ => None,
    }) else {
        return Vec::new();
    };
    let documented_written = documented.arguments.iter().find_map(|item| match item {
        DocumentedItem::Written(group) => Some(&group.written),
        _ => None,
    });
    let written = called.written.as_ref().or(documented_written);

    signatures
        .into_iter()
        .map(|signature| RenderEntry {
            signature: signature.signature,
            render: CommandRender {
                subject_variable: subject_variable.clone(),
                parameters: signature.parameters,
                called: join_called_text(&called.called),
                written: written.map(join_written_text),
            },
        })
        .collect()
}

pub(super) fn join_called_text(section: &CalledSection) -> String {
    section
        .arguments
        .iter()
        .map(|text| text.0.as_str())
        .collect::<Vec<_>>()
        .join(" ")
}

pub(super) fn join_written_text(section: &WrittenSection) -> String {
    section
        .arguments
        .iter()
        .map(|text| text.0.as_str())
        .collect::<Vec<_>>()
        .join(" ")
}

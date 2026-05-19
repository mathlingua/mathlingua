use super::*;

/// Lookup table of documented rendering metadata by canonical command signature.
///
/// The registry is built once from all parsed files before individual
/// formulations are rendered.  Rendering can then resolve a command use such as
/// `\function:on{A}:to{B}` to its `called:` and optional `written:` templates.
#[derive(Clone, Debug, Default)]
pub(in crate::backend::view) struct RenderRegistry {
    /// Map from canonical command signature to the rendering data for that command.
    pub(super) commands: HashMap<String, CommandRender>,
}

/// Rendering metadata extracted from a definition-like group.
///
/// `called` is always present because semantic checking requires it for
/// renderable commands.  `written` is optional and is used only when a command
/// has a math-mode representation.
#[derive(Clone, Debug)]
pub(super) struct CommandRender {
    /// Primary subject variable from the defining form, such as `f` in `f(x__)`.
    pub(super) subject_variable: Option<String>,
    /// Parameter names supplied in the command header, in substitution order.
    pub(super) parameters: Vec<String>,
    /// Joined plain-LaTeX-mode `called:` template.
    pub(super) called: String,
    /// Joined math-LaTeX-mode `written:` template, if provided.
    pub(super) written: Option<String>,
}

/// Builds the render registry from all parsed source files.
///
/// Only `Defines`, `Describes`, and `Refines` groups with documented `called:`
/// metadata contribute entries.  Later entries with the same signature replace
/// earlier ones here; duplicate detection is handled by the semantic checker
/// before view rendering proceeds.
pub(in crate::backend::view) fn build_render_registry(
    files: &[ParsedSourceFile],
) -> RenderRegistry {
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

/// Attempts to render one formulation string as inline LaTeX.
///
/// The renderer first tries full formulation parsers and then falls back to a
/// small set-spec parser used for legacy/simple set-builder text.
pub(in crate::backend::view) fn render_formulation_latex(
    text: &str,
    registry: &RenderRegistry,
) -> Option<String> {
    render_parsed_formulation_latex(text, registry)
        .or_else(|| render_simple_set_spec_latex(text, registry))
}

/// Renders a group card heading as LaTeX when documented metadata is available.
///
/// Definition-like groups use their own `called:` template.  `Refines` groups
/// additionally combine the refinement's `called:` template with the called text
/// of the command being refined, so a card can display titles like
/// `continuous function on A to B`.
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

/// Builds the specialized heading for a `Refines` group.
///
/// The `Refines:` section tells us what command is being refined.  The heading
/// combines the refinement label, such as `continuous`, with the target command's
/// called template, such as `function on A to B`.
pub(super) fn render_refines_group_heading_latex(
    header: &CommandHeader,
    refines_argument: Option<&str>,
    refinement_render: &CommandRender,
    registry: &RenderRegistry,
) -> Option<String> {
    let spec = parse_is_or_refined_statement_spec(refines_argument?).ok()?;
    let target = refines_target_type(&spec)?;
    let target_called = type_expression_called_template(target, registry)?;
    let mut substitutions = target_called.substitutions;
    substitutions.extend(command_header_substitutions(header));
    let template = format!("{} {}", refinement_render.called, target_called.template);

    Some(render_called_template(&template, &substitutions))
}

/// Attempts all supported parsers for a formulation and renders the first match.
pub(super) fn render_parsed_formulation_latex(
    text: &str,
    registry: &RenderRegistry,
) -> Option<String> {
    if let Ok(expression) = parse_expression(text) {
        return Some(render_expression(&expression, registry));
    }

    if let Ok(spec) = parse_is_or_refined_statement_spec(text) {
        return Some(render_is_or_refined_spec(&spec, registry));
    }

    parse_form_or_declaration(text)
        .ok()
        .map(|form| render_form_or_declaration(&form))
}

/// Registry insertion record for one renderable command definition.
pub(super) struct RenderEntry {
    /// Canonical command signature used as the registry key.
    signature: String,
    /// Rendering metadata stored for that signature.
    render: CommandRender,
}

/// Extracts a render registry entry from a top-level structural item.
///
/// Non-definition-like items are ignored because they do not define reusable
/// command rendering templates.
pub(super) fn render_entry(item: &TopLevelItem) -> Option<RenderEntry> {
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

/// Builds a render entry from the common pieces of a definition-like group.
///
/// This function centralizes extraction of `called:`, optional `written:`, the
/// subject variable, and header parameter names across `Defines`, `Describes`,
/// and `Refines`.
pub(super) fn render_entry_from_parts(
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
    let documented_written = documented.arguments.iter().find_map(|item| match item {
        DocumentedItem::Written(group) => Some(&group.written),
        _ => None,
    });
    let written = called.written.as_ref().or(documented_written);

    Some(RenderEntry {
        signature,
        render: CommandRender {
            subject_variable,
            parameters,
            called: join_called_text(&called.called),
            written: written.map(join_written_text),
        },
    })
}

/// Joins all text arguments in a `called:` section into one template string.
pub(super) fn join_called_text(section: &CalledSection) -> String {
    section
        .arguments
        .iter()
        .map(|text| text.0.as_str())
        .collect::<Vec<_>>()
        .join(" ")
}

/// Joins all text arguments in a `written:` section into one template string.
pub(super) fn join_written_text(section: &WrittenSection) -> String {
    section
        .arguments
        .iter()
        .map(|text| text.0.as_str())
        .collect::<Vec<_>>()
        .join(" ")
}

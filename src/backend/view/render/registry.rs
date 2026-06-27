use super::*;

#[derive(Clone, Debug, Default)]
pub(in crate::backend::view) struct RenderRegistry {
    /// Map from canonical command signature to the rendering data for that command.
    pub(super) commands: HashMap<String, CommandRender>,
    /// Whether resolved command renderings should carry clickable reference keys.
    pub(super) link_references: bool,
}

#[derive(Clone, Debug)]
pub(super) struct CommandRender {
    pub(super) subject_variable: Option<String>,
    pub(super) parameters: Vec<String>,
    pub(super) called: String,
    pub(super) called_source: CalledRenderSource,
    pub(super) written: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum CalledRenderSource {
    Called,
    Written,
}

impl CommandRender {
    pub(super) fn render_called(&self, substitutions: &HashMap<String, String>) -> String {
        match self.called_source {
            CalledRenderSource::Called => render_called_template(&self.called, substitutions),
            CalledRenderSource::Written => substitute_math_template(&self.called, substitutions),
        }
    }
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

pub(in crate::backend::view) fn build_linked_render_registry(
    files: &[ParsedSourceFile],
) -> RenderRegistry {
    let mut registry = build_render_registry(files);
    registry.link_references = true;
    registry
}

pub(in crate::backend::view) fn definition_reference_keys_for_heading(
    heading: Option<&str>,
) -> Vec<String> {
    let Some(heading) = heading else {
        return Vec::new();
    };
    let Ok(header) = parse_command_header(heading) else {
        return Vec::new();
    };

    command_header_signatures(&header)
        .into_iter()
        .map(|signature| reference_key(&signature.signature))
        .collect()
}

pub(in crate::backend::view) fn render_formulation_latex(
    text: &str,
    registry: &RenderRegistry,
) -> Option<String> {
    render_parsed_formulation_latex(text, registry)
        .or_else(|| render_simple_set_spec_latex(text, registry))
}

pub(super) fn command_reference_latex(
    signature: &str,
    latex: String,
    registry: &RenderRegistry,
) -> String {
    if registry.link_references {
        format!(
            "\\htmlData{{mlg-ref={}}}{{{latex}}}",
            reference_key(signature)
        )
    } else {
        latex
    }
}

fn reference_key(signature: &str) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut key = String::with_capacity(signature.len() * 2);

    for byte in signature.bytes() {
        key.push(HEX[(byte >> 4) as usize] as char);
        key.push(HEX[(byte & 0x0f) as usize] as char);
    }

    key
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

    Some(render.render_called(&substitutions))
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
    let refinement_latex = refinement_render.render_called(&command_header_substitutions(header));

    Some(join_called_latex_parts(vec![
        refinement_latex,
        target_called.latex,
    ]))
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
        TopLevelItem::States(group) => render_entries_from_parts(
            command_header_signatures(&group.heading),
            None,
            group.documented.as_ref(),
        ),
        TopLevelItem::Axiom(group) => {
            render_theorem_like_entries(group.heading.as_ref(), group.documented.as_ref())
        }
        TopLevelItem::Theorem(group) => {
            render_theorem_like_entries(group.heading.as_ref(), group.documented.as_ref())
        }
        TopLevelItem::Corollary(group) => {
            render_theorem_like_entries(group.heading.as_ref(), group.documented.as_ref())
        }
        TopLevelItem::Lemma(group) => {
            render_theorem_like_entries(group.heading.as_ref(), group.documented.as_ref())
        }
        TopLevelItem::Conjecture(group) => {
            render_theorem_like_entries(group.heading.as_ref(), group.documented.as_ref())
        }
        _ => Vec::new(),
    }
}

pub(super) fn render_entries_from_parts(
    signatures: Vec<CommandHeaderSignature>,
    subject_variable: Option<String>,
    documented: Option<&DocumentedSection>,
) -> Vec<RenderEntry> {
    render_entries_from_parts_with_fallback(signatures, subject_variable, documented, None)
}

fn render_theorem_like_entries(
    heading: Option<&CommandHeader>,
    documented: Option<&DocumentedSection>,
) -> Vec<RenderEntry> {
    let Some(heading) = heading else {
        return Vec::new();
    };

    render_entries_from_parts_with_fallback(
        command_header_signatures(heading),
        None,
        documented,
        Some(fallback_theorem_like_called_text(heading)),
    )
}

fn render_entries_from_parts_with_fallback(
    signatures: Vec<CommandHeaderSignature>,
    subject_variable: Option<String>,
    documented: Option<&DocumentedSection>,
    fallback_called: Option<String>,
) -> Vec<RenderEntry> {
    let Some(documented) = documented else {
        return fallback_called
            .map(|called| {
                render_entries_from_signatures(
                    signatures,
                    subject_variable,
                    Some((called, CalledRenderSource::Called)),
                    None,
                )
            })
            .unwrap_or_default();
    };
    let called = documented.arguments.iter().find_map(|item| match item {
        DocumentedItem::Called(group) => Some(group),
        _ => None,
    });
    let documented_written = documented.arguments.iter().find_map(|item| match item {
        DocumentedItem::Written(group) => Some(&group.written),
        _ => None,
    });
    let written = called
        .and_then(|called| called.written.as_ref())
        .or(documented_written);

    if called.is_none() && written.is_none() && fallback_called.is_none() {
        return Vec::new();
    }

    let written_text = written.map(join_written_text);
    let called_text = called
        .map(|called| (join_called_text(&called.called), CalledRenderSource::Called))
        .or_else(|| {
            written_text
                .clone()
                .map(|written| (written, CalledRenderSource::Written))
        })
        .or_else(|| fallback_called.map(|called| (called, CalledRenderSource::Called)));
    render_entries_from_signatures(signatures, subject_variable, called_text, written_text)
}

fn render_entries_from_signatures(
    signatures: Vec<CommandHeaderSignature>,
    subject_variable: Option<String>,
    called: Option<(String, CalledRenderSource)>,
    written: Option<String>,
) -> Vec<RenderEntry> {
    signatures
        .into_iter()
        .map(|signature| {
            let signature_text = signature.signature;
            let (called_text, called_source) = called
                .clone()
                .unwrap_or_else(|| (signature_text.clone(), CalledRenderSource::Called));

            RenderEntry {
                signature: signature_text,
                render: CommandRender {
                    subject_variable: subject_variable.clone(),
                    parameters: signature.parameters,
                    called: called_text,
                    called_source,
                    written: written.clone(),
                },
            }
        })
        .collect()
}

fn fallback_theorem_like_called_text(heading: &CommandHeader) -> String {
    match heading {
        CommandHeader::Command(header) => title_case_label(&format_chain(&header.chain)),
        CommandHeader::Infix(header) => title_case_label(&format_chain(&header.chain)),
        CommandHeader::InfixSpec(header) => title_case_label(&format_chain(&header.chain)),
        CommandHeader::Refined(header) => title_case_label(&command_header_signature(
            &CommandHeader::Refined(header.clone()),
        )),
    }
}

fn title_case_label(label: &str) -> String {
    label
        .split(|ch: char| !ch.is_ascii_alphanumeric())
        .filter(|word| !word.is_empty())
        .enumerate()
        .map(|(index, word)| title_case_word(word, index == 0))
        .collect::<Vec<_>>()
        .join(" ")
}

fn title_case_word(word: &str, is_first: bool) -> String {
    let lower = word.to_ascii_lowercase();
    if !is_first && is_title_case_stop_word(&lower) {
        return lower;
    }

    let mut chars = lower.chars();
    let Some(first) = chars.next() else {
        return lower;
    };
    format!(
        "{}{}",
        first.to_ascii_uppercase(),
        chars.collect::<String>()
    )
}

fn is_title_case_stop_word(word: &str) -> bool {
    matches!(
        word,
        "a" | "an"
            | "and"
            | "as"
            | "at"
            | "but"
            | "by"
            | "for"
            | "from"
            | "in"
            | "into"
            | "nor"
            | "of"
            | "on"
            | "or"
            | "over"
            | "the"
            | "to"
            | "via"
            | "with"
    )
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

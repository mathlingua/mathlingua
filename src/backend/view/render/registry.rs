use super::*;

#[derive(Clone, Debug, Default)]
pub(in crate::backend::view) struct RenderRegistry {
    /// Map from canonical command signature to the rendering data for that command.
    pub(super) commands: HashMap<String, CommandRender>,
    /// Render templates for capabilities where the described item is callable,
    /// such as `R(a_, b_) :-> ...` with `written: "a_? \: R \: b_?"`.
    pub(super) provided_calls: Vec<ProvidedCallRender>,
    /// Collection-wide aliases for rendering plain names as LaTeX fragments.
    pub(super) writing: HashMap<String, String>,
    /// Explicit `Documented:called:` rendering overrides for top-level `Topic:`
    /// items, keyed by the raw `#some.name` heading string. Topics without an
    /// override are title-cased from their heading at render time.
    pub(super) topic_titles: HashMap<String, String>,
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

#[derive(Clone, Debug)]
pub(super) struct ProvidedCallRender {
    pub(super) owner_subject: String,
    pub(super) function_name: String,
    pub(super) parameters: Vec<String>,
    pub(super) written: String,
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
            collect_writing_aliases(item, &mut registry);
        }
    }

    for file in files {
        for item in &file.document.items {
            for entry in render_entries(item) {
                registry.commands.insert(entry.signature, entry.render);
            }
            collect_provided_call_render_rules(item, &mut registry);
            collect_topic_title(item, &mut registry);
        }
    }

    registry
}

/// Records the explicit `Documented:called:` rendering override for a `Topic:`
/// item, keyed by its `#some.name` heading. Topics without a `called:` are
/// title-cased at render time, so only overrides are stored here.
fn collect_topic_title(item: &TopLevelItem, registry: &mut RenderRegistry) {
    let TopLevelItem::Topic(group) = item else {
        return;
    };
    if let Some(called) = topic_explicit_called_text(group) {
        registry
            .topic_titles
            .insert(topic_heading_key(&group.heading.parts), called);
    }
}

/// Extracts the raw `called:` text from a topic's `Documented:` section, if any.
fn topic_explicit_called_text(group: &TopicGroup) -> Option<String> {
    group
        .documented
        .as_ref()?
        .arguments
        .iter()
        .find_map(|item| match item {
            DocumentedItem::Called(called) => Some(join_called_text(&called.called)),
            _ => None,
        })
}

/// The registry key for a topic heading: the sigil plus the dotted path, matching
/// the raw bracket-inner string the view builder sees (for example `#real.analysis`).
fn topic_heading_key(parts: &[String]) -> String {
    format!("#{}", parts.join("."))
}

/// Resolves the rendered title for a top-level `Topic:` heading string.
///
/// Uses the topic's explicit `Documented:called:` override when present, otherwise
/// title-cases the dotted heading path (so `#real.analysis` renders as "Real
/// Analysis").
pub(in crate::backend::view) fn resolve_topic_heading_latex(
    heading: Option<&str>,
    registry: &RenderRegistry,
) -> Option<String> {
    let header = parse_topic_header(heading?).ok()?;
    let text = registry
        .topic_titles
        .get(&topic_heading_key(&header.parts))
        .cloned()
        .unwrap_or_else(|| title_case_label(&header.parts.join(".")));
    render_documented_text_latex("called", &text)
}

fn collect_writing_aliases(item: &TopLevelItem, registry: &mut RenderRegistry) {
    let TopLevelItem::Writing(group) = item else {
        return;
    };

    for alias in &group.writing.arguments {
        if let FormOrDeclarationKind::Name(name) = &alias.form.kind {
            registry.writing.insert(name.clone(), alias.body.clone());
        }
    }
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

pub(in crate::backend::view) fn render_writing_alias_latex(
    text: &str,
    _registry: &RenderRegistry,
) -> Option<String> {
    let alias = parse_writing_alias(text).ok()?;
    let FormOrDeclarationKind::Name(name) = alias.form.kind else {
        return None;
    };

    Some(format!(
        "\\textrm{{{}}} \\mathrel{{:\\!\\rightsquigarrow}} {}",
        escape_latex_text(&name),
        alias.body
    ))
}

pub(in crate::backend::view) fn render_documented_text_latex(
    label: &str,
    text: &str,
) -> Option<String> {
    match label {
        "called" => Some(render_called_display_template(text)),
        "written" => Some(render_written_display_template(text)),
        _ => None,
    }
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
    if !supports_resolved_group_heading(kind) {
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

    let substitutions = command_header_substitutions(&header, registry);

    Some(render.render_called(&substitutions))
}

fn supports_resolved_group_heading(kind: &str) -> bool {
    matches!(
        kind,
        "Defines"
            | "Describes"
            | "Refines"
            | "States"
            | "Axiom"
            | "Theorem"
            | "Corollary"
            | "Lemma"
            | "Conjecture"
            | "Equivalent"
    )
}

pub(super) fn render_refines_group_heading_latex(
    header: &CommandHeader,
    _refines_argument: Option<&str>,
    refinement_render: &CommandRender,
    registry: &RenderRegistry,
) -> Option<String> {
    let CommandHeader::Refined(refined_header) = header else {
        return None;
    };
    let refinement_latex =
        refinement_render.render_called(&command_header_substitutions(header, registry));
    let base_signature = refined_command_header_base_signature(refined_header);
    let target_render = registry.commands.get(&base_signature)?;
    let target_latex = command_reference_latex(
        &base_signature,
        target_render.render_called(&command_header_substitutions(header, registry)),
        registry,
    );

    Some(join_called_latex_parts(vec![
        refinement_latex,
        target_latex,
    ]))
}

pub(super) fn render_parsed_formulation_latex(
    text: &str,
    registry: &RenderRegistry,
) -> Option<String> {
    if let Ok(expression) = parse_expression(text) {
        return Some(render_expression(&expression, registry));
    }

    if let Ok(statement) = parse_is_via_statement(text) {
        return Some(render_is_via_statement(&statement, registry));
    }

    if let Ok(statement) = parse_refined_declaration_statement(text) {
        return Some(render_declaration_statement(&statement, registry));
    }

    parse_form_or_declaration(text)
        .ok()
        .map(|form| render_form_or_declaration(&form, registry))
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
            primary_describes_target_name(&group.describes.argument),
            group.documented.as_ref(),
        ),
        TopLevelItem::Defines(group) => render_entries_from_parts(
            command_header_signatures(&group.heading),
            primary_declaration_statement_name(&group.defines.argument),
            group.documented.as_ref(),
        ),
        TopLevelItem::Refines(group) => render_refines_entries(
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
        TopLevelItem::Equivalent(group) => render_entries_from_parts(
            command_header_signatures(&group.heading),
            None,
            group.documented.as_ref(),
        ),
        _ => Vec::new(),
    }
}

fn collect_provided_call_render_rules(item: &TopLevelItem, registry: &mut RenderRegistry) {
    let Some(owner_subject) = top_level_item_subject(item) else {
        return;
    };

    match item {
        TopLevelItem::Describes(group) => {
            if let Some(requires) = &group.requires {
                collect_requires_provided_call_render_rules(requires, &owner_subject, registry);
            }
            if let Some(enables) = &group.enables {
                collect_enables_provided_call_render_rules(enables, &owner_subject, registry);
            }
        }
        TopLevelItem::Defines(group) => {
            if let Some(requires) = &group.requires {
                collect_requires_provided_call_render_rules(requires, &owner_subject, registry);
            }
            if let Some(enables) = &group.enables {
                collect_enables_provided_call_render_rules(enables, &owner_subject, registry);
            }
        }
        TopLevelItem::Refines(group) => {
            if let Some(requires) = &group.requires {
                collect_requires_provided_call_render_rules(requires, &owner_subject, registry);
            }
            if let Some(enables) = &group.enables {
                collect_enables_provided_call_render_rules(enables, &owner_subject, registry);
            }
        }
        TopLevelItem::States(group) => {
            if let Some(requires) = &group.requires {
                collect_requires_provided_call_render_rules(requires, &owner_subject, registry);
            }
            if let Some(enables) = &group.enables {
                collect_enables_provided_call_render_rules(enables, &owner_subject, registry);
            }
        }
        _ => {}
    }
}

fn top_level_item_subject(item: &TopLevelItem) -> Option<String> {
    match item {
        TopLevelItem::Describes(group) => primary_describes_target_name(&group.describes.argument),
        TopLevelItem::Defines(group) => primary_declaration_statement_name(&group.defines.argument),
        TopLevelItem::Refines(group) => primary_declaration_statement_name(&group.refines.argument),
        _ => None,
    }
}

fn collect_requires_provided_call_render_rules(
    requires: &RequiresSection,
    owner_subject: &str,
    registry: &mut RenderRegistry,
) {
    for item in &requires.arguments {
        let RequiresItem::Capability(group) = item else {
            continue;
        };
        collect_capability_provided_call_render_rule(
            &group.capability.argument,
            group.written.as_ref(),
            owner_subject,
            registry,
        );
    }
}

fn collect_enables_provided_call_render_rules(
    enables: &EnablesSection,
    owner_subject: &str,
    registry: &mut RenderRegistry,
) {
    for item in &enables.arguments {
        match item {
            EnablesItem::Capability(group) => collect_capability_provided_call_render_rule(
                &group.capability.argument,
                group.written.as_ref(),
                owner_subject,
                registry,
            ),
            EnablesItem::FromCapability(group) => collect_capability_provided_call_render_rule(
                &group.capability.argument,
                group.written.as_ref(),
                owner_subject,
                registry,
            ),
            EnablesItem::FromAs(_) | EnablesItem::Relation(_) => {}
        }
    }
}

fn collect_capability_provided_call_render_rule(
    alias: &AliasKind,
    written: Option<&WrittenSection>,
    owner_subject: &str,
    registry: &mut RenderRegistry,
) {
    let Some(written) = written else {
        return;
    };
    let AliasKind::Expression(alias) = alias else {
        return;
    };
    let ExpressionAliasLhs::Form(FormOrDeclaration {
        kind: FormOrDeclarationKind::FunctionDeclaration { name, form },
        ..
    }) = &alias.lhs
    else {
        return;
    };
    let function_name = name.as_ref().unwrap_or(&form.name);
    if function_name != owner_subject {
        return;
    }

    registry.provided_calls.push(ProvidedCallRender {
        owner_subject: owner_subject.to_owned(),
        function_name: function_name.clone(),
        parameters: function_form_render_parameters(form),
        written: join_written_text(written),
    });
}

fn function_form_render_parameters(form: &FunctionForm) -> Vec<String> {
    form.magnetic_placeholder
        .iter()
        .map(|placeholder| placeholder.name.clone())
        .chain(
            form.placeholders
                .iter()
                .map(|placeholder| placeholder.name.clone()),
        )
        .collect()
}

fn primary_describes_target_name(target: &DescribesTarget) -> Option<String> {
    match target {
        DescribesTarget::Form(form) => primary_form_name(form),
        DescribesTarget::Declaration(statement) => primary_declaration_statement_name(statement),
    }
}

pub(super) fn render_entries_from_parts(
    signatures: Vec<CommandHeaderSignature>,
    subject_variable: Option<String>,
    documented: Option<&DocumentedSection>,
) -> Vec<RenderEntry> {
    render_entries_from_parts_with_fallback(signatures, subject_variable, documented, None)
}

pub(super) fn render_refines_entries(
    signatures: Vec<CommandHeaderSignature>,
    subject_variable: Option<String>,
    documented: Option<&DocumentedSection>,
) -> Vec<RenderEntry> {
    let Some(documented) = documented else {
        return Vec::new();
    };

    let adjective = documented.arguments.iter().find_map(|item| match item {
        DocumentedItem::Adjective(group) => Some(&group.adjective),
        _ => None,
    });
    let written = documented.arguments.iter().find_map(|item| match item {
        DocumentedItem::Written(group) => Some(&group.written),
        _ => None,
    });

    let Some(adjective) = adjective else {
        return Vec::new();
    };

    render_entries_from_signatures(
        signatures,
        subject_variable,
        Some((join_adjective_text(adjective), CalledRenderSource::Called)),
        written.map(join_written_text),
    )
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

pub(super) fn join_adjective_text(section: &AdjectiveSection) -> String {
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

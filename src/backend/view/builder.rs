use super::model::{
    ArgumentView, CollectionView, DirectoryView, FileView, GroupView, PageView, SectionView,
};
use super::render::{
    RenderRegistry, build_linked_render_registry, definition_reference_keys_for_heading,
    render_documented_text_latex, render_formulation_latex, render_group_heading_latex,
    render_writing_alias_latex, resolve_topic_heading_latex,
};
use crate::events::{Audience, Event, EventLog, Level};
use crate::frontend::{
    ParsedSourceFile, ProtoArgument, ProtoGroup, ProtoParser, ProtoSection, SourceFileViewMetadata,
    top_level_group_id, unescape_quoted_text,
};
use std::path::{Path, PathBuf};

/// Builds the complete serialized view model for a MathLingua collection.
///
/// Files must already have passed the shared structural and semantic checking
/// passes.  This pass indexes the checked files for rendering and converts them
/// into JSON-friendly view models.  Returning `None` means a user-facing
/// blocking diagnostic has already been emitted to `event_log`.
pub fn build_collection_view(
    collection_root: &Path,
    parsed_files: &[ParsedSourceFile],
    directory_metadata: &[(PathBuf, SourceFileViewMetadata)],
    event_log: &mut EventLog,
) -> Option<CollectionView> {
    let registry = build_linked_render_registry(parsed_files);
    let rendered_files = parsed_files
        .iter()
        .filter(|file| !file.view_metadata.hidden)
        .map(|file| build_file_view(collection_root, file, &registry, event_log))
        .collect::<Option<Vec<_>>>()?;

    Some(CollectionView {
        title: collection_title(collection_root),
        directories: directory_metadata
            .iter()
            .filter(|(_, metadata)| !metadata.hidden)
            .filter_map(|(path, metadata)| {
                let path = relative_path(collection_root, path);
                if path == "content" {
                    return None;
                }

                Some(DirectoryView {
                    path,
                    title: metadata.title.clone(),
                })
            })
            .collect(),
        files: rendered_files,
    })
}

fn build_file_view(
    collection_root: &Path,
    file: &ParsedSourceFile,
    registry: &RenderRegistry,
    event_log: &mut EventLog,
) -> Option<FileView> {
    let mut proto_log = EventLog::new();
    let groups = ProtoParser::new(&file.source, &mut proto_log).parse();
    for event in proto_log.events() {
        event_log.push(event.clone().with_file_path(file.path.clone()));
    }
    if has_blocking_user_issues(proto_log.events()) {
        return None;
    }

    let group_sources = source_for_groups(&file.source, &groups);

    Some(FileView {
        path: relative_path(collection_root, &file.path),
        title: file.view_metadata.title.clone(),
        items: groups
            .into_iter()
            .zip(group_sources)
            .flat_map(|(group, source)| group_views(group, source, registry))
            .collect(),
    })
}

fn has_blocking_user_issues(events: &[Event]) -> bool {
    events.iter().filter_map(Event::as_message).any(|event| {
        event.audience == Audience::User && matches!(event.level, Level::Error | Level::Debug)
    })
}

fn collection_title(collection_root: &Path) -> String {
    collection_root
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.is_empty())
        .unwrap_or("Mathlingua Collection")
        .to_string()
}

fn relative_path(collection_root: &Path, file: &Path) -> String {
    file.strip_prefix(collection_root)
        .unwrap_or(file)
        .display()
        .to_string()
}

fn source_for_groups(source: &str, groups: &[ProtoGroup]) -> Vec<String> {
    let lines = source.split('\n').collect::<Vec<_>>();

    groups
        .iter()
        .enumerate()
        .map(|(index, group)| {
            let start = group.metadata.row.min(lines.len());
            let mut end = groups
                .get(index + 1)
                .map(|next| next.metadata.row)
                .unwrap_or(lines.len())
                .min(lines.len());

            while end > start && is_trailing_source_gap(lines[end - 1]) {
                end -= 1;
            }

            lines[start..end].join("\n")
        })
        .collect()
}

fn is_trailing_source_gap(line: &str) -> bool {
    let trimmed = line.trim_start();
    trimmed.is_empty() || trimmed.starts_with("--")
}

fn group_views(group: ProtoGroup, source: String, registry: &RenderRegistry) -> Vec<GroupView> {
    let id = top_level_group_id(&group).unwrap_or_default();
    let description = documented_description_text(&group);
    let card = group_view(group, source, registry, id.clone());

    match description {
        Some(description) if !description.trim().is_empty() => {
            vec![description_page_group(&id, description), card]
        }
        _ => vec![card],
    }
}

fn description_page_group(source_id: &str, text: String) -> GroupView {
    GroupView {
        id: if source_id.is_empty() {
            "description".to_string()
        } else {
            format!("{source_id}-description")
        },
        kind: "Text".to_string(),
        definition_keys: Vec::new(),
        heading: None,
        heading_latex: None,
        body_text: None,
        page: Some(PageView {
            kind: "Text".to_string(),
            text,
        }),
        source: String::new(),
        sections: Vec::new(),
    }
}

fn group_view(
    group: ProtoGroup,
    source: String,
    registry: &RenderRegistry,
    id: String,
) -> GroupView {
    let kind = group
        .sections
        .first()
        .map(|section| section.label.clone())
        .unwrap_or_else(|| "Group".to_string());
    let primary_inline_argument = group
        .sections
        .first()
        .and_then(|section| section.inline_argument.as_deref());
    let page = page_view(&kind, &group.sections);
    let heading_latex = person_heading_latex(&kind, &group.sections)
        .or_else(|| topic_heading_latex(&kind, group.heading.as_deref(), registry))
        .or_else(|| {
            render_group_heading_latex(
                &kind,
                group.heading.as_deref(),
                primary_inline_argument,
                registry,
            )
        })
        .or_else(|| theorem_like_heading_latex(&kind, group.heading.as_deref(), &group.sections));
    let body_text = person_body_text(&kind, &group.sections);

    GroupView {
        id,
        heading_latex,
        body_text,
        definition_keys: definition_reference_keys_for_heading(group.heading.as_deref()),
        kind,
        heading: group.heading,
        page,
        source,
        sections: group
            .sections
            .into_iter()
            .map(|section| section_view(section, registry, SectionContext::Default))
            .collect(),
    }
}

fn person_heading_latex(kind: &str, sections: &[ProtoSection]) -> Option<String> {
    if kind != "Person" {
        return None;
    }

    let name = first_section_text(sections.iter().find(|section| section.label == "Person")?)?;
    render_documented_text_latex("called", &name)
}

/// Renders a heading-less theorem-like card's title from its `Documented:` `called:`.
///
/// Command-headed theorem-like items resolve their title through the command-signature
/// registry (like definitions); a heading-less `Axiom:`/`Theorem:`/`Corollary:`/`Lemma:`/
/// `Conjecture:` instead takes its name from `Documented:` `called:`.
fn theorem_like_heading_latex(
    kind: &str,
    heading: Option<&str>,
    sections: &[ProtoSection],
) -> Option<String> {
    const THEOREM_LIKE: [&str; 5] = ["Axiom", "Theorem", "Corollary", "Lemma", "Conjecture"];
    if !THEOREM_LIKE.contains(&kind) || heading.is_some() {
        return None;
    }

    render_documented_text_latex("called", &documented_called_text(sections)?)
}

/// Extracts the `Documented:` `called:` text from a group's proto sections.
fn documented_called_text(sections: &[ProtoSection]) -> Option<String> {
    let documented = sections
        .iter()
        .find(|section| section.label == "Documented")?;
    documented.arguments.iter().find_map(|argument| {
        let ProtoArgument::Group(group) = argument else {
            return None;
        };
        let called = group
            .sections
            .iter()
            .find(|section| section.label == "called")?;
        first_section_text(called)
    })
}

/// Renders a top-level `Topic:` heading as its human title: the explicit
/// `Documented:called:` override when present, otherwise the title-cased `#some.name`
/// heading (so `#real.analysis` renders as "Real Analysis").
fn topic_heading_latex(
    kind: &str,
    heading: Option<&str>,
    registry: &RenderRegistry,
) -> Option<String> {
    if kind != "Topic" {
        return None;
    }

    resolve_topic_heading_latex(heading, registry)
}

fn person_body_text(kind: &str, sections: &[ProtoSection]) -> Option<String> {
    if kind != "Person" {
        return None;
    }

    sections
        .iter()
        .find(|section| section.label == "biography")
        .and_then(section_text)
        .map(|text| unindent_description_text(&text))
        .filter(|text| !text.trim().is_empty())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SectionContext {
    Default,
    Documented,
}

fn section_view(
    section: ProtoSection,
    registry: &RenderRegistry,
    context: SectionContext,
) -> SectionView {
    let label = section.label;
    if context == SectionContext::Documented && label == "description" {
        return SectionView {
            label,
            inline_argument: Some("see above".to_string()),
            inline_latex: Some(r#"\langle\textrm{see above}\rangle"#.to_string()),
            arguments: Vec::new(),
        };
    }

    let inline_latex = section
        .inline_argument
        .as_deref()
        .and_then(|text| render_section_inline_latex(&label, text, registry));
    let render_kind = documented_render_kind(&label);

    SectionView {
        label: label.clone(),
        inline_argument: section.inline_argument,
        inline_latex,
        arguments: section
            .arguments
            .into_iter()
            .map(|argument| argument_view(argument, registry, &label, render_kind))
            .collect(),
    }
}

#[derive(Clone, Copy)]
enum DocumentedRenderKind {
    Called,
    Written,
}

fn documented_render_kind(label: &str) -> Option<DocumentedRenderKind> {
    match label {
        "called" => Some(DocumentedRenderKind::Called),
        "written" => Some(DocumentedRenderKind::Written),
        _ => None,
    }
}

fn render_section_inline_latex(
    label: &str,
    text: &str,
    registry: &RenderRegistry,
) -> Option<String> {
    documented_render_kind(label)
        .and_then(|kind| render_documented_template_argument(kind, text))
        .or_else(|| render_formulation_latex(text, registry))
}

fn render_documented_template_argument(kind: DocumentedRenderKind, text: &str) -> Option<String> {
    let template = strip_quoted_text(text)?;
    let label = match kind {
        DocumentedRenderKind::Called => "called",
        DocumentedRenderKind::Written => "written",
    };

    render_documented_text_latex(label, &template)
}

fn page_view(kind: &str, sections: &[ProtoSection]) -> Option<PageView> {
    if !matches!(kind, "Title" | "SectionTitle" | "SubsectionTitle" | "Text") {
        return None;
    }

    let text = sections.first().and_then(section_text).unwrap_or_default();

    Some(PageView {
        kind: kind.to_string(),
        text,
    })
}

fn documented_description_text(group: &ProtoGroup) -> Option<String> {
    let descriptions = group
        .sections
        .iter()
        .find(|section| section.label == "Documented")?
        .arguments
        .iter()
        .filter_map(|argument| match argument {
            ProtoArgument::Group(group) => group.sections.first(),
            _ => None,
        })
        .filter(|section| section.label == "description")
        .filter_map(section_text)
        .map(|text| unindent_description_text(&text))
        .filter(|text| !text.trim().is_empty())
        .collect::<Vec<_>>();

    if descriptions.is_empty() {
        None
    } else {
        Some(descriptions.join("\n\n"))
    }
}

fn section_text(section: &ProtoSection) -> Option<String> {
    first_section_text(section)
}

fn first_section_text(section: &ProtoSection) -> Option<String> {
    if let Some(text) = section.inline_argument.as_deref() {
        return strip_quoted_text(text);
    }

    section
        .arguments
        .iter()
        .find_map(|argument| match argument {
            ProtoArgument::Text(text) => strip_quoted_text(&text.text),
            _ => None,
        })
}

fn unindent_description_text(input: &str) -> String {
    let mut lines = input
        .replace("\r\n", "\n")
        .replace('\r', "\n")
        .split('\n')
        .map(str::to_string)
        .collect::<Vec<_>>();

    while lines.first().is_some_and(|line| line.trim().is_empty()) {
        lines.remove(0);
    }
    while lines.last().is_some_and(|line| line.trim().is_empty()) {
        lines.pop();
    }

    let skip_unindented_first_line =
        lines.len() > 1 && lines.first().is_some_and(|line| leading_indent(line) == 0);
    let common_indent = lines
        .iter()
        .enumerate()
        .filter(|(index, line)| {
            !line.trim().is_empty() && !(skip_unindented_first_line && *index == 0)
        })
        .map(|(_, line)| leading_indent(line))
        .min()
        .unwrap_or(0);

    lines
        .into_iter()
        .map(|line| strip_leading_indent(&line, common_indent).to_string())
        .collect::<Vec<_>>()
        .join("\n")
}

fn leading_indent(line: &str) -> usize {
    line.chars()
        .take_while(|character| matches!(character, ' ' | '\t'))
        .count()
}

fn strip_leading_indent(line: &str, count: usize) -> &str {
    if count == 0 {
        return line;
    }

    let mut stripped = 0;
    for (index, character) in line.char_indices() {
        if stripped == count || !matches!(character, ' ' | '\t') {
            return &line[index..];
        }
        stripped += 1;
    }

    ""
}

fn strip_quoted_text(input: &str) -> Option<String> {
    let input = input.trim();
    if input.len() < 2 || !input.starts_with('"') || !input.ends_with('"') {
        return None;
    }

    Some(unescape_quoted_text(&input[1..input.len() - 1]))
}

fn argument_view(
    argument: ProtoArgument,
    registry: &RenderRegistry,
    section_label: &str,
    documented_render_kind: Option<DocumentedRenderKind>,
) -> ArgumentView {
    match argument {
        ProtoArgument::Formulation(formulation) => ArgumentView::Formulation {
            latex: render_argument_formulation_latex(section_label, &formulation.text, registry),
            text: formulation.text,
        },
        ProtoArgument::Text(text) => {
            let latex = documented_render_kind
                .and_then(|kind| render_documented_template_argument(kind, &text.text));
            ArgumentView::Text {
                text: strip_quoted_text(&text.text).unwrap_or(text.text),
                latex,
            }
        }
        ProtoArgument::Group(group) => ArgumentView::Group {
            heading: group.heading,
            sections: group
                .sections
                .into_iter()
                .map(|section| {
                    let context = if section_label == "Documented" {
                        SectionContext::Documented
                    } else {
                        SectionContext::Default
                    };
                    section_view(section, registry, context)
                })
                .collect(),
        },
    }
}

fn render_argument_formulation_latex(
    section_label: &str,
    text: &str,
    registry: &RenderRegistry,
) -> Option<String> {
    if section_label == "Writing" {
        render_writing_alias_latex(text, registry)
    } else {
        render_formulation_latex(text, registry)
    }
}

// ===============================[ tests ]=====================================

#[cfg(test)]
mod tests {
    use super::build_collection_view;
    use crate::backend::view::ArgumentView;
    use crate::events::EventLog;
    use crate::frontend::{
        ParsedSourceFile, SourceFileViewMetadata, parse_document, top_level_item_ids,
    };
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    // Distinguishes concurrently-created test directories whose nanosecond
    // timestamps could otherwise collide under parallel test execution.
    static NEXT_TEST_DIR_ID: AtomicUsize = AtomicUsize::new(0);

    #[test]
    fn builds_a_collection_view_from_valid_files() {
        let temp_dir = TestDir::new();
        let root = temp_dir.path().join("repo");
        let content = root.join("content");
        let file = content.join("sets.mlg");
        let source = "[\\set]\nDescribes: S\nDocumented:\n. called:\n  . \"set\"\n";

        fs::create_dir_all(&content).unwrap();
        fs::write(&file, source).unwrap();

        let mut parse_log = EventLog::new();
        let document = parse_document(source, &mut parse_log);
        assert!(!parse_log.has_errors(), "{:#?}", parse_log.events());
        let parsed_file = ParsedSourceFile {
            path: file,
            source: source.to_string(),
            document,
            item_ids: top_level_item_ids(source),
            view_metadata: SourceFileViewMetadata::default(),
        };
        let mut event_log = EventLog::new();
        let view = build_collection_view(&root, &[parsed_file], &[], &mut event_log)
            .expect("expected view");

        assert_eq!(view.title, "repo");
        assert_eq!(view.files.len(), 1);
        assert_eq!(view.files[0].path, "content/sets.mlg");
        assert_eq!(view.files[0].items[0].kind, "Describes");
        assert_eq!(view.files[0].items[0].definition_keys, vec!["5c736574"]);
        assert_eq!(view.files[0].items[0].source, source.trim_end());
        assert_eq!(
            view.files[0].items[0].heading_latex,
            Some(r"\textrm{set}".to_string())
        );
        assert!(!event_log.has_errors());
    }

    #[test]
    fn renders_documented_called_and_written_text_in_view_details() {
        let temp_dir = TestDir::new();
        let root = temp_dir.path().join("repo");
        let content = root.join("content");
        let file = content.join("sets.mlg");
        let source = r#"[\membership]
Describes: X
Documented:
. called: "membership of $x_?$ in $X?$"
. written: "x_? \in X?"
"#;

        fs::create_dir_all(&content).unwrap();
        fs::write(&file, source).unwrap();

        let mut parse_log = EventLog::new();
        let document = parse_document(source, &mut parse_log);
        assert!(!parse_log.has_errors(), "{:#?}", parse_log.events());
        let parsed_file = ParsedSourceFile {
            path: file,
            source: source.to_string(),
            document,
            item_ids: top_level_item_ids(source),
            view_metadata: SourceFileViewMetadata::default(),
        };
        let mut event_log = EventLog::new();
        let view = build_collection_view(&root, &[parsed_file], &[], &mut event_log)
            .expect("expected view");
        let documented = view.files[0].items[0]
            .sections
            .iter()
            .find(|section| section.label == "Documented")
            .expect("expected documented section");
        let ArgumentView::Group { sections, .. } = &documented.arguments[0] else {
            panic!("expected called group");
        };
        assert_eq!(
            sections[0].inline_latex,
            Some(r#"\textrm{membership of }x\textrm{ in }X"#.to_string())
        );
        let ArgumentView::Group { sections, .. } = &documented.arguments[1] else {
            panic!("expected written group");
        };
        assert_eq!(sections[0].inline_latex, Some(r#"x \in X"#.to_string()));
        assert!(!event_log.has_errors());
    }

    #[test]
    fn renders_documented_description_as_dedented_prose_before_card() {
        let temp_dir = TestDir::new();
        let root = temp_dir.path().join("repo");
        let content = root.join("content");
        let file = content.join("sets.mlg");
        let source = r#"[\set]
Describes: X
Documented:
. called: "set"
. description: "Some text
                          Some indented text
                          \[
                              \some.math
                          \]
                          "
Id: "11111111-1111-4111-8111-111111111111"
"#;

        fs::create_dir_all(&content).unwrap();
        fs::write(&file, source).unwrap();

        let mut parse_log = EventLog::new();
        let document = parse_document(source, &mut parse_log);
        assert!(!parse_log.has_errors(), "{:#?}", parse_log.events());
        let parsed_file = ParsedSourceFile {
            path: file,
            source: source.to_string(),
            document,
            item_ids: top_level_item_ids(source),
            view_metadata: SourceFileViewMetadata::default(),
        };
        let mut event_log = EventLog::new();
        let view = build_collection_view(&root, &[parsed_file], &[], &mut event_log)
            .expect("expected view");

        assert_eq!(view.files[0].items.len(), 2);
        assert_eq!(
            view.files[0].items[0].id,
            "11111111-1111-4111-8111-111111111111-description"
        );
        assert_eq!(view.files[0].items[0].kind, "Text");
        assert_eq!(
            view.files[0].items[0]
                .page
                .as_ref()
                .map(|page| page.text.as_str()),
            Some("Some text\nSome indented text\n\\[\n    \\some.math\n\\]")
        );

        let documented = view.files[0].items[1]
            .sections
            .iter()
            .find(|section| section.label == "Documented")
            .expect("expected documented section");
        let ArgumentView::Group { sections, .. } = &documented.arguments[1] else {
            panic!("expected description group");
        };
        assert_eq!(sections[0].label, "description");
        assert_eq!(sections[0].inline_argument.as_deref(), Some("see above"));
        assert_eq!(
            sections[0].inline_latex.as_deref(),
            Some(r#"\langle\textrm{see above}\rangle"#)
        );
        assert!(sections[0].arguments.is_empty());
        assert!(!event_log.has_errors());
    }

    #[test]
    fn renders_person_name_as_title_and_biography_as_card_text() {
        let temp_dir = TestDir::new();
        let root = temp_dir.path().join("repo");
        let content = root.join("content");
        let file = content.join("people.mlg");
        let source = r#"[@ada.lovelace]
Person: "Ada Lovelace"
biography: "Known for notes on the Analytical Engine.

Worked with $Babbage$."
Id: "11111111-1111-4111-8111-111111111111"
"#;

        fs::create_dir_all(&content).unwrap();
        fs::write(&file, source).unwrap();

        let mut parse_log = EventLog::new();
        let document = parse_document(source, &mut parse_log);
        assert!(!parse_log.has_errors(), "{:#?}", parse_log.events());
        let parsed_file = ParsedSourceFile {
            path: file,
            source: source.to_string(),
            document,
            item_ids: top_level_item_ids(source),
            view_metadata: SourceFileViewMetadata::default(),
        };
        let mut event_log = EventLog::new();
        let view = build_collection_view(&root, &[parsed_file], &[], &mut event_log)
            .expect("expected view");
        let person = &view.files[0].items[0];

        assert_eq!(person.kind, "Person");
        assert_eq!(
            person.heading_latex.as_deref(),
            Some(r#"\textrm{Ada Lovelace}"#)
        );
        assert_eq!(
            person.body_text.as_deref(),
            Some("Known for notes on the Analytical Engine.\n\nWorked with $Babbage$.")
        );
        assert!(!event_log.has_errors());
    }

    #[test]
    fn renders_topic_heading_from_dotted_path_and_called_override() {
        let temp_dir = TestDir::new();
        let root = temp_dir.path().join("repo");
        let content = root.join("content");
        let file = content.join("topics.mlg");
        let source = r#"[#complex.analysis]
Topic: "Analysis over the complex numbers."
Id: "11111111-1111-4111-8111-111111111111"

[#real.analysis]
Topic: "Analysis over the real numbers."
Documented:
. called: "Real Variable Theory"
Id: "22222222-2222-4222-8222-222222222222"
"#;

        fs::create_dir_all(&content).unwrap();
        fs::write(&file, source).unwrap();

        let mut parse_log = EventLog::new();
        let document = parse_document(source, &mut parse_log);
        assert!(!parse_log.has_errors(), "{:#?}", parse_log.events());
        let parsed_file = ParsedSourceFile {
            path: file,
            source: source.to_string(),
            document,
            item_ids: top_level_item_ids(source),
            view_metadata: SourceFileViewMetadata::default(),
        };
        let mut event_log = EventLog::new();
        let view = build_collection_view(&root, &[parsed_file], &[], &mut event_log)
            .expect("expected view");

        // Without a `Documented:called:`, the dotted heading is title-cased.
        let title_cased = &view.files[0].items[0];
        assert_eq!(title_cased.kind, "Topic");
        assert_eq!(
            title_cased.heading_latex.as_deref(),
            Some(r#"\textrm{Complex Analysis}"#)
        );

        // With a `Documented:called:`, that text overrides the title.
        let with_called = &view.files[0].items[1];
        assert_eq!(
            with_called.heading_latex.as_deref(),
            Some(r#"\textrm{Real Variable Theory}"#)
        );

        assert!(!event_log.has_errors());
    }

    #[test]
    fn links_resolved_command_references_to_definition_keys() {
        let temp_dir = TestDir::new();
        let root = temp_dir.path().join("repo");
        let content = root.join("content");
        let file = content.join("sets.mlg");
        let source = r#"[\set]
Describes: X
Documented:
. called: "set"


[\nonempty.set]
Describes: X
extends: X is \set
Documented:
. called: "non-empty set"
"#;

        fs::create_dir_all(&content).unwrap();
        fs::write(&file, source).unwrap();

        let mut parse_log = EventLog::new();
        let document = parse_document(source, &mut parse_log);
        let parsed_file = ParsedSourceFile {
            path: file,
            source: source.to_string(),
            document,
            item_ids: top_level_item_ids(source),
            view_metadata: SourceFileViewMetadata::default(),
        };
        let mut event_log = EventLog::new();
        let view = build_collection_view(&root, &[parsed_file], &[], &mut event_log)
            .expect("expected view");
        let extends = &view.files[0].items[1].sections[1];

        assert_eq!(view.files[0].items[0].definition_keys, vec!["5c736574"]);
        assert_eq!(
            extends.inline_latex,
            Some(r#"X \textrm{ is } \htmlData{mlg-ref=5c736574}{\textrm{set}}"#.to_string())
        );
        assert!(!event_log.has_errors());
    }

    #[test]
    fn renders_headings_for_states_and_theorem_like_cards() {
        let temp_dir = TestDir::new();
        let root = temp_dir.path().join("repo");
        let content = root.join("content");
        let file = content.join("logic.mlg");
        let source = r#"[A \.and./ B]
States:
that:
. allOf:
  . A
  . B
Documented:
. written: "A? \text{ and } B?"


[\axiom.of.choice]
Axiom:
then: X = X
Documented:
. called: "axiom of choice"


[\axiom.of.unordered.pair]
Axiom:
then: X = X
"#;

        fs::create_dir_all(&content).unwrap();
        fs::write(&file, source).unwrap();

        let mut parse_log = EventLog::new();
        let document = parse_document(source, &mut parse_log);
        let parsed_file = ParsedSourceFile {
            path: file,
            source: source.to_string(),
            document,
            item_ids: top_level_item_ids(source),
            view_metadata: SourceFileViewMetadata::default(),
        };
        let mut event_log = EventLog::new();
        let view = build_collection_view(&root, &[parsed_file], &[], &mut event_log)
            .expect("expected view");

        assert!(!event_log.has_errors());
        assert_eq!(
            view.files[0].items[0].heading_latex,
            Some(r#"A \text{ and } B"#.to_string())
        );
        assert_eq!(
            view.files[0].items[1].heading_latex,
            Some(r#"\textrm{axiom of choice}"#.to_string())
        );
        assert_eq!(
            view.files[0].items[2].heading_latex,
            Some(r#"\textrm{Axiom of Unordered Pair}"#.to_string())
        );
    }

    #[test]
    fn renders_heading_less_theorem_like_called_as_title() {
        let temp_dir = TestDir::new();
        let root = temp_dir.path().join("repo");
        let content = root.join("content");
        let file = content.join("thm.mlg");
        let source = r#"Theorem:
given: a is \\expression
then: a = a
Documented:
. called: "Commutativity"
Id: "11111111-1111-4111-8111-111111111111"


Theorem:
then: X = X
Id: "22222222-2222-4222-8222-222222222222"
"#;

        fs::create_dir_all(&content).unwrap();
        fs::write(&file, source).unwrap();

        let mut parse_log = EventLog::new();
        let document = parse_document(source, &mut parse_log);
        assert!(!parse_log.has_errors(), "{:#?}", parse_log.events());
        let parsed_file = ParsedSourceFile {
            path: file,
            source: source.to_string(),
            document,
            item_ids: top_level_item_ids(source),
            view_metadata: SourceFileViewMetadata::default(),
        };
        let mut event_log = EventLog::new();
        let view = build_collection_view(&root, &[parsed_file], &[], &mut event_log)
            .expect("expected view");

        // A heading-less theorem-like takes its title from Documented:called:.
        assert_eq!(
            view.files[0].items[0].heading_latex,
            Some(r#"\textrm{Commutativity}"#.to_string())
        );
        // With no called:, a heading-less theorem-like has no title.
        assert_eq!(view.files[0].items[1].heading_latex, None);
    }

    #[test]
    fn builds_page_items_for_outline_and_text_groups() {
        let temp_dir = TestDir::new();
        let root = temp_dir.path().join("repo");
        let content = root.join("content");
        let file = content.join("intro.mlg");
        let source = r#"Title: "Foundations"

SectionTitle: "Sets"

Text: "First paragraph

Second paragraph with $x \in X$."
"#;

        fs::create_dir_all(&content).unwrap();
        fs::write(&file, source).unwrap();

        let mut parse_log = EventLog::new();
        let document = parse_document(source, &mut parse_log);
        let parsed_file = ParsedSourceFile {
            path: file,
            source: source.to_string(),
            document,
            item_ids: top_level_item_ids(source),
            view_metadata: SourceFileViewMetadata::default(),
        };
        let mut event_log = EventLog::new();
        let view = build_collection_view(&root, &[parsed_file], &[], &mut event_log)
            .expect("expected view");

        assert!(!event_log.has_errors());
        assert_eq!(
            view.files[0].items[0]
                .page
                .as_ref()
                .map(|page| page.text.as_str()),
            Some("Foundations")
        );
        assert_eq!(
            view.files[0].items[1]
                .page
                .as_ref()
                .map(|page| page.kind.as_str()),
            Some("SectionTitle")
        );
        assert_eq!(
            view.files[0].items[1]
                .page
                .as_ref()
                .map(|page| page.text.as_str()),
            Some("Sets")
        );
        assert_eq!(
            view.files[0].items[2]
                .page
                .as_ref()
                .map(|page| page.text.as_str()),
            Some("First paragraph\n\nSecond paragraph with $x \\in X$.")
        );
    }

    #[test]
    fn renders_writing_groups_as_latex_mapping_items() {
        let temp_dir = TestDir::new();
        let root = temp_dir.path().join("repo");
        let content = root.join("content");
        let file = content.join("writing.mlg");
        let source = r#"Writing:
. alpha :~> \alpha
. Gamma :~> \Gamma
Id: "11111111-1111-4111-8111-111111111111"
"#;

        fs::create_dir_all(&content).unwrap();
        fs::write(&file, source).unwrap();

        let mut parse_log = EventLog::new();
        let document = parse_document(source, &mut parse_log);
        let parsed_file = ParsedSourceFile {
            path: file,
            source: source.to_string(),
            document,
            item_ids: top_level_item_ids(source),
            view_metadata: SourceFileViewMetadata::default(),
        };
        let mut event_log = EventLog::new();
        let view = build_collection_view(&root, &[parsed_file], &[], &mut event_log)
            .expect("expected view");

        assert!(!event_log.has_errors());
        assert_eq!(view.files[0].items.len(), 1);
        assert_eq!(view.files[0].items[0].kind, "Writing");
        let writing = &view.files[0].items[0].sections[0];
        assert_eq!(writing.label, "Writing");
        match &writing.arguments[0] {
            ArgumentView::Formulation { latex, .. } => assert_eq!(
                latex.as_deref(),
                Some(r#"\textrm{alpha} \mathrel{:\!\rightsquigarrow} \alpha"#)
            ),
            other => panic!("expected formulation argument, got {other:?}"),
        }
        match &writing.arguments[1] {
            ArgumentView::Formulation { latex, .. } => assert_eq!(
                latex.as_deref(),
                Some(r#"\textrm{Gamma} \mathrel{:\!\rightsquigarrow} \Gamma"#)
            ),
            other => panic!("expected formulation argument, got {other:?}"),
        }
    }

    struct TestDir {
        path: PathBuf,
    }

    impl TestDir {
        fn new() -> Self {
            let unique = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let id = NEXT_TEST_DIR_ID.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                "mlg-view-builder-test-{}-{}-{}",
                std::process::id(),
                unique,
                id
            ));
            fs::create_dir(&path).unwrap();
            Self { path }
        }

        fn path(&self) -> &Path {
            &self.path
        }
    }

    impl Drop for TestDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }
}

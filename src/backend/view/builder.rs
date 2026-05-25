use super::model::{ArgumentView, CollectionView, FileView, GroupView, SectionView};
use super::render::{
    RenderRegistry, build_render_registry, render_formulation_latex, render_group_heading_latex,
};
use crate::events::{Audience, Event, EventLog, Level};
use crate::frontend::{ParsedSourceFile, ProtoArgument, ProtoGroup, ProtoParser, ProtoSection};
use std::path::Path;

/// Builds the complete serialized view model for a MathLingua collection.
///
/// Files must already have passed the shared structural and semantic checking
/// passes.  This pass indexes the checked files for rendering and converts them
/// into JSON-friendly view models.  Returning `None` means a user-facing
/// blocking diagnostic has already been emitted to `event_log`.
pub fn build_collection_view(
    collection_root: &Path,
    parsed_files: &[ParsedSourceFile],
    event_log: &mut EventLog,
) -> Option<CollectionView> {
    let registry = build_render_registry(parsed_files);
    let rendered_files = parsed_files
        .iter()
        .map(|file| build_file_view(collection_root, file, &registry, event_log))
        .collect::<Option<Vec<_>>>()?;

    Some(CollectionView {
        title: collection_title(collection_root),
        files: rendered_files,
    })
}

/// Builds the view model for one already-parsed source file.
///
/// The proto parser is rerun here because the viewer wants to preserve the
/// source's broad group/section layout, while semantic checks use the stricter
/// structural AST.
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

    Some(FileView {
        path: relative_path(collection_root, &file.path),
        items: groups
            .into_iter()
            .map(|group| group_view(group, registry))
            .collect(),
    })
}

/// Returns true when any event should stop viewer generation.
///
/// Debug-level user messages are treated as blocking here because they represent
/// parser recovery states that would make the rendered view misleading.
fn has_blocking_user_issues(events: &[Event]) -> bool {
    events.iter().filter_map(Event::as_message).any(|event| {
        event.audience == Audience::User && matches!(event.level, Level::Error | Level::Debug)
    })
}

/// Derives the display title for a collection from its root directory name.
fn collection_title(collection_root: &Path) -> String {
    collection_root
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.is_empty())
        .unwrap_or("Mathlingua Collection")
        .to_string()
}

/// Formats a source file path relative to the collection root when possible.
fn relative_path(collection_root: &Path, file: &Path) -> String {
    file.strip_prefix(collection_root)
        .unwrap_or(file)
        .display()
        .to_string()
}

/// Converts a proto group into the JSON-friendly group view.
///
/// The first section label becomes the group kind, and its inline argument is
/// supplied to heading rendering so `Refines` cards can combine their own
/// `called:` text with the called text of the thing being refined.
fn group_view(group: ProtoGroup, registry: &RenderRegistry) -> GroupView {
    let kind = group
        .sections
        .first()
        .map(|section| section.label.clone())
        .unwrap_or_else(|| "Group".to_string());
    let primary_inline_argument = group
        .sections
        .first()
        .and_then(|section| section.inline_argument.as_deref());

    GroupView {
        heading_latex: render_group_heading_latex(
            &kind,
            group.heading.as_deref(),
            primary_inline_argument,
            registry,
        ),
        kind,
        heading: group.heading,
        sections: group
            .sections
            .into_iter()
            .map(|section| section_view(section, registry))
            .collect(),
    }
}

/// Converts one proto section into a rendered section view.
fn section_view(section: ProtoSection, registry: &RenderRegistry) -> SectionView {
    let inline_latex = section
        .inline_argument
        .as_deref()
        .and_then(|text| render_formulation_latex(text, registry));

    SectionView {
        label: section.label,
        inline_argument: section.inline_argument,
        inline_latex,
        arguments: section
            .arguments
            .into_iter()
            .map(|argument| argument_view(argument, registry))
            .collect(),
    }
}

/// Converts one proto argument into a rendered argument view.
///
/// Formulation arguments are rendered to LaTeX when possible; text arguments are
/// passed through as display text; nested groups are recursively converted.
fn argument_view(argument: ProtoArgument, registry: &RenderRegistry) -> ArgumentView {
    match argument {
        ProtoArgument::Formulation(formulation) => ArgumentView::Formulation {
            latex: render_formulation_latex(&formulation.text, registry),
            text: formulation.text,
        },
        ProtoArgument::Text(text) => ArgumentView::Text { text: text.text },
        ProtoArgument::Group(group) => ArgumentView::Group {
            heading: group.heading,
            sections: group
                .sections
                .into_iter()
                .map(|section| section_view(section, registry))
                .collect(),
        },
    }
}

// ===============================[ tests ]=====================================

#[cfg(test)]
mod tests {
    use super::build_collection_view;
    use crate::events::EventLog;
    use crate::frontend::{ParsedSourceFile, parse_document};
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

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
        let parsed_file = ParsedSourceFile {
            path: file,
            source: source.to_string(),
            document,
        };
        let mut event_log = EventLog::new();
        let view =
            build_collection_view(&root, &[parsed_file], &mut event_log).expect("expected view");

        assert_eq!(view.title, "repo");
        assert_eq!(view.files.len(), 1);
        assert_eq!(view.files[0].path, "content/sets.mlg");
        assert_eq!(view.files[0].items[0].kind, "Describes");
        assert_eq!(
            view.files[0].items[0].heading_latex,
            Some(r"\textrm{set}".to_string())
        );
        assert!(!event_log.has_errors());
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
            let path = std::env::temp_dir().join(format!(
                "mlg-view-builder-test-{}-{}",
                std::process::id(),
                unique
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

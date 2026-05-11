use super::model::{ArgumentView, CollectionView, FileView, GroupView, SectionView};
use crate::events::{Audience, Event, EventLog, Level};
use crate::frontend::proto::Parser as ProtoParser;
use crate::frontend::proto::ast::{Argument, Group, Section};
use crate::frontend::structural::parse_document;
use std::fs;
use std::path::{Path, PathBuf};

const ORIGIN: &str = "view_builder";

pub fn build_collection_view(
    collection_root: &Path,
    files: &[PathBuf],
    event_log: &mut EventLog,
) -> Option<CollectionView> {
    let mut rendered_files = Vec::new();
    let mut has_blocking_issues = false;

    for file in files {
        match build_file_view(collection_root, file, event_log) {
            Some(file_view) => rendered_files.push(file_view),
            None => has_blocking_issues = true,
        }
    }

    if has_blocking_issues {
        None
    } else {
        Some(CollectionView {
            title: collection_title(collection_root),
            files: rendered_files,
        })
    }
}

fn build_file_view(
    collection_root: &Path,
    file: &Path,
    event_log: &mut EventLog,
) -> Option<FileView> {
    event_log.system_debug(Some(ORIGIN), format!("Rendering {}", file.display()));

    let source = match fs::read_to_string(file) {
        Ok(source) => source,
        Err(error) => {
            event_log.user_error_at_path(
                Some(ORIGIN),
                file.to_path_buf(),
                format!("Failed to read file: {error}"),
            );
            return None;
        }
    };

    let mut validation_log = EventLog::new();
    let _ = parse_document(&source, &mut validation_log);

    for event in validation_log.events() {
        event_log.push(event.clone().with_file_path(file.to_path_buf()));
    }

    if has_blocking_user_issues(&validation_log) {
        return None;
    }

    let mut proto_log = EventLog::new();
    let groups = ProtoParser::new(&source, &mut proto_log).parse();

    Some(FileView {
        path: relative_path(collection_root, file),
        items: groups.into_iter().map(group_view).collect(),
    })
}

fn has_blocking_user_issues(event_log: &EventLog) -> bool {
    event_log
        .events()
        .iter()
        .filter_map(Event::as_message)
        .any(|event| {
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

fn group_view(group: Group) -> GroupView {
    let kind = group
        .sections
        .first()
        .map(|section| section.label.clone())
        .unwrap_or_else(|| "Group".to_string());

    GroupView {
        kind,
        heading: group.heading,
        sections: group.sections.into_iter().map(section_view).collect(),
    }
}

fn section_view(section: Section) -> SectionView {
    SectionView {
        label: section.label,
        inline_argument: section.inline_argument,
        arguments: section.arguments.into_iter().map(argument_view).collect(),
    }
}

fn argument_view(argument: Argument) -> ArgumentView {
    match argument {
        Argument::Formulation(formulation) => ArgumentView::Formulation {
            text: formulation.text,
        },
        Argument::Text(text) => ArgumentView::Text { text: text.text },
        Argument::Group(group) => ArgumentView::Group {
            heading: group.heading,
            sections: group.sections.into_iter().map(section_view).collect(),
        },
    }
}

// =============================================================================

#[cfg(test)]
mod tests {
    use super::build_collection_view;
    use crate::events::EventLog;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn builds_a_collection_view_from_valid_files() {
        let temp_dir = TestDir::new();
        let root = temp_dir.path().join("repo");
        let content = root.join("content");
        let file = content.join("sets.mlg");

        fs::create_dir_all(&content).unwrap();
        fs::write(
            &file,
            "[\\set]\nDescribes: S\nDocumented:\n. called:\n  . \"set\"\n",
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let view = build_collection_view(&root, &[file], &mut event_log).expect("expected view");

        assert_eq!(view.title, "repo");
        assert_eq!(view.files.len(), 1);
        assert_eq!(view.files[0].path, "content/sets.mlg");
        assert_eq!(view.files[0].items[0].kind, "Describes");
        assert!(event_log.has_errors().not());
    }

    trait BoolExt {
        fn not(self) -> bool;
    }

    impl BoolExt for bool {
        fn not(self) -> bool {
            !self
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

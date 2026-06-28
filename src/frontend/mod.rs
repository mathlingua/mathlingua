use crate::events::EventLog;
use std::fs;
use std::path::{Path, PathBuf};

pub(crate) mod formulation;
pub(crate) mod proto;
pub(crate) mod structural;

pub use formulation::ast::*;
pub use structural::ast::*;
pub use structural::parse_document;

pub(crate) use formulation::{
    parse_command_header, parse_expression, parse_form_or_declaration,
    parse_refined_declaration_statement,
};
pub(crate) use proto::Parser as ProtoParser;
pub(crate) use proto::ast::{
    Argument as ProtoArgument, Group as ProtoGroup, Section as ProtoSection,
};

#[derive(Clone, Debug)]
pub struct ParsedSourceFile {
    pub path: PathBuf,
    pub source: String,
    pub document: Document,
    pub item_ids: Vec<TopLevelItemId>,
    pub view_metadata: SourceFileViewMetadata,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TopLevelItemId {
    pub value: Option<String>,
    pub group_row: usize,
    pub id_row: Option<usize>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SourceFileViewMetadata {
    pub hidden: bool,
    pub title: Option<String>,
}

pub fn parse_source_file(
    path: &Path,
    event_log: &mut EventLog,
    origin: &str,
) -> Option<ParsedSourceFile> {
    event_log.system_debug(Some(origin), format!("Parsing {}", path.display()));

    let source = match fs::read_to_string(path) {
        Ok(source) => source,
        Err(error) => {
            event_log.user_error_at_path(
                Some(origin),
                path.to_path_buf(),
                format!("Failed to read file: {error}"),
            );
            return None;
        }
    };

    let mut file_event_log = EventLog::new();
    let document = parse_document(&source, &mut file_event_log);
    let item_ids = top_level_item_ids(&source);

    for event in file_event_log.events() {
        event_log.push(event.clone().with_file_path(path.to_path_buf()));
    }

    Some(ParsedSourceFile {
        path: path.to_path_buf(),
        source,
        document,
        item_ids,
        view_metadata: SourceFileViewMetadata::default(),
    })
}

pub(crate) fn top_level_item_ids(source: &str) -> Vec<TopLevelItemId> {
    let mut event_log = EventLog::new();
    let groups = {
        let mut parser = ProtoParser::new(source, &mut event_log);
        parser.parse()
    };

    groups.iter().map(top_level_item_id).collect()
}

pub(crate) fn top_level_group_id(group: &ProtoGroup) -> Option<String> {
    group
        .sections
        .iter()
        .find(|section| section.label == "Id")
        .and_then(section_quoted_text)
}

fn top_level_item_id(group: &ProtoGroup) -> TopLevelItemId {
    let id_section = group.sections.iter().find(|section| section.label == "Id");

    TopLevelItemId {
        value: id_section.and_then(section_quoted_text),
        group_row: group.metadata.row,
        id_row: id_section.map(|section| section.metadata.row),
    }
}

fn section_quoted_text(section: &ProtoSection) -> Option<String> {
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

fn strip_quoted_text(input: &str) -> Option<String> {
    let input = input.trim();
    let inner = input.strip_prefix('"')?.strip_suffix('"')?;
    Some(inner.to_owned())
}

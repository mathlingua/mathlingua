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
    parse_is_or_refined_statement_spec,
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

    for event in file_event_log.events() {
        event_log.push(event.clone().with_file_path(path.to_path_buf()));
    }

    Some(ParsedSourceFile {
        path: path.to_path_buf(),
        source,
        document,
    })
}

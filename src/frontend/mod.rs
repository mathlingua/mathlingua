//! Frontend parsing pipeline for MathLingua source text.
//!
//! The frontend has three layers: proto parsing for indentation/group structure,
//! formulation parsing for mathematical expressions, and structural parsing that
//! maps proto groups into typed MathLingua document groups.

use crate::events::EventLog;
use std::fs;
use std::path::{Path, PathBuf};

/// Formula/expression lexer, parser, and AST.
pub(crate) mod formulation;
/// Indentation-sensitive proto lexer/parser and AST.
pub(crate) mod proto;
/// Typed structural document parser and AST.
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

/// A source file after the frontend has parsed it into a structural document.
///
/// Backend passes operate on this type instead of repeatedly reading or
/// reparsing files. It keeps the original source text for diagnostic location
/// lookup, the filesystem path for reporting, and the parsed structural AST for
/// semantic traversal.
#[derive(Clone, Debug)]
pub struct ParsedSourceFile {
    /// Path of the file on disk, used in diagnostics and duplicate reports.
    pub path: PathBuf,
    /// Original file contents, used to recover line and column information.
    pub source: String,
    /// Structural representation produced by the frontend parser.
    pub document: Document,
}

/// Reads and structurally parses one source file.
///
/// Parser diagnostics are rewritten with the file path so downstream console
/// output can report precise file locations.
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

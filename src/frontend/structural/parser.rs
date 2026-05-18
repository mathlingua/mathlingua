use std::collections::{HashMap, VecDeque};

use crate::events::EventLog;

/// Diagnostic origin attached to structural parser errors.
///
/// This distinguishes section-shape and group-shape diagnostics from lower
/// proto parsing errors and later backend semantic checks.
const ORIGIN: &str = "structural_parser";
use crate::frontend::formulation::{
    ParseError as FormulationParseError, parse_author_header, parse_command_header,
    parse_expression, parse_expression_alias, parse_form_or_declaration,
    parse_is_or_refined_statement_spec, parse_is_or_spec, parse_is_via_statement,
    parse_label_header, parse_resource_header, parse_spec_operator_alias, parse_writing_alias,
};
use crate::frontend::proto::Parser as ProtoParser;
use crate::frontend::proto::ast::{
    Argument as ProtoArgument, Formulation as ProtoFormulation, Group as ProtoGroup,
    Section as ProtoSection, TextLiteral as ProtoText,
};

use super::ast::*;

/// Parses raw MathLingua source into the strongly typed structural AST.
///
/// This function composes the proto parser with structural recognition.  Proto
/// groups that cannot be recognized are diagnosed and skipped, allowing valid
/// neighboring groups to continue into backend checks and rendering.
pub fn parse_document(input: &str, tracker: &mut EventLog) -> Document {
    let groups = {
        let mut proto_parser = ProtoParser::new(input, tracker);
        proto_parser.parse()
    };

    let mut items = Vec::new();
    for group in &groups {
        if let Some(item) = parse_top_level_group(group, tracker) {
            items.push(item);
        }
    }

    Document {
        items: ZeroOrMore::from(items),
    }
}

include!("parser/top_level.rs");
include!("parser/nested.rs");
include!("parser/clauses.rs");
include!("parser/helpers.rs");

#[cfg(test)]
mod tests;

//! Formulation lexer, parser, and AST for inline mathematical language.
//!
//! Formulations are the math-like fragments that appear in section arguments,
//! command headings, aliases, clauses, and rendering templates.

/// Parsed formulation AST types.
pub mod ast;
/// Logos-backed lexer adapter used by the generated parser.
pub mod lexer;
/// Public parser helpers for each supported formulation grammar entrypoint.
pub mod parser;
/// Source span type used by formulation AST nodes.
pub mod span;
/// Token definitions for the formulation lexer.
pub mod token;

// Generated LALRPOP parser module.  This is a macro invocation rather than a
// regular item, so it uses a normal comment instead of rustdoc.
lalrpop_util::lalrpop_mod!(pub grammar, "/frontend/formulation/grammar.rs");

/// Public parser entrypoints and shared parse error type.
pub use parser::{
    ParseError, parse_author_header, parse_command_header, parse_expression,
    parse_expression_alias, parse_form_or_declaration, parse_is_or_refined_statement_spec,
    parse_is_or_spec, parse_is_via_statement, parse_label_header, parse_resource_header,
    parse_spec_operator_alias, parse_writing_alias,
};

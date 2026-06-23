pub mod ast;
pub mod lexer;
pub mod parser;
pub mod token;

lalrpop_util::lalrpop_mod!(pub grammar, "/frontend/formulation/grammar.rs");

pub use parser::{
    ParseError, parse_author_header, parse_command_header, parse_expression,
    parse_expression_alias, parse_expression_binding, parse_form_or_declaration,
    parse_is_or_refined_statement_spec, parse_is_or_spec, parse_is_via_statement,
    parse_label_header, parse_resource_header, parse_spec_operator_alias, parse_writing_alias,
};

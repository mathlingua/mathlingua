pub mod ast;
pub mod lexer;
pub mod parser;
pub mod token;

lalrpop_util::lalrpop_mod!(pub grammar, "/frontend/formulation/grammar.rs");

#[allow(unused_imports)]
pub use parser::parse_declaration_statement;
pub use parser::{
    ParseError, parse_author_header, parse_command_header, parse_expression,
    parse_expression_alias, parse_form_or_declaration, parse_is_via_statement, parse_label_header,
    parse_ordinary_declaration_statement, parse_refined_declaration_statement,
    parse_resource_header, parse_spec_operator_alias, parse_writing_alias,
};

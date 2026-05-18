use std::collections::HashMap;

use crate::backend::semantic::ParsedSourceFile;
use crate::frontend::formulation::ast::{
    BinaryOperator, Chain, ChainPart, CommandExpression, CommandExpressionTailPart, CommandHeader,
    CommandHeaderNode, Expression, ExpressionKind, FormOrDeclaration, FormOrDeclarationKind,
    InfixCommandHeader, IsOrRefinedStatementSpec, IsOrSpec, IsStatement, IsSubject, IsSubjectForm,
    IsSubjectKind, RefinedCommandExpression, RefinedCommandHeader, RefinedExpressionPart,
    RefinedTail, SetExpression, SpecStatement, SpecSubject, SpecSubjectKind,
    TupleExpressionElement, TupleFormElement, TypeExpression, UnaryOperator,
};
use crate::frontend::formulation::{
    parse_command_header, parse_expression, parse_form_or_declaration,
    parse_is_or_refined_statement_spec,
};
use crate::frontend::structural::ast::*;

include!("render/registry.rs");
include!("render/expressions.rs");
include!("render/statements.rs");
include!("render/commands.rs");
include!("render/fallbacks.rs");
include!("render/templates.rs");
include!("render/names.rs");
include!("render/signatures.rs");
include!("render/escaping.rs");

#[cfg(test)]
mod tests;

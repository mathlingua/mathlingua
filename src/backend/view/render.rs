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

mod commands;
mod escaping;
mod expressions;
mod fallbacks;
mod names;
mod registry;
mod signatures;
mod statements;
mod templates;

use commands::*;
use escaping::*;
use expressions::*;
use fallbacks::*;
use names::*;
use registry::*;
pub(super) use registry::{
    RenderRegistry, build_render_registry, render_formulation_latex, render_group_heading_latex,
};
use signatures::*;
use statements::*;
use templates::*;

#[cfg(test)]
mod tests;

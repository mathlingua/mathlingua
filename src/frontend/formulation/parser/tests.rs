use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use super::{
    parse_author_header, parse_command_header, parse_expression, parse_expression_alias,
    parse_form_or_declaration, parse_is_or_refined_statement_spec, parse_is_or_spec,
    parse_is_via_statement, parse_label_header, parse_resource_header, parse_spec_operator_alias,
    parse_writing_alias,
};
use crate::frontend::formulation::ast::{
    BinaryOperator, ChainPart, CommandHeader, CommandHeaderNode, Expression, ExpressionAliasLhs,
    ExpressionKind, FormOrDeclaration, FormOrDeclarationKind, FunctionNamedExpressionElementLhs,
    IsOrRefinedStatementSpec, IsOrSpec, IsSubjectForm, IsSubjectKind, NamedOperatorKind,
    PlaceholderFormKind, RefinedTail, SpecSubjectKind, SubsetCall, TypeExpression,
};

include!("tests/support.rs");
include!("tests/operators.rs");
include!("tests/commands.rs");
include!("tests/expressions.rs");
include!("tests/refinements.rs");
include!("tests/golden.rs");

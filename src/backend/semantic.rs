use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::events::{EventLocation, EventLog, EventPosition, EventSpan, Level};
use crate::frontend::formulation::ast::{
    Chain, ChainPart, CommandExpression, CommandExpressionTailPart, CommandHeader,
    CommandHeaderNode, CommandHeaderTailPart, Expression, ExpressionKind, FormOrDeclaration,
    FormOrDeclarationKind, InfixCommand, InfixCommandHeader, IsOrRefinedStatementSpec, IsOrSpec,
    IsStatement, IsSubject, IsSubjectForm, IsSubjectKind, PlaceholderForm, PlaceholderFormKind,
    RefinedCommandExpression, RefinedCommandHeader, RefinedExpressionPart, RefinedTail,
    SpecSubject, SpecSubjectKind, TupleExpressionElement, TupleFormElement, TypeExpression,
};
use crate::frontend::structural::ast::*;

/// Event origin used for all diagnostics produced by the semantic checker.
const ORIGIN: &str = "semantic_check";

include!("semantic/types.rs");
include!("semantic/check.rs");
include!("semantic/validation.rs");
include!("semantic/shapes.rs");
include!("semantic/locator.rs");
include!("semantic/walk.rs");

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::events::{EventLocation, EventLog, EventPosition, EventSpan, Level};
use crate::frontend::formulation::ast::{
    Chain, ChainPart, CommandExpression, CommandExpressionTailPart, CommandHeader,
    CommandHeaderNode, CommandHeaderTailPart, CurlyExpressionArgs, Expression, ExpressionKind,
    FormOrDeclaration, FormOrDeclarationKind, InfixCommand, InfixCommandHeader,
    IsOrRefinedStatementSpec, IsOrSpec, IsStatement, IsSubject, IsSubjectForm, IsSubjectKind,
    PlaceholderForm, PlaceholderFormKind, RefinedCommandExpression, RefinedCommandHeader,
    RefinedExpressionPart, RefinedTail, SpecOperatorAlias, SpecOperatorAliasTarget, SpecSubject,
    SpecSubjectKind, TupleExpressionElement, TupleFormElement, TypeExpression,
};
use crate::frontend::structural::ast::*;

/// Event origin used for all diagnostics produced by the semantic checker.
const ORIGIN: &str = "semantic_check";

mod check;
mod locator;
mod shapes;
mod typecheck;
mod types;
mod validation;
mod walk;

pub use check::check_documents;
pub use types::ParsedSourceFile;

use locator::*;
use shapes::*;
use typecheck::*;
use types::*;
use validation::*;
use walk::*;

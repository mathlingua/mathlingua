use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::events::{EventLocation, EventLog, EventPosition, EventSpan, Level};
use crate::frontend::*;

const ORIGIN: &str = "semantic_check";

mod check;
mod definition;
mod locator;
mod rename;
mod shapes;
mod typecheck;
mod typeinfo;
mod types;
mod uses;
mod validation;
mod walk;

pub use check::{check_documents, check_documents_collecting_type_info};
pub use definition::{DefinitionSite, find_definition};
pub use rename::{
    RenameEditPlan, RenameError, RenamePreparation, RenameSpan, plan_rename, prepare_rename,
};
pub use typeinfo::{DocumentTypeInfo, TypeEntry};
pub(crate) use uses::{collect_definition_locations, command_occurrences};

use locator::*;
use shapes::*;
use typecheck::*;
use typeinfo::*;
use types::*;
use validation::*;
use walk::*;

/// Specialized mapping-parameter signature data needed by the view registry.
/// Keeping the semantic pattern types private prevents the renderer from
/// acquiring a second, subtly different interpretation of header syntax.
pub(crate) fn mapping_parameter_header_signatures(
    header: &CommandHeader,
) -> Option<(String, String)> {
    placeholder_signature_for_header(header)
        .ok()
        .flatten()
        .map(|(signature, pattern)| (signature, pattern.general_signature))
}

/// Concrete and general signatures plus actual mapping-slot information for a
/// command invocation that uses mapping parameters.
pub(crate) fn mapping_parameter_invocation_signatures(
    command: &CommandExpression,
) -> Option<(String, String, usize, Vec<usize>)> {
    placeholder_invocation_for_command_expression(command).map(|invocation| {
        (
            invocation.signature,
            invocation.general_signature,
            invocation.mapping_arity,
            invocation.selected_positions,
        )
    })
}

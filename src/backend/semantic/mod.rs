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
mod types;
mod validation;
mod walk;

pub use check::check_documents;
pub use definition::{DefinitionSite, find_definition};
pub use rename::{
    RenameEditPlan, RenameError, RenamePreparation, RenameSpan, plan_rename, prepare_rename,
};

use locator::*;
use shapes::*;
use typecheck::*;
use types::*;
use validation::*;
use walk::*;

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::events::{EventLocation, EventLog, EventPosition, EventSpan, Level};
use crate::frontend::*;

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

use locator::*;
use shapes::*;
use typecheck::*;
use types::*;
use validation::*;
use walk::*;

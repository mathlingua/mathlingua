use std::collections::BTreeSet;
use std::path::Path;

use super::parse_document;
use crate::events::{Event, EventLog};
use crate::frontend::structural::ast::{
    AliasItem, AliasKind, Clause, Document, DocumentedItem, IsOrViaItem, JustifiedItem,
    MetadataItem, ProvidesItem, ResourceItem, SpecifyItem, TopLevelItem,
};

mod definitions;
mod diagnostics;
mod metadata;
mod overview;
mod support;
mod theorems;

use support::*;

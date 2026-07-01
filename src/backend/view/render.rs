use std::collections::HashMap;

use crate::frontend::formulation::parse_writing_alias;
use crate::frontend::*;

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
#[cfg(test)]
pub(super) use registry::build_render_registry;
use registry::*;
pub(super) use registry::{
    RenderRegistry, build_linked_render_registry, definition_reference_keys_for_heading,
    render_documented_text_latex, render_formulation_latex, render_group_heading_latex,
    render_writing_alias_latex,
};
use signatures::*;
use statements::*;
use templates::*;

#[cfg(test)]
mod tests;

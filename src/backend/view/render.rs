use std::collections::HashMap;

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
use registry::*;
pub(super) use registry::{
    RenderRegistry, build_render_registry, render_formulation_latex, render_group_heading_latex,
};
use signatures::*;
use statements::*;
use templates::*;

#[cfg(test)]
mod tests;

//! LaTeX rendering support for the serialized collection viewer.
//!
//! This module indexes documented `called:` and `written:` templates from
//! parsed MathLingua sources, then uses that registry to render formulation
//! fragments and group headings into the LaTeX strings consumed by the web app.

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

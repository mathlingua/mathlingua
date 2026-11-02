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
mod text;

use commands::*;
use escaping::*;
use expressions::*;
use fallbacks::*;
use names::*;
pub use registry::{RenderRegistry, build_render_registry};
use registry::*;
pub(super) use registry::{
    build_linked_render_registry, definition_reference_keys_for_heading,
    join_title_parts, render_documented_text_latex, render_formulation_latex,
    render_group_heading_latex, render_group_parameter_destructurings,
    render_refines_section_latex, render_refines_specifies_latex, render_resource_reference,
    render_writing_alias_latex, resolve_topic_heading_latex, writing_alias_override,
};
use signatures::*;
use statements::*;
use templates::*;
pub(super) use commands::{
    render_command_expression, render_infix_command_expression,
};
pub(super) use expressions::{render_builtin_command_expression, render_expression};
pub(super) use templates::render_written_display_template;
pub(super) use text::{
    markdown_code_span_len, math_fragment, render_scoped_text_markdown,
    render_scoped_text_markdown_with_labels, theorem_reference_fragment,
};

#[cfg(test)]
mod tests;

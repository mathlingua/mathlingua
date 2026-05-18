use super::*;

mod clauses;
mod formulations;
mod groups;
mod headings;
mod sections;
mod text;

pub(in crate::frontend::structural::parser) use clauses::*;
pub(in crate::frontend::structural::parser) use formulations::*;
pub(in crate::frontend::structural::parser) use groups::*;
pub(in crate::frontend::structural::parser) use headings::*;
pub(in crate::frontend::structural::parser) use sections::*;
pub(in crate::frontend::structural::parser) use text::*;

use super::*;

mod documentation;
mod justification;
mod metadata;
mod resource_items;
mod spec_items;

pub(in crate::frontend::structural::parser) use documentation::*;
pub(in crate::frontend::structural::parser) use justification::*;
pub(in crate::frontend::structural::parser) use metadata::*;
pub(in crate::frontend::structural::parser) use resource_items::*;
pub(in crate::frontend::structural::parser) use spec_items::*;

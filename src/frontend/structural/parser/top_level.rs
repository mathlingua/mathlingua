use super::*;

mod definitions;
mod dispatch;
mod metadata;
mod outline;
mod theorems;

pub(in crate::frontend::structural::parser) use definitions::*;
pub(in crate::frontend::structural::parser) use dispatch::*;
pub(in crate::frontend::structural::parser) use metadata::*;
pub(in crate::frontend::structural::parser) use outline::*;
pub(in crate::frontend::structural::parser) use theorems::*;

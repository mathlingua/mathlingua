//! High-level implementations of the `mlg` command family.
//!
//! These modules sit between the binary entrypoint and the lower-level parser,
//! backend, event, and viewer layers.

/// Check command result and entrypoints.
pub use check::{CheckResult, check, check_in};
/// Collection initialization entrypoint.
pub use init::init;
/// Version command entrypoint.
pub use version::version;
/// Viewer command entrypoints.
pub use view::{view, view_in};

pub use facade::Mlg;

mod check;
mod collection;
mod config;
mod facade;
mod init;
mod version;
mod view;

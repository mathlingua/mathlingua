//! High-level implementations of the `mlg` command family.
//!
//! These modules sit between the binary entrypoint and the lower-level parser,
//! backend, event, and viewer layers.

/// Check command result and entrypoint.
pub use check::{CheckResult, check};
/// Collection initialization result and entrypoint.
pub use init::{InitResult, init};
/// Version command result and entrypoint.
pub use version::{VersionResult, version};
/// Viewer command result and entrypoint.
pub use view::{ViewResult, view};

mod check;
mod init;
mod util;
mod version;
mod view;

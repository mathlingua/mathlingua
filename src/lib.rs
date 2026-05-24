//! Public library surface for the MathLingua CLI implementation.
//!
//! The crate exposes the top-level `mlg` operations for tests and embedding
//! while keeping implementation modules organized by parser/frontend, backend,
//! events, and command orchestration.

/// Public command helpers used by the binary and integration callers.
pub use mlg::{CheckResult, InitResult, VersionResult, ViewResult, check, init, version, view};

/// Backend semantic checks and viewer model generation.
pub mod backend;
/// Command-line argument definitions.
pub mod cli;
/// Event model and console rendering infrastructure.
pub mod events;
/// Frontend lexing, parsing, and structural AST modules.
pub mod frontend;

mod mlg;

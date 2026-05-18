//! Structural parser for typed MathLingua documents.
//!
//! This layer converts proto groups into strongly typed document structures such
//! as definitions, theorems, clauses, metadata, and resource records.

/// Structural document AST.
pub mod ast;
/// Structural parser entrypoints and helpers.
pub mod parser;

/// Parses source text into a structural document.
pub use parser::parse_document;

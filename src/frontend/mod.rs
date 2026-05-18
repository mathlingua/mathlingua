//! Frontend parsing pipeline for MathLingua source text.
//!
//! The frontend has three layers: proto parsing for indentation/group structure,
//! formulation parsing for mathematical expressions, and structural parsing that
//! maps proto groups into typed MathLingua document groups.

/// Formula/expression lexer, parser, and AST.
pub mod formulation;
/// Indentation-sensitive proto lexer/parser and AST.
pub mod proto;
/// Typed structural document parser and AST.
pub mod structural;

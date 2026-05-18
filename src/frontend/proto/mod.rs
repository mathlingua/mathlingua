//! Proto parser for the indentation-sensitive MathLingua surface syntax.
//!
//! The proto layer preserves source groups, sections, text arguments, and nested
//! arguments before the stricter structural parser assigns domain meaning.

/// Proto AST types.
pub mod ast;
/// Line-oriented lexer for proto syntax.
pub mod lexer;
/// Proto parser that builds groups from lexed lines.
pub mod parser;

/// Public proto parser type.
pub use parser::Parser;

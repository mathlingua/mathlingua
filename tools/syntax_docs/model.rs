//! Data model shared by the syntax-doc extractors and renderers.
//!
//! Every type here records something read out of the implementation sources. The
//! extractors never invent information: if a fact cannot be derived from the code,
//! extraction fails with an error rather than guessing, so a generated document is
//! either faithful to the code or absent.

use std::fmt;

/// How many values a section holds, as encoded by the section macro used to
/// declare it in `src/frontend/structural/ast.rs`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Arity {
    /// `argument_section!` — exactly one value.
    One,
    /// `arguments_section!` — `OneOrMore`.
    OneOrMore,
    /// `zero_or_more_arguments_section!` — `ZeroOrMore`.
    ZeroOrMore,
}

impl Arity {
    /// The suffix used in the rendered notation (`X`, `X+`, `X*`).
    pub fn suffix(self) -> &'static str {
        match self {
            Self::One => "",
            Self::OneOrMore => "+",
            Self::ZeroOrMore => "*",
        }
    }
}

/// A `*Section` type declared by one of the three section macros.
#[derive(Clone, Debug)]
pub struct SectionDef {
    /// The section struct name, e.g. `ThenSection`.
    pub type_name: String,
    pub arity: Arity,
    /// The element type held by the section, e.g. `Clause`.
    pub element: String,
}

/// What kind of heading a group accepts, derived from the heading helper the
/// group's parse function calls.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HeadingKind {
    CommandRequired,
    CommandOptional,
    LabelOptional,
    AuthorRequired,
    ResourceRequired,
    TopicRequired,
    /// The parse function calls no heading helper, so the group takes no heading.
    None,
}

impl fmt::Display for HeadingKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            Self::CommandRequired => "[CommandHeader]",
            Self::CommandOptional => "[CommandHeader]?",
            Self::LabelOptional => "[LabelHeader]?",
            Self::AuthorRequired => "[AuthorHeader]",
            Self::ResourceRequired => "[ResourceHeader]",
            Self::TopicRequired => "[TopicHeader]",
            Self::None => "",
        };
        write!(f, "{text}")
    }
}

/// One field of a `*Group` struct.
#[derive(Clone, Debug)]
pub struct GroupField {
    /// The Rust field name, e.g. `where_`.
    pub name: String,
    /// True when the field is `Option<...>`, meaning the section may be omitted.
    pub optional: bool,
    /// The field's inner type with any `Option`/`Box` stripped, e.g. `WhereSection`.
    pub type_name: String,
}

/// A `*Group` struct declared in the structural AST.
#[derive(Clone, Debug)]
pub struct GroupStruct {
    pub name: String,
    pub fields: Vec<GroupField>,
}

/// A label in a group's expected-section sequence, as declared in the
/// `identify_sections` call inside the group's parse function.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExpectedLabel {
    pub label: String,
    /// True when the label carried a `?` suffix in the parser's expected list.
    pub optional: bool,
}

/// What a parse function declares about one group: the ordered section labels it
/// accepts and the heading it requires.
#[derive(Clone, Debug)]
pub struct ParsedGroup {
    /// The `*Group` types the parse function may build. Usually one; a parse function
    /// whose optional sections select between shapes builds several (`parse_from_group`
    /// builds `FromCapabilityGroup` or `FromAsGroup` from the same `from:` syntax).
    pub group_types: Vec<String>,
    /// The name passed to `identify_sections` for diagnostics; also the head label.
    pub head_label: String,
    pub heading: HeadingKind,
    pub labels: Vec<ExpectedLabel>,
    /// Group field name -> the section label whose contents it stores, read from the
    /// field's initializer in the parse function. A field name is not always its label:
    /// `parse_have_clause` stores the `have:` section in `IffGroup::then`.
    pub field_labels: std::collections::HashMap<String, String>,
}

/// A fully resolved section row, ready to render.
#[derive(Clone, Debug)]
pub struct ResolvedSection {
    pub label: String,
    /// True when the section itself may be omitted.
    pub optional: bool,
    /// The element type, e.g. `Clause`. `None` for a head section that takes no
    /// argument (it has no backing AST field), such as `Theorem:`.
    pub element: Option<String>,
    /// Arity of the values inside the section, when it has a backing field.
    pub arity: Option<Arity>,
}

/// A group with its parser-declared labels joined to its AST-declared types.
#[derive(Clone, Debug)]
pub struct ResolvedGroup {
    /// The AST types this syntax may build; more than one when optional sections
    /// select between shapes.
    pub group_types: Vec<String>,
    pub head_label: String,
    pub heading: HeadingKind,
    pub sections: Vec<ResolvedSection>,
}

/// A group whose parse function does not follow the `identify_sections` pattern, so its
/// shape cannot be derived.
#[derive(Clone, Debug)]
pub struct UnderivableGroup {
    /// The `*Group` types the function builds.
    pub types: Vec<String>,
    /// The parse function, so a reader knows where to look.
    pub parse_fn: String,
}

/// An enum used as a union of alternatives.
#[derive(Clone, Debug)]
pub struct EnumDef {
    pub name: String,
    /// Each variant's inner type, or the variant name when it holds no payload.
    pub variants: Vec<EnumVariant>,
}

#[derive(Clone, Debug)]
pub struct EnumVariant {
    pub name: String,
    /// The single payload type with `Box`/`Option` stripped, when present.
    pub payload: Option<String>,
}

/// A plain (non-group, non-section) struct in an AST, e.g. a formulation node.
#[derive(Clone, Debug)]
pub struct StructDef {
    pub name: String,
    pub fields: Vec<StructField>,
}

#[derive(Clone, Debug)]
pub struct StructField {
    pub name: String,
    /// The field type rendered back to source-like text, e.g. `Option<Box<Expression>>`.
    pub type_text: String,
}

/// A public formulation parser entry point: `pub fn parse_x(input: &str) -> Result<T, ParseError>`.
///
/// These are the functions the structural parser calls to turn section text into
/// formulation nodes. Headers and statements are parsed here by hand-written code
/// rather than by the LALRPOP grammar, so they have no production.
#[derive(Clone, Debug)]
pub struct EntryPoint {
    /// The function name, e.g. `parse_command_header`.
    pub name: String,
    /// The type it produces, e.g. `CommandHeader`.
    pub result_type: String,
}

/// A terminal declared in the grammar's `extern { enum Token { ... } }` block.
#[derive(Clone, Debug)]
pub struct Terminal {
    /// The quoted name the grammar uses, e.g. `is?`.
    pub name: String,
    /// The `Token` variant it maps to, e.g. `Token::IsPredicate`.
    pub variant: String,
    /// The payload type carried by the token, when any, e.g. `String`.
    pub payload: Option<String>,
}

/// One alternative of a grammar nonterminal: the symbol sequence with the action
/// code stripped.
#[derive(Clone, Debug)]
pub struct Production {
    /// The symbols in order, already normalized (bindings unwrapped, `@L`/`@R` dropped).
    pub symbols: Vec<String>,
}

/// A nonterminal defined in the LALRPOP grammar.
#[derive(Clone, Debug)]
pub struct Nonterminal {
    pub name: String,
    /// True for `pub` entry points.
    pub public: bool,
    /// The declared result type, e.g. `ast::Expression`.
    pub result_type: String,
    /// The result type reduced to a bare AST type name, e.g. `Expression`.
    pub result_ast_type: Option<String>,
    pub productions: Vec<Production>,
}

/// A generated markdown document.
#[derive(Clone, Debug)]
pub struct GeneratedFile {
    /// Path relative to the repository root.
    pub path: String,
    pub contents: String,
}

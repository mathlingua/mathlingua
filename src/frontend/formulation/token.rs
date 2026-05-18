use std::fmt;

use logos::Logos;

/// Error emitted when the formulation lexer cannot classify a token.
#[derive(Default, Clone, Debug, PartialEq, Eq)]
pub enum LexicalError {
    /// Unrecognized token in formulation input.
    #[default]
    InvalidToken,
}

impl fmt::Display for LexicalError {
    /// Formats the lexical error for parser diagnostics.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidToken => write!(f, "invalid token"),
        }
    }
}

/// Tokens recognized by the formulation lexer.
///
/// These tokens cover math expressions, command headers, command expressions,
/// aliases, labels, placeholders, named operators, and punctuation used by the
/// generated grammar.
#[derive(Logos, Clone, Debug, PartialEq, Eq)]
#[logos(skip r"[ \t\r\n\f]+", error = LexicalError)]
pub enum Token {
    /// `(.` grouped-expression opener.
    #[token("(.")]
    DotLParen,
    /// `.)` grouped-expression closer.
    #[token(".)")]
    DotRParen,
    /// `[|` named-argument opener.
    #[token("[|")]
    LNamedArguments,
    /// `|]` named-argument closer.
    #[token("|]")]
    RNamedArguments,
    /// `\:` infix-command opener.
    #[token("\\:")]
    InfixCommandStart,
    /// `:/` infix-command closer.
    #[token(":/")]
    InfixCommandEnd,
    /// Named operator with both-side colon binding, such as `:|foo|:`.
    #[regex(
        r":\|[A-Za-z0-9]+(?:[A-Za-z0-9_]*[A-Za-z0-9]+)?\|:",
        both_named_operator
    )]
    BothNamedOperator(String),
    /// Named operator with left-colon binding, such as `:|foo|`.
    #[regex(
        r":\|[A-Za-z0-9]+(?:[A-Za-z0-9_]*[A-Za-z0-9]+)?\|",
        left_named_operator
    )]
    LeftNamedOperator(String),
    /// Named operator with right-colon binding, such as `|foo|:`.
    #[regex(
        r"\|[A-Za-z0-9]+(?:[A-Za-z0-9_]*[A-Za-z0-9]+)?\|:",
        right_named_operator
    )]
    RightNamedOperator(String),
    /// Plain named operator, such as `|foo|`.
    #[regex(
        r"\|[A-Za-z0-9]+(?:[A-Za-z0-9_]*[A-Za-z0-9]+)?\|",
        plain_named_operator
    )]
    NamedOperator(String),
    /// Label token, such as `[:some.label:]`.
    #[regex(r"\[:[A-Za-z0-9]+(?:[A-Za-z0-9_]*[A-Za-z0-9]+)?(?:\.[A-Za-z0-9]+(?:[A-Za-z0-9_]*[A-Za-z0-9]+)?)*:\]", parse_label)]
    Label(Vec<String>),
    /// Expression alias operator `:=>`.
    #[token(":=>")]
    ExpressionAlias,
    /// Spec-operator alias operator `:->`.
    #[token(":->")]
    SpecOperatorAlias,
    /// Writing alias operator `:~>`.
    #[token(":~>")]
    WritingAlias,
    /// Declaration operator `:=`.
    #[token(":=")]
    Declare,
    /// Predicate token `is_not?`.
    #[token("is_not?")]
    IsNotPredicate,
    /// Predicate token `is?`.
    #[token("is?")]
    IsPredicate,
    /// Type assertion token `is`.
    #[token("is")]
    Is,
    /// Is-via token `via`.
    #[token("via")]
    Via,
    /// Quoted command/operator name.
    #[regex(r#""[A-Za-z0-9]+(?:[A-Za-z0-9_]*[A-Za-z0-9]+)?""#, parse_quoted_name)]
    QuotedName(String),
    /// Magnetic placeholder, written with a double underscore suffix.
    #[regex(
        r"[A-Za-z0-9]+(?:[A-Za-z0-9_]*[A-Za-z0-9]+)?__",
        parse_magnetic_placeholder
    )]
    MagneticPlaceholder(String),
    /// Placeholder, written with a single underscore suffix.
    #[regex(r"[A-Za-z0-9]+(?:[A-Za-z0-9_]*[A-Za-z0-9]+)?_", parse_placeholder)]
    Placeholder(String),
    /// Ordinary name or stropped symbolic name.
    #[regex(
        r"`[-~!#%^&*\\+=|<>/]+`|[A-Za-z0-9]+(?:[A-Za-z0-9_]*[A-Za-z0-9]+)?",
        parse_name
    )]
    Name(String),
    /// Symbolic operator token that is not one of the dedicated punctuation tokens.
    #[regex(r"(?:[-~!#%^&*+=|<>/]{2,}|[~!#%&<>])", parse_special_operator)]
    SpecialOperator(String),
    /// Backslash starting a command expression/header.
    #[token("\\")]
    CommandStart,
    /// `+`.
    #[token("+")]
    Plus,
    /// `-`.
    #[token("-")]
    Minus,
    /// `*`.
    #[token("*")]
    Star,
    /// `/`.
    #[token("/")]
    Slash,
    /// `=`.
    #[token("=")]
    Equals,
    /// `^`.
    #[token("^")]
    Caret,
    /// `(`.
    #[token("(")]
    LParen,
    /// `)`.
    #[token(")")]
    RParen,
    /// `{`.
    #[token("{")]
    LBrace,
    /// `}`.
    #[token("}")]
    RBrace,
    /// `[`.
    #[token("[")]
    LBracket,
    /// `]`.
    #[token("]")]
    RBracket,
    /// `,`.
    #[token(",")]
    Comma,
    /// `:`.
    #[token(":")]
    Colon,
    /// `.`.
    #[token(".")]
    Dot,
    /// `|`.
    #[token("|")]
    Pipe,
    /// `$`.
    #[token("$")]
    Dollar,
}

/// Extracts the operator name from `|name|`.
fn plain_named_operator(lex: &mut logos::Lexer<'_, Token>) -> String {
    let slice = lex.slice();
    slice[1..slice.len() - 1].to_owned()
}

/// Extracts the operator name from `:|name|:`.
fn both_named_operator(lex: &mut logos::Lexer<'_, Token>) -> String {
    let slice = lex.slice();
    slice[2..slice.len() - 2].to_owned()
}

/// Extracts the operator name from `:|name|`.
fn left_named_operator(lex: &mut logos::Lexer<'_, Token>) -> String {
    let slice = lex.slice();
    slice[2..slice.len() - 1].to_owned()
}

/// Extracts the operator name from `|name|:`.
fn right_named_operator(lex: &mut logos::Lexer<'_, Token>) -> String {
    let slice = lex.slice();
    slice[1..slice.len() - 2].to_owned()
}

/// Splits a label token into its dotted path segments.
fn parse_label(lex: &mut logos::Lexer<'_, Token>) -> Vec<String> {
    let slice = lex.slice();
    slice[2..slice.len() - 2]
        .split('.')
        .map(ToOwned::to_owned)
        .collect()
}

/// Removes quotes from a quoted name token.
fn parse_quoted_name(lex: &mut logos::Lexer<'_, Token>) -> String {
    let slice = lex.slice();
    slice[1..slice.len() - 1].to_owned()
}

/// Returns the raw name token text.
fn parse_name(lex: &mut logos::Lexer<'_, Token>) -> String {
    lex.slice().to_owned()
}

/// Removes the trailing underscore from a placeholder token.
fn parse_placeholder(lex: &mut logos::Lexer<'_, Token>) -> String {
    let slice = lex.slice();
    slice[..slice.len() - 1].to_owned()
}

/// Removes the trailing double underscore from a magnetic placeholder token.
fn parse_magnetic_placeholder(lex: &mut logos::Lexer<'_, Token>) -> String {
    let slice = lex.slice();
    slice[..slice.len() - 2].to_owned()
}

/// Returns the raw special-operator token text.
fn parse_special_operator(lex: &mut logos::Lexer<'_, Token>) -> String {
    lex.slice().to_owned()
}

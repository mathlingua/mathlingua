use std::fmt;

use logos::Logos;

#[derive(Default, Clone, Debug, PartialEq, Eq)]
pub enum LexicalError {
    #[default]
    InvalidToken,
}

impl fmt::Display for LexicalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidToken => write!(f, "invalid token"),
        }
    }
}

#[derive(Logos, Clone, Debug, PartialEq, Eq)]
#[logos(skip r"[ \t\r\n\f]+", error = LexicalError)]
pub enum Token {
    #[token("(.")]
    DotLParen,
    #[token(".)")]
    DotRParen,
    #[token("...")]
    Ellipsis,
    #[token("[|")]
    LNamedArguments,
    #[token("|]")]
    RNamedArguments,
    #[token("\\.")]
    InfixCommandStart,
    #[token("./")]
    InfixCommandEnd,
    #[token("\\:")]
    InfixSpecStart,
    #[token("?:/")]
    InfixSpecPredicateEnd,
    #[token(":/")]
    InfixSpecEnd,
    #[regex(
        r":\|[A-Za-z0-9]+(?:[A-Za-z0-9_]*[A-Za-z0-9]+)?\|:",
        both_named_operator
    )]
    BothNamedOperator(String),
    #[regex(
        r":\|[A-Za-z0-9]+(?:[A-Za-z0-9_]*[A-Za-z0-9]+)?\|",
        left_named_operator
    )]
    LeftNamedOperator(String),
    #[regex(
        r"\|[A-Za-z0-9]+(?:[A-Za-z0-9_]*[A-Za-z0-9]+)?\|:",
        right_named_operator
    )]
    RightNamedOperator(String),
    #[regex(
        r"\|[A-Za-z0-9]+(?:[A-Za-z0-9_]*[A-Za-z0-9]+)?\|",
        plain_named_operator
    )]
    NamedOperator(String),
    #[regex(
        r"[A-Za-z0-9]+(?:[A-Za-z0-9_]*[A-Za-z0-9]+)?\|[ \t\r\n\f]+",
        prefix_named_operator
    )]
    PrefixNamedOperator(String),
    #[regex(
        r"\|[A-Za-z0-9]+(?:[A-Za-z0-9_]*[A-Za-z0-9]+)?",
        postfix_named_operator
    )]
    PostfixNamedOperator(String),
    #[regex(r"\[:[A-Za-z0-9]+(?:[A-Za-z0-9_]*[A-Za-z0-9]+)?(?:\.[A-Za-z0-9]+(?:[A-Za-z0-9_]*[A-Za-z0-9]+)?)*:\]", parse_label)]
    Label(Vec<String>),
    #[token(":=>")]
    ExpressionAlias,
    #[token(":->")]
    SpecOperatorAlias,
    #[token(":~>")]
    WritingAlias,
    #[token("::=")]
    Introduce,
    #[token(":=")]
    Declare,
    #[token("is_not?")]
    IsNotPredicate,
    #[token("is?")]
    IsPredicate,
    #[token("is")]
    Is,
    #[token("via")]
    Via,
    #[regex(r#""[A-Za-z0-9]+(?:[A-Za-z0-9_]*[A-Za-z0-9]+)?""#, parse_quoted_name)]
    QuotedName(String),
    #[regex(
        r"[A-Za-z0-9]+(?:[A-Za-z0-9_]*[A-Za-z0-9]+)?__",
        parse_magnetic_placeholder
    )]
    MagneticPlaceholder(String),
    #[regex(r"[A-Za-z0-9]+(?:[A-Za-z0-9_]*[A-Za-z0-9]+)?_", parse_placeholder)]
    Placeholder(String),
    #[regex(
        r"`[-~!#%^&*\\+=|<>/]+`|[A-Za-z0-9]+(?:[A-Za-z0-9_]*[A-Za-z0-9]+)?",
        parse_name
    )]
    Name(String),
    #[regex(r"(?:[-~!#%^&*+=|<>/]{2,}|[~!#%&<>])", parse_special_operator)]
    SpecialOperator(String),
    #[token("\\")]
    CommandStart,
    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("*")]
    Star,
    #[token("/")]
    Slash,
    #[token("=")]
    Equals,
    #[token("^")]
    Caret,
    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token("{")]
    LBrace,
    #[token("}")]
    RBrace,
    #[token("[")]
    LBracket,
    #[token("]")]
    RBracket,
    #[token(",")]
    Comma,
    #[token(":")]
    Colon,
    #[token(".")]
    Dot,
    #[token("|")]
    Pipe,
    #[token("$")]
    Dollar,
    #[token("?")]
    Question,
}

fn plain_named_operator(lex: &mut logos::Lexer<'_, Token>) -> String {
    let slice = lex.slice();
    slice[1..slice.len() - 1].to_owned()
}

fn both_named_operator(lex: &mut logos::Lexer<'_, Token>) -> String {
    let slice = lex.slice();
    slice[2..slice.len() - 2].to_owned()
}

fn left_named_operator(lex: &mut logos::Lexer<'_, Token>) -> String {
    let slice = lex.slice();
    slice[2..slice.len() - 1].to_owned()
}

fn right_named_operator(lex: &mut logos::Lexer<'_, Token>) -> String {
    let slice = lex.slice();
    slice[1..slice.len() - 2].to_owned()
}

fn prefix_named_operator(lex: &mut logos::Lexer<'_, Token>) -> String {
    let slice = lex.slice();
    let end = slice.find('|').unwrap_or(slice.len());
    slice[..end].to_owned()
}

fn postfix_named_operator(lex: &mut logos::Lexer<'_, Token>) -> String {
    let slice = lex.slice();
    slice[1..].to_owned()
}

fn parse_label(lex: &mut logos::Lexer<'_, Token>) -> Vec<String> {
    let slice = lex.slice();
    slice[2..slice.len() - 2]
        .split('.')
        .map(ToOwned::to_owned)
        .collect()
}

fn parse_quoted_name(lex: &mut logos::Lexer<'_, Token>) -> String {
    let slice = lex.slice();
    slice[1..slice.len() - 1].to_owned()
}

fn parse_name(lex: &mut logos::Lexer<'_, Token>) -> String {
    lex.slice().to_owned()
}

fn parse_placeholder(lex: &mut logos::Lexer<'_, Token>) -> String {
    let slice = lex.slice();
    slice[..slice.len() - 1].to_owned()
}

fn parse_magnetic_placeholder(lex: &mut logos::Lexer<'_, Token>) -> String {
    let slice = lex.slice();
    slice[..slice.len() - 2].to_owned()
}

fn parse_special_operator(lex: &mut logos::Lexer<'_, Token>) -> String {
    lex.slice().to_owned()
}

use logos::{Logos, SpannedIter};

use super::token::{LexicalError, Token};

pub type Spanned<Tok, Loc, Error> = Result<(Loc, Tok, Loc), Error>;

pub struct Lexer<'input> {
    tokens: SpannedIter<'input, Token>,
}

impl<'input> Lexer<'input> {
    pub fn new(input: &'input str) -> Self {
        Self {
            tokens: Token::lexer(input).spanned(),
        }
    }
}

impl<'input> Iterator for Lexer<'input> {
    type Item = Spanned<Token, usize, LexicalError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.tokens
            .next()
            .map(|(token, span)| token.map(|token| (span.start, token, span.end)))
    }
}

// ===============================[ tests ]=====================================

#[cfg(test)]
mod tests {
    use super::Lexer;
    use crate::frontend::formulation::token::{LexicalError, Token};

    #[test]
    fn lexes_placeholders_names_and_operators() {
        let tokens: Vec<_> = Lexer::new("f(x_, y__) + |op| neg| |prime")
            .map(|item| item.expect("expected valid token").1)
            .collect();

        assert_eq!(
            tokens,
            vec![
                Token::Name("f".to_string()),
                Token::LParen,
                Token::Placeholder("x".to_string()),
                Token::Comma,
                Token::MagneticPlaceholder("y".to_string()),
                Token::RParen,
                Token::Plus,
                Token::NamedOperator("op".to_string()),
                Token::Name("neg".to_string()),
                Token::Pipe,
                Token::Pipe,
                Token::Name("prime".to_string()),
            ]
        );
    }

    #[test]
    fn lexes_command_related_tokens() {
        let tokens: Vec<_> = Lexer::new(r#"\function:on{A}:to{B}(x)"#)
            .map(|item| item.expect("expected valid token").1)
            .collect();

        assert_eq!(
            tokens,
            vec![
                Token::CommandStart,
                Token::Name("function".to_string()),
                Token::Colon,
                Token::Name("on".to_string()),
                Token::LBrace,
                Token::Name("A".to_string()),
                Token::RBrace,
                Token::Colon,
                Token::Name("to".to_string()),
                Token::LBrace,
                Token::Name("B".to_string()),
                Token::RBrace,
                Token::LParen,
                Token::Name("x".to_string()),
                Token::RParen,
            ]
        );
    }

    #[test]
    fn lexes_labels_and_quoted_names() {
        let tokens: Vec<_> = Lexer::new(r#"(x)[:some.label:] "in" X"#)
            .map(|item| item.expect("expected valid token").1)
            .collect();

        assert_eq!(
            tokens,
            vec![
                Token::LParen,
                Token::Name("x".to_string()),
                Token::RParen,
                Token::Label(vec!["some".to_string(), "label".to_string()]),
                Token::QuotedName("in".to_string()),
                Token::Name("X".to_string()),
            ]
        );
    }

    #[test]
    fn lexes_alias_tokens_named_argument_delimiters_and_predicates() {
        let tokens: Vec<_> = Lexer::new(
            r#"(.x.) [| |] :|left| |mid| |right|: :|both|: := :=> :-> :~> is via is? is_not? \: :/"#,
        )
        .map(|item| item.expect("expected valid token").1)
        .collect();

        assert_eq!(
            tokens,
            vec![
                Token::DotLParen,
                Token::Name("x".to_string()),
                Token::DotRParen,
                Token::LNamedArguments,
                Token::RNamedArguments,
                Token::LeftNamedOperator("left".to_string()),
                Token::NamedOperator("mid".to_string()),
                Token::RightNamedOperator("right".to_string()),
                Token::BothNamedOperator("both".to_string()),
                Token::Declare,
                Token::ExpressionAlias,
                Token::SpecOperatorAlias,
                Token::WritingAlias,
                Token::Is,
                Token::Via,
                Token::IsPredicate,
                Token::IsNotPredicate,
                Token::InfixCommandStart,
                Token::InfixCommandEnd,
            ]
        );
    }

    #[test]
    fn lexes_stropped_operator_names() {
        let tokens: Vec<_> = Lexer::new(r#"`*` `*+`"#)
            .map(|item| item.expect("expected valid token").1)
            .collect();

        assert_eq!(
            tokens,
            vec![
                Token::Name("`*`".to_string()),
                Token::Name("`*+`".to_string()),
            ]
        );
    }

    #[test]
    fn rejects_invalid_stropped_names() {
        let mut lexer = Lexer::new(r#"`some.thing`"#);

        let token = lexer.next().expect("expected one token result");

        assert_eq!(token, Err(LexicalError::InvalidToken));
    }

    #[test]
    fn reports_invalid_tokens() {
        let mut lexer = Lexer::new("?");

        let token = lexer.next().expect("expected one token result");

        assert_eq!(token, Err(LexicalError::InvalidToken));
        assert!(lexer.next().is_none());
    }
}

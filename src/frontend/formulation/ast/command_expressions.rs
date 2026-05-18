use super::*;

/// Dotted chain used for commands, aliases, and named paths.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Chain {
    /// Source span covered by the chain.
    pub span: Span,
    /// Ordered chain parts.
    pub parts: Vec<ChainPart>,
}

/// One part of a command/name chain.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ChainPart {
    /// Ordinary name part.
    Name(String),
    /// Alias part prefixed by `$`.
    Alias(String),
    /// Operator part.
    Operator(String),
}

/// Curly argument group supplied to a command expression.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CurlyExpressionArgs {
    /// Source span covered by the argument group.
    pub span: Span,
    /// Expressions inside the group.
    pub expressions: Vec<Expression>,
}

/// Parenthesized argument group supplied to a command expression.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParenExpressionArgs {
    /// Source span covered by the argument group.
    pub span: Span,
    /// Expressions inside the group.
    pub expressions: Vec<Expression>,
}

/// Tail segment of a command expression, such as `:to{B}`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommandExpressionTailPart {
    /// Source span covered by the tail part.
    pub span: Span,
    /// Tail label chain.
    pub chain: Chain,
    /// Curly argument groups attached to the tail.
    pub args: Vec<CurlyExpressionArgs>,
}

/// Prefix command expression.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommandExpression {
    /// Source span covered by the command expression.
    pub span: Span,
    /// Head command chain.
    pub chain: Chain,
    /// Curly argument groups after the head.
    pub head_args: Vec<CurlyExpressionArgs>,
    /// Tail segments after the head.
    pub tail: Vec<CommandExpressionTailPart>,
    /// Optional parenthesized invocation groups.
    pub paren_args: Vec<ParenExpressionArgs>,
}

/// Infix command expression between two operands.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InfixCommand {
    /// Source span covered by the infix command token.
    pub span: Span,
    /// Infix command chain.
    pub chain: Chain,
    /// Head curly argument groups.
    pub head_args: Vec<CurlyExpressionArgs>,
    /// Tail segments.
    pub tail: Vec<CommandExpressionTailPart>,
}

/// Type expression used on the right-hand side of `is`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TypeExpression {
    /// Normal command type.
    Command(CommandExpression),
    /// Refined command type.
    RefinedCommand(RefinedCommandExpression),
}

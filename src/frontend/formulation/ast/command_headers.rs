use super::*;

/// Curly argument group supplied in a command header.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CurlyHeadingArgs {
    /// Source span covered by the argument group.
    pub span: Span,
    /// Forms inside the group.
    pub forms: Vec<FormOrDeclaration>,
}

/// Parenthesized argument group supplied in a command header.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParenHeadingArgs {
    /// Source span covered by the argument group.
    pub span: Span,
    /// Forms inside the group.
    pub forms: Vec<FormOrDeclaration>,
}

/// Tail segment of a command header, such as `:to{B}`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommandHeaderTailPart {
    /// Source span covered by the tail segment.
    pub span: Span,
    /// Tail label chain.
    pub chain: Chain,
    /// Curly argument groups attached to the tail.
    pub args: Vec<CurlyHeadingArgs>,
}

/// Normal prefix command header.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommandHeaderNode {
    /// Source span covered by the command header.
    pub span: Span,
    /// Head command chain.
    pub chain: Chain,
    /// Curly argument groups after the head.
    pub head_args: Vec<CurlyHeadingArgs>,
    /// Tail segments after the head.
    pub tail: Vec<CommandHeaderTailPart>,
    /// Parenthesized argument groups after the command.
    pub paren_args: Vec<ParenHeadingArgs>,
}

/// Infix command header.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InfixCommandHeader {
    /// Source span covered by the infix command header.
    pub span: Span,
    /// Infix command chain.
    pub chain: Chain,
    /// Curly argument groups after the head.
    pub head_args: Vec<CurlyHeadingArgs>,
    /// Tail segments.
    pub tail: Vec<CommandHeaderTailPart>,
}

/// Refinement part inside a refined command header.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RefinedHeaderPart {
    /// Source span covered by the refinement part.
    pub span: Span,
    /// Refinement chain.
    pub chain: Chain,
    /// Tail segments attached to this refinement.
    pub tail: Vec<CommandHeaderTailPart>,
}

/// Base tail of a refined command.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RefinedTail {
    /// Tail represented by a normal chain.
    Chain(Chain),
    /// Tail represented by a raw name.
    Name { span: Span, name: String },
}

/// Refined command header, such as `\(continuous)::function:on{A}:to{B}`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RefinedCommandHeader {
    /// Source span covered by the header.
    pub span: Span,
    /// Optional prefix chain before the refinement parts.
    pub prefix_chain: Option<Chain>,
    /// Refinement parts.
    pub parts: Vec<RefinedHeaderPart>,
    /// Base command being refined.
    pub refined_tail: RefinedTail,
    /// Curly argument groups attached to the base command head.
    pub head_args: Vec<CurlyHeadingArgs>,
    /// Tail segments attached to the base command.
    pub tail: Vec<CommandHeaderTailPart>,
    /// Parenthesized argument groups attached to the base command.
    pub paren_args: Vec<ParenHeadingArgs>,
}

/// Any supported command heading form.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CommandHeader {
    /// Normal prefix command header.
    Command(CommandHeaderNode),
    /// Infix command header.
    Infix(InfixCommandHeader),
    /// Refined command header.
    Refined(RefinedCommandHeader),
}

/// Refinement part inside a refined command expression.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RefinedExpressionPart {
    /// Source span covered by the refinement part.
    pub span: Span,
    /// Refinement chain.
    pub chain: Chain,
    /// Tail segments attached to this refinement.
    pub tail: Vec<CommandExpressionTailPart>,
}

/// Refined command expression.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RefinedCommandExpression {
    /// Source span covered by the expression.
    pub span: Span,
    /// Optional prefix chain before the refinement parts.
    pub prefix_chain: Option<Chain>,
    /// Refinement parts.
    pub parts: Vec<RefinedExpressionPart>,
    /// Base command being refined.
    pub refined_tail: RefinedTail,
    /// Curly argument groups attached to the base command head.
    pub head_args: Vec<CurlyExpressionArgs>,
    /// Tail segments attached to the base command.
    pub tail: Vec<CommandExpressionTailPart>,
    /// Parenthesized invocation groups attached to the base command.
    pub paren_args: Vec<ParenExpressionArgs>,
}

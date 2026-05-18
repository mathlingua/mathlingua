use super::*;

/// Writing alias mapping a form to a textual rendering body.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WritingAlias {
    /// Source span covered by the alias.
    pub span: Span,
    /// Alias left-hand form.
    pub form: FormOrDeclaration,
    /// Alias body.
    pub body: String,
}

/// Left-hand side of an expression alias.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExpressionAliasLhs {
    /// Form alias target.
    Form(FormOrDeclaration),
    /// Prefix command alias target.
    Command(CommandHeaderNode),
    /// Infix command alias target.
    InfixCommand(InfixCommandHeader),
}

/// Expression alias mapping a form or command header to an expression.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExpressionAlias {
    /// Source span covered by the alias.
    pub span: Span,
    /// Alias left-hand side.
    pub lhs: ExpressionAliasLhs,
    /// Alias target expression.
    pub expression: Expression,
}

/// Spec-operator alias mapping a placeholder spec to a target spec.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpecOperatorAlias {
    /// Source span covered by the alias.
    pub span: Span,
    /// Placeholder-side spec.
    pub placeholder_spec: PlaceholderSpecStatement,
    /// Target spec.
    pub target: IsOrSpec,
}

/// Parsed label header.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LabelHeader {
    /// Source span covered by the header.
    pub span: Span,
    /// Header path parts.
    pub parts: Vec<String>,
}

/// Parsed author header.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AuthorHeader {
    /// Source span covered by the header.
    pub span: Span,
    /// Header path parts.
    pub parts: Vec<String>,
}

/// Parsed resource header.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResourceHeader {
    /// Source span covered by the header.
    pub span: Span,
    /// Header path parts.
    pub parts: Vec<String>,
}

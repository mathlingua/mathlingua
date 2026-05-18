/// Operator token with source span.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Operator {
    /// Source span covered by the operator.
    pub span: Span,
    /// Operator text.
    pub text: String,
}

impl Operator {
    /// Creates an operator from a span and text.
    pub fn new(span: Span, text: impl Into<String>) -> Self {
        Self {
            span,
            text: text.into(),
        }
    }
}

/// Unary operator categories.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UnaryOperator {
    /// Arithmetic prefix operator.
    Arithmetic(Operator),
}

impl UnaryOperator {
    /// Returns the source span of the wrapped operator token.
    pub fn span(&self) -> Span {
        match self {
            Self::Arithmetic(operator) => operator.span,
        }
    }
}

/// Binary operator categories ordered by the grammar's precedence rules.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BinaryOperator {
    /// Equality operator.
    Equality(Operator),
    /// Special symbolic operator.
    Special(Operator),
    /// Addition operator.
    Add(Operator),
    /// Subtraction operator.
    Subtract(Operator),
    /// Multiplication operator.
    Multiply(Operator),
    /// Division operator.
    Divide(Operator),
    /// Power operator.
    Power(Operator),
    /// Named operator.
    Named(NamedOperator),
}

/// High-precedence infix item parsed before lower-precedence expression folding.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HighPrecedenceInfix {
    /// Built-in binary operator.
    Binary(BinaryOperator),
    /// User command used infix.
    Command(InfixCommand),
}

/// Named operator token and its binding style.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NamedOperator {
    /// Source span covered by the operator.
    pub span: Span,
    /// Operator name without delimiters.
    pub name: String,
    /// Binding style indicated by colon delimiters.
    pub kind: NamedOperatorKind,
}

/// Binding style for a named operator.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NamedOperatorKind {
    /// No colon markers.
    Plain,
    /// Left colon marker.
    LeftColon,
    /// Right colon marker.
    RightColon,
    /// Both left and right colon markers.
    BothColon,
}


use super::*;

/// Root AST node for a mathematical expression formulation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Expression {
    /// Source span covered by the expression.
    pub span: Span,
    /// Specific expression form.
    pub kind: ExpressionKind,
}

impl Expression {
    /// Creates an expression from a source span and kind.
    pub fn new(span: Span, kind: ExpressionKind) -> Self {
        Self { span, kind }
    }
}

/// Specific forms an expression can take in MathLingua formulations.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExpressionKind {
    /// Bare identifier.
    Name(String),
    /// Positional function call, such as `f(x, y)`.
    FunctionCall {
        /// Function name.
        name: String,
        /// Positional call arguments.
        arguments: Vec<Expression>,
    },
    /// Named function call using `[|name: value|]` style elements.
    FunctionNamedCall {
        /// Function name.
        name: String,
        /// Named call elements.
        elements: Vec<FunctionNamedExpressionElement>,
    },
    /// Tuple expression.
    Tuple(Vec<TupleExpressionElement>),
    /// Set-builder expression.
    Set(SetExpression),
    /// Grouped expression.
    Grouped {
        /// Inner expression.
        expression: Box<Expression>,
        /// Whether the group used dot delimiters `(.` and `.)`.
        dot_delimited: bool,
    },
    /// Labeled expression.
    Labeled {
        /// Inner expression.
        expression: Box<Expression>,
        /// Attached label.
        label: Label,
    },
    /// Subset/index call expression.
    SubsetCall(SubsetCall),
    /// Prefix command expression.
    Command(CommandExpression),
    /// Infix command expression.
    InfixCommand {
        /// Left operand.
        left: Box<Expression>,
        /// Infix command.
        command: InfixCommand,
        /// Right operand.
        right: Box<Expression>,
    },
    /// Unary prefix expression.
    Prefix {
        /// Prefix operator.
        operator: UnaryOperator,
        /// Operand.
        expression: Box<Expression>,
    },
    /// Binary expression.
    Binary {
        /// Left operand.
        left: Box<Expression>,
        /// Binary operator.
        operator: BinaryOperator,
        /// Right operand.
        right: Box<Expression>,
    },
    /// Inline specification statement.
    SpecStatement(SpecStatement),
    /// Predicate form `subject is? command`.
    IsPredicate {
        /// Predicate subject.
        subject: Box<Expression>,
        /// Predicate command.
        command: CommandExpression,
    },
    /// Predicate form `subject is_not? command`.
    IsNotPredicate {
        /// Predicate subject.
        subject: Box<Expression>,
        /// Predicate command.
        command: CommandExpression,
    },
    /// Type assertion expression.
    IsType {
        /// Subject expression.
        subject: Box<Expression>,
        /// Type expression.
        ty: TypeExpression,
    },
}

/// Element inside a tuple expression.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TupleExpressionElement {
    /// Ordinary expression element.
    Expression(Expression),
    /// Operator element such as `*` in `(X, *, e)`.
    Operator(Operator),
}

/// Set-builder expression with target, specification, and optional predicate.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SetExpression {
    /// Source span covered by the set expression.
    pub span: Span,
    /// Type/specification attached to the target placeholder.
    pub spec: SpecStatement,
    /// Placeholder or function placeholder introduced by the set.
    pub target: PlaceholderForm,
    /// Optional predicate after `|`.
    pub predicate: Option<Box<Expression>>,
}

/// Named argument element inside a function call.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FunctionNamedExpressionElement {
    /// Source span covered by this named element.
    pub span: Span,
    /// Left-hand name or subset call.
    pub lhs: FunctionNamedExpressionElementLhs,
    /// Expression supplied for the named element.
    pub expression: Expression,
}

/// Left-hand side of a named function-call element.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FunctionNamedExpressionElementLhs {
    /// Simple named element.
    Name(String),
    /// Subset/indexed named element.
    SubsetCall(SubsetCall),
}

/// Dotted label attached to an expression.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Label {
    /// Source span covered by the label.
    pub span: Span,
    /// Label path components.
    pub parts: Vec<String>,
}

/// Subset/index call forms.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SubsetCall {
    /// One-level call, such as `A[i]`.
    One {
        /// Source span covered by the call.
        span: Span,
        /// Target being indexed.
        target: String,
        /// First index/subset name.
        first: String,
    },
    /// Two-argument call, such as `A[i, j]`.
    Two {
        /// Source span covered by the call.
        span: Span,
        /// Target being indexed.
        target: String,
        /// First index/subset name.
        first: String,
        /// Second index/subset name.
        second: String,
    },
    /// Nested call, such as `A[B[i]]`.
    Nested {
        /// Source span covered by the call.
        span: Span,
        /// Outer target being indexed.
        target: String,
        /// Outer subset/index name.
        outer: String,
        /// Inner target name.
        inner_target: String,
    },
}

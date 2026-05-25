//! Strongly typed AST produced by the formulation parser.

/// Byte span of a parsed formulation token or AST node.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Span {
    /// Inclusive start byte offset.
    pub start: usize,
    /// Exclusive end byte offset.
    pub end: usize,
}

impl Span {
    /// Creates a span from start and end byte offsets.
    pub const fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }
}

// ===============================[ aliases ]=====================================

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

/// Right-hand side accepted by a specification-operator alias.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SpecOperatorAliasTarget {
    /// An ordinary `is` statement or subject specification.
    IsOrSpec(Box<IsOrSpec>),
    /// A built-in keyword such as `\\abstract`.
    Builtin(Chain),
}

/// Spec-operator alias mapping a placeholder spec to a target spec.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpecOperatorAlias {
    /// Source span covered by the alias.
    pub span: Span,
    /// Placeholder-side spec.
    pub placeholder_spec: PlaceholderSpecStatement,
    /// Target spec.
    pub target: SpecOperatorAliasTarget,
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

// ===============================[ command expressions ]=====================================

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
    /// Function type from input specs to one output spec.
    Function(FunctionType),
}

/// Function type expression such as `(_ "in" A) => (_ "in" B)`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FunctionType {
    /// Source span covered by the function type.
    pub span: Span,
    /// Input specifications accepted by the function.
    pub inputs: Vec<FunctionTypeSpec>,
    /// Output specification produced by the function.
    pub output: FunctionTypeSpec,
}

/// One spec inside a function type.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FunctionTypeSpec {
    /// Source span covered by the spec.
    pub span: Span,
    /// Parameter text. The semantic checker requires this to be `_`.
    pub subject: String,
    /// Spec shape attached to the parameter.
    pub kind: FunctionTypeSpecKind,
}

/// Function type spec variants.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FunctionTypeSpecKind {
    /// Type spec of the form `_ is \type`.
    Is(Box<TypeExpression>),
    /// Quoted-operator spec of the form `_ "operator" Target`.
    Spec {
        /// Quoted operator text.
        operator: String,
        /// Right-hand target name.
        target: String,
    },
}

// ===============================[ command headers ]=====================================

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

// ===============================[ expression ]=====================================

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

// ===============================[ forms ]=====================================

/// Form or declaration used in definitions, command headers, and subjects.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FormOrDeclaration {
    /// Source span covered by the form.
    pub span: Span,
    /// Specific form/declaration shape.
    pub kind: FormOrDeclarationKind,
}

impl FormOrDeclaration {
    /// Creates a form or declaration from a span and kind.
    pub fn new(span: Span, kind: FormOrDeclarationKind) -> Self {
        Self { span, kind }
    }
}

/// Specific forms and declarations supported by formulation syntax.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FormOrDeclarationKind {
    /// Simple named form.
    Name(String),
    /// Function declaration, optionally binding a display name.
    FunctionDeclaration {
        /// Optional external name for the function declaration.
        name: Option<String>,
        /// Function form.
        form: FunctionForm,
    },
    /// Tuple declaration, optionally binding a name.
    TupleDeclaration {
        /// Optional tuple name.
        name: Option<String>,
        /// Tuple form.
        form: TupleForm,
    },
    /// Set declaration, optionally binding a name.
    SetDeclaration {
        /// Optional set name.
        name: Option<String>,
        /// Set form.
        form: SetForm,
    },
    /// Infix operator declaration.
    InfixOperator {
        /// Left placeholder.
        left: Placeholder,
        /// Operator being declared.
        operator: Operator,
        /// Right placeholder.
        right: Placeholder,
    },
    /// Prefix operator declaration.
    PrefixOperator {
        /// Operator being declared.
        operator: Operator,
        /// Operand placeholder.
        placeholder: Placeholder,
    },
    /// Postfix operator declaration.
    PostfixOperator {
        /// Operand placeholder.
        placeholder: Placeholder,
        /// Operator being declared.
        operator: Operator,
    },
}

/// Function form declaration, such as `f(x_)` or `f(x__)`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FunctionForm {
    /// Source span covered by the function form.
    pub span: Span,
    /// Function form name.
    pub name: String,
    /// Optional magnetic placeholder.
    pub magnetic_placeholder: Option<MagneticPlaceholder>,
    /// Ordinary placeholders.
    pub placeholders: Vec<Placeholder>,
}

/// Tuple form declaration.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TupleForm {
    /// Source span covered by the tuple form.
    pub span: Span,
    /// Tuple elements.
    pub elements: Vec<TupleFormElement>,
}

/// Element inside a tuple form declaration.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TupleFormElement {
    /// Nested form element.
    Form(FormOrDeclaration),
    /// Operator element.
    Operator(Operator),
}

/// Set form declaration.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SetForm {
    /// Source span covered by the set form.
    pub span: Span,
    /// Placeholder form introduced by the set.
    pub placeholder_form: PlaceholderForm,
}

/// Placeholder variable ending with `_` in source.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Placeholder {
    /// Source span covered by the placeholder.
    pub span: Span,
    /// Placeholder name without the trailing underscore.
    pub name: String,
}

impl Placeholder {
    /// Creates a placeholder from a span and name.
    pub fn new(span: Span, name: impl Into<String>) -> Self {
        Self {
            span,
            name: name.into(),
        }
    }
}

/// Magnetic placeholder variable ending with `__` in source.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MagneticPlaceholder {
    /// Source span covered by the placeholder.
    pub span: Span,
    /// Placeholder name without the trailing double underscore.
    pub name: String,
}

/// Placeholder form used in subjects and set-builder targets.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlaceholderForm {
    /// Source span covered by the placeholder form.
    pub span: Span,
    /// Specific placeholder form shape.
    pub kind: PlaceholderFormKind,
}

impl PlaceholderForm {
    /// Creates a placeholder form from a span and kind.
    pub fn new(span: Span, kind: PlaceholderFormKind) -> Self {
        Self { span, kind }
    }
}

/// Specific placeholder form shapes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PlaceholderFormKind {
    /// Single placeholder.
    Placeholder(Placeholder),
    /// Function placeholder, such as `f(x_)`.
    Function {
        /// Placeholder naming the function.
        placeholder: Placeholder,
        /// Function argument placeholders.
        arguments: Vec<Placeholder>,
    },
}

// ===============================[ operators ]=====================================

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

// ===============================[ statements ]=====================================

/// Statement of the form `subject "operator" name`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpecStatement {
    /// Source span covered by the statement.
    pub span: Span,
    /// Subject expression.
    pub subject: Box<Expression>,
    /// Quoted operator text.
    pub operator: String,
    /// Name on the right-hand side.
    pub name: String,
}

/// Subject of a specification statement.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpecSubject {
    /// Source span covered by the subject.
    pub span: Span,
    /// Subject shape.
    pub kind: SpecSubjectKind,
}

/// Specific specification subject shapes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SpecSubjectKind {
    /// Form subject.
    Form(FormOrDeclaration),
    /// Operator subject.
    Operator(Operator),
}

/// Subject of an `is` statement.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IsSubject {
    /// Source span covered by the subject.
    pub span: Span,
    /// Subject shape.
    pub kind: IsSubjectKind,
}

/// One form inside an `is` subject.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IsSubjectForm {
    /// Ordinary form/declaration.
    Form(FormOrDeclaration),
    /// Placeholder form.
    PlaceholderForm(PlaceholderForm),
}

/// Specific `is` subject shapes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IsSubjectKind {
    /// One or more forms.
    Forms(Vec<IsSubjectForm>),
    /// Operator subject.
    Operator(Operator),
}

/// Statement of the form `subject is type`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IsStatement {
    /// Source span covered by the statement.
    pub span: Span,
    /// Subject being typed.
    pub subject: IsSubject,
    /// Type expression.
    pub ty: TypeExpression,
}

/// Specification statement whose subject is a form/operator.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SubjectSpecStatement {
    /// Source span covered by the statement.
    pub span: Span,
    /// Subject being specified.
    pub subject: SpecSubject,
    /// Quoted operator text.
    pub operator: String,
    /// Right-hand name.
    pub name: String,
}

/// Either an `is` statement or a specification statement.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IsOrSpec {
    /// `is` statement.
    Is(IsStatement),
    /// Specification statement.
    Spec(SubjectSpecStatement),
}

/// Either an `is` statement with refined-command support or a specification.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IsOrRefinedStatementSpec {
    /// `is` statement.
    Is(IsStatement),
    /// Specification statement.
    Spec(SubjectSpecStatement),
}

/// `is ... via ...` statement used by structural sections.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IsViaStatement {
    /// Source span covered by the statement.
    pub span: Span,
    /// Main `is` statement.
    pub is_statement: IsStatement,
    /// Form used to view the subject as the extended type.
    pub via: FormOrDeclaration,
}

/// Specification statement whose subject is a placeholder form.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlaceholderSpecStatement {
    /// Source span covered by the statement.
    pub span: Span,
    /// Placeholder form being specified.
    pub placeholder_form: PlaceholderForm,
    /// Quoted operator text.
    pub operator: String,
    /// Right-hand name.
    pub name: String,
}

/// Local syntactic equality binding used by semantic checks.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExpressionBinding {
    /// Source span covered by the binding.
    pub span: Span,
    /// Left-hand expression.
    pub left: Expression,
    /// Right-hand expression.
    pub right: Expression,
}

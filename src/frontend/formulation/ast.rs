#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub const fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }
}

// ===============================[ aliases ]=====================================

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WritingAlias {
    pub span: Span,
    pub form: FormOrDeclaration,
    pub body: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExpressionAliasLhs {
    Form(FormOrDeclaration),
    Command(CommandHeaderNode),
    InfixCommand(InfixCommandHeader),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExpressionAlias {
    pub span: Span,
    pub lhs: ExpressionAliasLhs,
    pub expression: Expression,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SpecOperatorAliasTarget {
    IsOrSpec(Box<IsOrSpec>),
    Builtin(Chain),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpecOperatorAlias {
    pub span: Span,
    pub placeholder_spec: PlaceholderSpecStatement,
    pub target: SpecOperatorAliasTarget,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LabelHeader {
    pub span: Span,
    pub parts: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AuthorHeader {
    pub span: Span,
    pub parts: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResourceHeader {
    pub span: Span,
    pub parts: Vec<String>,
}

// ===============================[ command expressions ]=====================================

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Chain {
    pub span: Span,
    pub parts: Vec<ChainPart>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ChainPart {
    Name(String),
    Alias(String),
    Operator(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CurlyExpressionArgs {
    pub span: Span,
    pub expressions: Vec<Expression>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParenExpressionArgs {
    pub span: Span,
    pub expressions: Vec<Expression>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommandExpressionTailPart {
    pub span: Span,
    pub chain: Chain,
    pub args: Vec<CurlyExpressionArgs>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommandExpression {
    pub span: Span,
    pub chain: Chain,
    pub head_args: Vec<CurlyExpressionArgs>,
    pub tail: Vec<CommandExpressionTailPart>,
    pub paren_args: Vec<ParenExpressionArgs>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InfixCommand {
    pub span: Span,
    pub chain: Chain,
    pub head_args: Vec<CurlyExpressionArgs>,
    pub tail: Vec<CommandExpressionTailPart>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InfixSpec {
    pub span: Span,
    pub chain: Chain,
    pub head_args: Vec<CurlyExpressionArgs>,
    pub tail: Vec<CommandExpressionTailPart>,
    pub predicate: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TypeExpression {
    Builtin { span: Span, chain: Chain },
    Command(CommandExpression),
    RefinedCommand(RefinedCommandExpression),
    Function(FunctionType),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FunctionType {
    pub span: Span,
    pub inputs: Vec<FunctionTypeSpec>,
    pub output: FunctionTypeSpec,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FunctionTypeSpec {
    pub span: Span,
    pub subject: String,
    pub kind: FunctionTypeSpecKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FunctionTypeSpecKind {
    Is(Box<TypeExpression>),
    Spec { operator: String, target: String },
}

// ===============================[ command headers ]=====================================

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CurlyHeadingArgs {
    pub span: Span,
    pub forms: Vec<FormOrDeclaration>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParenHeadingArgs {
    pub span: Span,
    pub forms: Vec<FormOrDeclaration>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommandHeaderTailPart {
    pub span: Span,
    pub chain: Chain,
    pub args: Vec<CurlyHeadingArgs>,
    pub optional: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommandHeaderNode {
    pub span: Span,
    pub chain: Chain,
    pub head_args: Vec<CurlyHeadingArgs>,
    pub tail: Vec<CommandHeaderTailPart>,
    pub paren_args: Vec<ParenHeadingArgs>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InfixCommandHeader {
    pub span: Span,
    pub left: Option<FormOrDeclaration>,
    pub chain: Chain,
    pub head_args: Vec<CurlyHeadingArgs>,
    pub tail: Vec<CommandHeaderTailPart>,
    pub right: Option<FormOrDeclaration>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InfixSpecHeader {
    pub span: Span,
    pub left: FormOrDeclaration,
    pub chain: Chain,
    pub head_args: Vec<CurlyHeadingArgs>,
    pub tail: Vec<CommandHeaderTailPart>,
    pub right: FormOrDeclaration,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RefinedHeaderPart {
    pub span: Span,
    pub chain: Chain,
    pub tail: Vec<CommandHeaderTailPart>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RefinedTail {
    Chain(Chain),
    Name { span: Span, name: String },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RefinedCommandHeader {
    pub span: Span,
    pub prefix_chain: Option<Chain>,
    pub parts: Vec<RefinedHeaderPart>,
    pub refined_tail: RefinedTail,
    pub head_args: Vec<CurlyHeadingArgs>,
    pub tail: Vec<CommandHeaderTailPart>,
    pub paren_args: Vec<ParenHeadingArgs>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CommandHeader {
    Command(CommandHeaderNode),
    Infix(InfixCommandHeader),
    InfixSpec(InfixSpecHeader),
    Refined(RefinedCommandHeader),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RefinedExpressionPart {
    pub span: Span,
    pub chain: Chain,
    pub tail: Vec<CommandExpressionTailPart>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RefinedCommandExpression {
    pub span: Span,
    pub prefix_chain: Option<Chain>,
    pub parts: Vec<RefinedExpressionPart>,
    pub refined_tail: RefinedTail,
    pub head_args: Vec<CurlyExpressionArgs>,
    pub tail: Vec<CommandExpressionTailPart>,
    pub paren_args: Vec<ParenExpressionArgs>,
}

// ===============================[ expression ]=====================================

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Expression {
    pub span: Span,
    pub kind: ExpressionKind,
}

impl Expression {
    pub fn new(span: Span, kind: ExpressionKind) -> Self {
        Self { span, kind }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExpressionKind {
    Name(String),
    FunctionCall {
        name: String,
        arguments: Vec<Expression>,
    },
    FunctionNamedCall {
        name: String,
        elements: Vec<FunctionNamedExpressionElement>,
    },
    Tuple(Vec<TupleExpressionElement>),
    Set(SetExpression),
    Grouped {
        expression: Box<Expression>,
        dot_delimited: bool,
    },
    Labeled {
        expression: Box<Expression>,
        label: Label,
    },
    SubsetCall(SubsetCall),
    Command(CommandExpression),
    InfixCommand {
        left: Box<Expression>,
        command: InfixCommand,
        right: Box<Expression>,
    },
    InfixSpecStatement {
        left: Box<Expression>,
        spec: InfixSpec,
        right: Box<Expression>,
    },
    Prefix {
        operator: UnaryOperator,
        expression: Box<Expression>,
    },
    Postfix {
        expression: Box<Expression>,
        operator: Operator,
    },
    Binary {
        left: Box<Expression>,
        operator: BinaryOperator,
        right: Box<Expression>,
    },
    SpecStatement(SpecStatement),
    SpecPredicate(SpecStatement),
    IsPredicate {
        subject: Box<Expression>,
        command: CommandExpression,
    },
    IsNotPredicate {
        subject: Box<Expression>,
        command: CommandExpression,
    },
    IsType {
        subject: Box<Expression>,
        ty: TypeExpression,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TupleExpressionElement {
    Expression(Expression),
    Operator(Operator),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SetExpression {
    pub span: Span,
    pub target: SetTarget,
    pub specs: Vec<Expression>,
    pub predicate: Option<Box<Expression>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SetTarget {
    pub span: Span,
    pub kind: SetTargetKind,
}

impl SetTarget {
    pub fn new(span: Span, kind: SetTargetKind) -> Self {
        Self { span, kind }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SetTargetKind {
    Name(String),
    PlaceholderForm(PlaceholderForm),
    Function {
        name: String,
        arguments: Vec<SetTarget>,
    },
    Tuple(Vec<SetTargetElement>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SetTargetElement {
    Target(SetTarget),
    Operator(Operator),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FunctionNamedExpressionElement {
    pub span: Span,
    pub lhs: FunctionNamedExpressionElementLhs,
    pub expression: Expression,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FunctionNamedExpressionElementLhs {
    Name(String),
    SubsetCall(SubsetCall),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Label {
    pub span: Span,
    pub parts: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SubsetCall {
    One {
        span: Span,
        target: String,
        first: String,
    },
    Two {
        span: Span,
        target: String,
        first: String,
        second: String,
    },
    Nested {
        span: Span,
        target: String,
        outer: String,
        inner_target: String,
    },
}

// ===============================[ forms ]=====================================

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FormOrDeclaration {
    pub span: Span,
    pub kind: FormOrDeclarationKind,
}

impl FormOrDeclaration {
    pub fn new(span: Span, kind: FormOrDeclarationKind) -> Self {
        Self { span, kind }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FormOrDeclarationKind {
    Name(String),
    FunctionDeclaration {
        name: Option<String>,
        form: FunctionForm,
    },
    TupleDeclaration {
        name: Option<String>,
        form: TupleForm,
    },
    SetDeclaration {
        name: Option<String>,
        form: SetForm,
    },
    InfixOperator {
        left: Placeholder,
        operator: Operator,
        right: Placeholder,
    },
    PrefixOperator {
        operator: Operator,
        placeholder: Placeholder,
    },
    PostfixOperator {
        placeholder: Placeholder,
        operator: Operator,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FunctionForm {
    pub span: Span,
    pub name: String,
    pub magnetic_placeholder: Option<MagneticPlaceholder>,
    pub placeholders: Vec<Placeholder>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TupleForm {
    pub span: Span,
    pub elements: Vec<TupleFormElement>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TupleFormElement {
    Form(FormOrDeclaration),
    Operator(Operator),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SetForm {
    pub span: Span,
    pub placeholder_form: PlaceholderForm,
    pub has_condition_placeholder: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Placeholder {
    pub span: Span,
    pub name: String,
}

impl Placeholder {
    pub fn new(span: Span, name: impl Into<String>) -> Self {
        Self {
            span,
            name: name.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MagneticPlaceholder {
    pub span: Span,
    pub name: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlaceholderForm {
    pub span: Span,
    pub kind: PlaceholderFormKind,
}

impl PlaceholderForm {
    pub fn new(span: Span, kind: PlaceholderFormKind) -> Self {
        Self { span, kind }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PlaceholderFormKind {
    Placeholder(Placeholder),
    Function {
        placeholder: Placeholder,
        arguments: Vec<Placeholder>,
    },
}

// ===============================[ operators ]=====================================

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Operator {
    pub span: Span,
    pub text: String,
}

impl Operator {
    pub fn new(span: Span, text: impl Into<String>) -> Self {
        Self {
            span,
            text: text.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UnaryOperator {
    Arithmetic(Operator),
    Named(Operator),
}

impl UnaryOperator {
    pub fn span(&self) -> Span {
        match self {
            Self::Arithmetic(operator) | Self::Named(operator) => operator.span,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BinaryOperator {
    Equality(Operator),
    Special(Operator),
    Add(Operator),
    Subtract(Operator),
    Multiply(Operator),
    Divide(Operator),
    Power(Operator),
    Named(NamedOperator),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HighPrecedenceInfix {
    Binary(BinaryOperator),
    Command(InfixCommand),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NamedOperator {
    pub span: Span,
    pub name: String,
    pub kind: NamedOperatorKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NamedOperatorKind {
    Plain,
    LeftColon,
    RightColon,
    BothColon,
}

// ===============================[ statements ]=====================================

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpecStatement {
    pub span: Span,
    pub subject: Box<Expression>,
    pub operator: String,
    pub name: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpecSubject {
    pub span: Span,
    pub kind: SpecSubjectKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SpecSubjectKind {
    Form(FormOrDeclaration),
    Operator(Operator),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IsSubject {
    pub span: Span,
    pub kind: IsSubjectKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IsSubjectForm {
    Form(FormOrDeclaration),
    PlaceholderForm(PlaceholderForm),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IsSubjectKind {
    Forms(Vec<IsSubjectForm>),
    Operator(Operator),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IsStatement {
    pub span: Span,
    pub subject: IsSubject,
    pub ty: TypeExpression,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SubjectSpecStatement {
    pub span: Span,
    pub subject: SpecSubject,
    pub operator: String,
    pub name: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IsOrSpec {
    Is(IsStatement),
    Spec(SubjectSpecStatement),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IsOrRefinedStatementSpec {
    Is(IsStatement),
    Spec(SubjectSpecStatement),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeclarationStatement {
    pub span: Span,
    pub subject: IsSubject,
    pub expansion: Option<IsSubject>,
    pub definition: Option<Expression>,
    pub relation: Option<DeclarationRelation>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DeclarationRelation {
    Is(TypeExpression),
    Spec {
        operator: String,
        target: Box<Expression>,
    },
    InfixSpec {
        spec: InfixSpec,
        target: Box<Expression>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IsViaStatement {
    pub span: Span,
    pub is_statement: IsStatement,
    pub via: FormOrDeclaration,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlaceholderSpecStatement {
    pub span: Span,
    pub placeholder_form: PlaceholderForm,
    pub operator: String,
    pub name: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExpressionBinding {
    pub span: Span,
    pub left: Expression,
    pub right: Expression,
}

use super::*;

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
    /// Tuple form after `via`.
    pub tuple_form: TupleForm,
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

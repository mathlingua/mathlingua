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


use super::*;

pub(super) const BUILTIN_EXPRESSION_SIGNATURE: &str = "\\\\expression";
pub(super) const BUILTIN_STATEMENT_SIGNATURE: &str = "\\\\statement";
pub(super) const BUILTIN_SPECIFICATION_SIGNATURE: &str = "\\\\specification";
pub(super) const BUILTIN_TYPE_SIGNATURE: &str = "\\\\type";
pub(super) const BUILTIN_OPAQUE_SIGNATURE: &str = "\\\\opaque";

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct SignatureShape {
    pub(super) signature: String,
    pub(super) arg_groups: Vec<ArgGroupShape>,
    pub(super) fallback_shapes: Vec<SignatureShape>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct HeaderShape {
    pub(super) shape: SignatureShape,
    pub(super) parameters: Vec<String>,
    pub(super) hidden_parameters: Vec<String>,
    pub(super) type_key: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ArgDelimiter {
    Curly,
    Paren,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct ArgGroupShape {
    pub(super) delimiter: ArgDelimiter,
    pub(super) count: usize,
}

#[derive(Clone, Debug)]
pub(super) struct DefinitionEntry {
    pub(super) kind: DefinitionKind,
    pub(super) shape: SignatureShape,
    pub(super) path: PathBuf,
    pub(super) position: Option<SourcePosition>,
}

#[derive(Clone, Debug)]
pub(super) struct DefinitionTypeInfo {
    pub(super) signature: String,
    pub(super) parameters: Vec<String>,
    pub(super) hidden_parameters: Vec<String>,
    pub(super) using_parameters: Vec<String>,
    pub(super) given_parameters: Vec<String>,
    pub(super) requirements: Vec<TypeFact>,
    pub(super) outputs: Vec<TypeFact>,
    pub(super) substitutions: Vec<(String, String)>,
    pub(super) described: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(super) enum TypeFact {
    Is {
        subject: String,
        ty: String,
        signature: String,
    },
    Spec {
        subject: String,
        operator: String,
        target: String,
    },
    InfixSpec {
        subject: String,
        signature: String,
        args: Vec<String>,
        target: String,
    },
    RefinedIs {
        subject: String,
        ty: String,
        signature: String,
        base_ty: String,
        base_signature: String,
    },
    MemberOf {
        subject: String,
        collection: String,
    },
    FunctionType {
        subject: String,
        inputs: Vec<FunctionTypeFactSpec>,
        output: FunctionTypeFactSpec,
        variadic_tuple_input: bool,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(super) enum FunctionTypeFactSpec {
    Is { ty: String, signature: String },
    Spec { operator: String, target: String },
}

#[derive(Clone, Debug)]
pub(super) struct SpecOperatorRule {
    pub(super) owner_signature: String,
    pub(super) source_subject: Option<String>,
    pub(super) source_requires_literal: bool,
    pub(super) placeholder: String,
    pub(super) operator: String,
    pub(super) target: String,
    pub(super) target_alias: SpecOperatorAliasTarget,
}

#[derive(Clone, Debug)]
pub(super) struct ProvidedSymbolRule {
    pub(super) owner_signature: String,
    pub(super) owner_subject: String,
    pub(super) source_subject: Option<String>,
    pub(super) key: DisambiguationKey,
    pub(super) parameters: Vec<String>,
    pub(super) target: Expression,
}

#[derive(Clone, Debug)]
pub(super) struct CastAsRule {
    pub(super) owner_signature: String,
    pub(super) owner_subject: String,
    pub(super) source_subject: String,
    pub(super) left: Expression,
    pub(super) right: Expression,
}

#[derive(Clone, Debug)]
pub(super) struct ViewableRule {
    pub(super) source_signature: String,
    pub(super) source_subject: String,
    pub(super) parameters: Vec<String>,
    pub(super) target_subject: String,
    pub(super) target: TypeFact,
}

#[derive(Clone, Debug)]
pub(super) struct AbstractionRule {
    pub(super) source_signature: String,
    pub(super) source_subject: String,
    pub(super) parameters: Vec<String>,
    pub(super) target: TypeFact,
}

#[derive(Clone, Debug)]
pub(super) struct TypeExtensionRule {
    pub(super) subtype_signature: String,
    pub(super) subject: String,
    pub(super) parameters: Vec<String>,
    pub(super) target: TypeFact,
}

#[derive(Clone, Debug)]
pub(super) struct RefinementExtensionRule {
    pub(super) subtype_signature: String,
    pub(super) subject: String,
    pub(super) parameters: Vec<String>,
    pub(super) target: RefinementExtensionTarget,
}

#[derive(Clone, Debug)]
pub(super) enum RefinementExtensionTarget {
    Fact(TypeFact),
    DynamicRefinedIs {
        subject: String,
        command: RefinedCommandExpression,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum DisambiguationKey {
    BinaryOperator(String),
    Function { name: String, arity: usize },
    PrefixOperator(String),
    PostfixOperator(String),
}

#[derive(Clone, Debug)]
pub(super) struct DisambiguationRule {
    pub(super) key: DisambiguationKey,
    pub(super) parameters: Vec<String>,
    pub(super) branches: Vec<DisambiguationBranch>,
    pub(super) else_expression: Option<Expression>,
}

#[derive(Clone, Debug)]
pub(super) struct DisambiguationBranch {
    pub(super) requirements: Vec<TypeFact>,
    pub(super) substitutions: Vec<(String, String)>,
    pub(super) to: Expression,
}

/// The form of the object a definition declares, compared across the members of
/// a top-level `Equivalent:` item (its "target shape" — rule 2). Two members can
/// be interchangeable only if they declare the same shape of object.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum TargetShape {
    Name,
    Function(usize),
    Tuple(usize),
    Set,
    Operator,
    Statement,
    /// A subject whose shape is not one of the comparable cases above (e.g. a
    /// multi-subject declaration). Two `Other`s compare equal, so exotic subjects
    /// are treated leniently rather than reported as a spurious mismatch.
    Other,
}

/// Precomputed facts about a definition that are needed to validate a top-level
/// `Equivalent:` item but are not otherwise recoverable from the registry by
/// signature alone. Built in pass 1 (which has the defining item's AST).
#[derive(Clone, Debug)]
pub(super) struct DefinitionSummary {
    pub(super) target_shape: TargetShape,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum DefinitionKind {
    Describes,
    Defines,
    Refines,
    States,
    Axiom,
    Theorem,
    Corollary,
    Lemma,
    Conjecture,
    Equivalent,
}

impl DefinitionKind {
    pub(super) fn label(self) -> &'static str {
        match self {
            Self::Describes => "Describes",
            Self::Defines => "Defines",
            Self::Refines => "Refines",
            Self::States => "States",
            Self::Axiom => "Axiom",
            Self::Theorem => "Theorem",
            Self::Corollary => "Corollary",
            Self::Lemma => "Lemma",
            Self::Conjecture => "Conjecture",
            Self::Equivalent => "Equivalent",
        }
    }
}

#[derive(Default)]
pub(super) struct SignatureRegistry {
    pub(super) definitions: HashMap<String, DefinitionEntry>,
    pub(super) definition_summaries: HashMap<String, DefinitionSummary>,
    pub(super) type_infos: HashMap<String, DefinitionTypeInfo>,
    pub(super) spec_rules: Vec<SpecOperatorRule>,
    pub(super) extension_rules: Vec<TypeExtensionRule>,
    pub(super) refinement_extension_rules: Vec<RefinementExtensionRule>,
    pub(super) disambiguations: Vec<DisambiguationRule>,
    pub(super) provided_symbols: Vec<ProvidedSymbolRule>,
    pub(super) cast_as_rules: Vec<CastAsRule>,
    pub(super) viewable_rules: Vec<ViewableRule>,
    pub(super) abstraction_rules: Vec<AbstractionRule>,
    pub(super) collection_type_signatures: Vec<String>,
}

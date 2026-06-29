use super::*;

pub(super) const BUILTIN_EXPRESSION_SIGNATURE: &str = "\\\\expression";
pub(super) const BUILTIN_STATEMENT_SIGNATURE: &str = "\\\\statement";
pub(super) const BUILTIN_SPECIFICATION_SIGNATURE: &str = "\\\\specification";

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
    pub(super) placeholder: String,
    pub(super) operator: String,
    pub(super) target: String,
    pub(super) target_alias: SpecOperatorAliasTarget,
}

#[derive(Clone, Debug)]
pub(super) struct ProvidedSymbolRule {
    pub(super) owner_signature: String,
    pub(super) owner_subject: String,
    pub(super) key: DisambiguationKey,
    pub(super) parameters: Vec<String>,
    pub(super) target: Expression,
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
        }
    }
}

#[derive(Default)]
pub(super) struct SignatureRegistry {
    pub(super) definitions: HashMap<String, DefinitionEntry>,
    pub(super) type_infos: HashMap<String, DefinitionTypeInfo>,
    pub(super) spec_rules: Vec<SpecOperatorRule>,
    pub(super) extension_rules: Vec<TypeExtensionRule>,
    pub(super) refinement_extension_rules: Vec<RefinementExtensionRule>,
    pub(super) disambiguations: Vec<DisambiguationRule>,
    pub(super) provided_symbols: Vec<ProvidedSymbolRule>,
}

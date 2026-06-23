use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct SignatureShape {
    pub(super) signature: String,
    pub(super) arg_groups: Vec<ArgGroupShape>,
    pub(super) fallback_shapes: Vec<SignatureShape>,
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
    pub(super) requirements: Vec<TypeFact>,
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
    FunctionType {
        subject: String,
        inputs: Vec<FunctionTypeFactSpec>,
        output: FunctionTypeFactSpec,
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
pub(super) struct TypeExtensionRule {
    pub(super) subtype_signature: String,
    pub(super) subject: String,
    pub(super) parameters: Vec<String>,
    pub(super) target: TypeFact,
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
}

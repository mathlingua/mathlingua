use super::*;

use std::cell::RefCell;

pub(super) const BUILTIN_EXPRESSION_SIGNATURE: &str = "\\\\expression";
pub(super) const BUILTIN_STATEMENT_SIGNATURE: &str = "\\\\statement";
pub(super) const BUILTIN_SPECIFICATION_SIGNATURE: &str = "\\\\specification";
pub(super) const BUILTIN_TYPE_SIGNATURE: &str = "\\\\type";
pub(super) const BUILTIN_ANYTHING_SIGNATURE: &str = "\\\\anything";

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

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ArgGroupShape {
    pub(super) delimiter: ArgDelimiter,
    pub(super) count: ArgCount,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum ArgCount {
    Exact(usize),
    /// A semicolon-separated matrix argument. The row lengths are retained so
    /// ragged inputs can be rejected with the same shape diagnostic.
    Exact2D {
        row_lengths: Vec<usize>,
    },
    /// One or more arguments. Equal non-`None` length names across groups require
    /// those groups to receive the same number of arguments.
    Variadic {
        length: Option<String>,
    },
    /// A rectangular, nonempty two-dimensional variadic curly argument.
    Variadic2D {
        row_length: Option<String>,
        column_length: Option<String>,
    },
}

#[derive(Clone, Debug)]
pub(super) struct DefinitionEntry {
    pub(super) kind: DefinitionKind,
    pub(super) shape: SignatureShape,
    pub(super) path: PathBuf,
    pub(super) position: Option<SourcePosition>,
    pub(super) placeholder_pattern: Option<PlaceholderSignaturePattern>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct PlaceholderSignaturePattern {
    pub(super) general_signature: String,
    pub(super) mapping_arity: MappingArity,
    pub(super) selectors: Vec<MappingSelectorPattern>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum MappingArity {
    Exact(usize),
    Variadic,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum MappingSelectorPattern {
    Exact(usize),
    Arbitrary,
    Variadic,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct PlaceholderInvocation {
    pub(super) signature: String,
    pub(super) general_signature: String,
    pub(super) mapping_arity: usize,
    pub(super) selected_positions: Vec<usize>,
}

#[derive(Clone, Debug)]
pub(super) struct DefinitionTypeInfo {
    pub(super) signature: String,
    pub(super) type_key: String,
    pub(super) parameters: Vec<String>,
    pub(super) arg_groups: Vec<ArgGroupShape>,
    pub(super) variadic_parameters: Vec<VariadicParameter>,
    pub(super) hidden_parameters: Vec<String>,
    pub(super) using_parameters: Vec<String>,
    pub(super) given_parameters: Vec<String>,
    pub(super) requirements: Vec<TypeFact>,
    pub(super) outputs: Vec<TypeFact>,
    pub(super) substitutions: Vec<(String, String)>,
    /// The subject introduced by a `Defines:`/`Realizes:` target. Component
    /// facts can refer back to this owner and must replace it when the command
    /// is destructured or accessed directly.
    pub(super) defined_subject: Option<String>,
    pub(super) described: Option<String>,
    /// The structural shape of the subject in the `Declares:` target, normalized
    /// so a named form such as `G ::= (X, *, e)` is represented as a name subject
    /// plus a tuple expansion. `Refines:` may repeat this shape or omit a suffix.
    pub(super) described_subject_shape: Option<TargetShape>,
    /// The optional expansion shape in the normalized `Declares:` target, such
    /// as the output name in `f(x__) ::= y_` or the tuple in
    /// `G ::= (X, *, e)`.
    pub(super) described_expansion_shape: Option<TargetShape>,
    /// For a type described with a destructuring target `Name ::= (c1, ..., cn)`,
    /// the type facts of the components in tuple order (each fact's subject is a
    /// component name). Lets another definition that destructures a value of this
    /// type (e.g. a parameter `{M ::= (X, *)}` with `M is \magma`) recover the
    /// component types positionally. Empty when the target is not a tuple.
    pub(super) component_types: Vec<TypeFact>,
    /// Structural shape of each component in a destructuring `Declares:` target.
    /// Names are intentionally ignored, while distinctions such as a value versus
    /// an operator component are retained so `Refines: G ::= (...)` can be checked
    /// positionally against the base type.
    pub(super) component_shapes: Vec<TargetShape>,
    /// Element pattern and declared element facts for a type described by a set
    /// target (`Declares: X ::= {x__ : ...}`).  Command arguments that are set
    /// literals use this metadata for structural compatibility checks instead of
    /// requiring a nominal `literal is Type` fact.
    pub(super) set_element_target: Option<SetTarget>,
    pub(super) set_element_types: Vec<TypeFact>,
    /// Destructuring parameters in this definition's header (e.g. the `M ::= (X, *)`
    /// in `\magma.element:of{M ::= (X, *)}`), with their component names and types.
    /// Lets a provided-symbol capability's reduction target that mentions those
    /// components (`x_ |M.*| y_`) resolve them when the capability is applied.
    pub(super) parameter_destructurings: Vec<DestructuredParameter>,
    /// Inferred parameters introduced by `?`-suffixed names in the `when:`
    /// requirements (e.g. `A`, `B` in `g is \function:on{A?}:to{B?}`). At a use
    /// site these are solved by unifying the mentioning requirement against a
    /// fact already known about its subject.
    pub(super) inferred_parameters: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct DestructuredParameter {
    pub(super) name: String,
    pub(super) components: Vec<String>,
    /// The parameter's declared type (e.g. `\magma` for `{M ::= (X, *)}` with
    /// `when: M is \magma`). Component types are resolved from this type's own
    /// component types lazily, so collection order does not matter.
    pub(super) type_signature: String,
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
    pub(super) owner_is_defined_value: bool,
    pub(super) owner_parameters: Vec<String>,
    pub(super) source_subject: Option<String>,
    pub(super) source_requires_literal: bool,
    pub(super) placeholder: String,
    pub(super) operator: String,
    pub(super) target: String,
    pub(super) kind: SpecOperatorAliasKind,
    pub(super) target_alias: SpecOperatorAliasTarget,
    pub(super) substitutions: Vec<(String, String)>,
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
    pub(super) view_expression: Expression,
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

/// One command in an equivalence class declared by a top-level `Equivalent:`
/// item (a `to:` member, or the class-naming header itself).
#[derive(Clone, Debug)]
pub(super) struct EquivalenceMember {
    pub(super) signature: String,
    /// The header parameter names this member's arguments correspond to, in the
    /// order the member lists them.
    pub(super) params: Vec<String>,
}

/// A set of mutually interchangeable commands declared by one `Equivalent:` item.
/// Membership is by command signature; a value known to be one member is treated
/// as every other member instantiated at the same header parameters.
#[derive(Clone, Debug)]
pub(super) struct EquivalenceClass {
    pub(super) members: Vec<EquivalenceMember>,
}

impl EquivalenceClass {
    pub(super) fn member(&self, signature: &str) -> Option<&EquivalenceMember> {
        self.members
            .iter()
            .find(|member| member.signature == signature)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum DefinitionKind {
    Declares,
    Defines,
    Realizes,
    Refines,
    States,
    Axiom,
    Theorem,
    Conjecture,
    Example,
    Equivalent,
}

impl DefinitionKind {
    pub(super) fn label(self) -> &'static str {
        match self {
            Self::Declares => "Declares",
            Self::Defines => "Defines",
            Self::Realizes => "Realizes",
            Self::Refines => "Refines",
            Self::States => "States",
            Self::Axiom => "Axiom",
            Self::Theorem => "Theorem",
            Self::Conjecture => "Conjecture",
            Self::Example => "Example",
            Self::Equivalent => "Equivalent",
        }
    }
}

#[derive(Default)]
pub(super) struct SignatureRegistry {
    pub(super) definitions: HashMap<String, DefinitionEntry>,
    /// General command signature to the placeholder-specialized definitions
    /// sharing it, in source registration order.
    pub(super) placeholder_definitions: HashMap<String, Vec<String>>,
    pub(super) definition_summaries: HashMap<String, DefinitionSummary>,
    pub(super) equivalence_classes: Vec<EquivalenceClass>,
    pub(super) type_infos: HashMap<String, DefinitionTypeInfo>,
    pub(super) spec_rules: Vec<SpecOperatorRule>,
    pub(super) extension_rules: Vec<TypeExtensionRule>,
    pub(super) refinement_extension_rules: Vec<RefinementExtensionRule>,
    pub(super) disambiguations: Vec<DisambiguationRule>,
    pub(super) provided_symbols: Vec<ProvidedSymbolRule>,
    pub(super) cast_as_rules: Vec<CastAsRule>,
    pub(super) viewable_rules: Vec<ViewableRule>,
    pub(super) collection_type_signatures: Vec<String>,
    /// Maps an abstract `Defines:` signature — one marked `abstractly:` — to the
    /// symbols it specifies but leaves for a `Realizes:` to supply.
    pub(super) abstract_declarations: HashMap<String, AbstractDeclaration>,
    /// Maps a set-defining command signature (a `Defines` whose `:=` value is a
    /// set literal) to that set-builder body, so membership in a use of the
    /// command can be reduced to the body's element condition.
    pub(super) collection_bodies: HashMap<String, SetExpression>,
    pub(super) numeric_specifications: NumericSpecifications,
    /// Set only while a type-info pass walks a document, to collect the type the
    /// checker resolves for each expression it visits. The registry is the one
    /// value already threaded through every check function, so hanging the
    /// recorder off it keeps the walk's signatures unchanged; an ordinary check
    /// leaves it `None` and pays a null check per expression.
    pub(super) recorder: RefCell<Option<TypeRecorder>>,
}

/// A `Defines:` marked `abstractly:`, together with what a realization owes it.
///
/// The declaration states a specification for each abstract symbol but no value;
/// a `Realizes:` of it must define every one of them.
#[derive(Clone, Debug, Default)]
pub(super) struct AbstractDeclaration {
    /// The facts stated about symbols the declaration does not define, in source
    /// order. Their subjects are exactly what a `Realizes:` must supply.
    pub(super) abstract_facts: Vec<TypeFact>,
}

#[derive(Clone, Debug, Default)]
pub(super) struct NumericSpecifications {
    pub(super) decimal: Option<NumericTypeSpecification>,
    pub(super) zero_or_positive_int: Option<NumericTypeSpecification>,
    pub(super) positive_int: Option<NumericTypeSpecification>,
    pub(super) int: Option<NumericTypeSpecification>,
}

#[derive(Clone, Debug)]
pub(super) struct NumericTypeSpecification {
    pub(super) ty: String,
    pub(super) signature: String,
}

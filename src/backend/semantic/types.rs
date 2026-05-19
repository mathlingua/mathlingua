use super::*;

/// A source file after the frontend has parsed it into a structural document.
///
/// Backend passes operate on this type instead of repeatedly reading or
/// reparsing files.  It keeps the original source text for diagnostic location
/// lookup, the filesystem path for reporting, and the parsed structural AST for
/// semantic traversal.
#[derive(Clone, Debug)]
pub struct ParsedSourceFile {
    /// Path of the file on disk, used in diagnostics and duplicate reports.
    pub path: PathBuf,
    /// Original file contents, used to recover line and column information.
    pub source: String,
    /// Structural representation produced by the frontend parser.
    pub document: Document,
}

/// Canonical command signature plus the argument groups required by that form.
///
/// The signature intentionally strips concrete `{...}` and `(...)` contents, so
/// `\function{A, B}` and `\function{X}` share `\function`.  The accompanying
/// `arg_groups` records how many arguments each group must contain, while
/// `fallback_shapes` records secondary shapes that a composed refined command
/// may legally refer to.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct SignatureShape {
    /// Canonical signature text, such as `\function:on:to`.
    pub(super) signature: String,
    /// Ordered argument groups that must be supplied for this signature.
    pub(super) arg_groups: Vec<ArgGroupShape>,
    /// Alternate shapes to validate when a combined refined command is absent.
    pub(super) fallback_shapes: Vec<SignatureShape>,
}

/// Delimiter used by one command argument group.
///
/// Curly groups are required whenever present in a definition.  Parenthesized
/// groups are optional for use sites because they represent invocation of an
/// already-described callable object.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ArgDelimiter {
    /// A required `{...}` argument group.
    Curly,
    /// An optional-at-use-site `(...)` argument group.
    Paren,
}

/// Shape of one argument group in a command definition or reference.
///
/// Only the delimiter and arity are tracked here.  The semantic checker does not
/// yet type-check the individual argument expressions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct ArgGroupShape {
    /// Whether the group was written with `{}` or `()`.
    pub(super) delimiter: ArgDelimiter,
    /// Number of comma-separated arguments inside the group.
    pub(super) count: usize,
}

/// Registered definition metadata for one unique command signature.
///
/// This is the source of truth used to detect duplicate signatures and to check
/// references for undefined commands or argument-shape mismatches.
#[derive(Clone, Debug)]
pub(super) struct DefinitionEntry {
    /// Structural kind that introduced the signature.
    pub(super) kind: DefinitionKind,
    /// Canonical signature and expected argument shape.
    pub(super) shape: SignatureShape,
    /// Source path where the definition was found.
    pub(super) path: PathBuf,
    /// Best-effort source location of the command in the original file.
    pub(super) position: Option<SourcePosition>,
}

/// Type-checking metadata associated with a command definition.
#[derive(Clone, Debug)]
pub(super) struct DefinitionTypeInfo {
    /// Canonical command signature.
    pub(super) signature: String,
    /// Parameter names gathered from the command heading, in argument order.
    pub(super) parameters: Vec<String>,
    /// Facts that must be provable when this command is used.
    pub(super) requirements: Vec<TypeFact>,
    /// Local substitutions available while matching the requirements.
    pub(super) substitutions: Vec<(String, String)>,
    /// Described subject for `Describes:` definitions.
    pub(super) described: Option<String>,
}

/// A fact known or required by the type checker.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(super) enum TypeFact {
    /// A subject has a command type.
    Is {
        /// Subject key.
        subject: String,
        /// Full type key.
        ty: String,
        /// Canonical command signature for the type.
        signature: String,
    },
    /// A subject participates in a quoted-operator specification.
    Spec {
        /// Subject key.
        subject: String,
        /// Quoted operator text.
        operator: String,
        /// Right-hand target key.
        target: String,
    },
}

/// Type-check-time rewrite rule introduced by `:->`.
#[derive(Clone, Debug)]
pub(super) struct SpecOperatorRule {
    /// Signature of the definition that provides this rule.
    pub(super) owner_signature: String,
    /// Subject placeholder on the left-hand side, such as `x`.
    pub(super) placeholder: String,
    /// Quoted operator text on the left-hand side.
    pub(super) operator: String,
    /// Right-hand target key on the left-hand side.
    pub(super) target: String,
    /// Alias target to instantiate when the rule applies.
    pub(super) target_alias: SpecOperatorAliasTarget,
}

/// Top-level structural groups that can introduce a command signature.
///
/// The uniqueness rule is global across these kinds: the same signature cannot
/// be reused merely because it appears in a different group type.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum DefinitionKind {
    /// A `Describes` group.
    Describes,
    /// A `Defines` group.
    Defines,
    /// A `Refines` group.
    Refines,
    /// A `States` group.
    States,
    /// A named `Axiom`.
    Axiom,
    /// A named `Theorem`.
    Theorem,
    /// A named `Corollary`.
    Corollary,
    /// A named `Lemma`.
    Lemma,
    /// A named `Conjecture`.
    Conjecture,
}

impl DefinitionKind {
    /// Returns the user-facing label used in diagnostics for this definition kind.
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

/// Index of all definitions visible to a semantic checking run.
///
/// The key is the canonical command signature.  Because overloading is not
/// currently supported, one entry per signature is sufficient and duplicates can
/// be reported immediately during collection.
#[derive(Default)]
pub(super) struct SignatureRegistry {
    /// Map from canonical command signature to the definition that owns it.
    pub(super) definitions: HashMap<String, DefinitionEntry>,
    /// Type-checking metadata keyed by canonical command signature.
    pub(super) type_infos: HashMap<String, DefinitionTypeInfo>,
    /// Spec-operator rules collected from `Provides:` symbols.
    pub(super) spec_rules: Vec<SpecOperatorRule>,
}

/// Top-level document title group.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TitleGroup {
    /// Required `Title:` section.
    pub title: TitleSection,
}

/// Top-level first-level section group.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SectionGroup {
    /// Required `Section:` section.
    pub section: SectionSection,
}

/// Top-level second-level section group.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SubsectionGroup {
    /// Required `Subsection:` section.
    pub subsection: SubsectionSection,
}

/// Top-level third-level section group.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SubsubsectionGroup {
    /// Required `Subsubsection:` section.
    pub subsubsection: SubsubsectionSection,
}

/// Command group that describes the form introduced by a command heading.
///
/// `Describes` is the main definition-like group for mathematical objects that
/// have a presentation form plus optional assumptions, provides, documentation,
/// aliases, references, and metadata.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DescribesGroup {
    /// Required command heading that provides the command signature.
    pub heading: CommandHeader,
    /// Required described form or declaration.
    pub describes: DescribesSection,
    /// Optional `using:` specifications.
    pub using: Option<UsingSection>,
    /// Optional `when:` clauses.
    pub when: Option<WhenSection>,
    /// Optional `extends:` item.
    pub extends: Option<ExtendsSection>,
    /// Optional `specifies:` items.
    pub specifies: Option<DescribesSpecifiesSection>,
    /// Optional `satisfies:` clauses.
    pub satisfies: Option<SatisfiesSection>,
    /// Optional provided symbols/connections.
    pub provides: Option<ProvidesSection>,
    /// Optional justification notes.
    pub justified: Option<JustifiedSection>,
    /// Optional documentation/rendering metadata.
    pub documented: Option<DocumentedSection>,
    /// Optional aliases.
    pub aliases: Option<AliasesSection>,
    /// Optional resource references.
    pub references: Option<ReferencesSection>,
    /// Optional machine-readable metadata.
    pub metadata: Option<MetadataSection>,
}

/// Command group that defines a specification or type-level statement.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DefinesGroup {
    /// Required command heading that provides the command signature.
    pub heading: CommandHeader,
    /// Required `Defines:` statement.
    pub defines: DefinesSection,
    /// Optional `using:` specifications.
    pub using: Option<UsingSection>,
    /// Optional `when:` clauses.
    pub when: Option<WhenSection>,
    /// Optional `expresses:` clause.
    pub expresses: Option<ExpressesSection>,
    /// Optional provided symbols/connections.
    pub provides: Option<ProvidesSection>,
    /// Optional justification notes.
    pub justified: Option<JustifiedSection>,
    /// Optional documentation/rendering metadata.
    pub documented: Option<DocumentedSection>,
    /// Optional aliases.
    pub aliases: Option<AliasesSection>,
    /// Optional resource references.
    pub references: Option<ReferencesSection>,
    /// Optional machine-readable metadata.
    pub metadata: Option<MetadataSection>,
}

/// Command group that refines an existing command definition.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RefinesGroup {
    /// Required command heading for the refined signature.
    pub heading: CommandHeader,
    /// Required `Refines:` statement identifying the refined target.
    pub refines: RefinesSection,
    /// Optional `using:` specifications.
    pub using: Option<UsingSection>,
    /// Optional `when:` clauses.
    pub when: Option<WhenSection>,
    /// Optional refined `specifies:` statement.
    pub specifies: Option<RefinesSpecifiesSection>,
    /// Optional `satisfies:` clauses.
    pub satisfies: Option<SatisfiesSection>,
    /// Optional provided symbols/connections.
    pub provides: Option<ProvidesSection>,
    /// Optional justification notes.
    pub justified: Option<JustifiedSection>,
    /// Optional documentation/rendering metadata.
    pub documented: Option<DocumentedSection>,
    /// Optional aliases.
    pub aliases: Option<AliasesSection>,
    /// Optional resource references.
    pub references: Option<ReferencesSection>,
    /// Optional machine-readable metadata.
    pub metadata: Option<MetadataSection>,
}

/// Command-backed statement group with a required `that:` clause body.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StatesGroup {
    /// Required command heading for the statement signature.
    pub heading: CommandHeader,
    /// Optional prose attached to the `States:` section.
    pub states: StatesSection,
    /// Optional `using:` specifications.
    pub using: Option<UsingSection>,
    /// Optional `when:` clauses.
    pub when: Option<WhenSection>,
    /// Required statement clauses.
    pub that: ThatSection,
    /// Optional provided symbols/connections.
    pub provides: Option<ProvidesSection>,
    /// Optional justification notes.
    pub justified: Option<JustifiedSection>,
    /// Optional documentation/rendering metadata.
    pub documented: Option<DocumentedSection>,
    /// Optional aliases.
    pub aliases: Option<AliasesSection>,
    /// Optional resource references.
    pub references: Option<ReferencesSection>,
    /// Optional machine-readable metadata.
    pub metadata: Option<MetadataSection>,
}

/// Axiom group with optional command heading and theorem-like clauses.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AxiomGroup {
    /// Optional command heading for named axiom references.
    pub heading: Option<CommandHeader>,
    /// Optional axiom prose.
    pub axiom: AxiomSection,
    /// Optional assumptions.
    pub given: Option<GivenSection>,
    /// Optional local context clauses.
    pub where_: Option<WhereSection>,
    /// Required conclusion clauses.
    pub then: ThenSection,
    /// Optional iff clauses.
    pub iff: Option<IffSection>,
    /// Optional justification notes.
    pub justified: Option<JustifiedSection>,
    /// Optional documentation/rendering metadata.
    pub documented: Option<DocumentedSection>,
    /// Optional aliases.
    pub aliases: Option<AliasesSection>,
    /// Optional resource references.
    pub references: Option<ReferencesSection>,
    /// Optional machine-readable metadata.
    pub metadata: Option<MetadataSection>,
}

/// Theorem group with optional command heading and theorem-like clauses.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TheoremGroup {
    /// Optional command heading for named theorem references.
    pub heading: Option<CommandHeader>,
    /// Optional theorem prose.
    pub theorem: TheoremSection,
    /// Optional assumptions.
    pub given: Option<GivenSection>,
    /// Optional local context clauses.
    pub where_: Option<WhereSection>,
    /// Required conclusion clauses.
    pub then: ThenSection,
    /// Optional iff clauses.
    pub iff: Option<IffSection>,
    /// Optional justification notes.
    pub justified: Option<JustifiedSection>,
    /// Optional documentation/rendering metadata.
    pub documented: Option<DocumentedSection>,
    /// Optional aliases.
    pub aliases: Option<AliasesSection>,
    /// Optional resource references.
    pub references: Option<ReferencesSection>,
    /// Optional machine-readable metadata.
    pub metadata: Option<MetadataSection>,
}

/// Corollary group with an `of:` section plus theorem-like clauses.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CorollaryGroup {
    /// Optional command heading for named corollary references.
    pub heading: Option<CommandHeader>,
    /// Optional corollary prose.
    pub corollary: CorollarySection,
    /// Required `of:` relationship text.
    pub of: OfSection,
    /// Optional assumptions.
    pub given: Option<GivenSection>,
    /// Optional local context clauses.
    pub where_: Option<WhereSection>,
    /// Required conclusion clauses.
    pub then: ThenSection,
    /// Optional iff clauses.
    pub iff: Option<IffSection>,
    /// Optional justification notes.
    pub justified: Option<JustifiedSection>,
    /// Optional documentation/rendering metadata.
    pub documented: Option<DocumentedSection>,
    /// Optional aliases.
    pub aliases: Option<AliasesSection>,
    /// Optional resource references.
    pub references: Option<ReferencesSection>,
    /// Optional machine-readable metadata.
    pub metadata: Option<MetadataSection>,
}

/// Lemma group with optional command heading and theorem-like clauses.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LemmaGroup {
    /// Optional command heading for named lemma references.
    pub heading: Option<CommandHeader>,
    /// Optional lemma prose.
    pub lemma: LemmaSection,
    /// Optional assumptions.
    pub given: Option<GivenSection>,
    /// Optional local context clauses.
    pub where_: Option<WhereSection>,
    /// Required conclusion clauses.
    pub then: ThenSection,
    /// Optional iff clauses.
    pub iff: Option<IffSection>,
    /// Optional justification notes.
    pub justified: Option<JustifiedSection>,
    /// Optional documentation/rendering metadata.
    pub documented: Option<DocumentedSection>,
    /// Optional aliases.
    pub aliases: Option<AliasesSection>,
    /// Optional resource references.
    pub references: Option<ReferencesSection>,
    /// Optional machine-readable metadata.
    pub metadata: Option<MetadataSection>,
}

/// Conjecture group with optional command heading and theorem-like clauses.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConjectureGroup {
    /// Optional command heading for named conjecture references.
    pub heading: Option<CommandHeader>,
    /// Optional conjecture prose.
    pub conjecture: ConjectureSection,
    /// Optional assumptions.
    pub given: Option<GivenSection>,
    /// Optional local context clauses.
    pub where_: Option<WhereSection>,
    /// Required conclusion clauses.
    pub then: ThenSection,
    /// Optional iff clauses.
    pub iff: Option<IffSection>,
    /// Optional justification notes.
    pub justified: Option<JustifiedSection>,
    /// Optional documentation/rendering metadata.
    pub documented: Option<DocumentedSection>,
    /// Optional aliases.
    pub aliases: Option<AliasesSection>,
    /// Optional resource references.
    pub references: Option<ReferencesSection>,
    /// Optional machine-readable metadata.
    pub metadata: Option<MetadataSection>,
}

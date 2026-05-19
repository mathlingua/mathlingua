use super::*;

/// Root structural AST for one parsed MathLingua document.
///
/// The proto parser has already grouped sections by indentation; this document
/// contains only recognized top-level structural items.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Document {
    /// Top-level items in source order.
    pub items: ZeroOrMore<TopLevelItem>,
}

/// Every top-level group currently recognized by the structural parser.
///
/// Variants correspond to the leading section label of a proto group, with
/// headings and required sections normalized into strongly typed group structs.
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TopLevelItem {
    /// A document title group.
    Title(TitleGroup),
    /// A first-level section heading.
    Section(SectionGroup),
    /// A second-level section heading.
    Subsection(SubsectionGroup),
    /// A third-level section heading.
    Subsubsection(SubsubsectionGroup),
    /// A command definition that describes a mathematical form.
    Describes(DescribesGroup),
    /// A command definition that defines a specification or type.
    Defines(DefinesGroup),
    /// A command definition that refines another command.
    Refines(RefinesGroup),
    /// A named statement-like command with clauses.
    States(StatesGroup),
    /// An axiom block.
    Axiom(AxiomGroup),
    /// A theorem block.
    Theorem(TheoremGroup),
    /// A corollary block.
    Corollary(CorollaryGroup),
    /// A lemma block.
    Lemma(LemmaGroup),
    /// A conjecture block.
    Conjecture(ConjectureGroup),
    /// A person metadata block.
    Person(PersonGroup),
    /// A resource/bibliography metadata block.
    Resource(ResourceGroup),
    /// A numeric domain specification block.
    Specify(SpecifyGroup),
}

/// A logical clause inside theorem-like or definition-like sections.
///
/// Clauses may be structured groups such as `forAll` or inline formulation
/// expressions/specifications.  This enum lets backend passes traverse both
/// structured and inline clause forms uniformly.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Clause {
    /// Logical negation.
    Not(NotGroup),
    /// Conjunction of all child clauses.
    AllOf(AllOfGroup),
    /// Disjunction of any child clauses.
    AnyOf(AnyOfGroup),
    /// Exactly-one style disjunction.
    OneOf(OneOfGroup),
    /// Existential quantification.
    Exists(ExistsGroup),
    /// Unique existential quantification.
    ExistsUnique(ExistsUniqueGroup),
    /// Universal quantification.
    ForAll(ForAllGroup),
    /// Implication block.
    If(IfGroup),
    /// Bi-implication block.
    Iff(IffGroup),
    /// Piecewise conditional block.
    Piecewise(PiecewiseGroup),
    /// Nested given/then clause block.
    Given(GivenGroup),
    /// Local syntactic equality binding.
    Binding(ExpressionBinding),
    /// Inline `is` statement or specification.
    IsOrSpec(IsOrSpec),
    /// Inline expression clause.
    Expression(Expression),
}

/// Item accepted by sections that can reference either `is via` or `is/spec` syntax.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IsOrViaItem {
    /// A statement with an accompanying `via` tuple.
    IsVia(IsViaStatement),
    /// A plain `is` statement or specification.
    IsOrSpec(IsOrSpec),
}

/// Item accepted by quantifier sections that can bind equality or facts.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BindingOrSpec {
    /// Local syntactic equality binding.
    Binding(ExpressionBinding),
    /// A plain `is` statement or specification.
    IsOrSpec(IsOrSpec),
}

/// Kinds of alias definitions accepted by `alias:` and `symbol:` groups.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AliasKind {
    /// An expression alias using `:=>`.
    Expression(ExpressionAlias),
    /// A specification-operator alias using `:->`.
    SpecOperator(SpecOperatorAlias),
}

/// Nested item accepted inside an `Aliases:` section.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AliasItem {
    /// An `alias:` group.
    Alias(AliasGroup),
}

/// Nested item accepted inside a `Provides:` section.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProvidesItem {
    /// A provided symbol.
    Symbol(Box<SymbolGroup>),
    /// A provided connection.
    Connection(ConnectionGroup),
}

/// Nested item accepted inside a `Documented:` section.
///
/// Documentation groups carry rendering text and related prose used by the view
/// layer and, eventually, richer generated documentation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DocumentedItem {
    /// A `written:` group.
    Written(WrittenGroup),
    /// A `called:` group.
    Called(CalledGroup),
    /// A `writing:` alias group.
    Writing(WritingGroup),
    /// An `overview:` prose group.
    Overview(OverviewGroup),
    /// A `related:` prose group.
    Related(RelatedGroup),
    /// A `discoverer:` prose group.
    Discoverer(DiscovererGroup),
}

/// Nested item accepted inside a `Justified:` section.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum JustifiedItem {
    /// A labeled justification note.
    Label(LabelGroup),
    /// A by/comment justification note.
    By(ByGroup),
}

/// Nested item accepted inside a `Metadata:` section.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MetadataItem {
    /// Stable identifier metadata.
    Id(IdGroup),
    /// Version metadata.
    Version(VersionGroup),
}

/// Nested item accepted inside a top-level `Specify:` block.
///
/// These items currently model numeric-domain categories used by the language's
/// specification vocabulary.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SpecifyItem {
    /// Positive integer specification item.
    PositiveInt(PositiveIntGroup),
    /// Negative integer specification item.
    NegativeInt(NegativeIntGroup),
    /// Zero specification item.
    Zero(ZeroGroup),
    /// Positive decimal specification item.
    PositiveDecimal(PositiveDecimalGroup),
    /// Negative decimal specification item.
    NegativeDecimal(NegativeDecimalGroup),
}

/// Nested item accepted inside a `Resource:` group.
///
/// Resource items capture bibliography and web metadata while keeping each
/// field as its own typed group for validation and rendering.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ResourceItem {
    /// Resource title field.
    Title(ResourceTitleGroup),
    /// Resource author field.
    Author(ResourceAuthorGroup),
    /// Resource offset field, such as a page or location.
    Offset(ResourceOffsetGroup),
    /// Resource URL field.
    Url(ResourceUrlGroup),
    /// Resource homepage field.
    Homepage(ResourceHomepageGroup),
    /// Resource type field.
    Type(ResourceTypeGroup),
    /// Resource edition field.
    Edition(ResourceEditionGroup),
    /// Resource editor field.
    Editor(ResourceEditorGroup),
    /// Resource institution field.
    Institution(ResourceInstitutionGroup),
    /// Resource journal field.
    Journal(ResourceJournalGroup),
    /// Resource publisher field.
    Publisher(ResourcePublisherGroup),
    /// Resource volume field.
    Volume(ResourceVolumeGroup),
    /// Resource month field.
    Month(ResourceMonthGroup),
    /// Resource year field.
    Year(ResourceYearGroup),
    /// Resource description field.
    Description(ResourceDescriptionGroup),
}

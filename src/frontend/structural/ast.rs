//! Strongly typed AST produced by the structural parser.

use crate::frontend::formulation::ast::{
    AuthorHeader, CommandHeader, Expression, ExpressionAlias, ExpressionBinding, FormOrDeclaration,
    IsOrRefinedStatementSpec, IsOrSpec, IsViaStatement, LabelHeader, ResourceHeader,
    SpecOperatorAlias, WritingAlias,
};

// ===============================[ repeated ]=====================================

macro_rules! impl_repeated_items {
    ($name:ident) => {
        impl<T> std::ops::Deref for $name<T> {
            /// Slice view of the wrapped repeated items.
            type Target = [T];

            /// Exposes the repeated wrapper as a slice.
            ///
            /// Most downstream passes only need read-only list behavior, so the
            /// wrapper dereferences to the inner slice while preserving its
            /// cardinality guarantee in the type name.
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl<T> IntoIterator for $name<T> {
            /// Owned item yielded by value iteration.
            type Item = T;
            /// Owned vector iterator used after consuming the wrapper.
            type IntoIter = std::vec::IntoIter<T>;

            /// Consumes the wrapper and iterates over its owned items.
            fn into_iter(self) -> Self::IntoIter {
                self.0.into_iter()
            }
        }

        impl<'a, T> IntoIterator for &'a $name<T> {
            /// Shared item yielded by borrowed iteration.
            type Item = &'a T;
            /// Shared slice iterator over the wrapped values.
            type IntoIter = std::slice::Iter<'a, T>;

            /// Iterates immutably over the wrapped items.
            fn into_iter(self) -> Self::IntoIter {
                self.0.iter()
            }
        }

        impl<'a, T> IntoIterator for &'a mut $name<T> {
            /// Mutable item yielded by borrowed mutable iteration.
            type Item = &'a mut T;
            /// Mutable slice iterator over the wrapped values.
            type IntoIter = std::slice::IterMut<'a, T>;

            /// Iterates mutably over the wrapped items.
            fn into_iter(self) -> Self::IntoIter {
                self.0.iter_mut()
            }
        }
    };
}

/// A list that may contain any number of items, including none.
///
/// Structural AST nodes use this wrapper instead of bare `Vec<T>` to make the
/// MathLingua section grammar explicit in type signatures.  Optional sections
/// can still distinguish between a missing section and a present empty section.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ZeroOrMore<T>(Vec<T>);

impl<T> Default for ZeroOrMore<T> {
    /// Creates an empty repeated-item wrapper.
    fn default() -> Self {
        Self(Vec::new())
    }
}

impl<T> ZeroOrMore<T> {
    /// Consumes the wrapper and returns the underlying vector.
    ///
    /// This is used at API boundaries where callers need ordinary collection
    /// operations that are not worth mirroring on the wrapper type.
    pub fn into_vec(self) -> Vec<T> {
        self.0
    }
}

impl<T> From<Vec<T>> for ZeroOrMore<T> {
    /// Wraps a vector without changing its contents or cardinality.
    fn from(value: Vec<T>) -> Self {
        Self(value)
    }
}

impl<T> From<OneOrMore<T>> for ZeroOrMore<T> {
    /// Forgets the non-empty guarantee while preserving the items.
    fn from(value: OneOrMore<T>) -> Self {
        Self(value.into_vec())
    }
}

impl_repeated_items!(ZeroOrMore);

/// A list that is guaranteed to contain at least one item.
///
/// Required repeated sections use this wrapper so successful structural parsing
/// records the cardinality guarantee in the AST, rather than requiring every
/// backend pass to re-check emptiness.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OneOrMore<T>(Vec<T>);

impl<T> OneOrMore<T> {
    /// Creates a nonempty list from its first item plus any remaining items.
    pub fn new(first: T, rest: Vec<T>) -> Self {
        let mut items = Vec::with_capacity(rest.len() + 1);
        items.push(first);
        items.extend(rest);
        Self(items)
    }

    /// Consumes the wrapper and returns the underlying vector.
    pub fn into_vec(self) -> Vec<T> {
        self.0
    }
}

impl<T> TryFrom<Vec<T>> for OneOrMore<T> {
    /// Original vector returned when it is empty and cannot satisfy the invariant.
    type Error = Vec<T>;

    /// Converts a vector to a nonempty wrapper when it contains at least one item.
    ///
    /// Empty input is returned unchanged so callers can decide how to report the
    /// missing required content.
    fn try_from(value: Vec<T>) -> Result<Self, Self::Error> {
        if value.is_empty() {
            Err(value)
        } else {
            Ok(Self(value))
        }
    }
}

impl<T> TryFrom<ZeroOrMore<T>> for OneOrMore<T> {
    /// Original wrapper returned when it is empty and cannot satisfy the invariant.
    type Error = ZeroOrMore<T>;

    /// Converts a zero-or-more wrapper to a one-or-more wrapper when possible.
    ///
    /// On failure the original cardinality wrapper is returned, preserving the
    /// caller's type-level knowledge about the source section.
    fn try_from(value: ZeroOrMore<T>) -> Result<Self, Self::Error> {
        let items = value.into_vec();
        if items.is_empty() {
            Err(ZeroOrMore::default())
        } else {
            Ok(Self(items))
        }
    }
}

impl_repeated_items!(OneOrMore);

// ===============================[ sections ]=====================================

macro_rules! argument_section {
    ($name:ident, $ty:ty) => {
        /// Structural section wrapper for a section with exactly one parsed argument.
        ///
        /// The concrete type name records the section label while the `argument`
        /// field stores the formulation, text, or nested item parsed for that
        /// section.
        #[derive(Clone, Debug, PartialEq, Eq)]
        pub struct $name {
            /// Parsed argument carried by this section.
            pub argument: $ty,
        }
    };
}

macro_rules! arguments_section {
    ($name:ident, $ty:ty) => {
        /// Structural section wrapper for a required repeated section.
        ///
        /// The parser only constructs this type when at least one item was
        /// parsed successfully, so downstream code can rely on non-emptiness.
        #[derive(Clone, Debug, PartialEq, Eq)]
        pub struct $name {
            /// Nonempty parsed arguments for this section.
            pub arguments: OneOrMore<$ty>,
        }
    };
}

macro_rules! zero_or_more_arguments_section {
    ($name:ident, $ty:ty) => {
        /// Structural section wrapper for an optional or open repeated section.
        ///
        /// This captures section families where an explicitly present section
        /// may still carry no arguments after parsing.
        #[derive(Clone, Debug, PartialEq, Eq)]
        pub struct $name {
            /// Parsed arguments, possibly empty.
            pub arguments: ZeroOrMore<$ty>,
        }
    };
}

/// Open quoted text from a prose-oriented structural section.
///
/// The parser strips the outer quotes and keeps the inner text exactly as
/// written so rendering/backends can choose the appropriate interpretation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OpenText(pub String);

/// Quoted text from a `written:` rendering section.
///
/// Written text is interpreted later as math-mode LaTeX with MathLingua
/// substitution markers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WrittenText(pub String);

/// Quoted text from a `called:` rendering section.
///
/// Called text is interpreted later as plain LaTeX text, with explicit math
/// regions supplied by the author where substitutions should appear.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CalledText(pub String);

/// Quoted text from an `as:` entry inside a `writing:` group.
///
/// This captures rendering prose associated with a writing alias.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WritingText(pub String);

argument_section!(DescribesSection, FormOrDeclaration);
arguments_section!(UsingSection, IsOrSpec);
arguments_section!(WhenSection, Clause);
argument_section!(ExtendsSection, IsOrViaItem);
arguments_section!(DescribesSpecifiesSection, IsOrViaItem);
arguments_section!(SatisfiesSection, Clause);
arguments_section!(ProvidesSection, ProvidesItem);
arguments_section!(JustifiedSection, JustifiedItem);
arguments_section!(DocumentedSection, DocumentedItem);
arguments_section!(AliasesSection, AliasItem);
arguments_section!(ReferencesSection, ResourceHeader);
arguments_section!(MetadataSection, MetadataItem);
argument_section!(DefinesSection, IsOrSpec);
argument_section!(ExpressesSection, Clause);
argument_section!(RefinesSection, IsOrRefinedStatementSpec);
argument_section!(RefinesSpecifiesSection, IsOrRefinedStatementSpec);
zero_or_more_arguments_section!(StatesSection, OpenText);
arguments_section!(ThatSection, Clause);
zero_or_more_arguments_section!(AxiomSection, OpenText);
zero_or_more_arguments_section!(TheoremSection, OpenText);
zero_or_more_arguments_section!(CorollarySection, OpenText);
zero_or_more_arguments_section!(OfSection, OpenText);
zero_or_more_arguments_section!(LemmaSection, OpenText);
zero_or_more_arguments_section!(ConjectureSection, OpenText);
arguments_section!(GivenSection, IsOrRefinedStatementSpec);
arguments_section!(WhereSection, Clause);
arguments_section!(ThenSection, Clause);
arguments_section!(IffSection, Clause);
argument_section!(AliasSection, AliasKind);
arguments_section!(WrittenSection, WrittenText);
argument_section!(SymbolSection, AliasKind);
zero_or_more_arguments_section!(ConnectionSection, OpenText);
zero_or_more_arguments_section!(ToSection, OpenText);
zero_or_more_arguments_section!(MeansSection, OpenText);
zero_or_more_arguments_section!(SignifiesSection, OpenText);
zero_or_more_arguments_section!(ViewableSection, OpenText);
zero_or_more_arguments_section!(ThroughSection, OpenText);
arguments_section!(CalledSection, CalledText);
argument_section!(WritingSection, WritingAlias);
arguments_section!(AsSection, WritingText);
argument_section!(OverviewSection, OpenText);
arguments_section!(RelatedSection, OpenText);
zero_or_more_arguments_section!(DiscovererSection, OpenText);
zero_or_more_arguments_section!(LabelSection, OpenText);
zero_or_more_arguments_section!(BySection, OpenText);
argument_section!(CommentSection, OpenText);
argument_section!(IdSection, OpenText);
argument_section!(VersionSection, OpenText);
arguments_section!(SpecifySection, SpecifyItem);
zero_or_more_arguments_section!(PositiveSection, OpenText);
zero_or_more_arguments_section!(IntSection, OpenText);
zero_or_more_arguments_section!(IsSection, OpenText);
zero_or_more_arguments_section!(NegativeSection, OpenText);
zero_or_more_arguments_section!(ZeroSection, OpenText);
zero_or_more_arguments_section!(DecimalSection, OpenText);
zero_or_more_arguments_section!(PersonSection, OpenText);
arguments_section!(NameSection, OpenText);
argument_section!(BiographySection, OpenText);
arguments_section!(ResourceSection, ResourceItem);
argument_section!(ResourceTitleSection, OpenText);
arguments_section!(ResourceAuthorSection, OpenText);
argument_section!(ResourceOffsetSection, OpenText);
argument_section!(ResourceUrlSection, OpenText);
argument_section!(ResourceHomepageSection, OpenText);
argument_section!(ResourceTypeSection, OpenText);
argument_section!(ResourceEditionSection, OpenText);
argument_section!(ResourceEditorSection, OpenText);
argument_section!(ResourceInstitutionSection, OpenText);
argument_section!(ResourceJournalSection, OpenText);
argument_section!(ResourcePublisherSection, OpenText);
argument_section!(ResourceVolumeSection, OpenText);
argument_section!(ResourceMonthSection, OpenText);
argument_section!(ResourceYearSection, OpenText);
argument_section!(ResourceDescriptionSection, OpenText);
argument_section!(NotSection, Box<Clause>);
arguments_section!(AllOfSection, Clause);
arguments_section!(AnyOfSection, Clause);
arguments_section!(OneOfSection, Clause);
argument_section!(ExistsSection, BindingOrSpec);
arguments_section!(SuchThatSection, Clause);
argument_section!(ExistsUniqueSection, BindingOrSpec);
argument_section!(ForAllSection, BindingOrSpec);
arguments_section!(IfSection, Clause);
zero_or_more_arguments_section!(PiecewiseSection, OpenText);
arguments_section!(ElseSection, Clause);
argument_section!(GivenClauseSection, IsOrRefinedStatementSpec);
argument_section!(TitleSection, OpenText);
argument_section!(SectionSection, OpenText);
argument_section!(SubsectionSection, OpenText);
argument_section!(SubsubsectionSection, OpenText);

// ===============================[ items ]=====================================

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

// ===============================[ definition groups ]=====================================

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

// ===============================[ clause groups ]=====================================

/// Clause group representing logical negation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NotGroup {
    /// Optional local label heading.
    pub heading: Option<LabelHeader>,
    /// Required negated clause.
    pub not: NotSection,
}

/// Clause group requiring all child clauses.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AllOfGroup {
    /// Optional local label heading.
    pub heading: Option<LabelHeader>,
    /// Required nonempty child clause list.
    pub all_of: AllOfSection,
}

/// Clause group requiring any child clause.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AnyOfGroup {
    /// Optional local label heading.
    pub heading: Option<LabelHeader>,
    /// Required nonempty child clause list.
    pub any_of: AnyOfSection,
}

/// Clause group requiring exactly one child clause.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OneOfGroup {
    /// Optional local label heading.
    pub heading: Option<LabelHeader>,
    /// Required nonempty child clause list.
    pub one_of: OneOfSection,
}

/// Existential quantifier clause.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExistsGroup {
    /// Optional local label heading.
    pub heading: Option<LabelHeader>,
    /// Required bound variable/specification.
    pub exists: ExistsSection,
    /// Required predicate clauses.
    pub such_that: SuchThatSection,
}

/// Unique existential quantifier clause.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExistsUniqueGroup {
    /// Optional local label heading.
    pub heading: Option<LabelHeader>,
    /// Required uniquely bound variable/specification.
    pub exists_unique: ExistsUniqueSection,
    /// Required predicate clauses.
    pub such_that: SuchThatSection,
}

/// Universal quantifier clause.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ForAllGroup {
    /// Optional local label heading.
    pub heading: Option<LabelHeader>,
    /// Required bound variable/specification.
    pub for_all: ForAllSection,
    /// Optional guard clauses.
    pub where_: Option<WhereSection>,
    /// Required consequence clauses.
    pub then: ThenSection,
}

/// Implication clause.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IfGroup {
    /// Optional local label heading.
    pub heading: Option<LabelHeader>,
    /// Required antecedent clauses.
    pub if_: IfSection,
    /// Required consequent clauses.
    pub then: ThenSection,
}

/// Biconditional clause.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IffGroup {
    /// Optional local label heading.
    pub heading: Option<LabelHeader>,
    /// Required left/right condition clauses.
    pub iff: IffSection,
    /// Required equivalence clauses.
    pub then: ThenSection,
}

/// Piecewise conditional clause.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PiecewiseGroup {
    /// Optional local label heading.
    pub heading: Option<LabelHeader>,
    /// Optional piecewise prose.
    pub piecewise: PiecewiseSection,
    /// Required condition clauses.
    pub if_: IfSection,
    /// Required value/result clauses.
    pub then: ThenSection,
    /// Optional fallback clauses.
    pub else_: Option<ElseSection>,
}

/// Nested given/then clause group.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GivenGroup {
    /// Optional local label heading.
    pub heading: Option<LabelHeader>,
    /// Required given statement.
    pub given: GivenClauseSection,
    /// Optional local context clauses.
    pub where_: Option<WhereSection>,
    /// Required consequence clauses.
    pub then: ThenSection,
}

// ===============================[ support groups ]=====================================

/// Nested alias group inside an `Aliases:` section.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AliasGroup {
    /// Optional local label heading.
    pub heading: Option<LabelHeader>,
    /// Required alias body.
    pub alias: AliasSection,
    /// Optional rendered written form.
    pub written: Option<WrittenSection>,
}

/// Nested symbol group inside a `Provides:` section.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SymbolGroup {
    /// Optional local label heading.
    pub heading: Option<LabelHeader>,
    /// Required symbol alias body.
    pub symbol: SymbolSection,
    /// Optional rendered written form.
    pub written: Option<WrittenSection>,
}

/// Nested connection group inside a `Provides:` section.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConnectionGroup {
    /// Optional local label heading.
    pub heading: Option<LabelHeader>,
    /// Optional connection prose.
    pub connection: ConnectionSection,
    /// Required target text for the connection.
    pub to: ToSection,
    /// Optional `using:` specifications.
    pub using: Option<UsingSection>,
    /// Required meaning text.
    pub means: MeansSection,
    /// Optional signification text.
    pub signifies: Option<SignifiesSection>,
    /// Optional viewability text.
    pub viewable: Option<ViewableSection>,
    /// Optional through text.
    pub through: Option<ThroughSection>,
}

/// Documentation group containing math-mode written rendering text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WrittenGroup {
    /// Optional local label heading.
    pub heading: Option<LabelHeader>,
    /// Required written text entries.
    pub written: WrittenSection,
}

/// Documentation group containing plain-text called rendering text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CalledGroup {
    /// Optional local label heading.
    pub heading: Option<LabelHeader>,
    /// Required called text entries.
    pub called: CalledSection,
    /// Optional math-mode written text bundled with the called form.
    pub written: Option<WrittenSection>,
}

/// Documentation group defining a writing alias and its rendered text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WritingGroup {
    /// Optional local label heading.
    pub heading: Option<LabelHeader>,
    /// Required writing alias.
    pub writing: WritingSection,
    /// Required rendered alias text.
    pub as_: AsSection,
}

/// Documentation group containing overview prose.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OverviewGroup {
    /// Optional local label heading.
    pub heading: Option<LabelHeader>,
    /// Required overview text.
    pub overview: OverviewSection,
}

/// Documentation group containing related-entry prose.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RelatedGroup {
    /// Optional local label heading.
    pub heading: Option<LabelHeader>,
    /// Required related text entries.
    pub related: RelatedSection,
}

/// Documentation group containing discoverer prose.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiscovererGroup {
    /// Optional local label heading.
    pub heading: Option<LabelHeader>,
    /// Optional discoverer text entries.
    pub discoverer: DiscovererSection,
}

/// Justification label group with authoring note text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LabelGroup {
    /// Optional local label heading.
    pub heading: Option<LabelHeader>,
    /// Optional label text entries.
    pub label: LabelSection,
    /// Optional by text entries.
    pub by: BySection,
    /// Required comment text.
    pub comment: CommentSection,
}

/// Justification by group with authoring note text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ByGroup {
    /// Optional local label heading.
    pub heading: Option<LabelHeader>,
    /// Optional by text entries.
    pub by: BySection,
    /// Required comment text.
    pub comment: CommentSection,
}

/// Metadata group containing a stable identifier.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IdGroup {
    /// Required identifier text.
    pub id: IdSection,
}

/// Metadata group containing version text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VersionGroup {
    /// Required version text.
    pub version: VersionSection,
}

/// Top-level specification group containing numeric-domain items.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpecifyGroup {
    /// Required specification items.
    pub specify: SpecifySection,
}

// ===============================[ metadata resource groups ]=====================================

/// Specification item describing positive integers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PositiveIntGroup {
    /// Optional local label heading.
    pub heading: Option<LabelHeader>,
    /// Optional positive text.
    pub positive: PositiveSection,
    /// Optional integer text.
    pub int: IntSection,
    /// Optional descriptive text.
    pub is_: IsSection,
}

/// Specification item describing negative integers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NegativeIntGroup {
    /// Optional local label heading.
    pub heading: Option<LabelHeader>,
    /// Optional negative text.
    pub negative: NegativeSection,
    /// Optional integer text.
    pub int: IntSection,
    /// Optional descriptive text.
    pub is_: IsSection,
}

/// Specification item describing zero.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ZeroGroup {
    /// Optional local label heading.
    pub heading: Option<LabelHeader>,
    /// Optional zero text.
    pub zero: ZeroSection,
    /// Optional descriptive text.
    pub is_: IsSection,
}

/// Specification item describing positive decimals.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PositiveDecimalGroup {
    /// Optional local label heading.
    pub heading: Option<LabelHeader>,
    /// Optional positive text.
    pub positive: PositiveSection,
    /// Optional decimal text.
    pub decimal: DecimalSection,
    /// Optional descriptive text.
    pub is_: IsSection,
}

/// Specification item describing negative decimals.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NegativeDecimalGroup {
    /// Optional local label heading.
    pub heading: Option<LabelHeader>,
    /// Optional negative text.
    pub negative: NegativeSection,
    /// Optional decimal text.
    pub decimal: DecimalSection,
    /// Optional descriptive text.
    pub is_: IsSection,
}

/// Top-level person metadata group.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PersonGroup {
    /// Required author heading.
    pub heading: AuthorHeader,
    /// Optional person prose.
    pub person: PersonSection,
    /// Required name text entries.
    pub name: NameSection,
    /// Required biography text.
    pub biography: BiographySection,
}

/// Top-level resource metadata group.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResourceGroup {
    /// Required resource heading.
    pub heading: ResourceHeader,
    /// Required resource item entries.
    pub resource: ResourceSection,
}

/// Resource item containing a title.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResourceTitleGroup {
    /// Required title text.
    pub title: ResourceTitleSection,
}

/// Resource item containing one or more authors.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResourceAuthorGroup {
    /// Required author text entries.
    pub author: ResourceAuthorSection,
}

/// Resource item containing an offset such as page, chapter, or location.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResourceOffsetGroup {
    /// Required offset text.
    pub offset: ResourceOffsetSection,
}

/// Resource item containing a URL.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResourceUrlGroup {
    /// Required URL text.
    pub url: ResourceUrlSection,
}

/// Resource item containing a homepage URL.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResourceHomepageGroup {
    /// Required homepage text.
    pub homepage: ResourceHomepageSection,
}

/// Resource item containing a resource type.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResourceTypeGroup {
    /// Required type text.
    pub type_: ResourceTypeSection,
}

/// Resource item containing an edition.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResourceEditionGroup {
    /// Required edition text.
    pub edition: ResourceEditionSection,
}

/// Resource item containing an editor.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResourceEditorGroup {
    /// Required editor text.
    pub editor: ResourceEditorSection,
}

/// Resource item containing an institution.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResourceInstitutionGroup {
    /// Required institution text.
    pub institution: ResourceInstitutionSection,
}

/// Resource item containing a journal.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResourceJournalGroup {
    /// Required journal text.
    pub journal: ResourceJournalSection,
}

/// Resource item containing a publisher.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResourcePublisherGroup {
    /// Required publisher text.
    pub publisher: ResourcePublisherSection,
}

/// Resource item containing a volume.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResourceVolumeGroup {
    /// Required volume text.
    pub volume: ResourceVolumeSection,
}

/// Resource item containing a month.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResourceMonthGroup {
    /// Required month text.
    pub month: ResourceMonthSection,
}

/// Resource item containing a year.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResourceYearGroup {
    /// Required year text.
    pub year: ResourceYearSection,
}

/// Resource item containing a description.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResourceDescriptionGroup {
    /// Required description text.
    pub description: ResourceDescriptionSection,
}

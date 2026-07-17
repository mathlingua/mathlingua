use crate::frontend::formulation::ast::{
    AuthorHeader, CommandExpression, CommandHeader, DeclarationStatement, Expression,
    ExpressionAlias, ExpressionBinding, FormOrDeclaration, HardCastStatement, IsViaStatement,
    LabelHeader, ResourceHeader, SpecOperatorAlias, TopicHeader, TypeExpression, WritingAlias,
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
            type Item = T;
            type IntoIter = std::vec::IntoIter<T>;

            fn into_iter(self) -> Self::IntoIter {
                self.0.into_iter()
            }
        }

        impl<'a, T> IntoIterator for &'a $name<T> {
            type Item = &'a T;
            type IntoIter = std::slice::Iter<'a, T>;

            fn into_iter(self) -> Self::IntoIter {
                self.0.iter()
            }
        }

        impl<'a, T> IntoIterator for &'a mut $name<T> {
            type Item = &'a mut T;
            type IntoIter = std::slice::IterMut<'a, T>;

            fn into_iter(self) -> Self::IntoIter {
                self.0.iter_mut()
            }
        }
    };
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ZeroOrMore<T>(Vec<T>);

impl<T> Default for ZeroOrMore<T> {
    fn default() -> Self {
        Self(Vec::new())
    }
}

impl<T> ZeroOrMore<T> {
    pub fn into_vec(self) -> Vec<T> {
        self.0
    }
}

impl<T> From<Vec<T>> for ZeroOrMore<T> {
    fn from(value: Vec<T>) -> Self {
        Self(value)
    }
}

impl<T> From<OneOrMore<T>> for ZeroOrMore<T> {
    fn from(value: OneOrMore<T>) -> Self {
        Self(value.into_vec())
    }
}

impl_repeated_items!(ZeroOrMore);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OneOrMore<T>(Vec<T>);

impl<T> OneOrMore<T> {
    pub fn new(first: T, rest: Vec<T>) -> Self {
        let mut items = Vec::with_capacity(rest.len() + 1);
        items.push(first);
        items.extend(rest);
        Self(items)
    }

    pub fn into_vec(self) -> Vec<T> {
        self.0
    }
}

impl<T> TryFrom<Vec<T>> for OneOrMore<T> {
    type Error = Vec<T>;

    fn try_from(value: Vec<T>) -> Result<Self, Self::Error> {
        if value.is_empty() {
            Err(value)
        } else {
            Ok(Self(value))
        }
    }
}

impl<T> TryFrom<ZeroOrMore<T>> for OneOrMore<T> {
    type Error = ZeroOrMore<T>;

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
        #[derive(Clone, Debug, PartialEq, Eq)]
        pub struct $name {
            pub argument: $ty,
        }
    };
}

macro_rules! arguments_section {
    ($name:ident, $ty:ty) => {
        #[derive(Clone, Debug, PartialEq, Eq)]
        pub struct $name {
            /// Nonempty parsed arguments for this section.
            pub arguments: OneOrMore<$ty>,
        }
    };
}

macro_rules! zero_or_more_arguments_section {
    ($name:ident, $ty:ty) => {
        #[derive(Clone, Debug, PartialEq, Eq)]
        pub struct $name {
            /// Parsed arguments, possibly empty.
            pub arguments: ZeroOrMore<$ty>,
        }
    };
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OpenText(pub String);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WrittenText(pub String);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CalledText(pub String);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AdjectiveText(pub String);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WritingText(pub String);

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DescribesTarget {
    Form(FormOrDeclaration),
    Declaration(DeclarationStatement),
}

/// One side of a top-level `Relation:` (`between:`/`and:`). A relationship may
/// hold between declared concepts, documentation topics, or definitions, in any
/// combination, so each side is either an ordinary declaration (`a is \real`) or a
/// quoted-text `Reference` — a `"#topic"` or a `"\signature"` (see [`TopicRelatedItem`]
/// for the signature convention). Quoting keeps a `\signature` reference distinct
/// from a usage.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RelationSubject {
    Declaration(Box<DeclarationStatement>),
    Reference(OpenText),
}

/// The `means:` of a top-level `Relation:`. It is either a logical `Statement`
/// (a clause) or a quoted-text prose `Text` description of the relationship.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RelationMeans {
    Statement(Box<Clause>),
    Text(OpenText),
}

argument_section!(DescribesSection, DescribesTarget);
arguments_section!(UsingSection, DeclarationStatement);
arguments_section!(WhenSection, Clause);
argument_section!(ExtendsSection, IsOrViaItem);
arguments_section!(DescribesSpecifiesSection, IsOrViaItem);
arguments_section!(SatisfiesSection, Clause);
arguments_section!(RequiresSection, RequiresItem);
arguments_section!(EnablesSection, EnablesItem);
arguments_section!(JustifiedSection, JustifiedItem);
arguments_section!(DocumentedSection, DocumentedItem);
arguments_section!(AliasesSection, AliasItem);
arguments_section!(ReferencesSection, ResourceHeader);
arguments_section!(MetadataSection, MetadataItem);
argument_section!(DefinesSection, DeclarationStatement);
arguments_section!(ExpressesSection, Clause);
argument_section!(RefinesSection, DeclarationStatement);
argument_section!(RefinesExtendsSection, DeclarationStatement);
zero_or_more_arguments_section!(StatesSection, OpenText);
arguments_section!(ThatSection, Clause);
zero_or_more_arguments_section!(EquivalentSection, OpenText);
arguments_section!(EquivalentToSection, Expression);
// The `Axiom:`/`Theorem:`/`Corollary:` head sections take
// no argument; a result's name lives in `Documented:` `called:` (as for definitions).
zero_or_more_arguments_section!(OfSection, OpenText);
arguments_section!(GivenSection, DeclarationStatement);
arguments_section!(WhereSection, Clause);
arguments_section!(ThenSection, Clause);
arguments_section!(IffSection, Clause);
argument_section!(AliasSection, AliasKind);
arguments_section!(WrittenSection, WrittenText);
argument_section!(CapabilitySection, AliasKind);
argument_section!(DefinitionSection, DefinitionRequirement);
argument_section!(FromSection, DeclarationStatement);
argument_section!(CastAsSection, ExpressionBinding);
zero_or_more_arguments_section!(EnablesRelationSection, OpenText);
argument_section!(RelationToSection, RelationshipDeclaration);
arguments_section!(RelationWhenSection, RelationWhenItem);
arguments_section!(RelationRepresentsSection, RelationKind);
argument_section!(RelationshipMeansSection, Clause);
// Top-level `Relation:` item sections (distinct from the `Enables: relation:` group above).
zero_or_more_arguments_section!(RelationSection, OpenText);
argument_section!(RelationBetweenSection, RelationSubject);
argument_section!(RelationAndSection, RelationSubject);
argument_section!(RelationMeansSection, RelationMeans);
// Top-level `Topic:` item sections. References (`within:`/`to:`) are quoted text
// so a `#topic` or a bare `\signature` reads as a reference, never a usage.
// (`TopicRelated*` is distinct from the `related:` documentation item below.)
zero_or_more_arguments_section!(TopicSection, OpenText);
argument_section!(TopicWithinSection, OpenText);
arguments_section!(TopicRelatedToSection, OpenText);
argument_section!(TopicRelatedMeansSection, OpenText);
arguments_section!(TopicRelatedSection, TopicRelatedItem);
arguments_section!(CalledSection, CalledText);
arguments_section!(AdjectiveSection, AdjectiveText);
argument_section!(WritingSection, WritingAlias);
arguments_section!(AsSection, WritingText);
argument_section!(OverviewSection, OpenText);
argument_section!(DescriptionSection, OpenText);
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
arguments_section!(PersonSection, OpenText);
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
arguments_section!(EquivalentlySection, Clause);
arguments_section!(AnyOfSection, Clause);
arguments_section!(OneOfSection, Clause);
arguments_section!(ExistsSection, BindingOrSpec);
arguments_section!(SuchThatSection, Clause);
arguments_section!(ExistsUniqueSection, BindingOrSpec);
arguments_section!(ForAllSection, BindingOrSpec);
arguments_section!(IfSection, Clause);
zero_or_more_arguments_section!(PiecewiseSection, OpenText);
arguments_section!(ElseSection, Clause);
arguments_section!(GivenClauseSection, DeclarationStatement);
argument_section!(TitleSection, OpenText);
argument_section!(SectionTitleSection, OpenText);
argument_section!(SubsectionTitleSection, OpenText);
argument_section!(TextSection, OpenText);
arguments_section!(TopLevelWritingSection, WritingAlias);

// ===============================[ items ]=====================================

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Document {
    pub items: ZeroOrMore<TopLevelItem>,
}

#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TopLevelItem {
    Title(TitleGroup),
    SectionTitle(SectionTitleGroup),
    SubsectionTitle(SubsectionTitleGroup),
    Text(TextGroup),
    Writing(TopLevelWritingGroup),
    Disambiguates(DisambiguatesGroup),
    Describes(DescribesGroup),
    Defines(DefinesGroup),
    Refines(RefinesGroup),
    States(StatesGroup),
    Axiom(AxiomGroup),
    Theorem(TheoremGroup),
    Corollary(CorollaryGroup),
    Person(PersonGroup),
    Resource(ResourceGroup),
    Specify(SpecifyGroup),
    Relation(RelationGroup),
    Equivalent(EquivalentGroup),
    Topic(TopicGroup),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Clause {
    Not(NotGroup),
    AllOf(AllOfGroup),
    AnyOf(AnyOfGroup),
    OneOf(OneOfGroup),
    Exists(ExistsGroup),
    ExistsUnique(ExistsUniqueGroup),
    ForAll(ForAllGroup),
    If(IfGroup),
    Iff(IffGroup),
    Equivalently(EquivalentlyGroup),
    Piecewise(PiecewiseGroup),
    Given(GivenGroup),
    Declaration(DeclarationStatement),
    Expression(Expression),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IsOrViaItem {
    IsVia(IsViaStatement),
    Declaration(DeclarationStatement),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BindingOrSpec {
    Declaration(DeclarationStatement),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AliasKind {
    Expression(ExpressionAlias),
    SpecOperator(SpecOperatorAlias),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AliasItem {
    Alias(AliasGroup),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RequiresItem {
    Capability(Box<CapabilityGroup>),
    Definition(DefinitionGroup),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EnablesItem {
    Capability(Box<CapabilityGroup>),
    FromCapability(Box<FromCapabilityGroup>),
    FromAs(Box<FromAsGroup>),
    Relation(Box<EnablesRelationGroup>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DocumentedItem {
    Written(WrittenGroup),
    Called(CalledGroup),
    Adjective(AdjectiveGroup),
    Writing(WritingGroup),
    Overview(OverviewGroup),
    Description(DescriptionGroup),
    Related(RelatedGroup),
    Discoverer(DiscovererGroup),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum JustifiedItem {
    Label(LabelGroup),
    By(ByGroup),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MetadataItem {
    Id(IdGroup),
    Version(VersionGroup),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SpecifyItem {
    PositiveInt(PositiveIntGroup),
    NegativeInt(NegativeIntGroup),
    Zero(ZeroGroup),
    PositiveDecimal(PositiveDecimalGroup),
    NegativeDecimal(NegativeDecimalGroup),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ResourceItem {
    Title(ResourceTitleGroup),
    Author(ResourceAuthorGroup),
    Offset(ResourceOffsetGroup),
    Url(ResourceUrlGroup),
    Homepage(ResourceHomepageGroup),
    Type(ResourceTypeGroup),
    Edition(ResourceEditionGroup),
    Editor(ResourceEditorGroup),
    Institution(ResourceInstitutionGroup),
    Journal(ResourceJournalGroup),
    Publisher(ResourcePublisherGroup),
    Volume(ResourceVolumeGroup),
    Month(ResourceMonthGroup),
    Year(ResourceYearGroup),
    Description(ResourceDescriptionGroup),
}

// ===============================[ definition groups ]=====================================

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TitleGroup {
    pub title: TitleSection,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SectionTitleGroup {
    pub section_title: SectionTitleSection,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SubsectionTitleGroup {
    pub subsection_title: SubsectionTitleSection,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextGroup {
    pub text: TextSection,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TopLevelWritingGroup {
    pub writing: TopLevelWritingSection,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DisambiguatesGroup {
    pub heading: FormOrDeclaration,
    pub branches: Vec<DisambiguatesBranch>,
    pub else_: Option<DisambiguatesElseSection>,
    pub justified: Option<JustifiedSection>,
    pub documented: Option<DocumentedSection>,
    pub aliases: Option<AliasesSection>,
    pub references: Option<ReferencesSection>,
    pub metadata: Option<MetadataSection>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DisambiguatesBranch {
    pub when: WhenSection,
    pub to: DisambiguatesToSection,
}

argument_section!(DisambiguatesToSection, Expression);
argument_section!(DisambiguatesElseSection, Expression);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DescribesGroup {
    pub heading: CommandHeader,
    pub describes: DescribesSection,
    pub using: Option<UsingSection>,
    pub when: Option<WhenSection>,
    pub extends: Option<ExtendsSection>,
    pub specifies: Option<DescribesSpecifiesSection>,
    pub satisfies: Option<SatisfiesSection>,
    pub requires: Option<RequiresSection>,
    pub enables: Option<EnablesSection>,
    pub justified: Option<JustifiedSection>,
    pub documented: Option<DocumentedSection>,
    pub aliases: Option<AliasesSection>,
    pub references: Option<ReferencesSection>,
    pub metadata: Option<MetadataSection>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DefinesGroup {
    pub heading: CommandHeader,
    pub defines: DefinesSection,
    pub using: Option<UsingSection>,
    pub when: Option<WhenSection>,
    pub expresses: Option<ExpressesSection>,
    pub requires: Option<RequiresSection>,
    pub enables: Option<EnablesSection>,
    pub justified: Option<JustifiedSection>,
    pub documented: Option<DocumentedSection>,
    pub aliases: Option<AliasesSection>,
    pub references: Option<ReferencesSection>,
    pub metadata: Option<MetadataSection>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RefinesGroup {
    pub heading: CommandHeader,
    pub refines: RefinesSection,
    pub using: Option<UsingSection>,
    pub when: Option<WhenSection>,
    pub extends: Option<RefinesExtendsSection>,
    pub satisfies: Option<SatisfiesSection>,
    pub requires: Option<RequiresSection>,
    pub enables: Option<EnablesSection>,
    pub justified: Option<JustifiedSection>,
    pub documented: Option<DocumentedSection>,
    pub aliases: Option<AliasesSection>,
    pub references: Option<ReferencesSection>,
    pub metadata: Option<MetadataSection>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StatesGroup {
    pub heading: CommandHeader,
    pub states: StatesSection,
    pub using: Option<UsingSection>,
    pub when: Option<WhenSection>,
    pub that: ThatSection,
    pub requires: Option<RequiresSection>,
    pub enables: Option<EnablesSection>,
    pub justified: Option<JustifiedSection>,
    pub documented: Option<DocumentedSection>,
    pub aliases: Option<AliasesSection>,
    pub references: Option<ReferencesSection>,
    pub metadata: Option<MetadataSection>,
}

/// A top-level `Equivalent:` item. Its `[...]` command heading names an
/// equivalence class; each `to:` command (which must use the header parameters
/// exactly) is asserted to be interchangeable with the others. Unlike the
/// definition-like groups it has no `Enables:`/`Requires:`/`Aliases:`/`Metadata:`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EquivalentGroup {
    pub heading: CommandHeader,
    pub equivalent: EquivalentSection,
    pub using: Option<UsingSection>,
    pub when: Option<WhenSection>,
    pub to: EquivalentToSection,
    pub justified: Option<JustifiedSection>,
    pub documented: Option<DocumentedSection>,
    pub references: Option<ReferencesSection>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AxiomGroup {
    pub heading: Option<CommandHeader>,
    pub given: Option<GivenSection>,
    pub where_: Option<WhereSection>,
    pub then: ThenSection,
    pub iff: Option<IffSection>,
    pub justified: Option<JustifiedSection>,
    pub documented: Option<DocumentedSection>,
    pub aliases: Option<AliasesSection>,
    pub references: Option<ReferencesSection>,
    pub metadata: Option<MetadataSection>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TheoremGroup {
    pub heading: Option<CommandHeader>,
    pub given: Option<GivenSection>,
    pub where_: Option<WhereSection>,
    pub then: ThenSection,
    pub iff: Option<IffSection>,
    pub justified: Option<JustifiedSection>,
    pub documented: Option<DocumentedSection>,
    pub aliases: Option<AliasesSection>,
    pub references: Option<ReferencesSection>,
    pub metadata: Option<MetadataSection>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CorollaryGroup {
    pub heading: Option<CommandHeader>,
    pub of: OfSection,
    pub given: Option<GivenSection>,
    pub where_: Option<WhereSection>,
    pub then: ThenSection,
    pub iff: Option<IffSection>,
    pub justified: Option<JustifiedSection>,
    pub documented: Option<DocumentedSection>,
    pub aliases: Option<AliasesSection>,
    pub references: Option<ReferencesSection>,
    pub metadata: Option<MetadataSection>,
}

/// A top-level `Relation:` item, which states a bidirectional relationship
/// between the two concepts declared in `between:` and `and:`. Unlike the
/// directional `Enables: relation:` group ([`EnablesRelationGroup`]), it is a
/// heading-less, standalone item and does not register a cast/view rule.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RelationGroup {
    pub relation: RelationSection,
    pub using: Option<UsingSection>,
    pub between: RelationBetweenSection,
    pub and_: RelationAndSection,
    pub when: Option<WhenSection>,
    pub means: Option<RelationMeansSection>,
    pub justified: Option<JustifiedSection>,
    pub documented: Option<DocumentedSection>,
    pub aliases: Option<AliasesSection>,
    pub references: Option<ReferencesSection>,
    pub metadata: Option<MetadataSection>,
}

/// A top-level `Topic:` item, which names a documentation topic. Its `[#some.name]`
/// heading is a dotted, `#`-sigil path that renders as a human title (for example
/// `#real.analysis` renders as "Real Analysis") unless `Documented:called:` gives an
/// explicit rendering. The optional `within:` names a parent topic (making this a
/// sub-topic) as a quoted `"#..."` reference, and the optional `Related:` records
/// how this topic relates to other topics or definitions. It is stated, not proven,
/// and registers no type facts.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TopicGroup {
    pub heading: TopicHeader,
    pub topic: TopicSection,
    pub within: Option<TopicWithinSection>,
    pub related: Option<TopicRelatedSection>,
    pub documented: Option<DocumentedSection>,
}

/// One entry of a `Topic:`'s `Related:` section. Each entry points at one or more
/// other topics or definitions via `to:` — quoted `"#topic"` references or
/// `"\signature"` references (a `\command` with its arguments removed, such as
/// `\function:on:to`) — and explains the relationship in `means:`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TopicRelatedItem {
    pub to: TopicRelatedToSection,
    pub means: TopicRelatedMeansSection,
}

// ===============================[ clause groups ]=====================================

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NotGroup {
    pub heading: Option<LabelHeader>,
    pub not: NotSection,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AllOfGroup {
    pub heading: Option<LabelHeader>,
    pub all_of: AllOfSection,
}

/// An `equivalently:` clause — sugar for a chain of `iff`s asserting that all of
/// its sub-clauses are equivalent. Purely a checking convenience; each sub-clause
/// is validated like an `allOf:` entry.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EquivalentlyGroup {
    pub heading: Option<LabelHeader>,
    pub equivalently: EquivalentlySection,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AnyOfGroup {
    pub heading: Option<LabelHeader>,
    pub any_of: AnyOfSection,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OneOfGroup {
    pub heading: Option<LabelHeader>,
    pub one_of: OneOfSection,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExistsGroup {
    pub heading: Option<LabelHeader>,
    pub exists: ExistsSection,
    pub such_that: Option<SuchThatSection>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExistsUniqueGroup {
    pub heading: Option<LabelHeader>,
    pub exists_unique: ExistsUniqueSection,
    pub such_that: Option<SuchThatSection>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ForAllGroup {
    pub heading: Option<LabelHeader>,
    pub for_all: ForAllSection,
    pub where_: Option<WhereSection>,
    pub then: ThenSection,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IfGroup {
    pub heading: Option<LabelHeader>,
    pub if_: IfSection,
    pub then: ThenSection,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IffGroup {
    pub heading: Option<LabelHeader>,
    pub iff: IffSection,
    pub then: ThenSection,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PiecewiseGroup {
    pub heading: Option<LabelHeader>,
    pub piecewise: PiecewiseSection,
    pub if_: IfSection,
    pub then: ThenSection,
    pub else_: Option<ElseSection>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GivenGroup {
    pub heading: Option<LabelHeader>,
    pub given: GivenClauseSection,
    pub where_: Option<WhereSection>,
    pub then: ThenSection,
}

// ===============================[ support groups ]=====================================

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AliasGroup {
    pub heading: Option<LabelHeader>,
    pub alias: AliasSection,
    pub written: Option<WrittenSection>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CapabilityGroup {
    pub heading: Option<LabelHeader>,
    pub capability: CapabilitySection,
    pub written: Option<WrittenSection>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DefinitionRequirement {
    pub command: CommandExpression,
    pub ty: TypeExpression,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DefinitionGroup {
    pub heading: Option<LabelHeader>,
    pub definition: DefinitionSection,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FromCapabilityGroup {
    pub heading: Option<LabelHeader>,
    pub from: FromSection,
    pub capability: CapabilitySection,
    pub written: Option<WrittenSection>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FromAsGroup {
    pub heading: Option<LabelHeader>,
    pub from: FromSection,
    pub as_: CastAsSection,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RelationshipDeclaration {
    Command(CommandExpression),
    Declaration(DeclarationStatement),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RelationWhenItem {
    Declaration(DeclarationStatement),
    HardCast(HardCastStatement),
}

/// The kind of type-system cast a nested `Enables: relation:` registers via its
/// `represents:` field: `\\coercion` (a value of the described type may be used
/// wherever the related type is expected, and `as` casts to it) or `\\encoding`
/// (an abstraction boundary — no automatic coercion; only an explicit `as!` cast
/// drops through the encoding). The variants mirror those surface keywords.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RelationKind {
    Coercion,
    Encoding,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EnablesRelationGroup {
    pub heading: Option<LabelHeader>,
    pub relation: EnablesRelationSection,
    pub to: RelationToSection,
    pub when: Option<RelationWhenSection>,
    pub means: Option<RelationshipMeansSection>,
    pub represents: Option<RelationRepresentsSection>,
    pub by: Option<BySection>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WrittenGroup {
    pub heading: Option<LabelHeader>,
    pub written: WrittenSection,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CalledGroup {
    pub heading: Option<LabelHeader>,
    pub called: CalledSection,
    pub written: Option<WrittenSection>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AdjectiveGroup {
    pub heading: Option<LabelHeader>,
    pub adjective: AdjectiveSection,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WritingGroup {
    pub heading: Option<LabelHeader>,
    pub writing: WritingSection,
    pub as_: AsSection,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OverviewGroup {
    pub heading: Option<LabelHeader>,
    pub overview: OverviewSection,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DescriptionGroup {
    pub heading: Option<LabelHeader>,
    pub description: DescriptionSection,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RelatedGroup {
    pub heading: Option<LabelHeader>,
    pub related: RelatedSection,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiscovererGroup {
    pub heading: Option<LabelHeader>,
    pub discoverer: DiscovererSection,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LabelGroup {
    pub heading: Option<LabelHeader>,
    pub label: LabelSection,
    pub by: BySection,
    pub comment: CommentSection,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ByGroup {
    pub heading: Option<LabelHeader>,
    pub by: BySection,
    pub comment: CommentSection,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IdGroup {
    pub id: IdSection,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VersionGroup {
    pub version: VersionSection,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpecifyGroup {
    pub specify: SpecifySection,
}

// ===============================[ metadata resource groups ]=====================================

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PositiveIntGroup {
    pub heading: Option<LabelHeader>,
    pub positive: PositiveSection,
    pub int: IntSection,
    pub is_: IsSection,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NegativeIntGroup {
    pub heading: Option<LabelHeader>,
    pub negative: NegativeSection,
    pub int: IntSection,
    pub is_: IsSection,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ZeroGroup {
    pub heading: Option<LabelHeader>,
    pub zero: ZeroSection,
    pub is_: IsSection,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PositiveDecimalGroup {
    pub heading: Option<LabelHeader>,
    pub positive: PositiveSection,
    pub decimal: DecimalSection,
    pub is_: IsSection,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NegativeDecimalGroup {
    pub heading: Option<LabelHeader>,
    pub negative: NegativeSection,
    pub decimal: DecimalSection,
    pub is_: IsSection,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PersonGroup {
    pub heading: AuthorHeader,
    pub person: PersonSection,
    pub biography: Option<BiographySection>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResourceGroup {
    pub heading: ResourceHeader,
    pub resource: ResourceSection,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResourceTitleGroup {
    pub title: ResourceTitleSection,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResourceAuthorGroup {
    pub author: ResourceAuthorSection,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResourceOffsetGroup {
    pub offset: ResourceOffsetSection,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResourceUrlGroup {
    pub url: ResourceUrlSection,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResourceHomepageGroup {
    pub homepage: ResourceHomepageSection,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResourceTypeGroup {
    pub type_: ResourceTypeSection,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResourceEditionGroup {
    pub edition: ResourceEditionSection,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResourceEditorGroup {
    pub editor: ResourceEditorSection,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResourceInstitutionGroup {
    pub institution: ResourceInstitutionSection,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResourceJournalGroup {
    pub journal: ResourceJournalSection,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResourcePublisherGroup {
    pub publisher: ResourcePublisherSection,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResourceVolumeGroup {
    pub volume: ResourceVolumeSection,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResourceMonthGroup {
    pub month: ResourceMonthSection,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResourceYearGroup {
    pub year: ResourceYearSection,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResourceDescriptionGroup {
    pub description: ResourceDescriptionSection,
}

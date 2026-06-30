use crate::frontend::formulation::ast::{
    AuthorHeader, CommandExpression, CommandHeader, DeclarationStatement, Expression,
    ExpressionAlias, FormOrDeclaration, IsViaStatement, LabelHeader, ResourceHeader,
    SpecOperatorAlias, TypeExpression, WritingAlias,
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
argument_section!(ExpressesSection, Clause);
argument_section!(RefinesSection, DeclarationStatement);
argument_section!(RefinesExtendsSection, DeclarationStatement);
zero_or_more_arguments_section!(StatesSection, OpenText);
arguments_section!(ThatSection, Clause);
zero_or_more_arguments_section!(AxiomSection, OpenText);
zero_or_more_arguments_section!(TheoremSection, OpenText);
zero_or_more_arguments_section!(CorollarySection, OpenText);
zero_or_more_arguments_section!(OfSection, OpenText);
zero_or_more_arguments_section!(LemmaSection, OpenText);
zero_or_more_arguments_section!(ConjectureSection, OpenText);
arguments_section!(GivenSection, DeclarationStatement);
arguments_section!(WhereSection, Clause);
arguments_section!(ThenSection, Clause);
arguments_section!(IffSection, Clause);
argument_section!(AliasSection, AliasKind);
arguments_section!(WrittenSection, WrittenText);
argument_section!(CapabilitySection, AliasKind);
argument_section!(DefinitionSection, DefinitionRequirement);
zero_or_more_arguments_section!(ConnectionSection, OpenText);
zero_or_more_arguments_section!(ToSection, OpenText);
zero_or_more_arguments_section!(MeansSection, OpenText);
zero_or_more_arguments_section!(SignifiesSection, OpenText);
zero_or_more_arguments_section!(ViewableSection, OpenText);
zero_or_more_arguments_section!(ThroughSection, OpenText);
arguments_section!(CalledSection, CalledText);
arguments_section!(AdjectiveSection, AdjectiveText);
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
argument_section!(GivenClauseSection, DeclarationStatement);
argument_section!(TitleSection, OpenText);
argument_section!(SectionTitleSection, OpenText);
argument_section!(SubsectionTitleSection, OpenText);
argument_section!(TextSection, OpenText);

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
    Disambiguates(DisambiguatesGroup),
    Describes(DescribesGroup),
    Defines(DefinesGroup),
    Refines(RefinesGroup),
    States(StatesGroup),
    Axiom(AxiomGroup),
    Theorem(TheoremGroup),
    Corollary(CorollaryGroup),
    Lemma(LemmaGroup),
    Conjecture(ConjectureGroup),
    Person(PersonGroup),
    Resource(ResourceGroup),
    Specify(SpecifyGroup),
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
    Connection(ConnectionGroup),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DocumentedItem {
    Written(WrittenGroup),
    Called(CalledGroup),
    Adjective(AdjectiveGroup),
    Writing(WritingGroup),
    Overview(OverviewGroup),
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AxiomGroup {
    pub heading: Option<CommandHeader>,
    pub axiom: AxiomSection,
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
    pub theorem: TheoremSection,
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
    pub corollary: CorollarySection,
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LemmaGroup {
    pub heading: Option<CommandHeader>,
    pub lemma: LemmaSection,
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
pub struct ConjectureGroup {
    pub heading: Option<CommandHeader>,
    pub conjecture: ConjectureSection,
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
pub struct ConnectionGroup {
    pub heading: Option<LabelHeader>,
    pub connection: ConnectionSection,
    pub to: ToSection,
    pub using: Option<UsingSection>,
    pub means: MeansSection,
    pub signifies: Option<SignifiesSection>,
    pub viewable: Option<ViewableSection>,
    pub through: Option<ThroughSection>,
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
    pub name: NameSection,
    pub biography: BiographySection,
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

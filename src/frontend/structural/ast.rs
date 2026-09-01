use crate::frontend::formulation::ast::{
    AuthorHeader, CommandExpression, CommandHeader, DeclarationStatement, Expression,
    ExpressionAlias, ExpressionBinding, FormOrDeclaration, IsViaStatement, LabelHeader,
    ResourceHeader, SpecOperatorAlias, TopicHeader, TypeExpression, WritingAlias,
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

/// The left side of a documented mapping-rendering rule.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MappingWritingTarget {
    /// The exact placeholder-bearing mapping form from `Declares:` (for example,
    /// `x(i_)`). This controls how the mapping value itself is rendered.
    Mapping(FormOrDeclaration),
    /// The same mapping form with its placeholders replaced by matching ordinary
    /// names (for example, `x(i)`). This controls invocation rendering.
    Invocation(Expression),
}

/// The form or declaration a `Declares:` group describes.
///
/// A [`DeclaresTarget::Declaration`] target may carry an `is`/specification
/// relation, which states the type the definition extends — `Declares: A is \set`
/// makes every `A` of the defined type a `\set`. A bare
/// [`DeclaresTarget::Form`] target extends nothing on its own; it may still name
/// what it extends in an [`ExtendsSection`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DeclaresTarget {
    Form(FormOrDeclaration),
    Declaration(DeclarationStatement),
}

/// One clause of a definition's subtype declaration: the type the definition
/// extends, with the optional `via` view used to regard it as that type.
///
/// A single clause is normally written on the `Declares:` target itself
/// (`Declares: G ::= (X, *, e) is \monoid via (X, *)`). An `extends:` section
/// exists for the case a target cannot express: extending several types at once,
/// each through a different view of the same tuple.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExtendsItem {
    pub statement: DeclarationStatement,
    pub via: Option<FormOrDeclaration>,
}

/// One subtype clause of a `Declares` group, borrowed from whichever spelling
/// carried it. See [`extends_clauses`].
#[derive(Clone, Copy, Debug)]
pub struct ExtendsClause<'a> {
    pub statement: &'a DeclarationStatement,
    pub via: Option<&'a FormOrDeclaration>,
}

/// The types a `Declares` group extends, from either spelling.
///
/// `Declares: X is \foo` and `Declares: X` with `extends: X is \foo` mean the same
/// thing, so every consumer works from this normalized list rather than from one
/// spelling or the other. Writing both is rejected while parsing, so at most one
/// source contributes.
pub fn extends_clauses<'a>(
    declares: &'a DeclaresSection,
    extends: Option<&'a ExtendsSection>,
) -> Vec<ExtendsClause<'a>> {
    if let Some(extends) = extends {
        return extends
            .arguments
            .iter()
            .map(|item| ExtendsClause {
                statement: &item.statement,
                via: item.via.as_ref(),
            })
            .collect();
    }

    match &declares.argument {
        DeclaresTarget::Declaration(statement) if statement.relation.is_some() => {
            vec![ExtendsClause {
                statement,
                via: declares.via.as_ref(),
            }]
        }
        _ => Vec::new(),
    }
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

/// The `specifies:` of a top-level `Relation:`. It is either a logical `Statement`
/// (a clause) or a quoted-text prose `Text` description of the relationship.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RelationSpecifies {
    Statement(Box<Clause>),
    Text(OpenText),
}

/// The `Declares:` section: the described target plus the optional `via` view of
/// the type it extends, as in `Declares: G ::= (X, *, e) is \monoid via (X, *)`.
/// The `via` form is only meaningful together with an `is` relation on the
/// target, and a target that states a relation excludes an [`ExtendsSection`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeclaresSection {
    pub argument: DeclaresTarget,
    pub via: Option<FormOrDeclaration>,
}

arguments_section!(UsingSection, DeclarationStatement);
arguments_section!(WhenSection, Clause);
arguments_section!(ExtendsSection, ExtendsItem);
arguments_section!(DeclaresSpecifiesSection, IsOrViaItem);
arguments_section!(DefinesSpecifiesSection, IsOrViaItem);
argument_section!(RealizesSection, DeclarationStatement);
arguments_section!(SatisfiesSection, Clause);
arguments_section!(RequiresSection, RequiresItem);
arguments_section!(EnablesSection, EnablesItem);
arguments_section!(JustificationSection, HaveGroup);
arguments_section!(DocumentedSection, DocumentedItem);
arguments_section!(AliasesSection, AliasItem);
arguments_section!(ReferencesSection, ResourceHeader);
arguments_section!(MetadataSection, MetadataItem);
argument_section!(DefinesSection, DeclarationStatement);
arguments_section!(ExpressesSection, Clause);
argument_section!(RefinesSection, DeclarationStatement);
argument_section!(RefinesSpecifiesSection, DeclarationStatement);
arguments_section!(ThatSection, Clause);
arguments_section!(EquivalentToSection, Expression);
// The `Axiom:`/`Theorem:`/`Conjecture:` head sections take no argument; a
// result's name lives in `Documented:` `called:` (as for definitions).
arguments_section!(GivenSection, DeclarationStatement);
arguments_section!(WhereSection, Clause);
arguments_section!(ThenSection, Clause);
arguments_section!(IffSection, Clause);
arguments_section!(HaveSection, Clause);
arguments_section!(AssertingSection, Clause);
arguments_section!(BecauseSection, Clause);
arguments_section!(HaveBySection, Expression);
argument_section!(AliasSection, AliasKind);
arguments_section!(WrittenSection, WrittenText);
argument_section!(CapabilitySection, AliasKind);
argument_section!(DefinitionSection, DefinitionRequirement);
argument_section!(FromSection, DeclarationStatement);
argument_section!(CastAsSection, ExpressionBinding);
argument_section!(ViewAsSection, DeclarationStatement);
argument_section!(ViewSignifiesSection, Clause);
// Top-level `Relation:` item sections.
zero_or_more_arguments_section!(RelationSection, OpenText);
argument_section!(RelationBetweenSection, RelationSubject);
argument_section!(RelationAndSection, RelationSubject);
argument_section!(RelationSpecifiesSection, RelationSpecifies);
// Top-level `Topic:` item sections. References (`within:`/`to:`) are quoted text
// so a `#topic` or a bare `\signature` reads as a reference, never a usage.
// (`TopicRelated*` is distinct from the `related:` documentation item below.)
zero_or_more_arguments_section!(TopicSection, OpenText);
argument_section!(TopicWithinSection, OpenText);
arguments_section!(TopicRelatedToSection, OpenText);
argument_section!(TopicRelatedSpecifiesSection, OpenText);
arguments_section!(TopicRelatedSection, TopicRelatedItem);
arguments_section!(CalledSection, CalledText);
arguments_section!(AdjectiveSection, AdjectiveText);
argument_section!(WritingSection, MappingWritingTarget);
// An item-level `Writing:` section (after `Aliases:`). Each entry is a
// double-quoted `"name :~> body"` alias that overrides, for this item only, how
// the named identifier is rendered by the collection-wide `Writing:` group.
arguments_section!(ItemWritingSection, WritingAlias);
arguments_section!(AsSection, WritingText);
argument_section!(OverviewSection, OpenText);
argument_section!(DescriptionSection, OpenText);
arguments_section!(RelatedSection, OpenText);
zero_or_more_arguments_section!(DiscovererSection, OpenText);
// `notes:` documentation item: prose reminders (markdown) kept with an item —
// most useful on the opaque `Text*` placeholders, recording how to fill in a
// structured form later.
arguments_section!(NotesSection, OpenText);
zero_or_more_arguments_section!(BySection, OpenText);
argument_section!(IdSection, OpenText);
argument_section!(VersionSection, OpenText);
// The markdown-with-LaTeX body of an opaque `Text*` placeholder group.
argument_section!(TextItemSection, OpenText);
arguments_section!(SpecifySection, SpecifyItem);
argument_section!(NumericSpecificationIsSection, TypeExpression);
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
arguments_section!(LetSection, BindingOrSpec);
arguments_section!(IfSection, Clause);
arguments_section!(ElseIfSection, Clause);
arguments_section!(ElseSection, Clause);
argument_section!(TitleSection, OpenText);
argument_section!(SectionTitleSection, OpenText);
argument_section!(SubsectionTitleSection, OpenText);
argument_section!(TextSection, OpenText);
arguments_section!(TopLevelWritingSection, WritingAlias);
zero_or_more_arguments_section!(ExampleSection, ExampleItem);

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
    Example(ExampleGroup),
    Disambiguates(DisambiguatesGroup),
    Declares(DeclaresGroup),
    Defines(DefinesGroup),
    Realizes(RealizesGroup),
    Refines(RefinesGroup),
    States(StatesGroup),
    Axiom(AxiomGroup),
    Theorem(TheoremGroup),
    Conjecture(ConjectureGroup),
    Person(PersonGroup),
    Resource(ResourceGroup),
    Specify(SpecifyGroup),
    Relation(RelationGroup),
    Equivalent(EquivalentGroup),
    Topic(TopicGroup),
    TextItem(TextItemGroup),
}

/// One entry in a top-level `Example:` section.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExampleItem {
    Clause(Clause),
    Text(OpenText),
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
    Let(LetGroup),
    If(IfGroup),
    Iff(IffGroup),
    Equivalently(EquivalentlyGroup),
    Piecewise(PiecewiseGroup),
    Have(Box<HaveGroup>),
    Declaration(DeclarationStatement),
    Expression(Expression),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IsOrViaItem {
    IsVia(IsViaStatement),
    Declaration(DeclarationStatement),
    /// A `have:` group, optionally with `asserting:`, standing in for a
    /// specification (allowed in a `Declares:` group's `specifies:`).
    Have(Box<HaveGroup>),
    /// A specification wrapped in a `[:label:]` (e.g. `(.x is \foo.)[:1:]`) whose
    /// `label` may match a `Justification:` entry `[label]`. When it does, that
    /// entry's `have:` group is used to establish the inner `item`;
    /// otherwise the inner `item` is checked inline as normal.
    Labeled {
        label: Vec<String>,
        item: Box<IsOrViaItem>,
    },
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
    View(Box<EnablesViewGroup>),
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
    Notes(NotesGroup),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MetadataItem {
    Id(IdGroup),
    Version(VersionGroup),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SpecifyItem {
    Decimal(NumericSpecificationGroup),
    ZeroOrPositiveInt(NumericSpecificationGroup),
    PositiveInt(NumericSpecificationGroup),
    Int(NumericSpecificationGroup),
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
    pub justification: Option<JustificationSection>,
    pub documented: Option<DocumentedSection>,
    pub aliases: Option<AliasesSection>,
    pub writing: Option<ItemWritingSection>,
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
pub struct DeclaresGroup {
    pub heading: CommandHeader,
    pub declares: DeclaresSection,
    pub using: Option<UsingSection>,
    pub when: Option<WhenSection>,
    pub extends: Option<ExtendsSection>,
    pub specifies: Option<DeclaresSpecifiesSection>,
    pub satisfies: Option<SatisfiesSection>,
    pub requires: Option<RequiresSection>,
    pub enables: Option<EnablesSection>,
    pub justification: Option<JustificationSection>,
    pub documented: Option<DocumentedSection>,
    pub aliases: Option<AliasesSection>,
    pub writing: Option<ItemWritingSection>,
    pub references: Option<ReferencesSection>,
    pub metadata: Option<MetadataSection>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DefinesGroup {
    pub heading: CommandHeader,
    pub defines: DefinesSection,
    /// The `abstractly:` marker, present when this declaration leaves parts of
    /// its value for a [`RealizesGroup`] to supply.
    pub abstractly: bool,
    pub using: Option<UsingSection>,
    pub when: Option<WhenSection>,
    pub specifies: Option<DefinesSpecifiesSection>,
    pub expresses: Option<ExpressesSection>,
    pub requires: Option<RequiresSection>,
    pub enables: Option<EnablesSection>,
    pub justification: Option<JustificationSection>,
    pub documented: Option<DocumentedSection>,
    pub aliases: Option<AliasesSection>,
    pub writing: Option<ItemWritingSection>,
    pub references: Option<ReferencesSection>,
    pub metadata: Option<MetadataSection>,
}

/// A concrete realization of an abstract declaration.
///
/// `Realizes: Nb := \naturals` names the `Defines:` group marked `abstractly:`
/// that it realizes, and its `specifies:` supplies a definition for every symbol
/// that declaration left abstract. An abstract `Defines` is to a `Realizes`
/// roughly what an abstract base class is to a concrete subclass; a `Declares`
/// is the interface.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RealizesGroup {
    pub heading: CommandHeader,
    pub realizes: RealizesSection,
    pub using: Option<UsingSection>,
    pub when: Option<WhenSection>,
    pub specifies: Option<DefinesSpecifiesSection>,
    pub expresses: Option<ExpressesSection>,
    pub requires: Option<RequiresSection>,
    pub enables: Option<EnablesSection>,
    pub justification: Option<JustificationSection>,
    pub documented: Option<DocumentedSection>,
    pub aliases: Option<AliasesSection>,
    pub writing: Option<ItemWritingSection>,
    pub references: Option<ReferencesSection>,
    pub metadata: Option<MetadataSection>,
}

/// The optional `implicitly:`/`explicitly:` marker on a `Refines:` group.
///
/// These zero-argument sections signal to readers whether an explicitly written
/// refinement of a subtype merely restates the definition inherited from the
/// supertype's refinement (`Implicit`) or deliberately overrides it with extra
/// properties (`Explicit`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RefinementKind {
    Implicit,
    Explicit,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RefinesGroup {
    pub heading: CommandHeader,
    pub refines: RefinesSection,
    /// The optional `implicitly:`/`explicitly:` marker, if present.
    pub refinement_kind: Option<RefinementKind>,
    pub using: Option<UsingSection>,
    pub when: Option<WhenSection>,
    pub specifies: Option<RefinesSpecifiesSection>,
    pub satisfies: Option<SatisfiesSection>,
    pub requires: Option<RequiresSection>,
    pub enables: Option<EnablesSection>,
    pub justification: Option<JustificationSection>,
    pub documented: Option<DocumentedSection>,
    pub aliases: Option<AliasesSection>,
    pub writing: Option<ItemWritingSection>,
    pub references: Option<ReferencesSection>,
    pub metadata: Option<MetadataSection>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StatesGroup {
    pub heading: CommandHeader,
    pub using: Option<UsingSection>,
    pub when: Option<WhenSection>,
    pub that: ThatSection,
    pub requires: Option<RequiresSection>,
    pub enables: Option<EnablesSection>,
    pub justification: Option<JustificationSection>,
    pub documented: Option<DocumentedSection>,
    pub aliases: Option<AliasesSection>,
    pub writing: Option<ItemWritingSection>,
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
    pub using: Option<UsingSection>,
    pub when: Option<WhenSection>,
    pub to: EquivalentToSection,
    pub justification: Option<JustificationSection>,
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
    pub justification: Option<JustificationSection>,
    pub documented: Option<DocumentedSection>,
    pub aliases: Option<AliasesSection>,
    pub writing: Option<ItemWritingSection>,
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
    pub justification: Option<JustificationSection>,
    pub documented: Option<DocumentedSection>,
    pub aliases: Option<AliasesSection>,
    pub writing: Option<ItemWritingSection>,
    pub references: Option<ReferencesSection>,
    pub metadata: Option<MetadataSection>,
}

/// A named or anonymous example containing an ordered mixture of clauses and prose.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExampleGroup {
    pub heading: Option<CommandHeader>,
    pub example: ExampleSection,
    pub justification: Option<JustificationSection>,
    pub aliases: Option<AliasesSection>,
    pub writing: Option<ItemWritingSection>,
    pub references: Option<ReferencesSection>,
    pub metadata: Option<MetadataSection>,
}

/// A theorem-shaped assertion that is not claimed to have a proof.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConjectureGroup {
    pub heading: Option<CommandHeader>,
    pub given: Option<GivenSection>,
    pub where_: Option<WhereSection>,
    pub then: ThenSection,
    pub iff: Option<IffSection>,
    pub justification: Option<JustificationSection>,
    pub documented: Option<DocumentedSection>,
    pub aliases: Option<AliasesSection>,
    pub writing: Option<ItemWritingSection>,
    pub references: Option<ReferencesSection>,
    pub metadata: Option<MetadataSection>,
}

/// Which kind of prose placeholder a [`TextItemGroup`] is. Each maps to a leading
/// section label (`TextTheorem:`, `TextAxiom:`, `TextConjecture:`,
/// `TextDefinition:`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextItemKind {
    Theorem,
    Axiom,
    Conjecture,
    Definition,
}

impl TextItemKind {
    /// The leading section label that introduces this kind.
    pub fn label(self) -> &'static str {
        match self {
            Self::Theorem => "TextTheorem",
            Self::Axiom => "TextAxiom",
            Self::Conjecture => "TextConjecture",
            Self::Definition => "TextDefinition",
        }
    }
}

/// A top-level prose placeholder (`TextTheorem:`, `TextAxiom:`,
/// `TextConjecture:`, `TextDefinition:`): a markdown-with-LaTeX body standing in
/// for a structured theorem/axiom/conjecture/definition that will be written
/// later. It is opaque to the type-checker; `Documented:` (`called:`/`written:`/
/// `description:`/`notes:`) and `References:` record the naming, prose, and
/// citations to carry into the structured form.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextItemGroup {
    pub kind: TextItemKind,
    pub text: TextItemSection,
    pub documented: Option<DocumentedSection>,
    pub references: Option<ReferencesSection>,
    pub id: IdSection,
}

/// A top-level `Relation:` item, which states a bidirectional relationship
/// between the two concepts declared in `between:` and `and:`. It is a
/// heading-less, standalone item and does not register a view rule.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RelationGroup {
    pub relation: RelationSection,
    pub using: Option<UsingSection>,
    pub between: RelationBetweenSection,
    pub and_: RelationAndSection,
    pub when: Option<WhenSection>,
    pub specifies: Option<RelationSpecifiesSection>,
    pub justification: Option<JustificationSection>,
    pub documented: Option<DocumentedSection>,
    pub aliases: Option<AliasesSection>,
    pub writing: Option<ItemWritingSection>,
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
/// `\function:on:to`) — and explains the relationship in `specifies:`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TopicRelatedItem {
    pub to: TopicRelatedToSection,
    pub specifies: TopicRelatedSpecifiesSection,
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
pub struct LetGroup {
    pub heading: Option<LabelHeader>,
    pub let_: LetSection,
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

/// A `have:`/`asserting:`/`because?:`/`by?:` group: an escape hatch that states an
/// item (`have:`), optional facts (`asserting:`) it may take as true to establish
/// it, and optional justification (`because?:` clauses, `by?:` theorem
/// references) that the checker only well-forms rather than proves.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HaveGroup {
    pub heading: Option<LabelHeader>,
    pub have: HaveSection,
    pub asserting: Option<AssertingSection>,
    pub because: Option<BecauseSection>,
    pub by: Option<HaveBySection>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PiecewiseGroup {
    pub heading: Option<LabelHeader>,
    pub if_: IfSection,
    pub then: ThenSection,
    pub else_if: Vec<PiecewiseElseIf>,
    pub else_: Option<ElseSection>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PiecewiseElseIf {
    pub else_if: ElseIfSection,
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
pub struct EnablesViewGroup {
    pub heading: Option<LabelHeader>,
    pub as_: ViewAsSection,
    pub signifies: Option<ViewSignifiesSection>,
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
pub struct NotesGroup {
    pub heading: Option<LabelHeader>,
    pub notes: NotesSection,
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
pub struct NumericSpecificationGroup {
    pub is_: NumericSpecificationIsSection,
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

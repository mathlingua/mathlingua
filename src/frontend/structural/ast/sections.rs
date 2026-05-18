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
argument_section!(ExistsSection, IsOrSpec);
arguments_section!(SuchThatSection, Clause);
argument_section!(ExistsUniqueSection, IsOrSpec);
argument_section!(ForAllSection, IsOrSpec);
arguments_section!(IfSection, Clause);
zero_or_more_arguments_section!(PiecewiseSection, OpenText);
arguments_section!(ElseSection, Clause);
argument_section!(GivenClauseSection, IsOrRefinedStatementSpec);
argument_section!(TitleSection, OpenText);
argument_section!(SectionSection, OpenText);
argument_section!(SubsectionSection, OpenText);
argument_section!(SubsubsectionSection, OpenText);

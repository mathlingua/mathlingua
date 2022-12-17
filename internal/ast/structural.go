/*
 * Copyright 2022 Dominic Kramer
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     https://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

package ast

////////////////////////////////////////////////////////////////////////

const LowerAllOfName = "allOf"
const LowerAnyOfName = "anyOf"
const LowerAsName = "as"
const LowerAsQuestionName = LowerAsName + "?"
const LowerByName = "by"
const LowerCalledName = "called"
const LowerContentName = "content"
const LowerSpecifiesName = "specifies"
const LowerSpecifiesQuestionName = LowerSpecifiesName + "?"
const LowerDiscovererName = "discoverer"
const LowerElseName = "else"
const LowerElseQuestionName = LowerElseName + "?"
const LowerExampleName = "example"
const LowerExistsName = "exists"
const LowerExistsUniqueName = "existsUnique"
const LowerExtendsName = "extends"
const LowerExtendsQuestionName = LowerExtendsName + "?"
const LowerForAllName = "forAll"
const LowerGivenName = "given"
const LowerGivenQuestionName = LowerGivenName + "?"
const LowerUsingName = "using"
const LowerUsingQuestionName = LowerUsingName + "?"
const LowerHistoryName = "history"
const LowerIdName = "id"
const LowerIfName = "if"
const LowerIfQuestionName = LowerIfName + "?"
const LowerIffName = "iff"
const LowerIffQuestionName = LowerIffName + "?"
const LowerIsName = "is"
const LowerLabelName = "label"
const LowerDetailsName = "details"
const LowerMeansName = "means"
const LowerMeansQuestionName = LowerMeansName + "?"
const LowerMotivationName = "motivation"
const LowerNegativeFloatName = "negativeFloat"
const LowerNegativeIntName = "negativeInt"
const LowerNotName = "not"
const LowerNoteName = "note"
const LowerOfName = "of"
const LowerOneOfName = "oneOf"
const LowerOverviewName = "overview"
const LowerPiecewiseName = "piecewise"
const LowerPositiveFloatName = "positiveFloat"
const LowerPositiveIntName = "positiveInt"
const LowerRelatedName = "related"
const LowerSatisfiesName = "satisfies"
const LowerSatisfiesQuestionName = LowerSatisfiesName + "?"
const LowerStatesName = "states"
const LowerStatesQuestion = "states"
const LowerStatesQuestionName = LowerStatesName + "?"
const LowerSuchThatName = "suchThat"
const LowerSuchThatQuestionName = LowerSuchThatName + "?"
const LowerThatName = "that"
const LowerThenName = "then"
const LowerThroughName = "through"
const LowerViaName = "via"
const LowerViewName = "view"
const LowerWhenName = "when"
const LowerWhenQuestionName = LowerWhenName + "?"
const LowerWhereName = "where"
const LowerWhereQuestionName = LowerWhereName + "?"
const LowerWithName = "with"
const LowerWithQuestionName = LowerWithName + "?"
const LowerZeroName = "zero"
const UpperAxiomName = "Axiom"
const UpperConjectureName = "Conjecture"
const UpperDefinesName = "Defines"
const UpperDescribesName = "Describes"
const UpperDocumentedName = "Documented"
const UpperDocumentedQuestionName = UpperDocumentedName + "?"
const UpperJustifiedName = "Justified"
const UpperJustifiedQuestionName = UpperJustifiedName + "?"
const UpperNoteName = "Note"
const UpperProofName = "Proof"
const UpperProofQuestionName = UpperProofName + "?"
const UpperProvidesName = "Provides"
const UpperProvidesQuestionName = UpperProvidesName + "?"
const UpperReferencesName = "References"
const UpperReferencesQuestionName = UpperReferencesName + "?"
const UpperSpecifyName = "Specify"
const UpperStatesName = "States"
const UpperTheoremName = "Theorem"
const UpperTopicName = "Topic"
const UpperViewableName = "Viewable"
const UpperViewableQuestionName = UpperViewableName + "?"
const LowerMembersName = "members"
const LowerMemberName = "member"
const LowerOperationsName = "operations"
const LowerOperationName = "operation"
const LowerSpecifyName = "specify"
const UpperAliasesName = "Aliases"
const UpperAliasesQuestionName = UpperAliasesName + "?"
const LowerIntoName = "into"
const LowerAliasesName = "aliases"
const LowerOnName = "on"
const LowerOnQuestionName = LowerOnName + "?"
const LowerTitleName = "title"
const LowerAuthorName = "author"
const LowerOffsetName = "offset"
const LowerUrlName = "url"
const LowerHomepageName = "homepage"
const LowerTypeName = "type"
const LowerEditionName = "edition"
const LowerEditorName = "editor"
const LowerInstitutionName = "institution"
const LowerJournalName = "journal"
const LowerPublisherName = "publisher"
const LowerVolumeName = "volume"
const LowerMonthName = "month"
const LowerYearName = "year"
const LowerDescriptionName = "description"
const LowerSymbolName = "symbol"
const LowerWrittenName = "written"
const LowerWrittenQuestionName = LowerWrittenName + "?"
const LowerWritingName = "writing"
const LowerConnectionName = "connection"
const LowerToName = "to"
const LowerSignifiesName = "signifies"
const LowerSignifiesQuestionName = LowerSignifiesName + "?"
const LowerViewableName = "viewable"
const LowerViewableQuestionName = LowerViewableName + "?"
const LowerThroughQuestionName = LowerThroughName + "?"
const LowerDescribingName = "describing"
const LowerNameName = "name"
const LowerBiographyName = "biography"
const UpperPersonName = "Person"
const UpperResourceName = "Resource"
const UpperIdName = "Id"
const UpperIdQuestionName = UpperIdName + "?"

///////////////////////////////////////////////////////////////////////

type IdItem struct {
	RawText string
	Root    NodeType
	Label   *string
}

type Target struct {
	RawText string
	Root    NodeType
	Label   *string
}

type Spec struct {
	RawText string
	Root    NodeType
	Label   *string
}

type Alias struct {
	RawText string
	Root    NodeType
	Label   *string
}

func (Alias) ProvidesType() {}

type Formulation[T NodeType] struct {
	RawText string
	Root    T
	Label   *string
}

type MetaData struct {
	Start Position
}

type TextItem struct {
	RawText string
}

//////////////////////////////////////////////////////////////////////////////

type Clause interface {
	Clause()
}

func (Formulation[NodeType]) Clause() {}
func (AllOfGroup) Clause()            {}
func (NotGroup) Clause()              {}
func (AnyOfGroup) Clause()            {}
func (OneOfGroup) Clause()            {}
func (ExistsGroup) Clause()           {}
func (ExistsUniqueGroup) Clause()     {}
func (ForAllGroup) Clause()           {}
func (IfGroup) Clause()               {}
func (IffGroup) Clause()              {}
func (WhenGroup) Clause()             {}
func (PiecewiseGroup) Clause()        {}
func (GivenGroup) Clause()            {}

//////////////////////////////////////////////////////////////////////////////

var GivenSections = []string{
	LowerGivenName,
	LowerWhereQuestionName,
	LowerSuchThatQuestionName,
	LowerThenName,
}

type GivenGroup struct {
	Given    GivenSection
	Where    *WhereSection
	SuchThat *SuchThatSection
	Then     ThenSection
}

//////////////////////////////////////////////////////////////////////////////

var AllOfSections = []string{LowerAllOfName}

type AllOfGroup struct {
	AllOf AllOfSection
}

type AllOfSection struct {
	Clauses []Clause
}

//////////////////////////////////////////////////////////////////////////////

var NotSections = []string{LowerNotName}

type NotGroup struct {
	Not NotSection
}

type NotSection struct {
	Clause Clause
}

//////////////////////////////////////////////////////////////////////////////

var AnyOfSections = []string{LowerAnyOfName}

type AnyOfGroup struct {
	AnyOf AnyOfSection
}

type AnyOfSection struct {
	Clauses []Clause
}

//////////////////////////////////////////////////////////////////////////////

var OneOfSections = []string{LowerOneOfName}

type OneOfGroup struct {
	OneOf OneOfSection
}

type OneOfSection struct {
	Clauses []Clause
}

//////////////////////////////////////////////////////////////////////////////

var ExistsSections = []string{
	LowerExistsName,
	LowerWhereQuestionName,
	LowerSuchThatName,
}

type ExistsGroup struct {
	Exists   ExistsSection
	Where    *WhereSection
	SuchThat *SuchThatSection
}

type ExistsSection struct {
	Targets []Target
}

type WhereSection struct {
	Specs []Spec
}

type SuchThatSection struct {
	Clauses []Clause
}

//////////////////////////////////////////////////////////////////////////////

var ExistsUniqueSections = []string{
	LowerExistsUniqueName,
	LowerWhereQuestionName,
	LowerSuchThatName,
}

type ExistsUniqueGroup struct {
	ExistsUnique ExistsUniqueSection
	Where        *WhereSection
	SuchThat     SuchThatSection
}

type ExistsUniqueSection struct {
	Targets []Target
}

//////////////////////////////////////////////////////////////////////////////

var ForAllSections = []string{
	LowerForAllName,
	LowerWhereQuestionName,
	LowerSuchThatQuestionName,
	LowerThenName,
}

type ForAllGroup struct {
	ForAll   ForAllSection
	Where    *WhereSection
	SuchThat *SuchThatSection
	Then     ThenSection
}

type ForAllSection struct {
	Targets []Target
}

type ThenSection struct {
	Clauses []Clause
}

//////////////////////////////////////////////////////////////////////////////

var IfSections = []string{
	LowerIfName,
	LowerThenName,
}

type IfGroup struct {
	If   IfSection
	Then ThenSection
}

type IfSection struct {
	Clauses []Clause
}

//////////////////////////////////////////////////////////////////////////////

var IffSections = []string{
	LowerIffName,
	LowerThenName,
}

type IffGroup struct {
	Iff  IffSection
	Then ThenSection
}

type IffSection struct {
	Clauses []Clause
}

//////////////////////////////////////////////////////////////////////////////

type WhenSection struct {
	When []Clause
}

//////////////////////////////////////////////////////////////////////////////

type IfThen struct {
	If   IfSection
	Then ThenSection
}

var PiecewiseSections = []string{
	LowerPiecewiseName,
	LowerIfName,
	LowerThenName,
	LowerElseQuestionName,
}

type PiecewiseGroup struct {
	Piecewise PiecewiseSection
	IfThen    []IfThen
	Else      *ElseSection
}

type PiecewiseSection struct {
}

type ElseSection struct {
	Items []Clause
}

//////////////////////////////////////////////////////////////////////////////

var WhenSections = []string{
	LowerWhenName,
	LowerThenName,
}

type WhenGroup struct {
	When WhenSection
	Then ThenSection
}

//////////////////////////////////////////////////////////////////////////////

type ProvidesType interface {
	ProvidesType()
}

func (SymbolWrittenGroup) ProvidesType() {}
func (ConnectionGroup) ProvidesType()    {}

var SymbolSections = []string{
	LowerSymbolName,
	LowerWrittenQuestionName,
}

type SymbolSection struct {
	Symbol Alias
}

type WrittenSection struct {
	Written []TextItem
}

type SymbolWrittenGroup struct {
	Symbol  SymbolSection
	Written *WrittenSection
}

var ConnectionSections = []string{
	LowerConnectionName,
	LowerToName,
	LowerUsingQuestionName,
	LowerMeansName,
	LowerSignifiesQuestionName,
	LowerViewableQuestionName,
	LowerThroughQuestionName,
}

type ConnectionSection struct {
}

type ToSection struct {
	To Target
}

type SignifiesSection struct {
	Signifies Spec
}

type ConnectionViewableSection struct {
}

type ConnectionThroughSection struct {
	Through Formulation[NodeType]
}

type ConnectionGroup struct {
	Connection ConnectionSection
	Using      *UsingSection
	Means      MeansSection
	Signfies   *SignifiesSection
	Viewable   *ConnectionViewableSection
	Through    *ConnectionThroughSection
}

type SimpleOperationsSection struct {
	Operations []Alias
}

type SingleAliasesSection struct {
	Aliases Alias
}

type OperationSection struct {
	Operation Alias
}

type OperationsSection struct {
}

type OnSection struct {
	On []Target
}

type SpecifySection struct {
	Specify []Clause
}

//////////////////////////////////////////////////////////////////////////////

var WrittenSections = []string{LowerWrittenName}

type WrittenGroup struct {
	Written WrittenSection
}

var CalledSections = []string{LowerCalledName}

type CalledGroup struct {
	Called CalledSection
}

type CalledSection struct {
	Called []TextItem
}

var WritingSections = []string{LowerWritingName, LowerAsName}

type WritingGroup struct {
	Writing WritingSection
	As      WritingAsSection
}

type WritingSection struct {
	Writing Target
}

type WritingAsSection struct {
	As []TextItem
}

//////////////////////////////////////////////////////////////////////////////

type DocumentedType interface {
	DocumentedType()
}

func (OverviewGroup) DocumentedType()   {}
func (MotivationGroup) DocumentedType() {}
func (HistoryGroup) DocumentedType()    {}
func (ExampleGroup) DocumentedType()    {}
func (RelatedGroup) DocumentedType()    {}
func (DiscovererGroup) DocumentedType() {}
func (NoteGroup) DocumentedType()       {}
func (WrittenGroup) DocumentedType()    {}
func (WritingGroup) DocumentedType()    {}
func (CalledGroup) DocumentedType()     {}

var DetailsSections = []string{LowerDetailsName}

var OverviewSections = []string{LowerOverviewName}

type OverviewGroup struct {
	Overview OverviewSection
}

type OverviewSection struct {
	Overview TextItem
}

var MotivationSections = []string{LowerMotivationName}

type MotivationGroup struct {
	Motivation MotivationSection
}

type MotivationSection struct {
	Motivation TextItem
}

var HistorySections = []string{LowerHistoryName}

type HistoryGroup struct {
	History HistorySection
}

type HistorySection struct {
	History TextItem
}

var ExampleSections = []string{LowerExampleName}

type ExampleGroup struct {
	Examples ExampleSection
}

type ExampleSection struct {
	Examples []TextItem
}

var RelatedSections = []string{LowerRelatedName}

type RelatedGroup struct {
	Related RelatedSection
}

type RelatedSection struct {
	Related []TextItem
}

var DiscovererSections = []string{LowerDiscovererName}

type DiscovererGroup struct {
	Discoverer DiscovererSection
}

type DiscovererSection struct {
	Discoverer []TextItem
}

var NoteSections = []string{LowerNoteName}

type NoteGroup struct {
	Note NoteSection
}

type NoteSection struct {
	Note []NoteType
}

type NoteType interface {
	NoteType()
}

func (TextItem) NoteType()        {}
func (DescribingGroup) NoteType() {}

var DescribingSections = []string{LowerDescribingName, LowerContentName}

type DescribingGroup struct {
	Describing DescribingSection
	Content    ContentSection
}

type DescribingSection struct {
	Describing TextItem
}

//////////////////////////////////////////////////////////////////////////////

type ProvidesSection struct {
	Provides []ProvidesType
}

//////////////////////////////////////////////////////////////////////////////

type AliasesSection struct {
	Aliases []Alias
}

//////////////////////////////////////////////////////////////////////////////

type DocumentedSection struct {
	Documented []DocumentedType
}

//////////////////////////////////////////////////////////////////////////////

type JustifiedSection struct {
	Justified []JustifiedType
}

//////////////////////////////////////////////////////////////////////////////

type JustifiedType interface {
	JustifiedType()
}

func (LabelGroup) JustifiedType() {}
func (ByGroup) JustifiedType()    {}

var LabelSections = []string{LowerLabelName, LowerByName}

type LabelGroup struct {
	Label LabelSection
	By    BySection
}

type LabelSection struct {
	Label TextItem
}

var BySections = []string{LowerByName}

type ByGroup struct {
	By BySection
}

type BySection struct {
	By []TextItem
}

//////////////////////////////////////////////////////////////////////////////

type ReferencesSection struct {
	References []TextItem
}

//////////////////////////////////////////////////////////////////////////////

var DescribesSections = []string{
	UpperDescribesName,
	LowerWithQuestionName,
	LowerUsingQuestionName,
	LowerWhenQuestionName,
	LowerSuchThatQuestionName,
	LowerExtendsQuestionName,
	LowerSatisfiesQuestionName,
	UpperProvidesQuestionName,
	UpperJustifiedQuestionName,
	UpperDocumentedQuestionName,
	UpperReferencesQuestionName,
	UpperAliasesQuestionName,
	UpperIdQuestionName,
}

type DescribesGroup struct {
	Id         IdItem
	Describes  DescribesSection
	With       *WithSection
	Using      *UsingSection
	When       *WhenSection
	SuchThat   *SuchThatSection
	Extends    *ExtendsSection
	Satisfies  *SatisfiesSection
	Provides   *ProvidesSection
	Justified  *JustifiedSection
	Documented *DocumentedSection
	References *ReferencesSection
	Aliases    *AliasesSection
	MetaId     *MetaIdSection
}

type DescribesSection struct {
	Describes Target
}

type WithSection struct {
	With []Target
}

type GivenSection struct {
	Given []Target
}

type UsingSection struct {
	Using []Target
}

type ExtendsSection struct {
	Extends []Clause
}

type SatisfiesSection struct {
	Satisfies []Clause
}

//////////////////////////////////////////////////////////////////////////////

type MetaIdSection struct {
	Id TextItem
}

//////////////////////////////////////////////////////////////////////////////

var DefinesSections = []string{
	UpperDefinesName,
	LowerWithQuestionName,
	LowerUsingQuestionName,
	LowerWhenQuestionName,
	LowerSuchThatQuestionName,
	LowerMeansQuestionName,
	LowerSpecifiesQuestionName,
	UpperProvidesQuestionName,
	UpperJustifiedQuestionName,
	UpperDocumentedQuestionName,
	UpperReferencesQuestionName,
	UpperAliasesQuestionName,
	UpperIdQuestionName,
}

type DefinesGroup struct {
	Id         IdItem
	Defines    DefinesSection
	With       *WithSection
	Using      *UsingSection
	When       *WhenSection
	SuchThat   *SuchThatSection
	Means      *MeansSection
	Specifies  *SpecifiesSection
	Provides   *ProvidesSection
	Justified  *JustifiedSection
	Documented *DocumentedSection
	References *ReferencesSection
	Aliases    *AliasesSection
	MetaId     *MetaIdSection
}

type DefinesSection struct {
	Defines Target
}

type MeansSection struct {
	Means Clause
}

type SpecifiesSection struct {
	Specifies []Clause
}

//////////////////////////////////////////////////////////////////////////////

var StatesSections = []string{
	UpperStatesName,
	LowerWithQuestionName,
	LowerUsingQuestionName,
	LowerWhenQuestionName,
	LowerSuchThatQuestionName,
	LowerThatName,
	UpperDocumentedQuestionName,
	UpperJustifiedQuestionName,
	UpperReferencesQuestionName,
	UpperAliasesQuestionName,
	UpperIdQuestionName,
}

type StatesGroup struct {
	Id         IdItem
	States     StatesSection
	With       *WithSection
	Using      *UsingSection
	When       *WhenSection
	SuchThat   *SuchThatSection
	That       ThatSection
	Documented *DocumentedSection
	Justified  *JustifiedSection
	References *ReferencesSection
	Aliases    *AliasesSection
	MetaId     *MetaIdSection
}

type StatesSection struct {
}

type ThatSection struct {
	That []Clause
}

//////////////////////////////////////////////////////////////////////////////

var ProofSections = []string{
	UpperProofName,
	LowerOfName,
	LowerContentName,
	UpperReferencesQuestionName,
	UpperIdQuestionName,
}

type ProofGroup struct {
	Id         IdItem
	Proof      TopLevelProofSection
	Of         OfSection
	Content    ContentSection
	References *ReferencesSection
	MetaId     *MetaIdSection
}

type TopLevelProofSection struct {
}

type OfSection struct {
	Of TextItem
}

type ContentSection struct {
	Content TextItem
}

//////////////////////////////////////////////////////////////////////////////

var AxiomSections = []string{
	UpperAxiomName,
	LowerGivenQuestionName,
	LowerWhereQuestionName,
	LowerIfQuestionName,
	LowerIffQuestionName,
	LowerThenName,
	UpperDocumentedQuestionName,
	UpperReferencesQuestionName,
	UpperAliasesQuestionName,
	UpperIdQuestionName,
}

type AxiomGroup struct {
	Id         *IdItem
	Axiom      AxiomSection
	Given      *GivenSection
	Where      *WhereSection
	If         *IfSection
	Iff        *IffSection
	Then       ThenSection
	Documented *DocumentedSection
	References *ReferencesSection
	Aliases    *AliasesSection
	MetaId     *MetaIdSection
}

type AxiomSection struct {
	Axiom []TextItem
}

//////////////////////////////////////////////////////////////////////////////

var ConjectureSections = []string{
	UpperConjectureName,
	LowerGivenQuestionName,
	LowerWhereQuestionName,
	LowerIfQuestionName,
	LowerIffQuestionName,
	LowerThenName,
	UpperDocumentedQuestionName,
	UpperReferencesQuestionName,
	UpperAliasesQuestionName,
	UpperIdQuestionName,
}

type ConjectureGroup struct {
	Id         *IdItem
	Conjecture ConjectureSection
	Given      *GivenSection
	Where      *WhereSection
	If         *IfSection
	Iff        *IffSection
	Then       ThenSection
	Documented *DocumentedSection
	References *ReferencesSection
	Aliases    *AliasesSection
	MetaId     *MetaIdSection
}

type ConjectureSection struct {
	Conjecture []TextItem
}

//////////////////////////////////////////////////////////////////////////////

var TheoremSections = []string{
	UpperTheoremName,
	LowerGivenQuestionName,
	LowerWhereQuestionName,
	LowerIfQuestionName,
	LowerIffQuestionName,
	LowerThenName,
	UpperProofQuestionName,
	UpperDocumentedQuestionName,
	UpperReferencesQuestionName,
	UpperAliasesQuestionName,
	UpperIdQuestionName,
}

type TheoremGroup struct {
	Id         *IdItem
	Theorem    TheoremSection
	Given      *GivenSection
	Where      *WhereSection
	If         *IfSection
	Iff        *IffSection
	Then       ThenSection
	Proof      *ProofSection
	Documented *DocumentedSection
	References *ReferencesSection
	Aliases    *AliasesSection
	MetaId     *MetaIdSection
}

type TheoremSection struct {
	Theorem []TextItem
}

type ProofSection struct {
	Proof TextItem
}

//////////////////////////////////////////////////////////////////////////////

var TopicSections = []string{
	UpperTopicName,
	LowerContentName,
	UpperReferencesQuestionName,
	UpperIdQuestionName,
}

type TopicGroup struct {
	Id         IdItem
	Topic      TopicSection
	Content    ContentSection
	References *ReferencesSection
	MetaId     *MetaIdSection
}

type TopicSection struct {
}

//////////////////////////////////////////////////////////////////////////////

type SpecifyType interface {
	SpecifyType()
}

func (ZeroGroup) SpecifyType()          {}
func (PositiveIntGroup) SpecifyType()   {}
func (NegativeIntGroup) SpecifyType()   {}
func (PositiveFloatGroup) SpecifyType() {}
func (NegativeFloatGroup) SpecifyType() {}

var ZeroSections = []string{LowerZeroName, LowerMeansName}

type ZeroGroup struct {
	Zero  ZeroSection
	Means MeansSection
}

type ZeroSection struct {
}

var PositiveIntSections = []string{LowerPositiveIntName, LowerMeansName}

type PositiveIntGroup struct {
	PositiveInt PositiveIntSection
	Means       MeansSection
}

type PositiveIntSection struct {
	PositiveInt Target
}

var NegativeIntSections = []string{LowerNegativeIntName, LowerMeansName}

type NegativeIntGroup struct {
	NegativeInt NegativeIntSection
	Means       MeansSection
}

type NegativeIntSection struct {
	NegativeInt Target
}

var PositiveFloatSections = []string{LowerPositiveFloatName, LowerMeansName}

type PositiveFloatGroup struct {
	PositiveFloat PositiveFloatSection
	Means         MeansSection
}

type PositiveFloatSection struct {
	PositiveFloat Target
}

var NegativeFloatSections = []string{LowerNegativeFloatName, LowerMeansName}

type NegativeFloatGroup struct {
	NegativeFloat NegativeFloatSection
	Means         MeansSection
}

type NegativeFloatSection struct {
	NegativeFloat Target
}

//////////////////////////////////////////////////////////////////////////////

var SpecifySections = []string{UpperSpecifyName, UpperIdQuestionName}

type SpecifyGroup struct {
	Specify TopLevelSpecifySection
	MetaId  *MetaIdSection
}

type TopLevelSpecifySection struct {
	Specify []SpecifyType
}

//////////////////////////////////////////////////////////////////////////////

var PersonSections = []string{UpperPersonName, UpperIdQuestionName}

type PersonGroup struct {
	Id     string
	Person PersonSection
	MetaId *MetaIdSection
}

type PersonSection struct {
	Items []PersonType
}

type PersonType interface {
	PersonType()
}

func (NameGroup) PersonType()      {}
func (BiographyGroup) PersonType() {}

var NameSections = []string{LowerNameName}

type NameGroup struct {
	Name NameSection
}

type NameSection struct {
	Name []TextItem
}

var BiographySections = []string{LowerBiographyName}

type BiographyGroup struct {
	Biography BiographySection
}

type BiographySection struct {
	Biography TextItem
}

//////////////////////////////////////////////////////////////////////////////

var ResourceSections = []string{UpperResourceName, UpperIdQuestionName}

type ResourceGroup struct {
	Id       string
	Resource ResourceSection
	MetaId   *MetaIdSection
}

type ResourceSection struct {
	Items []ResourceType
}

func (TitleGroup) ResourceType()       {}
func (AuthorGroup) ResourceType()      {}
func (OffsetGroup) ResourceType()      {}
func (UrlGroup) ResourceType()         {}
func (HomepageGroup) ResourceType()    {}
func (TypeGroup) ResourceType()        {}
func (EditorGroup) ResourceType()      {}
func (EditionGroup) ResourceType()     {}
func (InstitutionGroup) ResourceType() {}
func (JournalGroup) ResourceType()     {}
func (PublisherGroup) ResourceType()   {}
func (VolumeGroup) ResourceType()      {}
func (MonthGroup) ResourceType()       {}
func (YearGroup) ResourceType()        {}
func (DescriptionGroup) ResourceType() {}

type ResourceType interface {
	ResourceType()
}

type TitleGroup struct {
	Title TitleSection
}

type TitleSection struct {
	Title TextItem
}

type AuthorGroup struct {
	Author AuthorSection
}

type AuthorSection struct {
	Author []TextItem
}

type OffsetGroup struct {
	Offset OffsetSection
}

type OffsetSection struct {
	Offset TextItem
}

type UrlGroup struct {
	Url UrlSection
}

type UrlSection struct {
	Url TextItem
}

type HomepageGroup struct {
	Homepage HomepageSection
}

type HomepageSection struct {
	Homepage TextItem
}

type TypeGroup struct {
	Type TypeSection
}

type TypeSection struct {
	Type TextItem
}

type EditorGroup struct {
	Editor EditorSection
}

type EditorSection struct {
	Editor []TextItem
}

type EditionGroup struct {
	Edition EditionSection
}

type EditionSection struct {
	Edition TextItem
}

type InstitutionGroup struct {
	Institution InstitutionSection
}

type InstitutionSection struct {
	Institution []TextItem
}

type JournalGroup struct {
	Journal JournalSection
}

type JournalSection struct {
	Journal []TextItem
}

type PublisherGroup struct {
	Publisher PublisherSection
}

type PublisherSection struct {
	Publisher []TextItem
}

type VolumeGroup struct {
	Volume VolumeSection
}

type VolumeSection struct {
	Volume TextItem
}

type MonthGroup struct {
	Month MonthSection
}

type MonthSection struct {
	Month TextItem
}

type YearGroup struct {
	Year YearSection
}

type YearSection struct {
	Year TextItem
}

type DescriptionGroup struct {
	Description DescriptionSection
}

type DescriptionSection struct {
	Description TextItem
}

//////////////////////////////////////////////////////////////////////////////

type TextBlockItem struct {
	Text string
}

type TopLevelItemType interface {
	TopLevelItemType()
}

func (TextBlockItem) TopLevelItemType()   {}
func (DefinesGroup) TopLevelItemType()    {}
func (DescribesGroup) TopLevelItemType()  {}
func (StatesGroup) TopLevelItemType()     {}
func (AxiomGroup) TopLevelItemType()      {}
func (ConjectureGroup) TopLevelItemType() {}
func (TheoremGroup) TopLevelItemType()    {}
func (SpecifyGroup) TopLevelItemType()    {}
func (TopicGroup) TopLevelItemType()      {}
func (PersonGroup) TopLevelItemType()     {}
func (ResourceGroup) TopLevelItemType()   {}
func (ProofGroup) TopLevelItemType()      {}

type Document struct {
	Items []TopLevelItemType
}

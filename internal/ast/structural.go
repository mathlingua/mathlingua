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

type StructuralNodeType interface {
	StructuralDebuggable
	StructuralNodeType()
}

func (IdItem) StructuralNodeType()             {}
func (Target) StructuralNodeType()             {}
func (Spec) StructuralNodeType()               {}
func (Alias) StructuralNodeType()              {}
func (Formulation[T]) StructuralNodeType()     {}
func (TextItem) StructuralNodeType()           {}
func (GivenGroup) StructuralNodeType()         {}
func (AllOfGroup) StructuralNodeType()         {}
func (NotGroup) StructuralNodeType()           {}
func (AnyOfGroup) StructuralNodeType()         {}
func (OneOfGroup) StructuralNodeType()         {}
func (ExistsGroup) StructuralNodeType()        {}
func (ExistsUniqueGroup) StructuralNodeType()  {}
func (ForAllGroup) StructuralNodeType()        {}
func (IfGroup) StructuralNodeType()            {}
func (IffGroup) StructuralNodeType()           {}
func (PiecewiseGroup) StructuralNodeType()     {}
func (WhenGroup) StructuralNodeType()          {}
func (SymbolWrittenGroup) StructuralNodeType() {}
func (ConnectionGroup) StructuralNodeType()    {}
func (WrittenGroup) StructuralNodeType()       {}
func (CalledGroup) StructuralNodeType()        {}
func (WritingGroup) StructuralNodeType()       {}
func (OverviewGroup) StructuralNodeType()      {}
func (MotivationGroup) StructuralNodeType()    {}
func (HistoryGroup) StructuralNodeType()       {}
func (ExampleGroup) StructuralNodeType()       {}
func (RelatedGroup) StructuralNodeType()       {}
func (DiscovererGroup) StructuralNodeType()    {}
func (NoteGroup) StructuralNodeType()          {}
func (DescribingGroup) StructuralNodeType()    {}
func (LabelGroup) StructuralNodeType()         {}
func (ByGroup) StructuralNodeType()            {}
func (DescribesGroup) StructuralNodeType()     {}
func (DefinesGroup) StructuralNodeType()       {}
func (StatesGroup) StructuralNodeType()        {}
func (ProofGroup) StructuralNodeType()         {}
func (AxiomGroup) StructuralNodeType()         {}
func (ConjectureGroup) StructuralNodeType()    {}
func (TheoremGroup) StructuralNodeType()       {}
func (TopicGroup) StructuralNodeType()         {}
func (ZeroGroup) StructuralNodeType()          {}
func (PositiveIntGroup) StructuralNodeType()   {}
func (NegativeIntGroup) StructuralNodeType()   {}
func (PositiveFloatGroup) StructuralNodeType() {}
func (NegativeFloatGroup) StructuralNodeType() {}
func (SpecifyGroup) StructuralNodeType()       {}
func (PersonGroup) StructuralNodeType()        {}
func (NameGroup) StructuralNodeType()          {}
func (BiographyGroup) StructuralNodeType()     {}
func (ResourceGroup) StructuralNodeType()      {}
func (TitleGroup) StructuralNodeType()         {}
func (AuthorGroup) StructuralNodeType()        {}
func (OffsetGroup) StructuralNodeType()        {}
func (UrlGroup) StructuralNodeType()           {}
func (HomepageGroup) StructuralNodeType()      {}
func (TypeGroup) StructuralNodeType()          {}
func (EditorGroup) StructuralNodeType()        {}
func (EditionGroup) StructuralNodeType()       {}
func (InstitutionGroup) StructuralNodeType()   {}
func (JournalGroup) StructuralNodeType()       {}
func (PublisherGroup) StructuralNodeType()     {}
func (VolumeGroup) StructuralNodeType()        {}
func (MonthGroup) StructuralNodeType()         {}
func (YearGroup) StructuralNodeType()          {}
func (DescriptionGroup) StructuralNodeType()   {}
func (Document) StructuralNodeType()           {}
func (TextBlockItem) StructuralNodeType()      {}

///////////////////////////////////////////////////////////////////////

type MetaData struct {
	Start Position
	Key   int
}

type IdItem struct {
	RawText  string
	Root     NodeType
	Label    *string
	MetaData MetaData
}

type Target struct {
	RawText  string
	Root     NodeType
	Label    *string
	MetaData MetaData
}

type Spec struct {
	RawText  string
	Root     NodeType
	Label    *string
	MetaData MetaData
}

type Alias struct {
	RawText  string
	Root     NodeType
	Label    *string
	MetaData MetaData
}

func (Alias) ProvidesType() {}

type Formulation[T NodeType] struct {
	RawText  string
	Root     T
	Label    *string
	MetaData MetaData
}

type TextItem struct {
	RawText  string
	MetaData MetaData
}

//////////////////////////////////////////////////////////////////////////////

type Clause interface {
	StructuralNodeType
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
	MetaData MetaData
}

//////////////////////////////////////////////////////////////////////////////

var AllOfSections = []string{LowerAllOfName}

type AllOfGroup struct {
	AllOf    AllOfSection
	MetaData MetaData
}

type AllOfSection struct {
	Clauses  []Clause
	MetaData MetaData
}

//////////////////////////////////////////////////////////////////////////////

var NotSections = []string{LowerNotName}

type NotGroup struct {
	Not      NotSection
	MetaData MetaData
}

type NotSection struct {
	Clause   Clause
	MetaData MetaData
}

//////////////////////////////////////////////////////////////////////////////

var AnyOfSections = []string{LowerAnyOfName}

type AnyOfGroup struct {
	AnyOf    AnyOfSection
	MetaData MetaData
}

type AnyOfSection struct {
	Clauses  []Clause
	MetaData MetaData
}

//////////////////////////////////////////////////////////////////////////////

var OneOfSections = []string{LowerOneOfName}

type OneOfGroup struct {
	OneOf    OneOfSection
	MetaData MetaData
}

type OneOfSection struct {
	Clauses  []Clause
	MetaData MetaData
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
	MetaData MetaData
}

type ExistsSection struct {
	Targets  []Target
	MetaData MetaData
}

type WhereSection struct {
	Specs    []Spec
	MetaData MetaData
}

type SuchThatSection struct {
	Clauses  []Clause
	MetaData MetaData
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
	MetaData     MetaData
}

type ExistsUniqueSection struct {
	Targets  []Target
	MetaData MetaData
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
	MetaData MetaData
}

type ForAllSection struct {
	Targets  []Target
	MetaData MetaData
}

type ThenSection struct {
	Clauses  []Clause
	MetaData MetaData
}

//////////////////////////////////////////////////////////////////////////////

var IfSections = []string{
	LowerIfName,
	LowerThenName,
}

type IfGroup struct {
	If       IfSection
	Then     ThenSection
	MetaData MetaData
}

type IfSection struct {
	Clauses  []Clause
	MetaData MetaData
}

//////////////////////////////////////////////////////////////////////////////

var IffSections = []string{
	LowerIffName,
	LowerThenName,
}

type IffGroup struct {
	Iff      IffSection
	Then     ThenSection
	MetaData MetaData
}

type IffSection struct {
	Clauses  []Clause
	MetaData MetaData
}

//////////////////////////////////////////////////////////////////////////////

type WhenSection struct {
	When     []Clause
	MetaData MetaData
}

//////////////////////////////////////////////////////////////////////////////

type IfThen struct {
	If       IfSection
	Then     ThenSection
	MetaData MetaData
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
	MetaData  MetaData
}

type PiecewiseSection struct {
	MetaData MetaData
}

type ElseSection struct {
	Items    []Clause
	MetaData MetaData
}

//////////////////////////////////////////////////////////////////////////////

var WhenSections = []string{
	LowerWhenName,
	LowerThenName,
}

type WhenGroup struct {
	When     WhenSection
	Then     ThenSection
	MetaData MetaData
}

//////////////////////////////////////////////////////////////////////////////

type ProvidesType interface {
	StructuralNodeType
	ProvidesType()
}

func (SymbolWrittenGroup) ProvidesType() {}
func (ConnectionGroup) ProvidesType()    {}

var SymbolSections = []string{
	LowerSymbolName,
	LowerWrittenQuestionName,
}

type SymbolSection struct {
	Symbol   Alias
	MetaData MetaData
}

type WrittenSection struct {
	Written  []TextItem
	MetaData MetaData
}

type SymbolWrittenGroup struct {
	Symbol   SymbolSection
	Written  *WrittenSection
	MetaData MetaData
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
	MetaData MetaData
}

type ToSection struct {
	To       Target
	MetaData MetaData
}

type SignifiesSection struct {
	Signifies Spec
	MetaData  MetaData
}

type ConnectionViewableSection struct {
	MetaData MetaData
}

type ConnectionThroughSection struct {
	Through  Formulation[NodeType]
	MetaData MetaData
}

type ConnectionGroup struct {
	Connection ConnectionSection
	Using      *UsingSection
	Means      MeansSection
	Signfies   *SignifiesSection
	Viewable   *ConnectionViewableSection
	Through    *ConnectionThroughSection
	MetaData   MetaData
}

type SimpleOperationsSection struct {
	Operations []Alias
	MetaData   MetaData
}

type SingleAliasesSection struct {
	Aliases  Alias
	MetaData MetaData
}

type OperationSection struct {
	Operation Alias
	MetaData  MetaData
}

type OperationsSection struct {
	MetaData MetaData
}

type OnSection struct {
	On       []Target
	MetaData MetaData
}

type SpecifySection struct {
	Specify  []Clause
	MetaData MetaData
}

//////////////////////////////////////////////////////////////////////////////

var WrittenSections = []string{LowerWrittenName}

type WrittenGroup struct {
	Written  WrittenSection
	MetaData MetaData
}

var CalledSections = []string{LowerCalledName}

type CalledGroup struct {
	Called   CalledSection
	MetaData MetaData
}

type CalledSection struct {
	Called   []TextItem
	MetaData MetaData
}

var WritingSections = []string{LowerWritingName, LowerAsName}

type WritingGroup struct {
	Writing  WritingSection
	As       WritingAsSection
	MetaData MetaData
}

type WritingSection struct {
	Writing  Target
	MetaData MetaData
}

type WritingAsSection struct {
	As       []TextItem
	MetaData MetaData
}

//////////////////////////////////////////////////////////////////////////////

type DocumentedType interface {
	StructuralNodeType
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
	MetaData MetaData
}

type OverviewSection struct {
	Overview TextItem
	MetaData MetaData
}

var MotivationSections = []string{LowerMotivationName}

type MotivationGroup struct {
	Motivation MotivationSection
	MetaData   MetaData
}

type MotivationSection struct {
	Motivation TextItem
	MetaData   MetaData
}

var HistorySections = []string{LowerHistoryName}

type HistoryGroup struct {
	History  HistorySection
	MetaData MetaData
}

type HistorySection struct {
	History  TextItem
	MetaData MetaData
}

var ExampleSections = []string{LowerExampleName}

type ExampleGroup struct {
	Examples ExampleSection
	MetaData MetaData
}

type ExampleSection struct {
	Examples []TextItem
	MetaData MetaData
}

var RelatedSections = []string{LowerRelatedName}

type RelatedGroup struct {
	Related  RelatedSection
	MetaData MetaData
}

type RelatedSection struct {
	Related  []TextItem
	MetaData MetaData
}

var DiscovererSections = []string{LowerDiscovererName}

type DiscovererGroup struct {
	Discoverer DiscovererSection
	MetaData   MetaData
}

type DiscovererSection struct {
	Discoverer []TextItem
	MetaData   MetaData
}

var NoteSections = []string{LowerNoteName}

type NoteGroup struct {
	Note     NoteSection
	MetaData MetaData
}

type NoteSection struct {
	Note     []NoteType
	MetaData MetaData
}

type NoteType interface {
	StructuralNodeType
	NoteType()
}

func (TextItem) NoteType()        {}
func (DescribingGroup) NoteType() {}

var DescribingSections = []string{LowerDescribingName, LowerContentName}

type DescribingGroup struct {
	Describing DescribingSection
	Content    ContentSection
	MetaData   MetaData
}

type DescribingSection struct {
	Describing TextItem
	MetaData   MetaData
}

//////////////////////////////////////////////////////////////////////////////

type ProvidesSection struct {
	Provides []ProvidesType
	MetaData MetaData
}

//////////////////////////////////////////////////////////////////////////////

type AliasesSection struct {
	Aliases  []Alias
	MetaData MetaData
}

//////////////////////////////////////////////////////////////////////////////

type DocumentedSection struct {
	Documented []DocumentedType
	MetaData   MetaData
}

//////////////////////////////////////////////////////////////////////////////

type JustifiedSection struct {
	Justified []JustifiedType
	MetaData  MetaData
}

//////////////////////////////////////////////////////////////////////////////

type JustifiedType interface {
	StructuralNodeType
	JustifiedType()
}

func (LabelGroup) JustifiedType() {}
func (ByGroup) JustifiedType()    {}

var LabelSections = []string{LowerLabelName, LowerByName}

type LabelGroup struct {
	Label    LabelSection
	By       BySection
	MetaData MetaData
}

type LabelSection struct {
	Label    TextItem
	MetaData MetaData
}

var BySections = []string{LowerByName}

type ByGroup struct {
	By       BySection
	MetaData MetaData
}

type BySection struct {
	By       []TextItem
	MetaData MetaData
}

//////////////////////////////////////////////////////////////////////////////

type ReferencesSection struct {
	References []TextItem
	MetaData   MetaData
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
	MetaData   MetaData
}

type DescribesSection struct {
	Describes Target
	MetaData  MetaData
}

type WithSection struct {
	With     []Target
	MetaData MetaData
}

type GivenSection struct {
	Given    []Target
	MetaData MetaData
}

type UsingSection struct {
	Using    []Target
	MetaData MetaData
}

type ExtendsSection struct {
	Extends  []Clause
	MetaData MetaData
}

type SatisfiesSection struct {
	Satisfies []Clause
	MetaData  MetaData
}

//////////////////////////////////////////////////////////////////////////////

type MetaIdSection struct {
	Id       TextItem
	MetaData MetaData
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
	MetaData   MetaData
}

type DefinesSection struct {
	Defines  Target
	MetaData MetaData
}

type MeansSection struct {
	Means    Clause
	MetaData MetaData
}

type SpecifiesSection struct {
	Specifies []Clause
	MetaData  MetaData
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
	MetaData   MetaData
}

type StatesSection struct {
	MetaData MetaData
}

type ThatSection struct {
	That     []Clause
	MetaData MetaData
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
	MetaData   MetaData
}

type TopLevelProofSection struct {
	MetaData MetaData
}

type OfSection struct {
	Of       TextItem
	MetaData MetaData
}

type ContentSection struct {
	Content  TextItem
	MetaData MetaData
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
	MetaData   MetaData
}

type AxiomSection struct {
	Axiom    []TextItem
	MetaData MetaData
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
	MetaData   MetaData
}

type ConjectureSection struct {
	Conjecture []TextItem
	MetaData   MetaData
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
	MetaData   MetaData
}

type TheoremSection struct {
	Theorem  []TextItem
	MetaData MetaData
}

type ProofSection struct {
	Proof    TextItem
	MetaData MetaData
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
	MetaData   MetaData
}

type TopicSection struct {
	MetaData MetaData
}

//////////////////////////////////////////////////////////////////////////////

type SpecifyType interface {
	StructuralNodeType
	SpecifyType()
}

func (ZeroGroup) SpecifyType()          {}
func (PositiveIntGroup) SpecifyType()   {}
func (NegativeIntGroup) SpecifyType()   {}
func (PositiveFloatGroup) SpecifyType() {}
func (NegativeFloatGroup) SpecifyType() {}

var ZeroSections = []string{LowerZeroName, LowerMeansName}

type ZeroGroup struct {
	Zero     ZeroSection
	Means    MeansSection
	MetaData MetaData
}

type ZeroSection struct {
	MetaData MetaData
}

var PositiveIntSections = []string{LowerPositiveIntName, LowerMeansName}

type PositiveIntGroup struct {
	PositiveInt PositiveIntSection
	Means       MeansSection
	MetaData    MetaData
}

type PositiveIntSection struct {
	PositiveInt Target
	MetaData    MetaData
}

var NegativeIntSections = []string{LowerNegativeIntName, LowerMeansName}

type NegativeIntGroup struct {
	NegativeInt NegativeIntSection
	Means       MeansSection
	MetaData    MetaData
}

type NegativeIntSection struct {
	NegativeInt Target
	MetaData    MetaData
}

var PositiveFloatSections = []string{LowerPositiveFloatName, LowerMeansName}

type PositiveFloatGroup struct {
	PositiveFloat PositiveFloatSection
	Means         MeansSection
	MetaData      MetaData
}

type PositiveFloatSection struct {
	PositiveFloat Target
	MetaData      MetaData
}

var NegativeFloatSections = []string{LowerNegativeFloatName, LowerMeansName}

type NegativeFloatGroup struct {
	NegativeFloat NegativeFloatSection
	Means         MeansSection
	MetaData      MetaData
}

type NegativeFloatSection struct {
	NegativeFloat Target
	MetaData      MetaData
}

//////////////////////////////////////////////////////////////////////////////

var SpecifySections = []string{UpperSpecifyName, UpperIdQuestionName}

type SpecifyGroup struct {
	Specify  TopLevelSpecifySection
	MetaId   *MetaIdSection
	MetaData MetaData
}

type TopLevelSpecifySection struct {
	Specify  []SpecifyType
	MetaData MetaData
}

//////////////////////////////////////////////////////////////////////////////

var PersonSections = []string{UpperPersonName, UpperIdQuestionName}

type PersonGroup struct {
	Id       string
	Person   PersonSection
	MetaId   *MetaIdSection
	MetaData MetaData
}

type PersonSection struct {
	Items    []PersonType
	MetaData MetaData
}

type PersonType interface {
	StructuralNodeType
	PersonType()
}

func (NameGroup) PersonType()      {}
func (BiographyGroup) PersonType() {}

var NameSections = []string{LowerNameName}

type NameGroup struct {
	Name     NameSection
	MetaData MetaData
}

type NameSection struct {
	Name     []TextItem
	MetaData MetaData
}

var BiographySections = []string{LowerBiographyName}

type BiographyGroup struct {
	Biography BiographySection
	MetaData  MetaData
}

type BiographySection struct {
	Biography TextItem
	MetaData  MetaData
}

//////////////////////////////////////////////////////////////////////////////

var ResourceSections = []string{UpperResourceName, UpperIdQuestionName}

type ResourceGroup struct {
	Id       string
	Resource ResourceSection
	MetaId   *MetaIdSection
	MetaData MetaData
}

type ResourceSection struct {
	Items    []ResourceType
	MetaData MetaData
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
	StructuralNodeType
	ResourceType()
}

type TitleGroup struct {
	Title    TitleSection
	MetaData MetaData
}

type TitleSection struct {
	Title    TextItem
	MetaData MetaData
}

type AuthorGroup struct {
	Author   AuthorSection
	MetaData MetaData
}

type AuthorSection struct {
	Author   []TextItem
	MetaData MetaData
}

type OffsetGroup struct {
	Offset   OffsetSection
	MetaData MetaData
}

type OffsetSection struct {
	Offset   TextItem
	MetaData MetaData
}

type UrlGroup struct {
	Url      UrlSection
	MetaData MetaData
}

type UrlSection struct {
	Url      TextItem
	MetaData MetaData
}

type HomepageGroup struct {
	Homepage HomepageSection
	MetaData MetaData
}

type HomepageSection struct {
	Homepage TextItem
	MetaData MetaData
}

type TypeGroup struct {
	Type     TypeSection
	MetaData MetaData
}

type TypeSection struct {
	Type     TextItem
	MetaData MetaData
}

type EditorGroup struct {
	Editor   EditorSection
	MetaData MetaData
}

type EditorSection struct {
	Editor   []TextItem
	MetaData MetaData
}

type EditionGroup struct {
	Edition  EditionSection
	MetaData MetaData
}

type EditionSection struct {
	Edition  TextItem
	MetaData MetaData
}

type InstitutionGroup struct {
	Institution InstitutionSection
	MetaData    MetaData
}

type InstitutionSection struct {
	Institution []TextItem
	MetaData    MetaData
}

type JournalGroup struct {
	Journal  JournalSection
	MetaData MetaData
}

type JournalSection struct {
	Journal  []TextItem
	MetaData MetaData
}

type PublisherGroup struct {
	Publisher PublisherSection
	MetaData  MetaData
}

type PublisherSection struct {
	Publisher []TextItem
	MetaData  MetaData
}

type VolumeGroup struct {
	Volume   VolumeSection
	MetaData MetaData
}

type VolumeSection struct {
	Volume   TextItem
	MetaData MetaData
}

type MonthGroup struct {
	Month    MonthSection
	MetaData MetaData
}

type MonthSection struct {
	Month    TextItem
	MetaData MetaData
}

type YearGroup struct {
	Year     YearSection
	MetaData MetaData
}

type YearSection struct {
	Year     TextItem
	MetaData MetaData
}

type DescriptionGroup struct {
	Description DescriptionSection
	MetaData    MetaData
}

type DescriptionSection struct {
	Description TextItem
	MetaData    MetaData
}

//////////////////////////////////////////////////////////////////////////////

type TextBlockItem struct {
	Text     string
	MetaData MetaData
}

type TopLevelItemType interface {
	StructuralNodeType
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

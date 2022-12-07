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
const LowerDiscoveredName = "discovered"
const LowerElseName = "else"
const LowerElseQuestionName = LowerElseName + "?"
const LowerExamplesName = "examples"
const LowerExistsName = "exists"
const LowerExistsUniqueName = "existsUnique"
const LowerExpressingName = "expressing"
const LowerExpressedName = "expressed"
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
const LowerNotesName = "notes"
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
const UpperMetadataName = "Metadata"
const UpperMetadataQuestionName = UpperMetadataName + "?"
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
const LowerConnectionName = "connection"
const LowerToName = "to"
const LowerSignifiesName = "signifies"
const LowerSignifiesQuestionName = LowerSignifiesName + "?"
const LowerViewableName = "viewable"
const LowerViewableQuestionName = LowerViewableName + "?"
const LowerThroughQuestionName = LowerThroughName + "?"

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

var ExpressedSections = []string{LowerExpressedName}

type ExpressedGroup struct {
	Expressed ExpressedSection
}

type ExpressedSection struct {
	Expressed []TextItem
}

var CalledSections = []string{LowerCalledName}

type CalledGroup struct {
	Called CalledSection
}

type CalledSection struct {
	Called []TextItem
}

var ExpressingSections = []string{LowerExpressingName, LowerAsName}

type ExpressingGroup struct {
	Expressing ExpressingSection
	As         ExpressingAsSection
}

type ExpressingSection struct {
	Expressing []Target
}

type ExpressingAsSection struct {
	As []TextItem
}

//////////////////////////////////////////////////////////////////////////////

type DocumentedType interface {
	DocumentedType()
}

func (DetailsGroup) DocumentedType()    {}
func (OverviewGroup) DocumentedType()   {}
func (MotivationGroup) DocumentedType() {}
func (HistoryGroup) DocumentedType()    {}
func (ExamplesGroup) DocumentedType()   {}
func (RelatedGroup) DocumentedType()    {}
func (DiscoveredGroup) DocumentedType() {}
func (NotesGroup) DocumentedType()      {}
func (ExpressedGroup) DocumentedType()  {}
func (ExpressingGroup) DocumentedType() {}
func (CalledGroup) DocumentedType()     {}

var DetailsSections = []string{LowerDetailsName}

type DetailsGroup struct {
	Details DetailsSection
}

type DetailsSection struct {
	Details TextItem
}

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

var ExamplesSections = []string{LowerExamplesName}

type ExamplesGroup struct {
	Examples ExamplesSection
}

type ExamplesSection struct {
	Examples []TextItem
}

var RelatedSections = []string{LowerRelatedName}

type RelatedGroup struct {
	Related RelatedSection
}

type RelatedSection struct {
	Related []Formulation[Signature]
}

var DiscoveredSections = []string{LowerDiscoveredName}

type DiscoveredGroup struct {
	Discovered DiscoveredSection
}

type DiscoveredSection struct {
	Discovered []TextItem
}

var NotesSections = []string{LowerNotesName}

type NotesGroup struct {
	Notes NotesSection
}

type NotesSection struct {
	Notes []TextItem
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

type MetadataSection struct {
	Metadata []MetadataType
}

//////////////////////////////////////////////////////////////////////////////

type MetadataType interface {
	MetadataType()
}

func (IdGroup) MetadataType() {}

var IdSections = []string{LowerIdName}

type IdGroup struct {
	Id IdSection
}

type IdSection struct {
	Id TextItem
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
	UpperViewableQuestionName,
	UpperJustifiedQuestionName,
	UpperDocumentedQuestionName,
	UpperReferencesQuestionName,
	UpperAliasesQuestionName,
	UpperMetadataQuestionName,
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
	Metadata   *MetadataSection
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

var DefinesSections = []string{
	UpperDefinesName,
	LowerWithQuestionName,
	LowerUsingQuestionName,
	LowerWhenQuestionName,
	LowerSuchThatQuestionName,
	LowerMeansQuestionName,
	LowerSpecifiesName,
	UpperProvidesQuestionName,
	UpperViewableQuestionName,
	UpperJustifiedQuestionName,
	UpperDocumentedQuestionName,
	UpperReferencesQuestionName,
	UpperAliasesQuestionName,
	UpperMetadataQuestionName,
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
	Metadata   *MetadataSection
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
	UpperMetadataQuestionName,
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
	Metadata   *MetadataSection
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
	UpperMetadataQuestionName,
}

type ProofGroup struct {
	Id         IdItem
	Of         OfSection
	Content    ContentSection
	References *ReferencesSection
	Metadata   *MetadataSection
}

type ProofSection struct {
	Proof TextItem
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
	LowerSuchThatQuestionName,
	LowerThenName,
	LowerIffQuestionName,
	UpperDocumentedQuestionName,
	UpperReferencesQuestionName,
	UpperAliasesQuestionName,
	UpperMetadataQuestionName,
}

type AxiomGroup struct {
	Id         *IdItem
	Axiom      AxiomSection
	Given      *GivenSection
	Where      *WhereSection
	SuchThat   *SuchThatSection
	Then       ThenSection
	Iff        *IffSection
	Documented *DocumentedSection
	References *ReferencesSection
	Aliases    *AliasesSection
	Metadata   *MetadataSection
}

type AxiomSection struct {
	Axiom []TextItem
}

//////////////////////////////////////////////////////////////////////////////

var ConjectureSections = []string{
	UpperConjectureName,
	LowerGivenQuestionName,
	LowerWhereQuestionName,
	LowerSuchThatQuestionName,
	LowerThenName,
	LowerIffQuestionName,
	UpperDocumentedQuestionName,
	UpperReferencesQuestionName,
	UpperAliasesQuestionName,
	UpperMetadataQuestionName,
}

type ConjectureGroup struct {
	Id         *IdItem
	Conjecture ConjectureSection
	Given      *GivenSection
	Where      *WhereSection
	SuchThat   *SuchThatSection
	Then       ThenSection
	Iff        *IffSection
	Documented *DocumentedSection
	References *ReferencesSection
	Aliases    *AliasesSection
	Metadata   *MetadataSection
}

type ConjectureSection struct {
	Conjecture []TextItem
}

//////////////////////////////////////////////////////////////////////////////

var TheoremSections = []string{
	UpperTheoremName,
	LowerGivenQuestionName,
	LowerWhereQuestionName,
	LowerSuchThatQuestionName,
	LowerThenName,
	LowerIffQuestionName,
	UpperProofQuestionName,
	UpperDocumentedQuestionName,
	UpperReferencesQuestionName,
	UpperAliasesQuestionName,
	UpperMetadataQuestionName,
}

type TheoremGroup struct {
	Id         *IdItem
	Theorem    TheoremSection
	Given      *GivenSection
	Where      *WhereSection
	SuchThat   *SuchThatSection
	Then       ThenSection
	Iff        *IffSection
	Proof      *ProofSection
	Documented *DocumentedSection
	References *ReferencesSection
	Aliases    *AliasesSection
	Metadata   *MetadataSection
}

type TheoremSection struct {
	Theorem []TextItem
}

//////////////////////////////////////////////////////////////////////////////

var TopicSections = []string{
	UpperTopicName,
	LowerContentName,
	UpperReferencesQuestionName,
	UpperMetadataQuestionName,
}

type TopicGroup struct {
	Id         IdItem
	Topic      TopicSection
	Content    ContentSection
	References *ReferencesSection
	Metadata   *MetadataSection
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

var ZeroSections = []string{LowerZeroName, LowerIsName}

type ZeroGroup struct {
}

type ZeroSection struct {
}

type IsSection struct {
	Is Formulation[Signature]
}

type ZeroDefinesSection struct {
	Defines Clause
}

var PositiveIntSections = []string{LowerPositiveIntName, LowerIsName}

type PositiveIntGroup struct {
}

type PositiveIntSection struct {
}

var NegativeIntSections = []string{LowerNegativeIntName, LowerIsName}

type NegativeIntGroup struct {
}

type NegativeIntSection struct {
}

var PositiveFloatSections = []string{LowerPositiveFloatName, LowerIsName}

type PositiveFloatGroup struct {
}

type PositiveFloatSection struct {
}

var NegativeFloatSections = []string{LowerNegativeFloatName, LowerIsName}

type NegativeFloatGroup struct {
}

type NegativeFloatSection struct {
}

//////////////////////////////////////////////////////////////////////////////

var SpecifySections = []string{UpperSpecifyName}

type SpecifyGroup struct {
	Specify []SpecifyType
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

type Document struct {
	Items []TopLevelItemType
}

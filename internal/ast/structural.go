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

var LowerAllOfName = "allOf"
var LowerNotName = "not"
var LowerAnyOfName = "anyOf"
var LowerOneOfName = "oneOf"
var LowerExistsName = "exists"
var LowerExistsUniqueName = "existsUnique"
var LowerWhereName = "where"
var LowerWhereQuestionName = LowerWhereName + "?"
var LowerSuchThatName = "suchThat"
var LowerSuchThatQuestionName = LowerSuchThatName + "?"
var LowerForAllName = "forAll"
var LowerThenName = "then"
var LowerIfName = "if"
var LowerIffName = "iff"
var LowerGeneratedName = "generated"
var LowerFromName = "from"
var LowerWhenName = "when"
var LowerWhenQuestionName = LowerWhenName + "?"
var LowerPiecewiseName = "piecewise"
var LowerElseName = "else"
var LowerElseQuestionName = LowerElseName + "?"
var LowerViaName = "via"
var LowerAsName = "as"
var LowerAsQuestionName = LowerAsName + "?"
var LowerStatesQuestion = "states"
var LowerInfixName = "infix"
var LowerStatesName = "states"
var LowerDefinesName = "defines"
var LowerPrefixName = "prefix"
var LowerPostfixName = "postfix"
var LowerSymbolName = "symbol"
var LowerWrittenName = "written"
var LowerCalledName = "called"
var LowerWritingName = "writing"
var LowerThatName = "that"
var LowerLooselyName = "loosely"
var LowerOverviewName = "overview"
var LowerMotivationName = "motivation"
var LowerHistoryName = "history"
var LowerExamplesName = "examples"
var LowerRelatedName = "related"
var LowerDiscoveredName = "discovered"
var LowerNotesName = "notes"
var LowerLabelName = "label"
var LowerByName = "by"
var LowerIdName = "id"
var UpperDescribesName = "Describes"
var LowerWithName = "with"
var LowerWithQuestionName = LowerWithName + "?"
var LowerGivenName = "given"
var LowerGivenQuestionName = LowerGivenName + "?"
var LowerExtendsName = "extends"
var LowerExtendsQuestionName = LowerExtendsName + "?"
var LowerSatisfiesName = "satisifies"
var LowerSatisfiesQuestionName = LowerSatisfiesName + "?"
var UpperViewableName = "Viewable"
var UpperViewableQuestionName = UpperViewableName + "?"
var UpperProvidesName = "Provides"
var UpperProvidesQuestionName = UpperProofName + "?"
var UpperUsingName = "Using"
var UpperUsingQuestionName = UpperUsingName + "?"
var UpperCodifiedName = "Codified"
var UpperDocumentedName = "Documented"
var UpperDocumentedQuestionName = UpperDocumentedName + "?"
var UpperJustifiedName = "Justified"
var UpperJustifiedQuestionName = UpperJustifiedName + "?"
var UpperReferencesName = "References"
var UpperReferencesQuestionName = UpperReferencesName + "?"
var UpperMetadataName = "Metadata"
var UpperMetadataQuestionName = UpperMetadataName + "?"
var UpperDeclaresName = "Declares"
var UpperStatesName = "States"
var UpperProofName = "Proof"
var LowerOfName = "of"
var LowerContentName = "content"
var UpperAxiomName = "Axiom"
var LowerIffQuestionName = "iff"
var UpperConjectureName = "Conjecture"
var UpperTheoremName = "Theorem"
var UpperTopicName = "Topic"
var UpperNoteName = "Note"
var LowerZeroName = "zero"
var LowerIsName = "is"
var LowerPositiveIntName = "positiveInt"
var LowerNegativeIntName = "negativeInt"
var LowerPositiveFloatName = "positiveFloat"
var LowerNegativeFloatName = "negativeFloat"
var UpperSpecifyName = "Specify"
var LowerThroughName = "through"
var LowerMeansName = "means"
var LowerMeansQuestionName = LowerMeansName + "?"
var LowerDefinesQuestionName = LowerDefinesName + "?"
var LowerStatesQuestionName = LowerStatesName + "?"
var LowerViewName = "view"

///////////////////////////////////////////////////////////////////////

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

type IdItem = Formulation[NodeType]
type Target = Formulation[NodeType]
type Spec = Formulation[NodeType]

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
	LowerSuchThatQuestionName,
}

type ExistsUniqueGroup struct {
	ExistsUnique ExistsUniqueSection
	Where        *WhereSection
	SuchThat     *SuchThatSection
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

var GeneratedSections = []string{
	LowerGeneratedName,
	LowerFromName,
	LowerWhenQuestionName,
}

type GeneratedGroup struct {
	Generated GeneratedSection
	From      FromSection
	When      *WhenSection
}

type GeneratedSection struct{}

type FromSection struct {
	Items []Target
}

type WhenSection struct {
	Items []Clause
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

type ViewableType interface {
	ViewableType()
}

func (AsViaGroup) ViewableType()     {}
func (AsThroughGroup) ViewableType() {}

var AsViaSections = []string{
	LowerAsName,
	"via",
}

type AsViaGroup struct {
	As  AsSection
	Via ViaSection
}

var AsThroughStatesSections = []string{
	LowerAsName,
	LowerThroughName,
	LowerAsQuestionName,
	LowerStatesQuestionName,
}

type AsThroughGroup struct {
	As        AsSection
	Through   ThroughSection
	ThroughAs *AsSection
	States    *AsStatesSection
}

type AsSection struct {
	As Formulation[Signature]
}

type ViaSection struct {
	Via Clause
}

type ThroughSection struct {
	Through Spec
}

type AsStatesSection struct {
	As Formulation[Signature]
}

//////////////////////////////////////////////////////////////////////////////

type ProvidesType interface {
	ProvidesType()
}

func (InfixGroup) ProvidesType()   {}
func (PrefixGroup) ProvidesType()  {}
func (PostfixGroup) ProvidesType() {}
func (SymbolGroup) ProvidesType()  {}

var InfixStatesSections = []string{
	LowerInfixName,
	LowerWhenQuestionName,
	LowerStatesName,
}

var InfixDefinesSections = []string{
	LowerInfixName,
	LowerWhenQuestionName,
	LowerDefinesName,
}

type InfixGroup struct {
	Infix         InfixSection
	When          *WhenSection
	StatesDefines ProvidesStatesDefinesType
}

var PrefixStatesSections = []string{
	LowerPrefixName,
	LowerWhenQuestionName,
	LowerStatesName,
}

var PrefixDefinesSections = []string{
	LowerPrefixName,
	LowerWhenQuestionName,
	LowerDefinesName,
}

type PrefixGroup struct {
	Prefix        PrefixSection
	When          *WhenSection
	StatesDefines ProvidesStatesDefinesType
}

var PostfixStatesSections = []string{
	LowerPostfixName,
	LowerWhenQuestionName,
	LowerStatesName,
}

var PostfixDefinesSections = []string{
	LowerPostfixName,
	LowerWhenQuestionName,
	LowerDefinesName,
}

type PostfixGroup struct {
	Postfix       PostfixSection
	When          *WhenSection
	StatesDefines ProvidesStatesDefinesType
}

var SymbolSections = []string{
	LowerSymbolName,
	LowerDefinesName,
}

type SymbolGroup struct {
	Symbol  SymbolSection
	Defines ProvidesDefinesSection
}

type InfixSection struct {
	Infix []Target
}

type PrefixSection struct {
	Prefix []Target
}

type PostfixSection struct {
	Postfix []Target
}

type SymbolSection struct {
	Symbol []Target
}

type ProvidesStatesDefinesType interface {
	ProvidesStatesDefinesType()
}

func (ProvidesStatesSection) ProvidesStatesDefinesType()  {}
func (ProvidesDefinesSection) ProvidesStatesDefinesType() {}

type ProvidesStatesSection struct {
	Clause Clause
}

type ProvidesDefinesSection struct {
	Clause Clause
}

//////////////////////////////////////////////////////////////////////////////

type CodifiedSection struct {
	Codified []CodifiedType
}

type CodifiedType interface {
	CodifiedType()
}

func (WrittenGroup) CodifiedType() {}
func (CalledGroup) CodifiedType()  {}
func (WritingGroup) CodifiedType() {}

var WrittenSections = []string{LowerWrittenName}

type WrittenGroup struct {
	Written WrittenSection
}

type WrittenSection struct {
	Written []TextItem
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
	Writing []Target
}

type WritingAsSection struct {
	As []TextItem
}

//////////////////////////////////////////////////////////////////////////////

type DocumentedType interface {
	DocumentedType()
}

func (LooselyGroup) DocumentedType()    {}
func (OverviewGroup) DocumentedType()   {}
func (MotivationGroup) DocumentedType() {}
func (HistoryGroup) DocumentedType()    {}
func (ExamplesGroup) DocumentedType()   {}
func (RelatedGroup) DocumentedType()    {}
func (DiscoveredGroup) DocumentedType() {}
func (NotesGroup) DocumentedType()      {}

var LooselySections = []string{LowerLooselyName}

type LooselyGroup struct {
	Loosely LooselySection
}

type LooselySection struct {
	Loosely TextItem
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

type ViewableSection struct {
	Viewable []ViewableType
}

//////////////////////////////////////////////////////////////////////////////

type ProvidesSection struct {
	Provides []ProvidesType
}

//////////////////////////////////////////////////////////////////////////////

type UsingSection struct {
	Using []Clause
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
	LowerGivenQuestionName,
	LowerWhenQuestionName,
	LowerSuchThatQuestionName,
	LowerExtendsQuestionName,
	LowerSatisfiesQuestionName,
	UpperViewableQuestionName,
	UpperProvidesQuestionName,
	UpperUsingQuestionName,
	UpperCodifiedName,
	UpperDocumentedQuestionName,
	UpperJustifiedQuestionName,
	UpperReferencesQuestionName,
	UpperMetadataQuestionName,
}

type DescribesGroup struct {
	Id         IdItem
	Describes  DescribesSection
	With       *WithSection
	Given      *GivenSection
	When       *WhenSection
	SuchThat   *SuchThatSection
	Extends    *ExtendsSection
	Satisfies  *SatisfiesSection
	Viewable   *ViewableSection
	Provides   *ProvidesSection
	Using      *UsingSection
	Codified   CodifiedSection
	Documented *DocumentedSection
	Justified  *JustifiedSection
	References *ReferencesSection
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

type ExtendsSection struct {
	Extends []Clause
}

type SatisfiesSection struct {
	Satisfies []Clause
}

//////////////////////////////////////////////////////////////////////////////

var DeclaresSections = []string{
	UpperDeclaresName,
	LowerWithQuestionName,
	LowerGivenQuestionName,
	LowerWhenQuestionName,
	LowerSuchThatQuestionName,
	LowerMeansQuestionName,
	LowerDefinesQuestionName,
	UpperViewableQuestionName,
	UpperProvidesQuestionName,
	UpperUsingQuestionName,
	UpperCodifiedName,
	UpperDocumentedQuestionName,
	UpperJustifiedQuestionName,
	UpperReferencesQuestionName,
	UpperMetadataQuestionName,
}

type DeclaresGroup struct {
	Id         IdItem
	Declares   DeclaresSection
	With       *WithSection
	Given      *GivenSection
	When       *WhenSection
	SuchThat   *SuchThatSection
	Means      *MeansSection
	Defines    *DefinesSection
	Viewable   *ViewableSection
	Provides   *ProvidesSection
	Using      *UsingSection
	Codified   CodifiedSection
	Documented *DocumentedSection
	Justified  *JustifiedSection
	References *ReferencesSection
	Metadata   *MetadataSection
}

type DeclaresSection struct {
	Declares Target
}

type MeansSection struct {
	Means Clause
}

type DefinesSection struct {
	Defines []Clause
}

//////////////////////////////////////////////////////////////////////////////

var StatesSections = []string{
	LowerStatesName,
	LowerGivenQuestionName,
	LowerWhenQuestionName,
	LowerSuchThatQuestionName,
	LowerThatName,
	UpperUsingQuestionName,
	UpperCodifiedName,
	UpperDocumentedQuestionName,
	UpperJustifiedQuestionName,
	UpperReferencesQuestionName,
	UpperMetadataQuestionName,
}

type StatesGroup struct {
	Id         IdItem
	States     StatesSection
	Given      *GivenSection
	When       *WhenSection
	SuchThat   *SuchThatSection
	That       ThatSection
	Using      *UsingSection
	Codified   CodifiedSection
	Documented *DocumentedSection
	Justified  *JustifiedSection
	References *ReferencesSection
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
	UpperUsingQuestionName,
	UpperDocumentedQuestionName,
	UpperReferencesQuestionName,
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
	Using      *UsingSection
	Documented *DocumentedSection
	References *ReferencesSection
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
	UpperUsingQuestionName,
	UpperDocumentedQuestionName,
	UpperReferencesQuestionName,
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
	Using      *UsingSection
	Documented *DocumentedSection
	References *ReferencesSection
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
	UpperUsingQuestionName,
	UpperProofName,
	UpperDocumentedQuestionName,
	UpperReferencesQuestionName,
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
	Using      *UsingSection
	Proof      *ProofSection
	Documented *DocumentedSection
	References *ReferencesSection
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

var NoteSections = []string{
	UpperNoteName,
	LowerContentName,
	UpperMetadataQuestionName,
}

type NoteGroup struct {
	Note     NoteSection
	Content  ContentSection
	Metadata *MetadataSection
}

type NoteSection struct {
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
func (DeclaresGroup) TopLevelItemType()   {}
func (DescribesGroup) TopLevelItemType()  {}
func (StatesGroup) TopLevelItemType()     {}
func (AxiomGroup) TopLevelItemType()      {}
func (ConjectureGroup) TopLevelItemType() {}
func (TheoremGroup) TopLevelItemType()    {}
func (SpecifyGroup) TopLevelItemType()    {}
func (NoteGroup) TopLevelItemType()       {}
func (TopicGroup) TopLevelItemType()      {}

type Document struct {
	Items []TopLevelItemType
}

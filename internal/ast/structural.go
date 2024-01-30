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

type GroupLabel struct {
	Label string
	Start Position
}

type IdItem struct {
	RawText        string
	Root           FormulationNodeKind
	CommonMetaData CommonMetaData
}

type Target struct {
	RawText        string
	Root           FormulationNodeKind
	CommonMetaData CommonMetaData
}

type Spec struct {
	RawText        string
	Root           FormulationNodeKind
	Label          *string
	CommonMetaData CommonMetaData
}

type Alias struct {
	RawText        string
	Root           FormulationNodeKind
	Label          *string
	CommonMetaData CommonMetaData
}

func (*Alias) ProvidesKind() {}

type TopFormulationMetaData struct {
	UsedSignatureStrings []string
}

type Formulation[T FormulationNodeKind] struct {
	RawText             string
	Root                T
	Label               *string
	CommonMetaData      CommonMetaData
	FormulationMetaData TopFormulationMetaData
}

type TextItem struct {
	RawText        string
	CommonMetaData CommonMetaData
}

////////////////////////////////////////////////////////////////////////////////////////////////////

var LetSections = []string{
	LowerLetName,
	LowerUsingQuestionName,
	LowerWhereQuestionName,
	LowerSuchThatQuestionName,
	LowerThenName,
}

type LetGroup struct {
	Label          *GroupLabel
	Let            LetSection
	Using          *UsingSection
	Where          *WhereSection
	SuchThat       *SuchThatSection
	Then           ThenSection
	CommonMetaData CommonMetaData
}

type LetSection struct {
	Let            []Target
	CommonMetaData CommonMetaData
}

////////////////////////////////////////////////////////////////////////////////////////////////////

var EquivalentlySections = []string{LowerEquivalentlyName}

type EquivalentlyGroup struct {
	Label          *GroupLabel
	Equivalently   EquivalentlySection
	CommonMetaData CommonMetaData
}

type EquivalentlySection struct {
	Clauses        []ClauseKind
	CommonMetaData CommonMetaData
}

////////////////////////////////////////////////////////////////////////////////////////////////////

var AllOfSections = []string{LowerAllOfName}

type AllOfGroup struct {
	Label          *GroupLabel
	AllOf          AllOfSection
	CommonMetaData CommonMetaData
}

type AllOfSection struct {
	Clauses        []ClauseKind
	CommonMetaData CommonMetaData
}

////////////////////////////////////////////////////////////////////////////////////////////////////

var NotSections = []string{LowerNotName}

type NotGroup struct {
	Label          *GroupLabel
	Not            NotSection
	CommonMetaData CommonMetaData
}

type NotSection struct {
	Clause         ClauseKind
	CommonMetaData CommonMetaData
}

////////////////////////////////////////////////////////////////////////////////////////////////////

var AnyOfSections = []string{LowerAnyOfName}

type AnyOfGroup struct {
	Label          *GroupLabel
	AnyOf          AnyOfSection
	CommonMetaData CommonMetaData
}

type AnyOfSection struct {
	Clauses        []ClauseKind
	CommonMetaData CommonMetaData
}

////////////////////////////////////////////////////////////////////////////////////////////////////

var OneOfSections = []string{LowerOneOfName}

type OneOfGroup struct {
	Label          *GroupLabel
	OneOf          OneOfSection
	CommonMetaData CommonMetaData
}

type OneOfSection struct {
	Clauses        []ClauseKind
	CommonMetaData CommonMetaData
}

////////////////////////////////////////////////////////////////////////////////////////////////////

var ExistsSections = []string{
	LowerExistsName,
	LowerUsingQuestionName,
	LowerWhereQuestionName,
	LowerSuchThatName,
}

type ExistsGroup struct {
	Label          *GroupLabel
	Exists         ExistsSection
	Using          *UsingSection
	Where          *WhereSection
	SuchThat       *SuchThatSection
	CommonMetaData CommonMetaData
}

type ExistsSection struct {
	Targets        []Target
	CommonMetaData CommonMetaData
}

type WhereSection struct {
	Specs          []Spec
	CommonMetaData CommonMetaData
}

type SuchThatSection struct {
	Clauses        []ClauseKind
	CommonMetaData CommonMetaData
}

////////////////////////////////////////////////////////////////////////////////////////////////////

var ExistsUniqueSections = []string{
	LowerExistsUniqueName,
	LowerUsingQuestionName,
	LowerWhereQuestionName,
	LowerSuchThatName,
}

type ExistsUniqueGroup struct {
	Label          *GroupLabel
	ExistsUnique   ExistsUniqueSection
	Using          *UsingSection
	Where          *WhereSection
	SuchThat       SuchThatSection
	CommonMetaData CommonMetaData
}

type ExistsUniqueSection struct {
	Targets        []Target
	CommonMetaData CommonMetaData
}

////////////////////////////////////////////////////////////////////////////////////////////////////

var ForAllSections = []string{
	LowerForAllName,
	LowerUsingQuestionName,
	LowerWhereQuestionName,
	LowerSuchThatQuestionName,
	LowerThenName,
}

type ForAllGroup struct {
	Label          *GroupLabel
	ForAll         ForAllSection
	Using          *UsingSection
	Where          *WhereSection
	SuchThat       *SuchThatSection
	Then           ThenSection
	CommonMetaData CommonMetaData
}

type ForAllSection struct {
	Targets        []Target
	CommonMetaData CommonMetaData
}

type ThenSection struct {
	Clauses        []ClauseKind
	CommonMetaData CommonMetaData
}

////////////////////////////////////////////////////////////////////////////////////////////////////

var IfSections = []string{
	LowerIfName,
	LowerThenName,
}

type IfGroup struct {
	Label          *GroupLabel
	If             IfSection
	Then           ThenSection
	CommonMetaData CommonMetaData
}

type IfSection struct {
	Clauses        []ClauseKind
	CommonMetaData CommonMetaData
}

////////////////////////////////////////////////////////////////////////////////////////////////////

var IffSections = []string{
	LowerIffName,
	LowerThenName,
}

type IffGroup struct {
	Label          *GroupLabel
	Iff            IffSection
	Then           ThenSection
	CommonMetaData CommonMetaData
}

type IffSection struct {
	Clauses        []ClauseKind
	CommonMetaData CommonMetaData
}

////////////////////////////////////////////////////////////////////////////////////////////////////

type WhenSection struct {
	When           []ClauseKind
	CommonMetaData CommonMetaData
}

////////////////////////////////////////////////////////////////////////////////////////////////////

type IfThen struct {
	If             IfSection
	Then           ThenSection
	CommonMetaData CommonMetaData
}

type ElseIfThen struct {
	ElseIf         ElseIfSection
	Then           ThenSection
	CommonMetaData CommonMetaData
}

type ElseIfSection struct {
	Clauses        []ClauseKind
	CommonMetaData CommonMetaData
}

var PiecewiseSections = []string{
	LowerPiecewiseName,
	LowerIfName,
	LowerThenName,
	LowerElseIfQuestionName,
	LowerThenQuestionName,
	LowerElseQuestionName,
}

type PiecewiseGroup struct {
	Label          *GroupLabel
	Piecewise      PiecewiseSection
	IfThen         IfThen
	ElseIfThen     []ElseIfThen
	Else           *ElseSection
	CommonMetaData CommonMetaData
}

type PiecewiseSection struct {
	CommonMetaData CommonMetaData
}

type ElseSection struct {
	Items          []ClauseKind
	CommonMetaData CommonMetaData
}

////////////////////////////////////////////////////////////////////////////////////////////////////

var WhenSections = []string{
	LowerWhenName,
	LowerThenName,
}

type WhenGroup struct {
	Label          *GroupLabel
	When           WhenSection
	Then           ThenSection
	CommonMetaData CommonMetaData
}

////////////////////////////////////////////////////////////////////////////////////////////////////

var SymbolWrittenSections = []string{
	LowerSymbolName,
	LowerWrittenQuestionName,
}

type SymbolWrittenGroup struct {
	Label          *GroupLabel
	Symbol         SymbolSection
	Written        *WrittenSection
	CommonMetaData CommonMetaData
}

type SymbolSection struct {
	Symbol         Alias
	CommonMetaData CommonMetaData
}

type WrittenSection struct {
	Written        []TextItem
	CommonMetaData CommonMetaData
}

var ViewSections = []string{
	LowerViewName,
	LowerAsName,
	LowerUsingQuestionName,
	LowerWhereQuestionName,
	LowerThroughQuestionName,
	LowerSignifiesQuestionName,
}

type ViewGroup struct {
	Label          *GroupLabel
	View           ViewSection
	As             AsSection
	Using          *UsingSection
	Where          *WhereSection
	Through        *ThroughSection
	Signfies       *SignifiesSection
	CommonMetaData CommonMetaData
}

type ViewSection struct {
	CommonMetaData CommonMetaData
}

type AsSection struct {
	As             Target
	CommonMetaData CommonMetaData
}

type SignifiesSection struct {
	Signifies      []Spec
	CommonMetaData CommonMetaData
}

type ThroughSection struct {
	Through        []Formulation[FormulationNodeKind]
	CommonMetaData CommonMetaData
}

////////////////////////////////////////////////////////////////////////////////////////////////////

var EncodingSections = []string{
	LowerEncodingName,
	LowerAsName,
	LowerUsingQuestionName,
	LowerWhereQuestionName,
	LowerThroughQuestionName,
}

type EncodingGroup struct {
	Label          *GroupLabel
	Encoding       EncodingSection
	As             AsSection
	Using          *UsingSection
	Where          *WhereSection
	Through        *ThroughSection
	CommonMetaData CommonMetaData
}

type EncodingSection struct {
	CommonMetaData CommonMetaData
}

////////////////////////////////////////////////////////////////////////////////////////////////////

var WrittenSections = []string{LowerWrittenName}

type WrittenGroup struct {
	Label          *GroupLabel
	Written        WrittenSection
	CommonMetaData CommonMetaData
}

var CalledSections = []string{LowerCalledName}

type CalledGroup struct {
	Label          *GroupLabel
	Called         CalledSection
	CommonMetaData CommonMetaData
}

type CalledSection struct {
	Called         []TextItem
	CommonMetaData CommonMetaData
}

var WritingSections = []string{LowerWritingName}

type WritingGroup struct {
	Label          *GroupLabel
	Writing        WritingSection
	CommonMetaData CommonMetaData
}

type WritingSection struct {
	Writing        []TextItem
	CommonMetaData CommonMetaData
}

////////////////////////////////////////////////////////////////////////////////////////////////////

var OverviewSections = []string{LowerOverviewName}

type OverviewGroup struct {
	Label          *GroupLabel
	Overview       OverviewSection
	CommonMetaData CommonMetaData
}

type OverviewSection struct {
	Overview       TextItem
	CommonMetaData CommonMetaData
}

var RelatedSections = []string{LowerRelatedName}

type RelatedGroup struct {
	Label          *GroupLabel
	Related        RelatedSection
	CommonMetaData CommonMetaData
}

type RelatedSection struct {
	Related        []TextItem
	CommonMetaData CommonMetaData
}

////////////////////////////////////////////////////////////////////////////////////////////////////

type ProvidesSection struct {
	Provides       []ProvidesKind
	CommonMetaData CommonMetaData
}

////////////////////////////////////////////////////////////////////////////////////////////////////

type AliasesSection struct {
	Aliases        []Alias
	CommonMetaData CommonMetaData
}

////////////////////////////////////////////////////////////////////////////////////////////////////

type DocumentedSection struct {
	Documented     []DocumentedKind
	CommonMetaData CommonMetaData
}

////////////////////////////////////////////////////////////////////////////////////////////////////

type JustifiedSection struct {
	Justified      []JustifiedKind
	CommonMetaData CommonMetaData
}

////////////////////////////////////////////////////////////////////////////////////////////////////

var LabelSections = []string{LowerLabelName, LowerByName}

type LabelGroup struct {
	Label          LabelSection
	By             BySection
	CommonMetaData CommonMetaData
}

type LabelSection struct {
	Label          TextItem
	CommonMetaData CommonMetaData
}

var BySections = []string{LowerByName}

type ByGroup struct {
	By             BySection
	CommonMetaData CommonMetaData
}

type BySection struct {
	By             []TextItem
	CommonMetaData CommonMetaData
}

////////////////////////////////////////////////////////////////////////////////////////////////////

type ReferencesSection struct {
	References     []TextItem
	CommonMetaData CommonMetaData
}

////////////////////////////////////////////////////////////////////////////////////////////////////

var DescribesSections = []string{
	UpperDescribesName,
	LowerUsingQuestionName,
	LowerWhenQuestionName,
	LowerSuchThatQuestionName,
	LowerExtendsQuestionName,
	LowerSpecifiesQuestionName,
	UpperProvidesQuestionName,
	UpperJustifiedQuestionName,
	UpperDocumentedQuestionName,
	UpperReferencesQuestionName,
	UpperAliasesQuestionName,
	UpperIdQuestionName,
}

type DescribesGroup struct {
	Id             IdItem
	Describes      DescribesSection
	Using          *UsingSection
	When           *WhenSection
	SuchThat       *SuchThatSection
	Extends        *ExtendsSection
	Specifies      *SpecifiesSection
	Provides       *ProvidesSection
	Justified      *JustifiedSection
	Documented     *DocumentedSection
	References     *ReferencesSection
	Aliases        *AliasesSection
	MetaId         *MetaIdSection
	CommonMetaData CommonMetaData
}

type DescribesSection struct {
	Describes      Target
	CommonMetaData CommonMetaData
}

type GivenSection struct {
	Given          []Target
	CommonMetaData CommonMetaData
}

type UsingSection struct {
	Using          []Target
	CommonMetaData CommonMetaData
}

type ExtendsSection struct {
	Extends        []ClauseKind
	CommonMetaData CommonMetaData
}

type SpecifiesSection struct {
	Specifies      []ClauseKind
	CommonMetaData CommonMetaData
}

////////////////////////////////////////////////////////////////////////////////////////////////////

type MetaIdSection struct {
	Id             TextItem
	CommonMetaData CommonMetaData
}

////////////////////////////////////////////////////////////////////////////////////////////////////

var LowerDefineSections = []string{
	LowerDefineName,
	LowerUsingQuestionName,
	LowerWhenQuestionName,
	LowerSuchThatQuestionName,
	LowerMeansQuestionName,
	LowerAsName,
}

type LowerDefineGroup struct {
	Define         LowerDefineSection
	Using          *UsingSection
	When           *WhenSection
	SuchThat       *SuchThatSection
	Means          *MeansSection
	As             DefineAsSection
	CommonMetaData CommonMetaData
}

type LowerDefineSection struct {
	Define         Target
	CommonMetaData CommonMetaData
}

type DefineAsSection struct {
	As             []ClauseKind
	CommonMetaData CommonMetaData
}

////////////////////////////////////////////////////////////////////////////////////////////////////

var DefinesSections = []string{
	UpperDefinesName,
	LowerUsingQuestionName,
	LowerWhenQuestionName,
	LowerSuchThatQuestionName,
	LowerMeansQuestionName,
	LowerExpressesQuestionName,
	UpperProvidesQuestionName,
	UpperJustifiedQuestionName,
	UpperDocumentedQuestionName,
	UpperReferencesQuestionName,
	UpperAliasesQuestionName,
	UpperIdQuestionName,
}

type DefinesGroup struct {
	Id             IdItem
	Defines        DefinesSection
	Using          *UsingSection
	When           *WhenSection
	SuchThat       *SuchThatSection
	Means          *MeansSection
	Expresses      *ExpressesSection
	Provides       *ProvidesSection
	Justified      *JustifiedSection
	Documented     *DocumentedSection
	References     *ReferencesSection
	Aliases        *AliasesSection
	MetaId         *MetaIdSection
	CommonMetaData CommonMetaData
}

type DefinesSection struct {
	Defines        Target
	CommonMetaData CommonMetaData
}

type MeansSection struct {
	Means          []ClauseKind
	CommonMetaData CommonMetaData
}

type ExpressesSection struct {
	Expresses      []ClauseKind
	CommonMetaData CommonMetaData
}

////////////////////////////////////////////////////////////////////////////////////////////////////

var CapturesSections = []string{
	UpperCapturesName,
	UpperJustifiedQuestionName,
	UpperDocumentedQuestionName,
	UpperReferencesQuestionName,
	UpperIdQuestionName,
}

type CapturesGroup struct {
	Id             IdItem
	Captures       CapturesSection
	Justified      *JustifiedSection
	Documented     *DocumentedSection
	References     *ReferencesSection
	MetaId         *MetaIdSection
	CommonMetaData CommonMetaData
}

type CapturesSection struct {
	Captures       []Formulation[FormulationNodeKind]
	CommonMetaData CommonMetaData
}

////////////////////////////////////////////////////////////////////////////////////////////////////

var StatesSections = []string{
	UpperStatesName,
	LowerUsingQuestionName,
	LowerWhenQuestionName,
	LowerSuchThatQuestionName,
	LowerThatName,
	UpperJustifiedQuestionName,
	UpperDocumentedQuestionName,
	UpperReferencesQuestionName,
	UpperAliasesQuestionName,
	UpperIdQuestionName,
}

type StatesGroup struct {
	Id             IdItem
	States         StatesSection
	Using          *UsingSection
	When           *WhenSection
	SuchThat       *SuchThatSection
	That           ThatSection
	Justified      *JustifiedSection
	Documented     *DocumentedSection
	References     *ReferencesSection
	Aliases        *AliasesSection
	MetaId         *MetaIdSection
	CommonMetaData CommonMetaData
}

type StatesSection struct {
	CommonMetaData CommonMetaData
}

type ThatSection struct {
	That           []ClauseKind
	CommonMetaData CommonMetaData
}

////////////////////////////////////////////////////////////////////////////////////////////////////

var AxiomSections = []string{
	UpperAxiomName,
	LowerGivenQuestionName,
	LowerUsingQuestionName,
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
	Id             *IdItem
	Axiom          AxiomSection
	Given          *GivenSection
	Using          *UsingSection
	Where          *WhereSection
	If             *IfSection
	Iff            *IffSection
	Then           ThenSection
	Documented     *DocumentedSection
	References     *ReferencesSection
	Aliases        *AliasesSection
	MetaId         *MetaIdSection
	CommonMetaData CommonMetaData
}

type AxiomSection struct {
	CommonMetaData CommonMetaData
}

////////////////////////////////////////////////////////////////////////////////////////////////////

var ConjectureSections = []string{
	UpperConjectureName,
	LowerGivenQuestionName,
	LowerUsingQuestionName,
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
	Id             *IdItem
	Conjecture     ConjectureSection
	Given          *GivenSection
	Using          *UsingSection
	Where          *WhereSection
	If             *IfSection
	Iff            *IffSection
	Then           ThenSection
	Documented     *DocumentedSection
	References     *ReferencesSection
	Aliases        *AliasesSection
	MetaId         *MetaIdSection
	CommonMetaData CommonMetaData
}

type ConjectureSection struct {
	CommonMetaData CommonMetaData
}

////////////////////////////////////////////////////////////////////////////////////////////////////

var TheoremSections = []string{
	UpperTheoremName,
	LowerGivenQuestionName,
	LowerUsingQuestionName,
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
	Id             *IdItem
	Theorem        TheoremSection
	Given          *GivenSection
	Using          *UsingSection
	Where          *WhereSection
	If             *IfSection
	Iff            *IffSection
	Then           ThenSection
	Proof          *ProofSection
	Documented     *DocumentedSection
	References     *ReferencesSection
	Aliases        *AliasesSection
	MetaId         *MetaIdSection
	CommonMetaData CommonMetaData
}

type TheoremSection struct {
	CommonMetaData CommonMetaData
}

type ProofSection struct {
	Proof          []ProofItemKind
	CommonMetaData CommonMetaData
}

////////////////////////////////////////////////////////////////////////////////////////////////////

var LemmaSections = []string{
	UpperLemmaName,
	LowerForName,
	LowerGivenQuestionName,
	LowerUsingQuestionName,
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

type LemmaGroup struct {
	Id             *IdItem
	Lemma          LemmaSection
	For            LemmaForSection
	Given          *GivenSection
	Using          *UsingSection
	Where          *WhereSection
	If             *IfSection
	Iff            *IffSection
	Then           ThenSection
	Proof          *ProofSection
	Documented     *DocumentedSection
	References     *ReferencesSection
	Aliases        *AliasesSection
	MetaId         *MetaIdSection
	CommonMetaData CommonMetaData
}

type LemmaSection struct {
	CommonMetaData CommonMetaData
}

type LemmaForSection struct {
	For            []TextItem
	CommonMetaData CommonMetaData
}

////////////////////////////////////////////////////////////////////////////////////////////////////

var CorollarySections = []string{
	UpperCorollaryName,
	LowerToName,
	LowerGivenQuestionName,
	LowerUsingQuestionName,
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

type CorollaryGroup struct {
	Id             *IdItem
	Corollary      CorollarySection
	To             CorollaryToSection
	Given          *GivenSection
	Using          *UsingSection
	Where          *WhereSection
	If             *IfSection
	Iff            *IffSection
	Then           ThenSection
	Proof          *ProofSection
	Documented     *DocumentedSection
	References     *ReferencesSection
	Aliases        *AliasesSection
	MetaId         *MetaIdSection
	CommonMetaData CommonMetaData
}

type CorollarySection struct {
	CommonMetaData CommonMetaData
}

type CorollaryToSection struct {
	To             []TextItem
	CommonMetaData CommonMetaData
}

////////////////////////////////////////////////////////////////////////////////////////////////////

var ZeroSections = []string{LowerZeroName, LowerMeansName}

type ZeroGroup struct {
	Zero           ZeroSection
	SingleMeans    SingleMeansSection
	CommonMetaData CommonMetaData
}

type SingleMeansSection struct {
	Means          ClauseKind
	CommonMetaData CommonMetaData
}

type ZeroSection struct {
	CommonMetaData CommonMetaData
}

var PositiveIntSections = []string{LowerPositiveIntName, LowerMeansName}

type PositiveIntGroup struct {
	PositiveInt    PositiveIntSection
	SingleMeans    SingleMeansSection
	CommonMetaData CommonMetaData
}

type PositiveIntSection struct {
	PositiveInt    Target
	CommonMetaData CommonMetaData
}

var NegativeIntSections = []string{LowerNegativeIntName, LowerMeansName}

type NegativeIntGroup struct {
	NegativeInt    NegativeIntSection
	SingleMeans    SingleMeansSection
	CommonMetaData CommonMetaData
}

type NegativeIntSection struct {
	NegativeInt    Target
	CommonMetaData CommonMetaData
}

var PositiveFloatSections = []string{LowerPositiveFloatName, LowerMeansName}

type PositiveFloatGroup struct {
	PositiveFloat  PositiveFloatSection
	SingleMeans    SingleMeansSection
	CommonMetaData CommonMetaData
}

type PositiveFloatSection struct {
	PositiveFloat  Target
	CommonMetaData CommonMetaData
}

var NegativeFloatSections = []string{LowerNegativeFloatName, LowerMeansName}

type NegativeFloatGroup struct {
	NegativeFloat  NegativeFloatSection
	SingleMeans    SingleMeansSection
	CommonMetaData CommonMetaData
}

type NegativeFloatSection struct {
	NegativeFloat  Target
	CommonMetaData CommonMetaData
}

////////////////////////////////////////////////////////////////////////////////////////////////////

var SpecifySections = []string{UpperSpecifyName, UpperIdQuestionName}

type SpecifyGroup struct {
	Specify        TopLevelSpecifySection
	MetaId         *MetaIdSection
	CommonMetaData CommonMetaData
}

type TopLevelSpecifySection struct {
	Specify        []SpecifyKind
	CommonMetaData CommonMetaData
}

////////////////////////////////////////////////////////////////////////////////////////////////////

var PersonSections = []string{UpperPersonName, UpperIdQuestionName}

type PersonGroup struct {
	Id             string
	Person         PersonSection
	MetaId         *MetaIdSection
	CommonMetaData CommonMetaData
}

type PersonSection struct {
	Items          []PersonKind
	CommonMetaData CommonMetaData
}

////////////////////////////////////////////////////////////////////////////////////////////////////

var NameSections = []string{LowerNameName}

type NameGroup struct {
	Name           NameSection
	CommonMetaData CommonMetaData
}

type NameSection struct {
	Name           []TextItem
	CommonMetaData CommonMetaData
}

var BiographySections = []string{LowerBiographyName}

type BiographyGroup struct {
	Biography      BiographySection
	CommonMetaData CommonMetaData
}

type BiographySection struct {
	Biography      TextItem
	CommonMetaData CommonMetaData
}

////////////////////////////////////////////////////////////////////////////////////////////////////

var ResourceSections = []string{UpperResourceName, UpperIdQuestionName}

type ResourceGroup struct {
	Id             string
	Resource       ResourceSection
	MetaId         *MetaIdSection
	CommonMetaData CommonMetaData
}

type ResourceSection struct {
	Items          []ResourceKind
	CommonMetaData CommonMetaData
}

var TitleSections = []string{LowerTitleName}

type TitleGroup struct {
	Title          TitleSection
	CommonMetaData CommonMetaData
}

type TitleSection struct {
	Title          TextItem
	CommonMetaData CommonMetaData
}

var AuthorSections = []string{LowerAuthorName}

type AuthorGroup struct {
	Author         AuthorSection
	CommonMetaData CommonMetaData
}

type AuthorSection struct {
	Author         []TextItem
	CommonMetaData CommonMetaData
}

var OffsetSections = []string{LowerOffsetName}

type OffsetGroup struct {
	Offset         OffsetSection
	CommonMetaData CommonMetaData
}

type OffsetSection struct {
	Offset         TextItem
	CommonMetaData CommonMetaData
}

var UrlSections = []string{LowerUrlName}

type UrlGroup struct {
	Url            UrlSection
	CommonMetaData CommonMetaData
}

type UrlSection struct {
	Url            TextItem
	CommonMetaData CommonMetaData
}

var HomepageSections = []string{LowerHomepageName}

type HomepageGroup struct {
	Homepage       HomepageSection
	CommonMetaData CommonMetaData
}

type HomepageSection struct {
	Homepage       TextItem
	CommonMetaData CommonMetaData
}

var TypeSections = []string{LowerTypeName}

type TypeGroup struct {
	Type           TypeSection
	CommonMetaData CommonMetaData
}

type TypeSection struct {
	Type           TextItem
	CommonMetaData CommonMetaData
}

var EditorSections = []string{LowerEditorName}

type EditorGroup struct {
	Editor         EditorSection
	CommonMetaData CommonMetaData
}

type EditorSection struct {
	Editor         []TextItem
	CommonMetaData CommonMetaData
}

var EditionSections = []string{LowerEditionName}

type EditionGroup struct {
	Edition        EditionSection
	CommonMetaData CommonMetaData
}

type EditionSection struct {
	Edition        TextItem
	CommonMetaData CommonMetaData
}

var InstitutionSections = []string{LowerInstitutionName}

type InstitutionGroup struct {
	Institution    InstitutionSection
	CommonMetaData CommonMetaData
}

type InstitutionSection struct {
	Institution    []TextItem
	CommonMetaData CommonMetaData
}

var JournalSections = []string{LowerJournalName}

type JournalGroup struct {
	Journal        JournalSection
	CommonMetaData CommonMetaData
}

type JournalSection struct {
	Journal        []TextItem
	CommonMetaData CommonMetaData
}

var PublisherSections = []string{LowerPublisherName}

type PublisherGroup struct {
	Publisher      PublisherSection
	CommonMetaData CommonMetaData
}

type PublisherSection struct {
	Publisher      []TextItem
	CommonMetaData CommonMetaData
}

var VolumeSections = []string{LowerVolumeName}

type VolumeGroup struct {
	Volume         VolumeSection
	CommonMetaData CommonMetaData
}

type VolumeSection struct {
	Volume         TextItem
	CommonMetaData CommonMetaData
}

var MonthSections = []string{LowerMonthName}

type MonthGroup struct {
	Month          MonthSection
	CommonMetaData CommonMetaData
}

type MonthSection struct {
	Month          TextItem
	CommonMetaData CommonMetaData
}

var YearSections = []string{LowerYearName}

type YearGroup struct {
	Year           YearSection
	CommonMetaData CommonMetaData
}

type YearSection struct {
	Year           TextItem
	CommonMetaData CommonMetaData
}

var DescriptionSections = []string{LowerDescriptionName}

type DescriptionGroup struct {
	Description    DescriptionSection
	CommonMetaData CommonMetaData
}

type DescriptionSection struct {
	Description    TextItem
	CommonMetaData CommonMetaData
}

////////////////////////////////////////////////////////////////////////////////////////////////////

type TextBlockItem struct {
	Text           string
	CommonMetaData CommonMetaData
}

type Document struct {
	Items          []TopLevelItemKind
	CommonMetaData CommonMetaData
}

////////////////////////////////////////////////////////////////////////////////////////////////////

var ProofThenSections = []string{
	LowerThenName,
	LowerByQuestionName,
	LowerBecauseQuestionName,
}

type ProofThenGroup struct {
	Label          *GroupLabel
	Then           ProofThenSection
	By             *ProofBySection
	Because        *ProofBecauseSection
	CommonMetaData CommonMetaData
}

type ProofThenSection struct {
	Then           []ProofItemKind
	CommonMetaData CommonMetaData
}

////////////////////////////////////////////////////////////////////////////////////////////////////

var ProofThusSections = []string{
	LowerThusName,
	LowerByQuestionName,
	LowerBecauseQuestionName,
}

type ProofThusGroup struct {
	Label          *GroupLabel
	Thus           ProofThusSection
	By             *ProofBySection
	Because        *ProofBecauseSection
	CommonMetaData CommonMetaData
}

type ProofThusSection struct {
	Thus           []ProofItemKind
	CommonMetaData CommonMetaData
}

////////////////////////////////////////////////////////////////////////////////////////////////////

var ProofThereforeSections = []string{
	LowerThereforeName,
	LowerByQuestionName,
	LowerBecauseQuestionName,
}

type ProofThereforeGroup struct {
	Label          *GroupLabel
	Therefore      ProofThereforeSection
	By             *ProofBySection
	Because        *ProofBecauseSection
	CommonMetaData CommonMetaData
}

////////////////////////////////////////////////////////////////////////////////////////////////////

var ProofHenceSections = []string{
	LowerHenceName,
	LowerByQuestionName,
	LowerBecauseQuestionName,
}

type ProofHenceGroup struct {
	Label          *GroupLabel
	Hence          ProofHenceSection
	By             *ProofBySection
	Because        *ProofBecauseSection
	CommonMetaData CommonMetaData
}

type ProofHenceSection struct {
	Hence          []ProofItemKind
	CommonMetaData CommonMetaData
}

////////////////////////////////////////////////////////////////////////////////////////////////////

var ProofNoticeSections = []string{
	LowerNoticeName,
	LowerByQuestionName,
	LowerBecauseQuestionName,
}

type ProofNoticeGroup struct {
	Label          *GroupLabel
	Notice         ProofNoticeSection
	By             *ProofBySection
	Because        *ProofBecauseSection
	CommonMetaData CommonMetaData
}

type ProofNoticeSection struct {
	Notice         []ProofItemKind
	CommonMetaData CommonMetaData
}

type ProofThereforeSection struct {
	Therefore      []ProofItemKind
	CommonMetaData CommonMetaData
}

////////////////////////////////////////////////////////////////////////////////////////////////////

var ProofNextSections = []string{
	LowerNextName,
	LowerByQuestionName,
	LowerBecauseQuestionName,
}

type ProofNextGroup struct {
	Label          *GroupLabel
	Next           ProofNextSection
	By             *ProofBySection
	Because        *ProofBecauseSection
	CommonMetaData CommonMetaData
}

type ProofNextSection struct {
	Next           []ProofItemKind
	CommonMetaData CommonMetaData
}

////////////////////////////////////////////////////////////////////////////////////////////////////

var ProofByBecauseThenSections = []string{
	LowerByName,
	LowerBecauseQuestionName,
	LowerThenName,
}

type ProofByBecauseThenGroup struct {
	Label          *GroupLabel
	By             ProofBySection
	Because        *ProofBecauseSection
	Then           ProofThenSection
	CommonMetaData CommonMetaData
}

////////////////////////////////////////////////////////////////////////////////////////////////////

var ProofBecauseThenSections = []string{
	LowerBecauseName,
	LowerThenName,
}

type ProofBecauseThenGroup struct {
	Label          *GroupLabel
	Because        ProofBecauseSection
	Then           ProofThenSection
	CommonMetaData CommonMetaData
}

type ProofBySection struct {
	Items          []TextItem
	CommonMetaData CommonMetaData
}

type ProofBecauseSection struct {
	Because        []ProofItemKind
	CommonMetaData CommonMetaData
}

////////////////////////////////////////////////////////////////////////////////////////////////////

var ProofStepwiseSections = []string{
	LowerStepwiseName,
}

type ProofStepwiseGroup struct {
	Label          *GroupLabel
	Stepwise       ProofStepwiseSection
	CommonMetaData CommonMetaData
}

type ProofStepwiseSection struct {
	Stepwise       []ProofItemKind
	CommonMetaData CommonMetaData
}

////////////////////////////////////////////////////////////////////////////////////////////////////

var ProofSupposeSections = []string{
	LowerSupposeName,
	LowerThenName,
}

type ProofSupposeGroup struct {
	Label          *GroupLabel
	Suppose        ProofSupposeSection
	Then           ProofThenSection
	CommonMetaData CommonMetaData
}

type ProofSupposeSection struct {
	Suppose        []ProofItemKind
	CommonMetaData CommonMetaData
}

////////////////////////////////////////////////////////////////////////////////////////////////////

var ProofBlockSections = []string{
	LowerBlockName,
}

type ProofBlockGroup struct {
	Label          *GroupLabel
	Block          ProofBlockSection
	CommonMetaData CommonMetaData
}

type ProofBlockSection struct {
	Block          []ProofItemKind
	CommonMetaData CommonMetaData
}

////////////////////////////////////////////////////////////////////////////////////////////////////

var ProofCasewiseSections = []string{
	LowerCasewiseName,
	LowerCaseName,
	LowerElseQuestionName,
}

type ProofCasewiseGroup struct {
	Label          *GroupLabel
	Casewise       ProofCasewiseSection
	Cases          []ProofCaseSection
	Else           *ProofElseSection
	CommonMetaData CommonMetaData
}

type ProofCasewiseSection struct {
	CommonMetaData CommonMetaData
}

type ProofCaseSection struct {
	Case           []ProofItemKind
	CommonMetaData CommonMetaData
}

type ProofElseSection struct {
	Else           []ProofItemKind
	CommonMetaData CommonMetaData
}

////////////////////////////////////////////////////////////////////////////////////////////////////

var ProofPartwiseSections = []string{
	LowerPartwiseName,
	LowerPartName,
}

type ProofPartwiseGroup struct {
	Label          *GroupLabel
	Partwise       ProofPartwiseSection
	Parts          []ProofPartSection
	CommonMetaData CommonMetaData
}

type ProofPartwiseSection struct {
	CommonMetaData CommonMetaData
}

type ProofPartSection struct {
	Part           []ProofItemKind
	CommonMetaData CommonMetaData
}

////////////////////////////////////////////////////////////////////////////////////////////////////

var ProofSufficesToShowSections = []string{
	LowerSufficesToShowName,
}

type ProofSufficesToShowGroup struct {
	Label          *GroupLabel
	SufficesToShow ProofSufficesToShowSection
	CommonMetaData CommonMetaData
}

type ProofSufficesToShowSection struct {
	Items          []ProofItemKind
	CommonMetaData CommonMetaData
}

////////////////////////////////////////////////////////////////////////////////////////////////////

var ProofToShowSections = []string{
	LowerToShowName,
	LowerObserveName,
}

type ProofToShowGroup struct {
	Label          *GroupLabel
	ToShow         ProofToShowSection
	Observe        ProofObserveSection
	CommonMetaData CommonMetaData
}

type ProofToShowSection struct {
	Items          []ProofItemKind
	CommonMetaData CommonMetaData
}

type ProofObserveSection struct {
	Items          []ProofItemKind
	CommonMetaData CommonMetaData
}

////////////////////////////////////////////////////////////////////////////////////////////////////

var ProofWithoutLossOfGeneralitySections = []string{
	LowerWithoutLossOfGeneralityName,
}

type ProofWithoutLossOfGeneralityGroup struct {
	Label                   *GroupLabel
	WithoutLossOfGenerality ProofWithoutLossOfGeneralitySection
	CommonMetaData          CommonMetaData
}

type ProofWithoutLossOfGeneralitySection struct {
	Items          []ProofItemKind
	CommonMetaData CommonMetaData
}

////////////////////////////////////////////////////////////////////////////////////////////////////

var ProofRemarkSections = []string{
	LowerRemarkName,
}

type ProofRemarkGroup struct {
	Label          *GroupLabel
	Remark         ProofRemarkSection
	CommonMetaData CommonMetaData
}

type ProofRemarkSection struct {
	Remark         TextItem
	CommonMetaData CommonMetaData
}

////////////////////////////////////////////////////////////////////////////////////////////////////

var ProofContradictionSections = []string{
	LowerContradictionName,
}

type ProofContradictionGroup struct {
	Label          *GroupLabel
	Contradiction  ProofContradictionSection
	CommonMetaData CommonMetaData
}

type ProofContradictionSection struct {
	CommonMetaData CommonMetaData
}

////////////////////////////////////////////////////////////////////////////////////////////////////

var ProofAbsurdSections = []string{
	LowerAbsurdName,
}

type ProofAbsurdGroup struct {
	Label          *GroupLabel
	Absurd         ProofAbsurdSection
	CommonMetaData CommonMetaData
}

type ProofAbsurdSection struct {
	CommonMetaData CommonMetaData
}

////////////////////////////////////////////////////////////////////////////////////////////////////

var ProofDoneSections = []string{
	LowerDoneName,
}

type ProofDoneGroup struct {
	Label          *GroupLabel
	Done           ProofDoneSection
	CommonMetaData CommonMetaData
}

type ProofDoneSection struct {
	CommonMetaData CommonMetaData
}

////////////////////////////////////////////////////////////////////////////////////////////////////

var ProofQedSections = []string{
	LowerQedName,
}

type ProofQedGroup struct {
	Label          *GroupLabel
	Qed            ProofQedSection
	CommonMetaData CommonMetaData
}

type ProofQedSection struct {
	CommonMetaData CommonMetaData
}

////////////////////////////////////////////////////////////////////////////////////////////////////

var ProofForContradictionSections = []string{
	LowerForContradictionName,
}

type ProofForContradictionGroup struct {
	Label            *GroupLabel
	ForContradiction ProofForContradictionSection
	CommonMetaData   CommonMetaData
}

type ProofForContradictionSection struct {
	Items          []ProofItemKind
	CommonMetaData CommonMetaData
}

////////////////////////////////////////////////////////////////////////////////////////////////////

var ProofForContrapositiveSections = []string{
	LowerForContrapositiveName,
}

type ProofForContrapositiveGroup struct {
	Label             *GroupLabel
	ForContrapositive ProofForContrapositiveSection
	CommonMetaData    CommonMetaData
}

type ProofForContrapositiveSection struct {
	Items          []ProofItemKind
	CommonMetaData CommonMetaData
}

////////////////////////////////////////////////////////////////////////////////////////////////////

var ProofForInductionSections = []string{
	LowerForInductionName,
}

type ProofForInductionGroup struct {
	Label          *GroupLabel
	ForInduction   ProofForInductionSection
	CommonMetaData CommonMetaData
}

type ProofForInductionSection struct {
	Items          []ProofItemKind
	CommonMetaData CommonMetaData
}

////////////////////////////////////////////////////////////////////////////////////////////////////

var ProofClaimSections = []string{
	LowerClaimName,
	LowerGivenQuestionName,
	LowerUsingQuestionName,
	LowerWhereQuestionName,
	LowerIfQuestionName,
	LowerIffQuestionName,
	LowerThenName,
	UpperProofQuestionName,
}

type ProofClaimGroup struct {
	Label          *GroupLabel
	Claim          ProofClaimSection
	Given          *GivenSection
	Using          *UsingSection
	Where          *WhereSection
	If             *IfSection
	Iff            *IffSection
	Then           ThenSection
	Proof          *ProofSection
	CommonMetaData CommonMetaData
}

type ProofClaimSection struct {
	CommonMetaData CommonMetaData
}

////////////////////////////////////////////////////////////////////////////////////////////////////

var ProofEquivalentlySections = []string{LowerEquivalentlyName}

type ProofEquivalentlyGroup struct {
	Label          *GroupLabel
	Equivalently   ProofEquivalentlySection
	CommonMetaData CommonMetaData
}

type ProofEquivalentlySection struct {
	Items          []ProofItemKind
	CommonMetaData CommonMetaData
}

////////////////////////////////////////////////////////////////////////////////////////////////////

var ProofAllOfSections = []string{LowerAllOfName}

type ProofAllOfGroup struct {
	Label          *GroupLabel
	AllOf          ProofAllOfSection
	CommonMetaData CommonMetaData
}

type ProofAllOfSection struct {
	Items          []ProofItemKind
	CommonMetaData CommonMetaData
}

////////////////////////////////////////////////////////////////////////////////////////////////////

var ProofNotSections = []string{LowerNotName}

type ProofNotGroup struct {
	Label          *GroupLabel
	Not            ProofNotSection
	CommonMetaData CommonMetaData
}

type ProofNotSection struct {
	Item           ProofItemKind
	CommonMetaData CommonMetaData
}

////////////////////////////////////////////////////////////////////////////////////////////////////

var ProofAnyOfSections = []string{LowerAnyOfName}

type ProofAnyOfGroup struct {
	Label          *GroupLabel
	AnyOf          ProofAnyOfSection
	CommonMetaData CommonMetaData
}

type ProofAnyOfSection struct {
	Items          []ProofItemKind
	CommonMetaData CommonMetaData
}

////////////////////////////////////////////////////////////////////////////////////////////////////

var ProofOneOfSections = []string{LowerOneOfName}

type ProofOneOfGroup struct {
	Label          *GroupLabel
	OneOf          ProofOneOfSection
	CommonMetaData CommonMetaData
}

type ProofOneOfSection struct {
	Items          []ProofItemKind
	CommonMetaData CommonMetaData
}

////////////////////////////////////////////////////////////////////////////////////////////////////

var ProofExistsSections = []string{
	LowerExistsName,
	LowerUsingQuestionName,
	LowerWhereQuestionName,
	LowerSuchThatName,
}

type ProofExistsGroup struct {
	Label          *GroupLabel
	Exists         ExistsSection
	Using          *UsingSection
	Where          *WhereSection
	SuchThat       *ProofSuchThatSection
	CommonMetaData CommonMetaData
}

type ProofSuchThatSection struct {
	Items          []ProofItemKind
	CommonMetaData CommonMetaData
}

////////////////////////////////////////////////////////////////////////////////////////////////////

var ProofExistsUniqueSections = []string{
	LowerExistsUniqueName,
	LowerUsingQuestionName,
	LowerWhereQuestionName,
	LowerSuchThatName,
}

type ProofExistsUniqueGroup struct {
	Label          *GroupLabel
	ExistsUnique   ExistsUniqueSection
	Using          *UsingSection
	Where          *WhereSection
	SuchThat       ProofSuchThatSection
	CommonMetaData CommonMetaData
}

////////////////////////////////////////////////////////////////////////////////////////////////////

var ProofForAllSections = []string{
	LowerForAllName,
	LowerUsingQuestionName,
	LowerWhereQuestionName,
	LowerSuchThatQuestionName,
	LowerThenName,
}

type ProofForAllGroup struct {
	Label          *GroupLabel
	ForAll         ForAllSection
	Using          *UsingSection
	Where          *WhereSection
	SuchThat       *ProofSuchThatSection
	Then           ProofThenSection
	CommonMetaData CommonMetaData
}

////////////////////////////////////////////////////////////////////////////////////////////////////

var ProofIfSections = []string{
	LowerIfName,
	LowerThenName,
}

type ProofIfGroup struct {
	Label          *GroupLabel
	If             ProofIfSection
	Then           ProofThenSection
	CommonMetaData CommonMetaData
}

type ProofIfSection struct {
	Items          []ProofItemKind
	CommonMetaData CommonMetaData
}

////////////////////////////////////////////////////////////////////////////////////////////////////

var ProofIffSections = []string{
	LowerIffName,
	LowerThenName,
}

type ProofIffGroup struct {
	Label          *GroupLabel
	Iff            ProofIffSection
	Then           ProofThenSection
	CommonMetaData CommonMetaData
}

type ProofIffSection struct {
	Items          []ProofItemKind
	CommonMetaData CommonMetaData
}

////////////////////////////////////////////////////////////////////////////////////////////////////

var ProofLetSections = []string{
	LowerLetName,
	LowerUsingQuestionName,
	LowerWhereQuestionName,
	LowerSuchThatQuestionName,
	LowerThenName,
}

type ProofLetGroup struct {
	Label          *GroupLabel
	Let            LetSection
	Using          *UsingSection
	Where          *WhereSection
	SuchThat       *ProofSuchThatSection
	Then           ProofThenSection
	CommonMetaData CommonMetaData
}

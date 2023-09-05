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

type ClauseKind interface {
	StructuralNodeKind
	ClauseKind()
}

func (*Formulation[NodeType]) ClauseKind() {}
func (*AllOfGroup) ClauseKind()            {}
func (*NotGroup) ClauseKind()              {}
func (*AnyOfGroup) ClauseKind()            {}
func (*OneOfGroup) ClauseKind()            {}
func (*ExistsGroup) ClauseKind()           {}
func (*ExistsUniqueGroup) ClauseKind()     {}
func (*ForAllGroup) ClauseKind()           {}
func (*IfGroup) ClauseKind()               {}
func (*IffGroup) ClauseKind()              {}
func (*WhenGroup) ClauseKind()             {}
func (*PiecewiseGroup) ClauseKind()        {}
func (*LetGroup) ClauseKind()              {}
func (*LowerDefineGroup) ClauseKind()      {}

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

var PiecewiseSections = []string{
	LowerPiecewiseName,
	LowerIfName,
	LowerThenName,
	LowerElseQuestionName,
}

type PiecewiseGroup struct {
	Label          *GroupLabel
	Piecewise      PiecewiseSection
	IfThen         []IfThen
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

type ProvidesKind interface {
	StructuralNodeKind
	ProvidesKind()
}

func (*SymbolWrittenGroup) ProvidesKind() {}
func (*ViewGroup) ProvidesKind()          {}
func (*EncodingGroup) ProvidesKind()      {}

var SymbolSections = []string{
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
	Through        *LinkThroughSection
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

type LinkThroughSection struct {
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
	Through        *LinkThroughSection
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

var ExpressedSections = []string{LowerExpressedName}

type ExpressedGroup struct {
	Label          *GroupLabel
	Expressed      ExpressedSection
	CommonMetaData CommonMetaData
}

type ExpressedSection struct {
	Expressed      []TextItem
	CommonMetaData CommonMetaData
}

////////////////////////////////////////////////////////////////////////////////////////////////////

type DocumentedKind interface {
	StructuralNodeKind
	DocumentedKind()
}

func (*OverviewGroup) DocumentedKind()  {}
func (*RelatedGroup) DocumentedKind()   {}
func (*WrittenGroup) DocumentedKind()   {}
func (*ExpressedGroup) DocumentedKind() {}
func (*CalledGroup) DocumentedKind()    {}

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

type JustifiedKind interface {
	StructuralNodeKind
	JustifiedKind()
}

func (*LabelGroup) JustifiedKind() {}
func (*ByGroup) JustifiedKind()    {}

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
	LowerSatisfiesQuestionName,
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
	Satisfies      *SatisfiesSection
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

type SatisfiesSection struct {
	Satisfies      []ClauseKind
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
	LowerSpecifiesQuestionName,
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
	Specifies      *SpecifiesSection
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

type SpecifiesSection struct {
	Specifies      []ClauseKind
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

var ProofSections = []string{
	UpperProofName,
	LowerOfName,
	LowerContentName,
	UpperReferencesQuestionName,
	UpperIdQuestionName,
}

type ProofGroup struct {
	Id             IdItem
	Proof          TopLevelProofSection
	Of             OfSection
	Content        ContentSection
	References     *ReferencesSection
	MetaId         *MetaIdSection
	CommonMetaData CommonMetaData
}

type TopLevelProofSection struct {
	CommonMetaData CommonMetaData
}

type OfSection struct {
	Of             TextItem
	CommonMetaData CommonMetaData
}

type ContentSection struct {
	Content        TextItem
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
	Proof          []TextItem
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

type SpecifyKind interface {
	StructuralNodeKind
	SpecifyKind()
}

func (*ZeroGroup) SpecifyKind()          {}
func (*PositiveIntGroup) SpecifyKind()   {}
func (*NegativeIntGroup) SpecifyKind()   {}
func (*PositiveFloatGroup) SpecifyKind() {}
func (*NegativeFloatGroup) SpecifyKind() {}

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

type PersonKind interface {
	StructuralNodeKind
	PersonKind()
}

func (*NameGroup) PersonKind()      {}
func (*BiographyGroup) PersonKind() {}

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

func (*TitleGroup) ResourceKind()       {}
func (*AuthorGroup) ResourceKind()      {}
func (*OffsetGroup) ResourceKind()      {}
func (*UrlGroup) ResourceKind()         {}
func (*HomepageGroup) ResourceKind()    {}
func (*TypeGroup) ResourceKind()        {}
func (*EditorGroup) ResourceKind()      {}
func (*EditionGroup) ResourceKind()     {}
func (*InstitutionGroup) ResourceKind() {}
func (*JournalGroup) ResourceKind()     {}
func (*PublisherGroup) ResourceKind()   {}
func (*VolumeGroup) ResourceKind()      {}
func (*MonthGroup) ResourceKind()       {}
func (*YearGroup) ResourceKind()        {}
func (*DescriptionGroup) ResourceKind() {}

type ResourceKind interface {
	StructuralNodeKind
	ResourceKind()
}

type TitleGroup struct {
	Title          TitleSection
	CommonMetaData CommonMetaData
}

type TitleSection struct {
	Title          TextItem
	CommonMetaData CommonMetaData
}

type AuthorGroup struct {
	Author         AuthorSection
	CommonMetaData CommonMetaData
}

type AuthorSection struct {
	Author         []TextItem
	CommonMetaData CommonMetaData
}

type OffsetGroup struct {
	Offset         OffsetSection
	CommonMetaData CommonMetaData
}

type OffsetSection struct {
	Offset         TextItem
	CommonMetaData CommonMetaData
}

type UrlGroup struct {
	Url            UrlSection
	CommonMetaData CommonMetaData
}

type UrlSection struct {
	Url            TextItem
	CommonMetaData CommonMetaData
}

type HomepageGroup struct {
	Homepage       HomepageSection
	CommonMetaData CommonMetaData
}

type HomepageSection struct {
	Homepage       TextItem
	CommonMetaData CommonMetaData
}

type TypeGroup struct {
	Type           TypeSection
	CommonMetaData CommonMetaData
}

type TypeSection struct {
	Type           TextItem
	CommonMetaData CommonMetaData
}

type EditorGroup struct {
	Editor         EditorSection
	CommonMetaData CommonMetaData
}

type EditorSection struct {
	Editor         []TextItem
	CommonMetaData CommonMetaData
}

type EditionGroup struct {
	Edition        EditionSection
	CommonMetaData CommonMetaData
}

type EditionSection struct {
	Edition        TextItem
	CommonMetaData CommonMetaData
}

type InstitutionGroup struct {
	Institution    InstitutionSection
	CommonMetaData CommonMetaData
}

type InstitutionSection struct {
	Institution    []TextItem
	CommonMetaData CommonMetaData
}

type JournalGroup struct {
	Journal        JournalSection
	CommonMetaData CommonMetaData
}

type JournalSection struct {
	Journal        []TextItem
	CommonMetaData CommonMetaData
}

type PublisherGroup struct {
	Publisher      PublisherSection
	CommonMetaData CommonMetaData
}

type PublisherSection struct {
	Publisher      []TextItem
	CommonMetaData CommonMetaData
}

type VolumeGroup struct {
	Volume         VolumeSection
	CommonMetaData CommonMetaData
}

type VolumeSection struct {
	Volume         TextItem
	CommonMetaData CommonMetaData
}

type MonthGroup struct {
	Month          MonthSection
	CommonMetaData CommonMetaData
}

type MonthSection struct {
	Month          TextItem
	CommonMetaData CommonMetaData
}

type YearGroup struct {
	Year           YearSection
	CommonMetaData CommonMetaData
}

type YearSection struct {
	Year           TextItem
	CommonMetaData CommonMetaData
}

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

type TopLevelItemKind interface {
	StructuralNodeKind
	TopLevelItemKind()
}

func (*TextBlockItem) TopLevelItemKind()   {}
func (*DefinesGroup) TopLevelItemKind()    {}
func (*DescribesGroup) TopLevelItemKind()  {}
func (*StatesGroup) TopLevelItemKind()     {}
func (*AxiomGroup) TopLevelItemKind()      {}
func (*ConjectureGroup) TopLevelItemKind() {}
func (*TheoremGroup) TopLevelItemKind()    {}
func (*CorollaryGroup) TopLevelItemKind()  {}
func (*LemmaGroup) TopLevelItemKind()      {}
func (*SpecifyGroup) TopLevelItemKind()    {}
func (*PersonGroup) TopLevelItemKind()     {}
func (*ResourceGroup) TopLevelItemKind()   {}
func (*ProofGroup) TopLevelItemKind()      {}
func (*CapturesGroup) TopLevelItemKind()   {}

type Document struct {
	Items          []TopLevelItemKind
	CommonMetaData CommonMetaData
}

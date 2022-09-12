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

type Formulation[T NodeType] struct {
	RawText string
	Root    T
	Label   *string
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

//////////////////////////////////////////////////////////////////////////////

/*
allOf:
*/
type AllOfGroup struct {
	AllOf AllOfSection
}

type AllOfSection struct {
	Clauses []Clause
}

//////////////////////////////////////////////////////////////////////////////

/*
not:
*/
type NotGroup struct {
	Not NotSection
}

type NotSection struct {
	Clause Clause
}

//////////////////////////////////////////////////////////////////////////////

/*
anyOf:
*/
type AnyOfGroup struct {
	AnyOf AnyOfSection
}

type AnyOfSection struct {
	Clauses []Clause
}

//////////////////////////////////////////////////////////////////////////////

/*
oneOf:
*/
type OneOfGroup struct {
	OneOf OneOfSection
}

type OneOfSection struct {
	Clauses []Clause
}

//////////////////////////////////////////////////////////////////////////////

/*
exists:
where?:
suchThat?:
*/
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

/*
existsUnique:
where?:
suchThat?:
*/
type ExistsUniqueGroup struct {
	ExistsUnique ExistsUniqueSection
	Where        *WhereSection
	SuchThat     *SuchThatSection
}

type ExistsUniqueSection struct {
	Specs []Spec
}

//////////////////////////////////////////////////////////////////////////////

/*
forAll:
where?:
suchThat?:
then:
*/
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

/*
if:
then:
*/
type IfGroup struct {
	If   IfSection
	Then ThenSection
}

type IfSection struct {
	Clauses []Clause
}

//////////////////////////////////////////////////////////////////////////////

/*
iff:
then:
*/
type IffGroup struct {
	Iff  IffSection
	Then ThenSection
}

type IffSection struct {
	Clauses []Clause
}

//////////////////////////////////////////////////////////////////////////////

/*
generated:
from:
when?:
*/
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

/*
piecewise:
(if:then:)+
else?:
*/
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

type AsViaGroup struct {
	As  AsSection
	Via ViaSection
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

/*
infix:
when?:
states|defines:
*/
type InfixGroup struct {
	Infix         InfixSection
	When          *WhenSection
	StatesDefines ProvidesStatesDefinesType
}

/*
prefix:
when?:
states|defines:
*/
type PrefixGroup struct {
	Prefix        PrefixSection
	When          *WhenSection
	StatesDefines ProvidesStatesDefinesType
}

/*
postfix:
when?:
states|defines:
*/
type PostfixGroup struct {
	Postfix       PostfixSection
	When          *WhenSection
	StatesDefines ProvidesStatesDefinesType
}

/*
symbol:
defines:
*/
type SymbolGroup struct {
	Symbol        SymbolSection
	When          *WhenSection
	StatesDefines ProvidesStatesDefinesType
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
}

type CodifiedType interface {
	CodifiedType()
}

func (WrittenGroup) CodifiedType() {}
func (CalledGroup) CodifiedType()  {}
func (WritingGroup) CodifiedType() {}

type WrittenGroup struct {
	Written WrittenSection
}

type WrittenSection struct {
	Written []TextItem
}

type CalledGroup struct {
	Called CalledSection
}

type CalledSection struct {
	Called []TextItem
}

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

type LooselyGroup struct {
	Loosely LooselySection
}

type LooselySection struct {
	Loosely TextItem
}

type OverviewGroup struct {
	Overview OverviewSection
}

type OverviewSection struct {
	Overview TextItem
}

type MotivationGroup struct {
	Motivation MotivationSection
}

type MotivationSection struct {
	Motivation TextItem
}

type HistoryGroup struct {
	History HistorySection
}

type HistorySection struct {
	History TextItem
}

type ExamplesGroup struct {
	Examples ExamplesSection
}

type ExamplesSection struct {
	Examples []TextItem
}

type RelatedGroup struct {
	Related RelatedSection
}

type RelatedSection struct {
	Related []Signature
}

type DiscoveredGroup struct {
	Discovered DiscoveredSection
}

type DiscoveredSection struct {
	Discovered TextItem
}

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

type LabelGroup struct {
	Label LabelSection
	By    BySection
}

type LabelSection struct {
	Label TextItem
}

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

type IdGroup struct {
	Id IdSection
}

type IdSection struct {
	Id TextItem
}

//////////////////////////////////////////////////////////////////////////////

/*
[]
Describes:
with?:
given?:
when?:
suchThat?:
extends?:
satisfies?:
Viewable?:
Provides?:
Using?:
Codified:
Documented?:
Justified?:
References?:
Metadata?:
*/
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

/*
[]
Declares:
with?:
given?:
when?:
suchThat?:
means?:
defines?:
Viewable?:
Provides?:
Using?:
Codified?:
Documented?:
Justified?:
References?:
Metadata?:
*/
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

/*
[]
States:
given?:
when?:
suchThat?:
that:
Using?:
Codified?:
Documented?:
Justified?:
References?:
Metadata?:
*/
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

/*
[]
Proof:
of:
content:
References?:
Metadata?:
*/
type ProofGroup struct {
	Id         IdItem
	Of         OfSection
	Content    ContentSection
	References *ReferencesSection
	Metadata   *MetadataSection
}

type ProofSection struct {
}

type OfSection struct {
	Of TextItem
}

type ContentSection struct {
	Content TextItem
}

//////////////////////////////////////////////////////////////////////////////

/*
[]
Axiom:
given?:
where?:
suchThat?:
then:
iff?:
Using?:
Documented?:
References?:
Metadata?:
*/
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

/*
[]
Conjecture:
given?:
where?:
suchThat?:
then:
iff?:
Using?:
Documented?:
References?:
Metadata?:
*/
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

/*
[]
Theorem:
given?:
where?:
suchThat?:
then:
iff?:
Using?:
Proof:
Documented?:
References?:
Metadata?:
*/
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

/*
[]
Topic:
content:
References?:
Metadata?:
*/
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

/*
Note:
content:
Metadata?:
*/
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

type PositiveIntGroup struct {
}

type PositiveIntSection struct {
}

type NegativeIntGroup struct {
}

type NegativeIntSection struct {
}

type PositiveFloatGroup struct {
}

type PositiveFloatSection struct {
}

type NegativeFloatGroup struct {
}

type NegativeFloatSection struct {
}

//////////////////////////////////////////////////////////////////////////////

/*
Specify:
*/
type SpecifyGroup struct {
	Specify []SpecifyType
}

//////////////////////////////////////////////////////////////////////////////

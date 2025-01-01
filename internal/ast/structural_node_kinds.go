/*
 * Copyright 2024 Dominic Kramer
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

type StructuralNodeKind interface {
	MlgNodeKind
	StructuralNodeKind()
	ToCode(indent int, hasDot bool) []string
}

func (*IdItem) StructuralNodeKind()                            {}
func (*Target) StructuralNodeKind()                            {}
func (*Spec) StructuralNodeKind()                              {}
func (*Alias) StructuralNodeKind()                             {}
func (*Formulation[T]) StructuralNodeKind()                    {}
func (*TextItem) StructuralNodeKind()                          {}
func (*DeclareGroup) StructuralNodeKind()                      {}
func (*AllOfGroup) StructuralNodeKind()                        {}
func (*EquivalentlyGroup) StructuralNodeKind()                 {}
func (*NotGroup) StructuralNodeKind()                          {}
func (*AnyOfGroup) StructuralNodeKind()                        {}
func (*OneOfGroup) StructuralNodeKind()                        {}
func (*ExistsGroup) StructuralNodeKind()                       {}
func (*ExistsUniqueGroup) StructuralNodeKind()                 {}
func (*ForAllGroup) StructuralNodeKind()                       {}
func (*IfGroup) StructuralNodeKind()                           {}
func (*IffGroup) StructuralNodeKind()                          {}
func (*PiecewiseGroup) StructuralNodeKind()                    {}
func (*AssertingGroup) StructuralNodeKind()                    {}
func (*SymbolWrittenGroup) StructuralNodeKind()                {}
func (*ComparisonGroup) StructuralNodeKind()                   {}
func (*ViewGroup) StructuralNodeKind()                         {}
func (*EncodingGroup) StructuralNodeKind()                     {}
func (*WrittenGroup) StructuralNodeKind()                      {}
func (*CalledGroup) StructuralNodeKind()                       {}
func (*WritingGroup) StructuralNodeKind()                      {}
func (*OverviewGroup) StructuralNodeKind()                     {}
func (*RelatedGroup) StructuralNodeKind()                      {}
func (*LabelGroup) StructuralNodeKind()                        {}
func (*ByGroup) StructuralNodeKind()                           {}
func (*DescribesGroup) StructuralNodeKind()                    {}
func (*DefinesGroup) StructuralNodeKind()                      {}
func (*CapturesGroup) StructuralNodeKind()                     {}
func (*StatesGroup) StructuralNodeKind()                       {}
func (*AxiomGroup) StructuralNodeKind()                        {}
func (*ConjectureGroup) StructuralNodeKind()                   {}
func (*TheoremGroup) StructuralNodeKind()                      {}
func (*CorollaryGroup) StructuralNodeKind()                    {}
func (*LemmaGroup) StructuralNodeKind()                        {}
func (*ZeroGroup) StructuralNodeKind()                         {}
func (*PositiveIntGroup) StructuralNodeKind()                  {}
func (*NegativeIntGroup) StructuralNodeKind()                  {}
func (*PositiveFloatGroup) StructuralNodeKind()                {}
func (*NegativeFloatGroup) StructuralNodeKind()                {}
func (*SpecifyGroup) StructuralNodeKind()                      {}
func (*PersonGroup) StructuralNodeKind()                       {}
func (*NameGroup) StructuralNodeKind()                         {}
func (*BiographyGroup) StructuralNodeKind()                    {}
func (*ResourceGroup) StructuralNodeKind()                     {}
func (*TitleGroup) StructuralNodeKind()                        {}
func (*AuthorGroup) StructuralNodeKind()                       {}
func (*OffsetGroup) StructuralNodeKind()                       {}
func (*UrlGroup) StructuralNodeKind()                          {}
func (*HomepageGroup) StructuralNodeKind()                     {}
func (*TypeGroup) StructuralNodeKind()                         {}
func (*EditorGroup) StructuralNodeKind()                       {}
func (*EditionGroup) StructuralNodeKind()                      {}
func (*InstitutionGroup) StructuralNodeKind()                  {}
func (*JournalGroup) StructuralNodeKind()                      {}
func (*PublisherGroup) StructuralNodeKind()                    {}
func (*VolumeGroup) StructuralNodeKind()                       {}
func (*MonthGroup) StructuralNodeKind()                        {}
func (*YearGroup) StructuralNodeKind()                         {}
func (*DescriptionGroup) StructuralNodeKind()                  {}
func (*Document) StructuralNodeKind()                          {}
func (*TextBlockItem) StructuralNodeKind()                     {}
func (*ProofThenGroup) StructuralNodeKind()                    {}
func (*ProofThusGroup) StructuralNodeKind()                    {}
func (*ProofThereforeGroup) StructuralNodeKind()               {}
func (*ProofHenceGroup) StructuralNodeKind()                   {}
func (*ProofNoticeGroup) StructuralNodeKind()                  {}
func (*ProofNextGroup) StructuralNodeKind()                    {}
func (*ProofByBecauseThenGroup) StructuralNodeKind()           {}
func (*ProofBecauseThenGroup) StructuralNodeKind()             {}
func (*ProofStepwiseGroup) StructuralNodeKind()                {}
func (*ProofSupposeGroup) StructuralNodeKind()                 {}
func (*ProofBlockGroup) StructuralNodeKind()                   {}
func (*ProofCasewiseGroup) StructuralNodeKind()                {}
func (*ProofWithoutLossOfGeneralityGroup) StructuralNodeKind() {}
func (*ProofContradictionGroup) StructuralNodeKind()           {}
func (*ProofForContradictionGroup) StructuralNodeKind()        {}
func (*ProofForInductionGroup) StructuralNodeKind()            {}
func (*ProofClaimGroup) StructuralNodeKind()                   {}
func (*ProofEquivalentlyGroup) StructuralNodeKind()            {}
func (*ProofAllOfGroup) StructuralNodeKind()                   {}
func (*ProofNotGroup) StructuralNodeKind()                     {}
func (*ProofAnyOfGroup) StructuralNodeKind()                   {}
func (*ProofOneOfGroup) StructuralNodeKind()                   {}
func (*ProofExistsGroup) StructuralNodeKind()                  {}
func (*ProofExistsUniqueGroup) StructuralNodeKind()            {}
func (*ProofForAllGroup) StructuralNodeKind()                  {}
func (*ProofDeclareGroup) StructuralNodeKind()                 {}
func (*ProofIfGroup) StructuralNodeKind()                      {}
func (*ProofIffGroup) StructuralNodeKind()                     {}
func (*ProofForContrapositiveGroup) StructuralNodeKind()       {}
func (*ProofQedGroup) StructuralNodeKind()                     {}
func (*ProofAbsurdGroup) StructuralNodeKind()                  {}
func (*ProofDoneGroup) StructuralNodeKind()                    {}
func (*ProofPartwiseGroup) StructuralNodeKind()                {}
func (*ProofSufficesToShowGroup) StructuralNodeKind()          {}
func (*ProofToShowGroup) StructuralNodeKind()                  {}
func (*ProofRemarkGroup) StructuralNodeKind()                  {}
func (*InductivelyGroup) StructuralNodeKind()                  {}
func (*InductivelyCaseGroup) StructuralNodeKind()              {}
func (*MatchingGroup) StructuralNodeKind()                     {}
func (*MatchingCaseGroup) StructuralNodeKind()                 {}

////////////////////////////////////////////////////////////////////////////////////////////////////

type ClauseKind interface {
	StructuralNodeKind
	ClauseKind()
}

func (*TextItem) ClauseKind()              {}
func (*Formulation[NodeType]) ClauseKind() {}
func (*AllOfGroup) ClauseKind()            {}
func (*NotGroup) ClauseKind()              {}
func (*AnyOfGroup) ClauseKind()            {}
func (*OneOfGroup) ClauseKind()            {}
func (*EquivalentlyGroup) ClauseKind()     {}
func (*ExistsGroup) ClauseKind()           {}
func (*ExistsUniqueGroup) ClauseKind()     {}
func (*ForAllGroup) ClauseKind()           {}
func (*IfGroup) ClauseKind()               {}
func (*IffGroup) ClauseKind()              {}
func (*AssertingGroup) ClauseKind()        {}
func (*PiecewiseGroup) ClauseKind()        {}
func (*DeclareGroup) ClauseKind()          {}
func (*InductivelyGroup) ClauseKind()      {}
func (*MatchingGroup) ClauseKind()         {}

////////////////////////////////////////////////////////////////////////////////////////////////////

type ProvidesKind interface {
	StructuralNodeKind
	ProvidesKind()
}

func (*SymbolWrittenGroup) ProvidesKind() {}
func (*ViewGroup) ProvidesKind()          {}
func (*EncodingGroup) ProvidesKind()      {}
func (*ComparisonGroup) ProvidesKind()    {}

////////////////////////////////////////////////////////////////////////////////////////////////////

type DocumentedKind interface {
	StructuralNodeKind
	DocumentedKind()
}

func (*OverviewGroup) DocumentedKind() {}
func (*RelatedGroup) DocumentedKind()  {}
func (*WrittenGroup) DocumentedKind()  {}
func (*WritingGroup) DocumentedKind()  {}
func (*CalledGroup) DocumentedKind()   {}

////////////////////////////////////////////////////////////////////////////////////////////////////

type JustifiedKind interface {
	StructuralNodeKind
	JustifiedKind()
}

func (*LabelGroup) JustifiedKind() {}
func (*ByGroup) JustifiedKind()    {}

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

////////////////////////////////////////////////////////////////////////////////////////////////////

type PersonKind interface {
	StructuralNodeKind
	PersonKind()
}

func (*NameGroup) PersonKind()      {}
func (*BiographyGroup) PersonKind() {}

////////////////////////////////////////////////////////////////////////////////////////////////////

type ResourceKind interface {
	StructuralNodeKind
	ResourceKind()
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

////////////////////////////////////////////////////////////////////////////////////////////////////

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
func (*CapturesGroup) TopLevelItemKind()   {}

////////////////////////////////////////////////////////////////////////////////////////////////////

type ProofItemKind interface {
	StructuralNodeKind
	ProofItemKind()
}

func (*ProofEquivalentlyGroup) ProofItemKind()            {}
func (*ProofAllOfGroup) ProofItemKind()                   {}
func (*ProofNotGroup) ProofItemKind()                     {}
func (*ProofAnyOfGroup) ProofItemKind()                   {}
func (*ProofOneOfGroup) ProofItemKind()                   {}
func (*ProofExistsGroup) ProofItemKind()                  {}
func (*ProofExistsUniqueGroup) ProofItemKind()            {}
func (*ProofForAllGroup) ProofItemKind()                  {}
func (*ProofDeclareGroup) ProofItemKind()                 {}
func (*ProofIfGroup) ProofItemKind()                      {}
func (*ProofIffGroup) ProofItemKind()                     {}
func (*ProofThenGroup) ProofItemKind()                    {}
func (*ProofThusGroup) ProofItemKind()                    {}
func (*ProofThereforeGroup) ProofItemKind()               {}
func (*ProofHenceGroup) ProofItemKind()                   {}
func (*ProofNoticeGroup) ProofItemKind()                  {}
func (*ProofNextGroup) ProofItemKind()                    {}
func (*ProofByBecauseThenGroup) ProofItemKind()           {}
func (*ProofBecauseThenGroup) ProofItemKind()             {}
func (*ProofStepwiseGroup) ProofItemKind()                {}
func (*ProofSupposeGroup) ProofItemKind()                 {}
func (*ProofBlockGroup) ProofItemKind()                   {}
func (*ProofCasewiseGroup) ProofItemKind()                {}
func (*ProofWithoutLossOfGeneralityGroup) ProofItemKind() {}
func (*ProofForContradictionGroup) ProofItemKind()        {}
func (*ProofForInductionGroup) ProofItemKind()            {}
func (*ProofClaimGroup) ProofItemKind()                   {}
func (*ProofForContrapositiveGroup) ProofItemKind()       {}
func (*ProofQedGroup) ProofItemKind()                     {}
func (*ProofAbsurdGroup) ProofItemKind()                  {}
func (*ProofDoneGroup) ProofItemKind()                    {}
func (*ProofContradictionGroup) ProofItemKind()           {}
func (*ProofPartwiseGroup) ProofItemKind()                {}
func (*ProofSufficesToShowGroup) ProofItemKind()          {}
func (*ProofToShowGroup) ProofItemKind()                  {}
func (*ProofRemarkGroup) ProofItemKind()                  {}
func (*TextItem) ProofItemKind()                          {}
func (*Formulation[FormulationNodeKind]) ProofItemKind()  {}

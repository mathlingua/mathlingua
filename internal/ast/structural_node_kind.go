/*
 * Copyright 2023 Dominic Kramer
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
func (*LetGroup) StructuralNodeKind()                          {}
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
func (*WhenGroup) StructuralNodeKind()                         {}
func (*SymbolWrittenGroup) StructuralNodeKind()                {}
func (*ViewGroup) StructuralNodeKind()                         {}
func (*EncodingGroup) StructuralNodeKind()                     {}
func (*WrittenGroup) StructuralNodeKind()                      {}
func (*CalledGroup) StructuralNodeKind()                       {}
func (*ExpressedGroup) StructuralNodeKind()                    {}
func (*OverviewGroup) StructuralNodeKind()                     {}
func (*RelatedGroup) StructuralNodeKind()                      {}
func (*LabelGroup) StructuralNodeKind()                        {}
func (*ByGroup) StructuralNodeKind()                           {}
func (*DescribesGroup) StructuralNodeKind()                    {}
func (*DefinesGroup) StructuralNodeKind()                      {}
func (*LowerDefineGroup) StructuralNodeKind()                  {}
func (*CapturesGroup) StructuralNodeKind()                     {}
func (*StatesGroup) StructuralNodeKind()                       {}
func (*ProofGroup) StructuralNodeKind()                        {}
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
func (*ProofThenByGroup) StructuralNodeKind()                  {}
func (*ProofThusByGroup) StructuralNodeKind()                  {}
func (*ProofThereforeByGroup) StructuralNodeKind()             {}
func (*ProofHenceByGroup) StructuralNodeKind()                 {}
func (*ProofNoticeByGroup) StructuralNodeKind()                {}
func (*ProofNextByGroup) StructuralNodeKind()                  {}
func (*ProofThenBecauseGroup) StructuralNodeKind()             {}
func (*ProofThusBecauseGroup) StructuralNodeKind()             {}
func (*ProofThereforeBecauseGroup) StructuralNodeKind()        {}
func (*ProofHenceBecauseGroup) StructuralNodeKind()            {}
func (*ProofNoticeBecauseGroup) StructuralNodeKind()           {}
func (*ProofNextBecauseGroup) StructuralNodeKind()             {}
func (*ProofByThenGroup) StructuralNodeKind()                  {}
func (*ProofBecauseThenGroup) StructuralNodeKind()             {}
func (*ProofIndependentlyGroup) StructuralNodeKind()           {}
func (*ProofStepwiseGroup) StructuralNodeKind()                {}
func (*ProofSupposeGroup) StructuralNodeKind()                 {}
func (*ProofBlockGroup) StructuralNodeKind()                   {}
func (*ProofCasewiseGroup) StructuralNodeKind()                {}
func (*ProofWithoutLossOfGeneralityGroup) StructuralNodeKind() {}
func (*ProofContradictingGroup) StructuralNodeKind()           {}
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
func (*ProofLetGroup) StructuralNodeKind()                     {}
func (*ProofIfGroup) StructuralNodeKind()                      {}
func (*ProofIffGroup) StructuralNodeKind()                     {}

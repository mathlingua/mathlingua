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

type MlgNodeKind interface {
	MlgNodeKind()
	GetCommonMetaData() *CommonMetaData
	ForEach(fn func(subNode MlgNodeKind))
}

func (*Root) MlgNodeKind()               {}
func (*IdItem) MlgNodeKind()             {}
func (*Target) MlgNodeKind()             {}
func (*Spec) MlgNodeKind()               {}
func (*Alias) MlgNodeKind()              {}
func (*Formulation[T]) MlgNodeKind()     {}
func (*TextItem) MlgNodeKind()           {}
func (*LetGroup) MlgNodeKind()           {}
func (*AllOfGroup) MlgNodeKind()         {}
func (*EquivalentlyGroup) MlgNodeKind()  {}
func (*NotGroup) MlgNodeKind()           {}
func (*AnyOfGroup) MlgNodeKind()         {}
func (*OneOfGroup) MlgNodeKind()         {}
func (*ExistsGroup) MlgNodeKind()        {}
func (*ExistsUniqueGroup) MlgNodeKind()  {}
func (*ForAllGroup) MlgNodeKind()        {}
func (*IfGroup) MlgNodeKind()            {}
func (*IffGroup) MlgNodeKind()           {}
func (*PiecewiseGroup) MlgNodeKind()     {}
func (*WhenGroup) MlgNodeKind()          {}
func (*SymbolWrittenGroup) MlgNodeKind() {}
func (*ViewGroup) MlgNodeKind()          {}
func (*EncodingGroup) MlgNodeKind()      {}
func (*WrittenGroup) MlgNodeKind()       {}
func (*CalledGroup) MlgNodeKind()        {}
func (*WritingGroup) MlgNodeKind()       {}
func (*OverviewGroup) MlgNodeKind()      {}
func (*RelatedGroup) MlgNodeKind()       {}
func (*LabelGroup) MlgNodeKind()         {}
func (*ByGroup) MlgNodeKind()            {}
func (*DescribesGroup) MlgNodeKind()     {}
func (*DefinesGroup) MlgNodeKind()       {}
func (*LowerDefineGroup) MlgNodeKind()   {}
func (*CapturesGroup) MlgNodeKind()      {}
func (*StatesGroup) MlgNodeKind()        {}
func (*AxiomGroup) MlgNodeKind()         {}
func (*ConjectureGroup) MlgNodeKind()    {}
func (*TheoremGroup) MlgNodeKind()       {}
func (*CorollaryGroup) MlgNodeKind()     {}
func (*LemmaGroup) MlgNodeKind()         {}
func (*ZeroGroup) MlgNodeKind()          {}
func (*PositiveIntGroup) MlgNodeKind()   {}
func (*NegativeIntGroup) MlgNodeKind()   {}
func (*PositiveFloatGroup) MlgNodeKind() {}
func (*NegativeFloatGroup) MlgNodeKind() {}
func (*SpecifyGroup) MlgNodeKind()       {}
func (*PersonGroup) MlgNodeKind()        {}
func (*NameGroup) MlgNodeKind()          {}
func (*BiographyGroup) MlgNodeKind()     {}
func (*ResourceGroup) MlgNodeKind()      {}
func (*TitleGroup) MlgNodeKind()         {}
func (*AuthorGroup) MlgNodeKind()        {}
func (*OffsetGroup) MlgNodeKind()        {}
func (*UrlGroup) MlgNodeKind()           {}
func (*HomepageGroup) MlgNodeKind()      {}
func (*TypeGroup) MlgNodeKind()          {}
func (*EditorGroup) MlgNodeKind()        {}
func (*EditionGroup) MlgNodeKind()       {}
func (*InstitutionGroup) MlgNodeKind()   {}
func (*JournalGroup) MlgNodeKind()       {}
func (*PublisherGroup) MlgNodeKind()     {}
func (*VolumeGroup) MlgNodeKind()        {}
func (*MonthGroup) MlgNodeKind()         {}
func (*YearGroup) MlgNodeKind()          {}
func (*DescriptionGroup) MlgNodeKind()   {}
func (*Document) MlgNodeKind()           {}
func (*TextBlockItem) MlgNodeKind()      {}

func (*NameForm) MlgNodeKind()                               {}
func (*FunctionForm) MlgNodeKind()                           {}
func (*TupleForm) MlgNodeKind()                              {}
func (*ConditionalSetForm) MlgNodeKind()                     {}
func (*ConditionalSetIdForm) MlgNodeKind()                   {}
func (*FunctionCallExpression) MlgNodeKind()                 {}
func (*TupleExpression) MlgNodeKind()                        {}
func (*LabeledGrouping) MlgNodeKind()                        {}
func (*ConditionalSetExpression) MlgNodeKind()               {}
func (*CommandExpression) MlgNodeKind()                      {}
func (*PrefixOperatorCallExpression) MlgNodeKind()           {}
func (*PostfixOperatorCallExpression) MlgNodeKind()          {}
func (*InfixOperatorCallExpression) MlgNodeKind()            {}
func (*IsExpression) MlgNodeKind()                           {}
func (*SatisfiesExpression) MlgNodeKind()                    {}
func (*ExtendsExpression) MlgNodeKind()                      {}
func (*AsExpression) MlgNodeKind()                           {}
func (*OrdinalCallExpression) MlgNodeKind()                  {}
func (*ChainExpression) MlgNodeKind()                        {}
func (*Signature) MlgNodeKind()                              {}
func (*TypeMetaKind) MlgNodeKind()                           {}
func (*FormulationMetaKind) MlgNodeKind()                    {}
func (*StructuralColonEqualsForm) MlgNodeKind()              {}
func (*ExpressionColonEqualsItem) MlgNodeKind()              {}
func (*ExpressionColonArrowItem) MlgNodeKind()               {}
func (*ExpressionColonDashArrowItem) MlgNodeKind()           {}
func (*EnclosedNonCommandOperatorTarget) MlgNodeKind()       {}
func (*NonEnclosedNonCommandOperatorTarget) MlgNodeKind()    {}
func (*InfixCommandExpression) MlgNodeKind()                 {}
func (*CommandId) MlgNodeKind()                              {}
func (*PrefixOperatorId) MlgNodeKind()                       {}
func (*PostfixOperatorId) MlgNodeKind()                      {}
func (*InfixOperatorId) MlgNodeKind()                        {}
func (*InfixCommandOperatorId) MlgNodeKind()                 {}
func (*PseudoTokenNode) MlgNodeKind()                        {}
func (*PseudoExpression) MlgNodeKind()                       {}
func (*MultiplexedInfixOperatorCallExpression) MlgNodeKind() {}
func (*InfixOperatorForm) MlgNodeKind()                      {}
func (*PrefixOperatorForm) MlgNodeKind()                     {}
func (*PostfixOperatorForm) MlgNodeKind()                    {}
func (*NamedArg) MlgNodeKind()                               {}
func (*NamedParam) MlgNodeKind()                             {}
func (*InfixCommandId) MlgNodeKind()                         {}
func (*FunctionLiteralExpression) MlgNodeKind()              {}
func (*CurlyParam) MlgNodeKind()                             {}
func (*CurlyArg) MlgNodeKind()                               {}
func (*DirectionalParam) MlgNodeKind()                       {}
func (*FunctionLiteralForm) MlgNodeKind()                    {}
func (*ProofThenGroup) MlgNodeKind()                         {}
func (*ProofThusGroup) MlgNodeKind()                         {}
func (*ProofThereforeGroup) MlgNodeKind()                    {}
func (*ProofHenceGroup) MlgNodeKind()                        {}
func (*ProofNoticeGroup) MlgNodeKind()                       {}
func (*ProofNextGroup) MlgNodeKind()                         {}
func (*ProofByBecauseThenGroup) MlgNodeKind()                {}
func (*ProofBecauseThenGroup) MlgNodeKind()                  {}
func (*ProofStepwiseGroup) MlgNodeKind()                     {}
func (*ProofSupposeGroup) MlgNodeKind()                      {}
func (*ProofBlockGroup) MlgNodeKind()                        {}
func (*ProofWithoutLossOfGeneralityGroup) MlgNodeKind()      {}
func (*ProofContradictionGroup) MlgNodeKind()                {}
func (*ProofForContradictionGroup) MlgNodeKind()             {}
func (*ProofForInductionGroup) MlgNodeKind()                 {}
func (*ProofClaimGroup) MlgNodeKind()                        {}
func (*ProofCasewiseGroup) MlgNodeKind()                     {}
func (*ProofEquivalentlyGroup) MlgNodeKind()                 {}
func (*ProofAllOfGroup) MlgNodeKind()                        {}
func (*ProofNotGroup) MlgNodeKind()                          {}
func (*ProofAnyOfGroup) MlgNodeKind()                        {}
func (*ProofOneOfGroup) MlgNodeKind()                        {}
func (*ProofExistsGroup) MlgNodeKind()                       {}
func (*ProofExistsUniqueGroup) MlgNodeKind()                 {}
func (*ProofForAllGroup) MlgNodeKind()                       {}
func (*ProofLetGroup) MlgNodeKind()                          {}
func (*ProofIfGroup) MlgNodeKind()                           {}
func (*ProofIffGroup) MlgNodeKind()                          {}
func (*SelectFromBuiltinExpression) MlgNodeKind()            {}
func (*DefinitionBuiltinExpression) MlgNodeKind()            {}
func (*MapToElseBuiltinExpression) MlgNodeKind()             {}
func (*CommandType) MlgNodeKind()                            {}
func (*InfixCommandType) MlgNodeKind()                       {}
func (*NamedTypeParam) MlgNodeKind()                         {}
func (*CurlyTypeParam) MlgNodeKind()                         {}
func (*DirectionalTypeParam) MlgNodeKind()                   {}
func (*DirectionType) MlgNodeKind()                          {}
func (*ProofForContrapositiveGroup) MlgNodeKind()            {}
func (*ProofQedGroup) MlgNodeKind()                          {}
func (*ProofAbsurdGroup) MlgNodeKind()                       {}
func (*ProofDoneGroup) MlgNodeKind()                         {}
func (*ProofPartwiseGroup) MlgNodeKind()                     {}
func (*ProofSufficesToShowGroup) MlgNodeKind()               {}
func (*ProofToShowGroup) MlgNodeKind()                       {}
func (*ProofRemarkGroup) MlgNodeKind()                       {}

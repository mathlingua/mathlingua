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

type MlgNodeType interface {
	MlgNodeType()
}

type Position struct {
	Offset int
	Row    int
	Column int
}

type MetaData struct {
	Start Position
	Key   int
}

func (IdItem) MlgNodeType()             {}
func (Target) MlgNodeType()             {}
func (Spec) MlgNodeType()               {}
func (Alias) MlgNodeType()              {}
func (Formulation[T]) MlgNodeType()     {}
func (TextItem) MlgNodeType()           {}
func (GivenGroup) MlgNodeType()         {}
func (AllOfGroup) MlgNodeType()         {}
func (NotGroup) MlgNodeType()           {}
func (AnyOfGroup) MlgNodeType()         {}
func (OneOfGroup) MlgNodeType()         {}
func (ExistsGroup) MlgNodeType()        {}
func (ExistsUniqueGroup) MlgNodeType()  {}
func (ForAllGroup) MlgNodeType()        {}
func (IfGroup) MlgNodeType()            {}
func (IffGroup) MlgNodeType()           {}
func (PiecewiseGroup) MlgNodeType()     {}
func (WhenGroup) MlgNodeType()          {}
func (SymbolWrittenGroup) MlgNodeType() {}
func (ConnectionGroup) MlgNodeType()    {}
func (WrittenGroup) MlgNodeType()       {}
func (CalledGroup) MlgNodeType()        {}
func (WritingGroup) MlgNodeType()       {}
func (OverviewGroup) MlgNodeType()      {}
func (MotivationGroup) MlgNodeType()    {}
func (HistoryGroup) MlgNodeType()       {}
func (ExampleGroup) MlgNodeType()       {}
func (RelatedGroup) MlgNodeType()       {}
func (DiscovererGroup) MlgNodeType()    {}
func (NoteGroup) MlgNodeType()          {}
func (DescribingGroup) MlgNodeType()    {}
func (LabelGroup) MlgNodeType()         {}
func (ByGroup) MlgNodeType()            {}
func (DescribesGroup) MlgNodeType()     {}
func (DefinesGroup) MlgNodeType()       {}
func (StatesGroup) MlgNodeType()        {}
func (ProofGroup) MlgNodeType()         {}
func (AxiomGroup) MlgNodeType()         {}
func (ConjectureGroup) MlgNodeType()    {}
func (TheoremGroup) MlgNodeType()       {}
func (TopicGroup) MlgNodeType()         {}
func (ZeroGroup) MlgNodeType()          {}
func (PositiveIntGroup) MlgNodeType()   {}
func (NegativeIntGroup) MlgNodeType()   {}
func (PositiveFloatGroup) MlgNodeType() {}
func (NegativeFloatGroup) MlgNodeType() {}
func (SpecifyGroup) MlgNodeType()       {}
func (PersonGroup) MlgNodeType()        {}
func (NameGroup) MlgNodeType()          {}
func (BiographyGroup) MlgNodeType()     {}
func (ResourceGroup) MlgNodeType()      {}
func (TitleGroup) MlgNodeType()         {}
func (AuthorGroup) MlgNodeType()        {}
func (OffsetGroup) MlgNodeType()        {}
func (UrlGroup) MlgNodeType()           {}
func (HomepageGroup) MlgNodeType()      {}
func (TypeGroup) MlgNodeType()          {}
func (EditorGroup) MlgNodeType()        {}
func (EditionGroup) MlgNodeType()       {}
func (InstitutionGroup) MlgNodeType()   {}
func (JournalGroup) MlgNodeType()       {}
func (PublisherGroup) MlgNodeType()     {}
func (VolumeGroup) MlgNodeType()        {}
func (MonthGroup) MlgNodeType()         {}
func (YearGroup) MlgNodeType()          {}
func (DescriptionGroup) MlgNodeType()   {}
func (Document) MlgNodeType()           {}
func (TextBlockItem) MlgNodeType()      {}

func (NameForm) MlgNodeType()                               {}
func (FunctionForm) MlgNodeType()                           {}
func (FunctionExpressionForm) MlgNodeType()                 {}
func (TupleForm) MlgNodeType()                              {}
func (FixedSetForm) MlgNodeType()                           {}
func (ConditionalSetForm) MlgNodeType()                     {}
func (ConditionalSetIdForm) MlgNodeType()                   {}
func (FunctionCallExpression) MlgNodeType()                 {}
func (TupleExpression) MlgNodeType()                        {}
func (FixedSetExpression) MlgNodeType()                     {}
func (ConditionalSetExpression) MlgNodeType()               {}
func (CommandExpression) MlgNodeType()                      {}
func (CommandAtExpression) MlgNodeType()                    {}
func (PrefixOperatorCallExpression) MlgNodeType()           {}
func (PostfixOperatorCallExpression) MlgNodeType()          {}
func (InfixOperatorCallExpression) MlgNodeType()            {}
func (IsExpression) MlgNodeType()                           {}
func (ExtendsExpression) MlgNodeType()                      {}
func (AsExpression) MlgNodeType()                           {}
func (NameOrdinalCallExpression) MlgNodeType()              {}
func (ChainExpression) MlgNodeType()                        {}
func (Signature) MlgNodeType()                              {}
func (MetaKinds) MlgNodeType()                              {}
func (StructuralColonEqualsForm) MlgNodeType()              {}
func (ExpressionColonEqualsItem) MlgNodeType()              {}
func (ExpressionColonArrowItem) MlgNodeType()               {}
func (EnclosedNonCommandOperatorTarget) MlgNodeType()       {}
func (NonEnclosedNonCommandOperatorTarget) MlgNodeType()    {}
func (CommandOperatorTarget) MlgNodeType()                  {}
func (CommandId) MlgNodeType()                              {}
func (CommandAtId) MlgNodeType()                            {}
func (PrefixOperatorId) MlgNodeType()                       {}
func (PostfixOperatorId) MlgNodeType()                      {}
func (InfixOperatorId) MlgNodeType()                        {}
func (InfixCommandOperatorId) MlgNodeType()                 {}
func (PseudoTokenNode) MlgNodeType()                        {}
func (PseudoExpression) MlgNodeType()                       {}
func (MultiplexedInfixOperatorCallExpression) MlgNodeType() {}
func (InfixOperatorForm) MlgNodeType()                      {}
func (PrefixOperatorForm) MlgNodeType()                     {}
func (PostfixOperatorForm) MlgNodeType()                    {}

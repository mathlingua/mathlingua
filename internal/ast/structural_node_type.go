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

type StructuralNodeType interface {
	MlgNodeType
	StructuralDebuggableType
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

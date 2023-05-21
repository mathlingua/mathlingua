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

func (*IdItem) StructuralNodeKind()             {}
func (*Target) StructuralNodeKind()             {}
func (*Spec) StructuralNodeKind()               {}
func (*Alias) StructuralNodeKind()              {}
func (*Formulation[T]) StructuralNodeKind()     {}
func (*TextItem) StructuralNodeKind()           {}
func (*GivenGroup) StructuralNodeKind()         {}
func (*AllOfGroup) StructuralNodeKind()         {}
func (*NotGroup) StructuralNodeKind()           {}
func (*AnyOfGroup) StructuralNodeKind()         {}
func (*OneOfGroup) StructuralNodeKind()         {}
func (*ExistsGroup) StructuralNodeKind()        {}
func (*ExistsUniqueGroup) StructuralNodeKind()  {}
func (*ForAllGroup) StructuralNodeKind()        {}
func (*IfGroup) StructuralNodeKind()            {}
func (*IffGroup) StructuralNodeKind()           {}
func (*PiecewiseGroup) StructuralNodeKind()     {}
func (*WhenGroup) StructuralNodeKind()          {}
func (*SymbolWrittenGroup) StructuralNodeKind() {}
func (*ConnectionGroup) StructuralNodeKind()    {}
func (*WrittenGroup) StructuralNodeKind()       {}
func (*CalledGroup) StructuralNodeKind()        {}
func (*WritingGroup) StructuralNodeKind()       {}
func (*OverviewGroup) StructuralNodeKind()      {}
func (*MotivationGroup) StructuralNodeKind()    {}
func (*HistoryGroup) StructuralNodeKind()       {}
func (*ExampleGroup) StructuralNodeKind()       {}
func (*RelatedGroup) StructuralNodeKind()       {}
func (*DiscovererGroup) StructuralNodeKind()    {}
func (*NoteGroup) StructuralNodeKind()          {}
func (*DescribingGroup) StructuralNodeKind()    {}
func (*LabelGroup) StructuralNodeKind()         {}
func (*ByGroup) StructuralNodeKind()            {}
func (*DescribesGroup) StructuralNodeKind()     {}
func (*DefinesGroup) StructuralNodeKind()       {}
func (*StatesGroup) StructuralNodeKind()        {}
func (*ProofGroup) StructuralNodeKind()         {}
func (*AxiomGroup) StructuralNodeKind()         {}
func (*ConjectureGroup) StructuralNodeKind()    {}
func (*TheoremGroup) StructuralNodeKind()       {}
func (*TopicGroup) StructuralNodeKind()         {}
func (*ZeroGroup) StructuralNodeKind()          {}
func (*PositiveIntGroup) StructuralNodeKind()   {}
func (*NegativeIntGroup) StructuralNodeKind()   {}
func (*PositiveFloatGroup) StructuralNodeKind() {}
func (*NegativeFloatGroup) StructuralNodeKind() {}
func (*SpecifyGroup) StructuralNodeKind()       {}
func (*PersonGroup) StructuralNodeKind()        {}
func (*NameGroup) StructuralNodeKind()          {}
func (*BiographyGroup) StructuralNodeKind()     {}
func (*ResourceGroup) StructuralNodeKind()      {}
func (*TitleGroup) StructuralNodeKind()         {}
func (*AuthorGroup) StructuralNodeKind()        {}
func (*OffsetGroup) StructuralNodeKind()        {}
func (*UrlGroup) StructuralNodeKind()           {}
func (*HomepageGroup) StructuralNodeKind()      {}
func (*TypeGroup) StructuralNodeKind()          {}
func (*EditorGroup) StructuralNodeKind()        {}
func (*EditionGroup) StructuralNodeKind()       {}
func (*InstitutionGroup) StructuralNodeKind()   {}
func (*JournalGroup) StructuralNodeKind()       {}
func (*PublisherGroup) StructuralNodeKind()     {}
func (*VolumeGroup) StructuralNodeKind()        {}
func (*MonthGroup) StructuralNodeKind()         {}
func (*YearGroup) StructuralNodeKind()          {}
func (*DescriptionGroup) StructuralNodeKind()   {}
func (*Document) StructuralNodeKind()           {}
func (*TextBlockItem) StructuralNodeKind()      {}
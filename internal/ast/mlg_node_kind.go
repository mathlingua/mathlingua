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

import (
	"fmt"
	"mathlingua/internal/mlglib"
)

type MlgNodeKind interface {
	MlgNodeKind()
	GetCommonMetaData() *CommonMetaData
	ForEach(fn func(subNode MlgNodeKind))
}

type CommonMetaData struct {
	Start Position
	Key   int
	Scope IScope
}

func (*Root) MlgNodeKind()               {}
func (*IdItem) MlgNodeKind()             {}
func (*Target) MlgNodeKind()             {}
func (*Spec) MlgNodeKind()               {}
func (*Alias) MlgNodeKind()              {}
func (*Formulation[T]) MlgNodeKind()     {}
func (*TextItem) MlgNodeKind()           {}
func (*GivenGroup) MlgNodeKind()         {}
func (*AllOfGroup) MlgNodeKind()         {}
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
func (*ConnectionGroup) MlgNodeKind()    {}
func (*WrittenGroup) MlgNodeKind()       {}
func (*CalledGroup) MlgNodeKind()        {}
func (*ExpressedGroup) MlgNodeKind()     {}
func (*OverviewGroup) MlgNodeKind()      {}
func (*MotivationGroup) MlgNodeKind()    {}
func (*HistoryGroup) MlgNodeKind()       {}
func (*ExampleGroup) MlgNodeKind()       {}
func (*RelatedGroup) MlgNodeKind()       {}
func (*DiscovererGroup) MlgNodeKind()    {}
func (*NoteGroup) MlgNodeKind()          {}
func (*DescribingGroup) MlgNodeKind()    {}
func (*LabelGroup) MlgNodeKind()         {}
func (*ByGroup) MlgNodeKind()            {}
func (*DescribesGroup) MlgNodeKind()     {}
func (*DefinesGroup) MlgNodeKind()       {}
func (*CapturesGroup) MlgNodeKind()      {}
func (*StatesGroup) MlgNodeKind()        {}
func (*ProofGroup) MlgNodeKind()         {}
func (*AxiomGroup) MlgNodeKind()         {}
func (*ConjectureGroup) MlgNodeKind()    {}
func (*TheoremGroup) MlgNodeKind()       {}
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
func (*ConditionalSetExpression) MlgNodeKind()               {}
func (*CommandExpression) MlgNodeKind()                      {}
func (*PrefixOperatorCallExpression) MlgNodeKind()           {}
func (*PostfixOperatorCallExpression) MlgNodeKind()          {}
func (*InfixOperatorCallExpression) MlgNodeKind()            {}
func (*IsExpression) MlgNodeKind()                           {}
func (*ExtendsExpression) MlgNodeKind()                      {}
func (*AsExpression) MlgNodeKind()                           {}
func (*OrdinalCallExpression) MlgNodeKind()                  {}
func (*ChainExpression) MlgNodeKind()                        {}
func (*Signature) MlgNodeKind()                              {}
func (*MetaKinds) MlgNodeKind()                              {}
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

// The lint checker incorrectly reports that this function needs a return statement.
// nolint:typecheck
func Debug(node MlgNodeKind, fn func(node MlgNodeKind) (string, bool)) string {
	switch node := node.(type) {
	case StructuralNodeKind:
		return StructuralNodeToCode(node)
	case FormulationNodeKind:
		return FormulationNodeToCode(node, fn)
	default:
		panic(fmt.Sprintf("Cannot debug a node: %s", mlglib.PrettyPrint(node)))
	}
}

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

import (
	"fmt"
	"mathlingua/internal/mlglib"
)

type MlgNodeType interface {
	MlgNodeType()
	ForEach(fn func(subNode MlgNodeType))
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

func Debug(node MlgNodeType) string {
	switch node := node.(type) {
	case StructuralNodeType:
		return DebugStructuralNode(node)
	case FormulationNodeType:
		return DebugFormulationNode(node)
	default:
		panic(fmt.Sprintf("Cannot debug a node: %s", mlglib.PrettyPrint(node)))
	}
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
func (NamedArg) MlgNodeType()                               {}
func (NamedParam) MlgNodeType()                             {}
func (InfixCommandId) MlgNodeType()                         {}

/////////////////////////////////////////// for each /////////////////////////////////////////////////

func (n *IdItem) ForEach(fn func(subNode MlgNodeType)) {
	fn(n.Root)
}

func (n *Target) ForEach(fn func(subNode MlgNodeType)) {
	fn(n.Root)
}

func (n *Spec) ForEach(fn func(subNode MlgNodeType)) {
	fn(n.Root)
}

func (n *Alias) ForEach(fn func(subNode MlgNodeType)) {
	fn(n.Root)
}

func (n *Formulation[T]) ForEach(fn func(subNode MlgNodeType)) {
	fn(n.Root)
}

func (n *TextItem) ForEach(fn func(subNode MlgNodeType)) {
	// this doesn't have any sub nodes
}

func (n *GivenGroup) ForEach(fn func(subNode MlgNodeType)) {
	forEachTarget(n.Given.Given, fn)
	if n.Where != nil {
		forEachSpec(n.Where.Specs, fn)
	}
	if n.SuchThat != nil {
		forEach(n.SuchThat.Clauses, fn)
	}
	forEach(n.Then.Clauses, fn)
}

func (n *AllOfGroup) ForEach(fn func(subNode MlgNodeType)) {
	forEach(n.AllOf.Clauses, fn)
}

func (n *NotGroup) ForEach(fn func(subNode MlgNodeType)) {
	fn(n.Not.Clause)
}

func (n *AnyOfGroup) ForEach(fn func(subNode MlgNodeType)) {
	forEach(n.AnyOf.Clauses, fn)
}

func (n *OneOfGroup) ForEach(fn func(subNode MlgNodeType)) {
	forEach(n.OneOf.Clauses, fn)
}

func (n *ExistsGroup) ForEach(fn func(subNode MlgNodeType)) {
	forEachTarget(n.Exists.Targets, fn)
	if n.Where != nil {
		forEachSpec(n.Where.Specs, fn)
	}
	if n.SuchThat != nil {
		forEach(n.SuchThat.Clauses, fn)
	}
}

func (n *ExistsUniqueGroup) ForEach(fn func(subNode MlgNodeType)) {
	forEachTarget(n.ExistsUnique.Targets, fn)
	if n.Where != nil {
		forEachSpec(n.Where.Specs, fn)
	}
	forEach(n.SuchThat.Clauses, fn)
}

func (n *ForAllGroup) ForEach(fn func(subNode MlgNodeType)) {
	forEachTarget(n.ForAll.Targets, fn)
	if n.Where != nil {
		forEachSpec(n.Where.Specs, fn)
	}
	if n.SuchThat != nil {
		forEach(n.SuchThat.Clauses, fn)
	}
	forEach(n.Then.Clauses, fn)
}

func (n *IfGroup) ForEach(fn func(subNode MlgNodeType)) {
	forEach(n.If.Clauses, fn)
	forEach(n.Then.Clauses, fn)
}

func (n *IffGroup) ForEach(fn func(subNode MlgNodeType)) {
	forEach(n.Iff.Clauses, fn)
	forEach(n.Then.Clauses, fn)
}

func (n *PiecewiseGroup) ForEach(fn func(subNode MlgNodeType)) {
	for i, _ := range n.IfThen {
		forEach(n.IfThen[i].If.Clauses, fn)
		forEach(n.IfThen[i].Then.Clauses, fn)
	}
	if n.Else != nil {
		forEach(n.Else.Items, fn)
	}
}

func (n *WhenGroup) ForEach(fn func(subNode MlgNodeType)) {
	forEach(n.When.When, fn)
	forEach(n.Then.Clauses, fn)
}

func (n *SymbolWrittenGroup) ForEach(fn func(subNode MlgNodeType)) {
	fn(&n.Symbol.Symbol)
	if n.Written != nil {
		forEachTextItem(n.Written.Written, fn)
	}
}

func (n *ConnectionGroup) ForEach(fn func(subNode MlgNodeType)) {
	if n.Using != nil {
		forEachTarget(n.Using.Using, fn)
	}
	fn(n.Means.Means)
	if n.Signfies != nil {
		fn(&n.Signfies.Signifies)
	}
	if n.Through != nil {
		fn(&n.Through.Through)
	}
}

func (n *WrittenGroup) ForEach(fn func(subNode MlgNodeType)) {
	forEachTextItem(n.Written.Written, fn)
}

func (n *CalledGroup) ForEach(fn func(subNode MlgNodeType)) {
	forEachTextItem(n.Called.Called, fn)
}

func (n *WritingGroup) ForEach(fn func(subNode MlgNodeType)) {
	forEachTextItem(n.As.As, fn)
	fn(&n.Writing.Writing)
}

func (n *OverviewGroup) ForEach(fn func(subNode MlgNodeType)) {
	fn(&n.Overview.Overview)
}

func (n *MotivationGroup) ForEach(fn func(subNode MlgNodeType)) {
	fn(&n.Motivation.Motivation)
}

func (n *HistoryGroup) ForEach(fn func(subNode MlgNodeType)) {
	fn(&n.History.History)
}

func (n *ExampleGroup) ForEach(fn func(subNode MlgNodeType)) {
	forEachTextItem(n.Examples.Examples, fn)
}

func (n *RelatedGroup) ForEach(fn func(subNode MlgNodeType)) {
	forEachTextItem(n.Related.Related, fn)
}

func (n *DiscovererGroup) ForEach(fn func(subNode MlgNodeType)) {
	forEachTextItem(n.Discoverer.Discoverer, fn)
}

func (n *NoteGroup) ForEach(fn func(subNode MlgNodeType)) {
	forEach(n.Note.Note, fn)
}

func (n *DescribingGroup) ForEach(fn func(subNode MlgNodeType)) {
	fn(&n.Describing.Describing)
	fn(&n.Content.Content)
}

func (n *LabelGroup) ForEach(fn func(subNode MlgNodeType)) {
	fn(&n.Label.Label)
	forEachTextItem(n.By.By, fn)
}

func (n *ByGroup) ForEach(fn func(subNode MlgNodeType)) {
	forEachTextItem(n.By.By, fn)
}

func (n *DescribesGroup) ForEach(fn func(subNode MlgNodeType)) {
	fn(&n.Id)
	fn(&n.Describes.Describes)
	if n.With != nil {
		forEachTarget(n.With.With, fn)
	}
	if n.Using != nil {
		forEachTarget(n.Using.Using, fn)
	}
	if n.When != nil {
		forEach(n.When.When, fn)
	}
	if n.SuchThat != nil {
		forEach(n.SuchThat.Clauses, fn)
	}
	if n.Extends != nil {
		forEach(n.Extends.Extends, fn)
	}
	if n.Satisfies != nil {
		forEach(n.Satisfies.Satisfies, fn)
	}
	if n.Provides != nil {
		forEach(n.Provides.Provides, fn)
	}
	if n.Justified != nil {
		forEach(n.Justified.Justified, fn)
	}
	if n.Documented != nil {
		forEach(n.Documented.Documented, fn)
	}
	if n.References != nil {
		forEachTextItem(n.References.References, fn)
	}
	if n.Aliases != nil {
		forEachAlias(n.Aliases.Aliases, fn)
	}
	if n.MetaId != nil {
		fn(&n.MetaId.Id)
	}
}

func (n *DefinesGroup) ForEach(fn func(subNode MlgNodeType)) {
	fn(&n.Id)
	fn(&n.Defines.Defines)
	if n.With != nil {
		forEachTarget(n.With.With, fn)
	}
	if n.Using != nil {
		forEachTarget(n.Using.Using, fn)
	}
	if n.When != nil {
		forEach(n.When.When, fn)
	}
	if n.SuchThat != nil {
		forEach(n.SuchThat.Clauses, fn)
	}
	if n.Means != nil {
		fn(n.Means.Means)
	}
	if n.Specifies != nil {
		forEach(n.Specifies.Specifies, fn)
	}
	if n.Provides != nil {
		forEach(n.Provides.Provides, fn)
	}
	if n.Justified != nil {
		forEach(n.Justified.Justified, fn)
	}
	if n.Documented != nil {
		forEach(n.Documented.Documented, fn)
	}
	if n.References != nil {
		forEachTextItem(n.References.References, fn)
	}
	if n.Aliases != nil {
		forEachAlias(n.Aliases.Aliases, fn)
	}
	if n.MetaId != nil {
		fn(&n.MetaId.Id)
	}
}

func (n *StatesGroup) ForEach(fn func(subNode MlgNodeType)) {
	fn(&n.Id)
	if n.With != nil {
		forEachTarget(n.With.With, fn)
	}
	if n.Using != nil {
		forEachTarget(n.Using.Using, fn)
	}
	if n.When != nil {
		forEach(n.When.When, fn)
	}
	if n.SuchThat != nil {
		forEach(n.SuchThat.Clauses, fn)
	}
	forEach(n.That.That, fn)
	if n.Justified != nil {
		forEach(n.Justified.Justified, fn)
	}
	if n.Documented != nil {
		forEach(n.Documented.Documented, fn)
	}
	if n.References != nil {
		forEachTextItem(n.References.References, fn)
	}
	if n.Aliases != nil {
		forEachAlias(n.Aliases.Aliases, fn)
	}
	if n.MetaId != nil {
		fn(&n.MetaId.Id)
	}
}

func (n *ProofGroup) ForEach(fn func(subNode MlgNodeType)) {
	fn(&n.Id)
	fn(&n.Of.Of)
	fn(&n.Content.Content)
}

func (n *AxiomGroup) ForEach(fn func(subNode MlgNodeType)) {
	if n.Id != nil {
		fn(n.Id)
	}
	if n.Given != nil {
		forEachTarget(n.Given.Given, fn)
	}
	if n.Where != nil {
		forEachSpec(n.Where.Specs, fn)
	}
	if n.If != nil {
		forEach(n.If.Clauses, fn)
	}
	if n.Iff != nil {
		forEach(n.Iff.Clauses, fn)
	}
	forEach(n.Then.Clauses, fn)
	if n.Documented != nil {
		forEach(n.Documented.Documented, fn)
	}
	if n.References != nil {
		forEachTextItem(n.References.References, fn)
	}
	if n.Aliases != nil {
		forEachAlias(n.Aliases.Aliases, fn)
	}
	if n.MetaId != nil {
		fn(&n.MetaId.Id)
	}
}

func (n *ConjectureGroup) ForEach(fn func(subNode MlgNodeType)) {
	if n.Id != nil {
		fn(n.Id)
	}
	if n.Given != nil {
		forEachTarget(n.Given.Given, fn)
	}
	if n.Where != nil {
		forEachSpec(n.Where.Specs, fn)
	}
	if n.If != nil {
		forEach(n.If.Clauses, fn)
	}
	if n.Iff != nil {
		forEach(n.Iff.Clauses, fn)
	}
	forEach(n.Then.Clauses, fn)
	if n.Documented != nil {
		forEach(n.Documented.Documented, fn)
	}
	if n.References != nil {
		forEachTextItem(n.References.References, fn)
	}
	if n.Aliases != nil {
		forEachAlias(n.Aliases.Aliases, fn)
	}
	if n.MetaId != nil {
		fn(&n.MetaId.Id)
	}
}

func (n *TheoremGroup) ForEach(fn func(subNode MlgNodeType)) {
	if n.Id != nil {
		fn(n.Id)
	}
	if n.Given != nil {
		forEachTarget(n.Given.Given, fn)
	}
	if n.Where != nil {
		forEachSpec(n.Where.Specs, fn)
	}
	if n.If != nil {
		forEach(n.If.Clauses, fn)
	}
	if n.Iff != nil {
		forEach(n.Iff.Clauses, fn)
	}
	forEach(n.Then.Clauses, fn)
	if n.Proof != nil {
		fn(&n.Proof.Proof)
	}
	if n.Documented != nil {
		forEach(n.Documented.Documented, fn)
	}
	if n.References != nil {
		forEachTextItem(n.References.References, fn)
	}
	if n.Aliases != nil {
		forEachAlias(n.Aliases.Aliases, fn)
	}
	if n.MetaId != nil {
		fn(&n.MetaId.Id)
	}
}

func (n *TopicGroup) ForEach(fn func(subNode MlgNodeType)) {
	fn(&n.Id)
	fn(&n.Content.Content)
	if n.References != nil {
		forEachTextItem(n.References.References, fn)
	}
	if n.MetaId != nil {
		fn(&n.MetaId.Id)
	}
}

func (n *ZeroGroup) ForEach(fn func(subNode MlgNodeType)) {
	fn(n.Means.Means)
}

func (n *PositiveIntGroup) ForEach(fn func(subNode MlgNodeType)) {
	fn(n.Means.Means)
}

func (n *NegativeIntGroup) ForEach(fn func(subNode MlgNodeType)) {
	fn(n.Means.Means)
}

func (n *PositiveFloatGroup) ForEach(fn func(subNode MlgNodeType)) {
	fn(n.Means.Means)
}

func (n *NegativeFloatGroup) ForEach(fn func(subNode MlgNodeType)) {
	fn(n.Means.Means)
}

func (n *SpecifyGroup) ForEach(fn func(subNode MlgNodeType)) {
	forEach(n.Specify.Specify, fn)
	fn(&n.MetaId.Id)
}

func (n *PersonGroup) ForEach(fn func(subNode MlgNodeType)) {
	forEach(n.Person.Items, fn)
	fn(&n.MetaId.Id)
}

func (n *NameGroup) ForEach(fn func(subNode MlgNodeType)) {
	forEachTextItem(n.Name.Name, fn)
}

func (n *BiographyGroup) ForEach(fn func(subNode MlgNodeType)) {
	fn(&n.Biography.Biography)
}

func (n *ResourceGroup) ForEach(fn func(subNode MlgNodeType)) {
	forEach(n.Resource.Items, fn)
}

func (n *TitleGroup) ForEach(fn func(subNode MlgNodeType)) {
	fn(&n.Title.Title)
}

func (n *AuthorGroup) ForEach(fn func(subNode MlgNodeType)) {
	forEachTextItem(n.Author.Author, fn)
}

func (n *OffsetGroup) ForEach(fn func(subNode MlgNodeType)) {
	fn(&n.Offset.Offset)
}

func (n *UrlGroup) ForEach(fn func(subNode MlgNodeType)) {
	fn(&n.Url.Url)
}

func (n *HomepageGroup) ForEach(fn func(subNode MlgNodeType)) {
	fn(&n.Homepage.Homepage)
}

func (n *TypeGroup) ForEach(fn func(subNode MlgNodeType)) {
	fn(&n.Type.Type)
}

func (n *EditorGroup) ForEach(fn func(subNode MlgNodeType)) {
	forEachTextItem(n.Editor.Editor, fn)
}

func (n *EditionGroup) ForEach(fn func(subNode MlgNodeType)) {
	fn(&n.Edition.Edition)
}

func (n *InstitutionGroup) ForEach(fn func(subNode MlgNodeType)) {
	forEachTextItem(n.Institution.Institution, fn)
}

func (n *JournalGroup) ForEach(fn func(subNode MlgNodeType)) {
	forEachTextItem(n.Journal.Journal, fn)
}

func (n *PublisherGroup) ForEach(fn func(subNode MlgNodeType)) {
	forEachTextItem(n.Publisher.Publisher, fn)
}

func (n *VolumeGroup) ForEach(fn func(subNode MlgNodeType)) {
	fn(&n.Volume.Volume)
}

func (n *MonthGroup) ForEach(fn func(subNode MlgNodeType)) {
	fn(&n.Month.Month)
}

func (n *YearGroup) ForEach(fn func(subNode MlgNodeType)) {
	fn(&n.Year.Year)
}

func (n *DescriptionGroup) ForEach(fn func(subNode MlgNodeType)) {
	fn(&n.Description.Description)
}

func (n *Document) ForEach(fn func(subNode MlgNodeType)) {
	for i, _ := range n.Items {
		fn(n.Items[i])
	}
}

func (n *TextBlockItem) ForEach(fn func(subNode MlgNodeType)) {
	// this doesn't have any sub nodes
}

func (n *NameForm) ForEach(fn func(subNode MlgNodeType)) {
	// this doesn't have any sub nodes
}

func (n *FunctionForm) ForEach(fn func(subNode MlgNodeType)) {
	fn(&n.Target)
	forEachNameForm(n.Params, fn)
}

func (n *FunctionExpressionForm) ForEach(fn func(subNode MlgNodeType)) {
	fn(&n.Target)
	forEachNameForm(n.Params, fn)
}

func (n *TupleForm) ForEach(fn func(subNode MlgNodeType)) {
	forEach(n.Params, fn)
}

func (n *FixedSetForm) ForEach(fn func(subNode MlgNodeType)) {
	forEach(n.Params, fn)
}

func (n *ConditionalSetForm) ForEach(fn func(subNode MlgNodeType)) {
	fn(n.Target)
}

func (n *ConditionalSetIdForm) ForEach(fn func(subNode MlgNodeType)) {
	forEach(n.Symbols, fn)
	fn(n.Target)
	fn(&n.Condition)
}

func (n *FunctionCallExpression) ForEach(fn func(subNode MlgNodeType)) {
	fn(n.Target)
	forEach(n.Args, fn)
}

func (n *TupleExpression) ForEach(fn func(subNode MlgNodeType)) {
	forEach(n.Args, fn)
}

func (n *FixedSetExpression) ForEach(fn func(subNode MlgNodeType)) {
	forEach(n.Args, fn)
}

func (n *ConditionalSetExpression) ForEach(fn func(subNode MlgNodeType)) {
	forEach(n.Symbols, fn)
	fn(n.Target)
	forEach(n.Conditions, fn)
}

func (n *CommandExpression) ForEach(fn func(subNode MlgNodeType)) {
	forEachNameForm(n.Names, fn)
	if n.SquareArgs != nil {
		forEach(*n.SquareArgs, fn)
	}
	if n.CurlyArgs != nil {
		forEach(*n.CurlyArgs, fn)
	}
	if n.NamedArgs != nil {
		forEachNamedArg(*n.NamedArgs, fn)
	}
	if n.ParenArgs != nil {
		forEach(*n.ParenArgs, fn)
	}
}

func (n *NamedArg) ForEach(fn func(subNode MlgNodeType)) {
	fn(&n.Name)
	if n.Args != nil {
		forEach(*n.Args, fn)
	}
}

func (n *PrefixOperatorCallExpression) ForEach(fn func(subNode MlgNodeType)) {
	fn(n.Target)
	fn(n.Arg)
}

func (n *PostfixOperatorCallExpression) ForEach(fn func(subNode MlgNodeType)) {
	fn(n.Arg)
	fn(n.Target)
}

func (n *InfixOperatorCallExpression) ForEach(fn func(subNode MlgNodeType)) {
	fn(n.Lhs)
	fn(n.Target)
	fn(n.Rhs)
}

func (n *IsExpression) ForEach(fn func(subNode MlgNodeType)) {
	forEach(n.Lhs, fn)
	forEach(n.Rhs, fn)
}

func (n *ExtendsExpression) ForEach(fn func(subNode MlgNodeType)) {
	forEach(n.Lhs, fn)
	forEach(n.Rhs, fn)
}

func (n *AsExpression) ForEach(fn func(subNode MlgNodeType)) {
	fn(n.Lhs)
	fn(&n.Rhs)
}

func (n *NameOrdinalCallExpression) ForEach(fn func(subNode MlgNodeType)) {
	fn(&n.Target)
	fn(n.Arg)
}

func (n *ChainExpression) ForEach(fn func(subNode MlgNodeType)) {
	forEach(n.Parts, fn)
}

func (n Signature) ForEach(fn func(subNode MlgNodeType)) {
	// this doesn't have any sub nodes
}

func (n *MetaKinds) ForEach(fn func(subNode MlgNodeType)) {
	// this doesn't have any sub nodes
}

func (n *StructuralColonEqualsForm) ForEach(fn func(subNode MlgNodeType)) {
	fn(n.Lhs)
	fn(n.Rhs)
}

func (n *ExpressionColonEqualsItem) ForEach(fn func(subNode MlgNodeType)) {
	fn(n.Lhs)
	fn(n.Rhs)
}

func (n *ExpressionColonArrowItem) ForEach(fn func(subNode MlgNodeType)) {
	fn(n.Lhs)
	fn(n.Rhs)
}

func (n *EnclosedNonCommandOperatorTarget) ForEach(fn func(subNode MlgNodeType)) {
	fn(n.Target)
}

func (n *NonEnclosedNonCommandOperatorTarget) ForEach(fn func(subNode MlgNodeType)) {
	// this doesn't have any sub nodes
}

func (n *CommandOperatorTarget) ForEach(fn func(subNode MlgNodeType)) {
	fn(&n.Command)
}

func (n *CommandId) ForEach(fn func(subNode MlgNodeType)) {
	forEachNameForm(n.Names, fn)
	if n.SquareParams != nil {
		forEach(*n.SquareParams, fn)
	}
	if n.CurlyParams != nil {
		forEach(*n.CurlyParams, fn)
	}
	if n.NamedParams != nil {
		forEachNamedParam(*n.NamedParams, fn)
	}
	if n.ParenParams != nil {
		forEachNameForm(*n.ParenParams, fn)
	}
}

func (n *NamedParam) ForEach(fn func(subNode MlgNodeType)) {
	fn(&n.Name)
	if n.Params != nil {
		forEach(*n.Params, fn)
	}
}

func (n *PrefixOperatorId) ForEach(fn func(subNode MlgNodeType)) {
	fn(&n.Operator)
	fn(n.Param)
}

func (n *PostfixOperatorId) ForEach(fn func(subNode MlgNodeType)) {
	fn(n.Param)
	fn(&n.Operator)
}

func (n *InfixOperatorId) ForEach(fn func(subNode MlgNodeType)) {
	fn(n.Lhs)
	fn(&n.Operator)
	fn(n.Rhs)
}

func (n *InfixCommandOperatorId) ForEach(fn func(subNode MlgNodeType)) {
	fn(n.Lhs)
	fn(&n.Operator)
	fn(n.Rhs)
}

func (n *InfixCommandId) ForEach(fn func(subNode MlgNodeType)) {
	forEachNameForm(n.Names, fn)
	if n.SquareParams != nil {
		forEach(*n.SquareParams, fn)
	}
	if n.CurlyParams != nil {
		forEach(*n.CurlyParams, fn)
	}
	if n.NamedParams != nil {
		forEachNamedParam(*n.NamedParams, fn)
	}
	if n.ParenParams != nil {
		forEachNameForm(*n.ParenParams, fn)
	}
}

func (n *PseudoTokenNode) ForEach(fn func(subNode MlgNodeType)) {
	// this doesn't have any sub nodes
}

func (n *PseudoExpression) ForEach(fn func(subNode MlgNodeType)) {
	forEach(n.Children, fn)
}

func (n *MultiplexedInfixOperatorCallExpression) ForEach(fn func(subNode MlgNodeType)) {
	forEach(n.Lhs, fn)
	fn(n.Target)
	forEach(n.Rhs, fn)
}

func (n *InfixOperatorForm) ForEach(fn func(subNode MlgNodeType)) {
	fn(&n.Lhs)
	fn(&n.Operator)
	fn(&n.Rhs)
}

func (n *PrefixOperatorForm) ForEach(fn func(subNode MlgNodeType)) {
	fn(&n.Operator)
	fn(&n.Param)
}

func (n *PostfixOperatorForm) ForEach(fn func(subNode MlgNodeType)) {
	fn(&n.Param)
	fn(&n.Operator)
}

/////////////////////////////////////////////////////////////////////////////////////////////

func forEach[T MlgNodeType](items []T, fn func(n MlgNodeType)) {
	for i, _ := range items {
		fn(items[i])
	}
}

func forEachNameForm(items []NameForm, fn func(n MlgNodeType)) {
	for i, _ := range items {
		fn(&items[i])
	}
}

func forEachNamedArg(items []NamedArg, fn func(n MlgNodeType)) {
	for i, _ := range items {
		fn(&items[i])
	}
}

func forEachNamedParam(items []NamedParam, fn func(n MlgNodeType)) {
	for i, _ := range items {
		fn(&items[i])
	}
}

func forEachTarget(items []Target, fn func(n MlgNodeType)) {
	for i, _ := range items {
		fn(&items[i])
	}
}

func forEachSpec(items []Spec, fn func(n MlgNodeType)) {
	for i, _ := range items {
		fn(&items[i])
	}
}

func forEachAlias(items []Alias, fn func(n MlgNodeType)) {
	for i, _ := range items {
		fn(&items[i])
	}
}

func forEachTextItem(items []TextItem, fn func(n MlgNodeType)) {
	for i, _ := range items {
		fn(&items[i])
	}
}

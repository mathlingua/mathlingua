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

func (n *Root) ForEach(fn func(subNode MlgNodeKind)) {
	for _, doc := range n.Documents {
		fn(&doc)
	}
}

func (n *IdItem) ForEach(fn func(subNode MlgNodeKind)) {
	fn(n.Root)
}

func (n *Target) ForEach(fn func(subNode MlgNodeKind)) {
	fn(n.Root)
}

func (n *Spec) ForEach(fn func(subNode MlgNodeKind)) {
	fn(n.Root)
}

func (n *Alias) ForEach(fn func(subNode MlgNodeKind)) {
	fn(n.Root)
}

func (n *Formulation[T]) ForEach(fn func(subNode MlgNodeKind)) {
	fn(n.Root)
}

func (n *TextItem) ForEach(fn func(subNode MlgNodeKind)) {
	// this doesn't have any sub nodes
}

func (n *GivenGroup) ForEach(fn func(subNode MlgNodeKind)) {
	forEachTarget(n.Given.Given, fn)
	if n.Where != nil {
		forEachSpec(n.Where.Specs, fn)
	}
	if n.SuchThat != nil {
		forEach(n.SuchThat.Clauses, fn)
	}
	forEach(n.Then.Clauses, fn)
}

func (n *AllOfGroup) ForEach(fn func(subNode MlgNodeKind)) {
	forEach(n.AllOf.Clauses, fn)
}

func (n *NotGroup) ForEach(fn func(subNode MlgNodeKind)) {
	fn(n.Not.Clause)
}

func (n *AnyOfGroup) ForEach(fn func(subNode MlgNodeKind)) {
	forEach(n.AnyOf.Clauses, fn)
}

func (n *OneOfGroup) ForEach(fn func(subNode MlgNodeKind)) {
	forEach(n.OneOf.Clauses, fn)
}

func (n *ExistsGroup) ForEach(fn func(subNode MlgNodeKind)) {
	forEachTarget(n.Exists.Targets, fn)
	if n.Where != nil {
		forEachSpec(n.Where.Specs, fn)
	}
	if n.SuchThat != nil {
		forEach(n.SuchThat.Clauses, fn)
	}
}

func (n *ExistsUniqueGroup) ForEach(fn func(subNode MlgNodeKind)) {
	forEachTarget(n.ExistsUnique.Targets, fn)
	if n.Where != nil {
		forEachSpec(n.Where.Specs, fn)
	}
	forEach(n.SuchThat.Clauses, fn)
}

func (n *ForAllGroup) ForEach(fn func(subNode MlgNodeKind)) {
	forEachTarget(n.ForAll.Targets, fn)
	if n.Where != nil {
		forEachSpec(n.Where.Specs, fn)
	}
	if n.SuchThat != nil {
		forEach(n.SuchThat.Clauses, fn)
	}
	forEach(n.Then.Clauses, fn)
}

func (n *IfGroup) ForEach(fn func(subNode MlgNodeKind)) {
	forEach(n.If.Clauses, fn)
	forEach(n.Then.Clauses, fn)
}

func (n *IffGroup) ForEach(fn func(subNode MlgNodeKind)) {
	forEach(n.Iff.Clauses, fn)
	forEach(n.Then.Clauses, fn)
}

func (n *PiecewiseGroup) ForEach(fn func(subNode MlgNodeKind)) {
	for i, _ := range n.IfThen {
		forEach(n.IfThen[i].If.Clauses, fn)
		forEach(n.IfThen[i].Then.Clauses, fn)
	}
	if n.Else != nil {
		forEach(n.Else.Items, fn)
	}
}

func (n *WhenGroup) ForEach(fn func(subNode MlgNodeKind)) {
	forEach(n.When.When, fn)
	forEach(n.Then.Clauses, fn)
}

func (n *SymbolWrittenGroup) ForEach(fn func(subNode MlgNodeKind)) {
	fn(&n.Symbol.Symbol)
	if n.Written != nil {
		forEachTextItem(n.Written.Written, fn)
	}
}

func (n *ConnectionGroup) ForEach(fn func(subNode MlgNodeKind)) {
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

func (n *WrittenGroup) ForEach(fn func(subNode MlgNodeKind)) {
	forEachTextItem(n.Written.Written, fn)
}

func (n *CalledGroup) ForEach(fn func(subNode MlgNodeKind)) {
	forEachTextItem(n.Called.Called, fn)
}

func (n *WritingGroup) ForEach(fn func(subNode MlgNodeKind)) {
	forEachTextItem(n.As.As, fn)
	fn(&n.Writing.Writing)
}

func (n *OverviewGroup) ForEach(fn func(subNode MlgNodeKind)) {
	fn(&n.Overview.Overview)
}

func (n *MotivationGroup) ForEach(fn func(subNode MlgNodeKind)) {
	fn(&n.Motivation.Motivation)
}

func (n *HistoryGroup) ForEach(fn func(subNode MlgNodeKind)) {
	fn(&n.History.History)
}

func (n *ExampleGroup) ForEach(fn func(subNode MlgNodeKind)) {
	forEachTextItem(n.Examples.Examples, fn)
}

func (n *RelatedGroup) ForEach(fn func(subNode MlgNodeKind)) {
	forEachTextItem(n.Related.Related, fn)
}

func (n *DiscovererGroup) ForEach(fn func(subNode MlgNodeKind)) {
	forEachTextItem(n.Discoverer.Discoverer, fn)
}

func (n *NoteGroup) ForEach(fn func(subNode MlgNodeKind)) {
	forEach(n.Note.Note, fn)
}

func (n *DescribingGroup) ForEach(fn func(subNode MlgNodeKind)) {
	fn(&n.Describing.Describing)
	fn(&n.Content.Content)
}

func (n *LabelGroup) ForEach(fn func(subNode MlgNodeKind)) {
	fn(&n.Label.Label)
	forEachTextItem(n.By.By, fn)
}

func (n *ByGroup) ForEach(fn func(subNode MlgNodeKind)) {
	forEachTextItem(n.By.By, fn)
}

func (n *DescribesGroup) ForEach(fn func(subNode MlgNodeKind)) {
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

func (n *DefinesGroup) ForEach(fn func(subNode MlgNodeKind)) {
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
	if n.Generalizes != nil {
		forEachFormulation(n.Generalizes.Generalizes, fn)
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

func (n *StatesGroup) ForEach(fn func(subNode MlgNodeKind)) {
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

func (n *ProofGroup) ForEach(fn func(subNode MlgNodeKind)) {
	fn(&n.Id)
	fn(&n.Of.Of)
	fn(&n.Content.Content)
}

func (n *AxiomGroup) ForEach(fn func(subNode MlgNodeKind)) {
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

func (n *ConjectureGroup) ForEach(fn func(subNode MlgNodeKind)) {
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

func (n *TheoremGroup) ForEach(fn func(subNode MlgNodeKind)) {
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

func (n *TopicGroup) ForEach(fn func(subNode MlgNodeKind)) {
	fn(&n.Id)
	fn(&n.Content.Content)
	if n.References != nil {
		forEachTextItem(n.References.References, fn)
	}
	if n.MetaId != nil {
		fn(&n.MetaId.Id)
	}
}

func (n *ZeroGroup) ForEach(fn func(subNode MlgNodeKind)) {
	fn(n.Means.Means)
}

func (n *PositiveIntGroup) ForEach(fn func(subNode MlgNodeKind)) {
	fn(n.Means.Means)
}

func (n *NegativeIntGroup) ForEach(fn func(subNode MlgNodeKind)) {
	fn(n.Means.Means)
}

func (n *PositiveFloatGroup) ForEach(fn func(subNode MlgNodeKind)) {
	fn(n.Means.Means)
}

func (n *NegativeFloatGroup) ForEach(fn func(subNode MlgNodeKind)) {
	fn(n.Means.Means)
}

func (n *SpecifyGroup) ForEach(fn func(subNode MlgNodeKind)) {
	forEach(n.Specify.Specify, fn)
	fn(&n.MetaId.Id)
}

func (n *PersonGroup) ForEach(fn func(subNode MlgNodeKind)) {
	forEach(n.Person.Items, fn)
	fn(&n.MetaId.Id)
}

func (n *NameGroup) ForEach(fn func(subNode MlgNodeKind)) {
	forEachTextItem(n.Name.Name, fn)
}

func (n *BiographyGroup) ForEach(fn func(subNode MlgNodeKind)) {
	fn(&n.Biography.Biography)
}

func (n *ResourceGroup) ForEach(fn func(subNode MlgNodeKind)) {
	forEach(n.Resource.Items, fn)
}

func (n *TitleGroup) ForEach(fn func(subNode MlgNodeKind)) {
	fn(&n.Title.Title)
}

func (n *AuthorGroup) ForEach(fn func(subNode MlgNodeKind)) {
	forEachTextItem(n.Author.Author, fn)
}

func (n *OffsetGroup) ForEach(fn func(subNode MlgNodeKind)) {
	fn(&n.Offset.Offset)
}

func (n *UrlGroup) ForEach(fn func(subNode MlgNodeKind)) {
	fn(&n.Url.Url)
}

func (n *HomepageGroup) ForEach(fn func(subNode MlgNodeKind)) {
	fn(&n.Homepage.Homepage)
}

func (n *TypeGroup) ForEach(fn func(subNode MlgNodeKind)) {
	fn(&n.Type.Type)
}

func (n *EditorGroup) ForEach(fn func(subNode MlgNodeKind)) {
	forEachTextItem(n.Editor.Editor, fn)
}

func (n *EditionGroup) ForEach(fn func(subNode MlgNodeKind)) {
	fn(&n.Edition.Edition)
}

func (n *InstitutionGroup) ForEach(fn func(subNode MlgNodeKind)) {
	forEachTextItem(n.Institution.Institution, fn)
}

func (n *JournalGroup) ForEach(fn func(subNode MlgNodeKind)) {
	forEachTextItem(n.Journal.Journal, fn)
}

func (n *PublisherGroup) ForEach(fn func(subNode MlgNodeKind)) {
	forEachTextItem(n.Publisher.Publisher, fn)
}

func (n *VolumeGroup) ForEach(fn func(subNode MlgNodeKind)) {
	fn(&n.Volume.Volume)
}

func (n *MonthGroup) ForEach(fn func(subNode MlgNodeKind)) {
	fn(&n.Month.Month)
}

func (n *YearGroup) ForEach(fn func(subNode MlgNodeKind)) {
	fn(&n.Year.Year)
}

func (n *DescriptionGroup) ForEach(fn func(subNode MlgNodeKind)) {
	fn(&n.Description.Description)
}

func (n *Document) ForEach(fn func(subNode MlgNodeKind)) {
	for i, _ := range n.Items {
		fn(n.Items[i])
	}
}

func (n *TextBlockItem) ForEach(fn func(subNode MlgNodeKind)) {
	// this doesn't have any sub nodes
}

func (n *NameForm) ForEach(fn func(subNode MlgNodeKind)) {
	// this doesn't have any sub nodes
}

func (n *FunctionForm) ForEach(fn func(subNode MlgNodeKind)) {
	fn(&n.Target)
	forEach(n.Params, fn)
}

func (n *TupleForm) ForEach(fn func(subNode MlgNodeKind)) {
	forEach(n.Params, fn)
}

func (n *ConditionalSetForm) ForEach(fn func(subNode MlgNodeKind)) {
	fn(n.Target)
}

func (n *ConditionalSetIdForm) ForEach(fn func(subNode MlgNodeKind)) {
	forEach(n.Symbols, fn)
	fn(n.Target)
	fn(&n.Condition)
}

func (n *FunctionCallExpression) ForEach(fn func(subNode MlgNodeKind)) {
	fn(n.Target)
	forEach(n.Args, fn)
}

func (n *TupleExpression) ForEach(fn func(subNode MlgNodeKind)) {
	forEach(n.Args, fn)
}

func (n *ConditionalSetExpression) ForEach(fn func(subNode MlgNodeKind)) {
	forEach(n.Symbols, fn)
	fn(n.Target)
	forEach(n.Conditions, fn)
}

func (n *CommandExpression) ForEach(fn func(subNode MlgNodeKind)) {
	forEachNameForm(n.Names, fn)
	if n.CurlyArg != nil {
		fn(n.CurlyArg)
	}
	if n.NamedArgs != nil {
		forEachNamedArg(*n.NamedArgs, fn)
	}
	if n.ParenArgs != nil {
		forEach(*n.ParenArgs, fn)
	}
}

func (n *NamedArg) ForEach(fn func(subNode MlgNodeKind)) {
	fn(&n.Name)
	if n.CurlyArg != nil {
		fn(n.CurlyArg)
	}
}

func (n *PrefixOperatorCallExpression) ForEach(fn func(subNode MlgNodeKind)) {
	fn(n.Target)
	fn(n.Arg)
}

func (n *PostfixOperatorCallExpression) ForEach(fn func(subNode MlgNodeKind)) {
	fn(n.Arg)
	fn(n.Target)
}

func (n *InfixOperatorCallExpression) ForEach(fn func(subNode MlgNodeKind)) {
	fn(n.Lhs)
	fn(n.Target)
	fn(n.Rhs)
}

func (n *IsExpression) ForEach(fn func(subNode MlgNodeKind)) {
	forEach(n.Lhs, fn)
	forEach(n.Rhs, fn)
}

func (n *ExtendsExpression) ForEach(fn func(subNode MlgNodeKind)) {
	forEach(n.Lhs, fn)
	forEach(n.Rhs, fn)
}

func (n *AsExpression) ForEach(fn func(subNode MlgNodeKind)) {
	fn(n.Lhs)
	fn(&n.Rhs)
}

func (n *OrdinalCallExpression) ForEach(fn func(subNode MlgNodeKind)) {
	fn(n.Target)
	forEach(n.Args, fn)
}

func (n *ChainExpression) ForEach(fn func(subNode MlgNodeKind)) {
	forEach(n.Parts, fn)
}

func (n Signature) ForEach(fn func(subNode MlgNodeKind)) {
	// this doesn't have any sub nodes
}

func (n *MetaKinds) ForEach(fn func(subNode MlgNodeKind)) {
	// this doesn't have any sub nodes
}

func (n *StructuralColonEqualsForm) ForEach(fn func(subNode MlgNodeKind)) {
	fn(n.Lhs)
	fn(n.Rhs)
}

func (n *ExpressionColonEqualsItem) ForEach(fn func(subNode MlgNodeKind)) {
	fn(n.Lhs)
	fn(n.Rhs)
}

func (n *ExpressionColonArrowItem) ForEach(fn func(subNode MlgNodeKind)) {
	fn(n.Lhs)
	fn(n.Rhs)
}

func (n *ExpressionColonDashArrowItem) ForEach(fn func(subNode MlgNodeKind)) {
	fn(n.Lhs)
	fn(n.Rhs)
}

func (n *EnclosedNonCommandOperatorTarget) ForEach(fn func(subNode MlgNodeKind)) {
	fn(n.Target)
}

func (n *NonEnclosedNonCommandOperatorTarget) ForEach(fn func(subNode MlgNodeKind)) {
	// this doesn't have any sub nodes
}

func (n *CommandOperatorTarget) ForEach(fn func(subNode MlgNodeKind)) {
	fn(&n.Command)
}

func (n *CommandId) ForEach(fn func(subNode MlgNodeKind)) {
	forEachNameForm(n.Names, fn)
	if n.CurlyParam != nil {
		fn(n.CurlyParam)
	}
	if n.NamedParams != nil {
		forEachNamedParam(*n.NamedParams, fn)
	}
	if n.ParenParams != nil {
		forEachNameForm(*n.ParenParams, fn)
	}
}

func (n *NamedParam) ForEach(fn func(subNode MlgNodeKind)) {
	fn(&n.Name)
	if n.CurlyParam != nil {
		fn(n.CurlyParam)
	}
}

func (n *PrefixOperatorId) ForEach(fn func(subNode MlgNodeKind)) {
	fn(&n.Operator)
	fn(n.Param)
}

func (n *PostfixOperatorId) ForEach(fn func(subNode MlgNodeKind)) {
	fn(n.Param)
	fn(&n.Operator)
}

func (n *InfixOperatorId) ForEach(fn func(subNode MlgNodeKind)) {
	fn(n.Lhs)
	fn(&n.Operator)
	fn(n.Rhs)
}

func (n *InfixCommandOperatorId) ForEach(fn func(subNode MlgNodeKind)) {
	fn(n.Lhs)
	fn(&n.Operator)
	fn(n.Rhs)
}

func (n *InfixCommandId) ForEach(fn func(subNode MlgNodeKind)) {
	forEachNameForm(n.Names, fn)
	if n.CurlyParam != nil {
		fn(n.CurlyParam)
	}
	if n.NamedParams != nil {
		forEachNamedParam(*n.NamedParams, fn)
	}
	if n.ParenParams != nil {
		forEachNameForm(*n.ParenParams, fn)
	}
}

func (n *PseudoTokenNode) ForEach(fn func(subNode MlgNodeKind)) {
	// this doesn't have any sub nodes
}

func (n *PseudoExpression) ForEach(fn func(subNode MlgNodeKind)) {
	forEach(n.Children, fn)
}

func (n *MultiplexedInfixOperatorCallExpression) ForEach(fn func(subNode MlgNodeKind)) {
	forEach(n.Lhs, fn)
	fn(n.Target)
	forEach(n.Rhs, fn)
}

func (n *InfixOperatorForm) ForEach(fn func(subNode MlgNodeKind)) {
	fn(n.Lhs)
	fn(&n.Operator)
	fn(n.Rhs)
}

func (n *PrefixOperatorForm) ForEach(fn func(subNode MlgNodeKind)) {
	fn(&n.Operator)
	fn(n.Param)
}

func (n *PostfixOperatorForm) ForEach(fn func(subNode MlgNodeKind)) {
	fn(n.Param)
	fn(&n.Operator)
}

func (n *FunctionLiteralExpression) ForEach(fn func(subNode MlgNodeKind)) {
	fn(&n.Lhs)
	fn(n.Rhs)
}

func (n *CurlyParam) ForEach(fn func(subNode MlgNodeKind)) {
	if n.SquareParams != nil {
		forEach(*n.SquareParams, fn)
	}
	forEach(n.CurlyParams, fn)
	if n.Direction != nil {
		fn(n.Direction)
	}
}

func (n *CurlyArg) ForEach(fn func(subNode MlgNodeKind)) {
	forEach(*n.CurlyArgs, fn)
	if n.Direction != nil {
		fn(n.Direction)
	}
}

func (n *DirectionalParam) ForEach(fn func(subNode MlgNodeKind)) {
	if n.Name != nil {
		fn(n.Name)
	}
	forEach(n.SquareParams, fn)
}

////////////////////////////////////////////////////////////////////////////////////////////////////

func forEach[T MlgNodeKind](items []T, fn func(n MlgNodeKind)) {
	for i, _ := range items {
		fn(items[i])
	}
}

func forEachNameForm(items []NameForm, fn func(n MlgNodeKind)) {
	for i, _ := range items {
		fn(&items[i])
	}
}

func forEachNamedArg(items []NamedArg, fn func(n MlgNodeKind)) {
	for i, _ := range items {
		fn(&items[i])
	}
}

func forEachNamedParam(items []NamedParam, fn func(n MlgNodeKind)) {
	for i, _ := range items {
		fn(&items[i])
	}
}

func forEachFormulation(items []Formulation[FormulationNodeKind], fn func(n MlgNodeKind)) {
	for i, _ := range items {
		fn(&items[i])
	}
}

func forEachTarget(items []Target, fn func(n MlgNodeKind)) {
	for i, _ := range items {
		fn(&items[i])
	}
}

func forEachSpec(items []Spec, fn func(n MlgNodeKind)) {
	for i, _ := range items {
		fn(&items[i])
	}
}

func forEachAlias(items []Alias, fn func(n MlgNodeKind)) {
	for i, _ := range items {
		fn(&items[i])
	}
}

func forEachTextItem(items []TextItem, fn func(n MlgNodeKind)) {
	for i, _ := range items {
		fn(&items[i])
	}
}

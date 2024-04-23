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

func (n *DeclareGroup) ForEach(fn func(subNode MlgNodeKind)) {
	forEachTarget(n.Declare.Declare, fn)
	if n.Using != nil {
		forEachTarget(n.Using.Using, fn)
	}
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

func (n *EquivalentlyGroup) ForEach(fn func(subNode MlgNodeKind)) {
	forEach(n.Equivalently.Clauses, fn)
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
	if n.Using != nil {
		forEachTarget(n.Using.Using, fn)
	}
	if n.Where != nil {
		forEachSpec(n.Where.Specs, fn)
	}
	if n.SuchThat != nil {
		forEach(n.SuchThat.Clauses, fn)
	}
}

func (n *ExistsUniqueGroup) ForEach(fn func(subNode MlgNodeKind)) {
	forEachTarget(n.ExistsUnique.Targets, fn)
	if n.Using != nil {
		forEachTarget(n.Using.Using, fn)
	}
	if n.Where != nil {
		forEachSpec(n.Where.Specs, fn)
	}
	forEach(n.SuchThat.Clauses, fn)
}

func (n *ForAllGroup) ForEach(fn func(subNode MlgNodeKind)) {
	forEachTarget(n.ForAll.Targets, fn)
	if n.Using != nil {
		forEachTarget(n.Using.Using, fn)
	}
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

func (n *ProofDeclareGroup) ForEach(fn func(subNode MlgNodeKind)) {
	forEachTarget(n.Declare.Declare, fn)
	if n.Using != nil {
		forEachTarget(n.Using.Using, fn)
	}
	if n.Where != nil {
		forEachSpec(n.Where.Specs, fn)
	}
	if n.SuchThat != nil {
		forEach(n.SuchThat.Items, fn)
	}
	forEach(n.Then.Then, fn)
}

func (n *ProofAllOfGroup) ForEach(fn func(subNode MlgNodeKind)) {
	forEach(n.AllOf.Items, fn)
}

func (n *ProofEquivalentlyGroup) ForEach(fn func(subNode MlgNodeKind)) {
	forEach(n.Equivalently.Items, fn)
}

func (n *ProofNotGroup) ForEach(fn func(subNode MlgNodeKind)) {
	fn(n.Not.Item)
}

func (n *ProofAnyOfGroup) ForEach(fn func(subNode MlgNodeKind)) {
	forEach(n.AnyOf.Items, fn)
}

func (n *ProofOneOfGroup) ForEach(fn func(subNode MlgNodeKind)) {
	forEach(n.OneOf.Items, fn)
}

func (n *ProofExistsGroup) ForEach(fn func(subNode MlgNodeKind)) {
	forEachTarget(n.Exists.Targets, fn)
	if n.Using != nil {
		forEachTarget(n.Using.Using, fn)
	}
	if n.Where != nil {
		forEachSpec(n.Where.Specs, fn)
	}
	if n.SuchThat != nil {
		forEach(n.SuchThat.Items, fn)
	}
}

func (n *ProofExistsUniqueGroup) ForEach(fn func(subNode MlgNodeKind)) {
	forEachTarget(n.ExistsUnique.Targets, fn)
	if n.Using != nil {
		forEachTarget(n.Using.Using, fn)
	}
	if n.Where != nil {
		forEachSpec(n.Where.Specs, fn)
	}
	forEach(n.SuchThat.Items, fn)
}

func (n *ProofForAllGroup) ForEach(fn func(subNode MlgNodeKind)) {
	forEachTarget(n.ForAll.Targets, fn)
	if n.Using != nil {
		forEachTarget(n.Using.Using, fn)
	}
	if n.Where != nil {
		forEachSpec(n.Where.Specs, fn)
	}
	if n.SuchThat != nil {
		forEach(n.SuchThat.Items, fn)
	}
	forEach(n.Then.Then, fn)
}

func (n *ProofIfGroup) ForEach(fn func(subNode MlgNodeKind)) {
	forEach(n.If.Items, fn)
	forEach(n.Then.Then, fn)
}

func (n *ProofIffGroup) ForEach(fn func(subNode MlgNodeKind)) {
	forEach(n.Iff.Items, fn)
	forEach(n.Then.Then, fn)
}

func (n *PiecewiseGroup) ForEach(fn func(subNode MlgNodeKind)) {
	forEach(n.IfThen.If.Clauses, fn)
	forEach(n.IfThen.Then.Clauses, fn)
	for i := range n.ElseIfThen {
		forEach(n.ElseIfThen[i].ElseIf.Clauses, fn)
		forEach(n.ElseIfThen[i].Then.Clauses, fn)
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
	if n.Replaces != nil {
		fn(&n.Replaces.Replaces)
	}
	if n.Written != nil {
		forEachTextItem(n.Written.Written, fn)
	}
}

func (n *ViewGroup) ForEach(fn func(subNode MlgNodeKind)) {
	fn(&n.As.As)
	if n.Using != nil {
		forEachTarget(n.Using.Using, fn)
	}
	if n.Where != nil {
		forEachSpec(n.Where.Specs, fn)
	}
	if n.Through != nil {
		forEachFormulation(n.Through.Through, fn)
	}
	if n.Signfies != nil {
		forEachSpec(n.Signfies.Signifies, fn)
	}
}

func (n *EncodingGroup) ForEach(fn func(subNode MlgNodeKind)) {
	fn(&n.As.As)
	if n.Using != nil {
		forEachTarget(n.Using.Using, fn)
	}
	if n.Where != nil {
		forEachSpec(n.Where.Specs, fn)
	}
	if n.Through != nil {
		forEachFormulation(n.Through.Through, fn)
	}
}

func (n *WrittenGroup) ForEach(fn func(subNode MlgNodeKind)) {
	forEachTextItem(n.Written.Written, fn)
}

func (n *CalledGroup) ForEach(fn func(subNode MlgNodeKind)) {
	forEachTextItem(n.Called.Called, fn)
}

func (n *WritingGroup) ForEach(fn func(subNode MlgNodeKind)) {
	forEachTextItem(n.Writing.Writing, fn)
}

func (n *OverviewGroup) ForEach(fn func(subNode MlgNodeKind)) {
	fn(&n.Overview.Overview)
}

func (n *RelatedGroup) ForEach(fn func(subNode MlgNodeKind)) {
	forEachTextItem(n.Related.Related, fn)
}

func (n *LabelGroup) ForEach(fn func(subNode MlgNodeKind)) {
	fn(&n.Label.Label)
	forEachProofItem(n.By.By, fn)
}

func (n *ByGroup) ForEach(fn func(subNode MlgNodeKind)) {
	forEachProofItem(n.By.By, fn)
}

func (n *DescribesGroup) ForEach(fn func(subNode MlgNodeKind)) {
	fn(&n.Id)
	fn(&n.Describes.Describes)
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
	if n.EquivalentTo != nil {
		forEach(n.EquivalentTo.EquivalentTo, fn)
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

func (n *DefinesGroup) ForEach(fn func(subNode MlgNodeKind)) {
	fn(&n.Id)
	fn(&n.Defines.Defines)
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
		forEach(n.Means.Means, fn)
	}
	if n.EquivalentTo != nil {
		forEach(n.EquivalentTo.EquivalentTo, fn)
	}
	if n.Expresses != nil {
		forEach(n.Expresses.Expresses, fn)
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

func (n *CapturesGroup) ForEach(fn func(subNode MlgNodeKind)) {
	fn(&n.Id)
	forEachFormulation(n.Captures.Captures, fn)
	if n.Justified != nil {
		forEach(n.Justified.Justified, fn)
	}
	if n.Documented != nil {
		forEach(n.Documented.Documented, fn)
	}
	if n.References != nil {
		forEachTextItem(n.References.References, fn)
	}
	if n.MetaId != nil {
		fn(&n.MetaId.Id)
	}
}

func (n *StatesGroup) ForEach(fn func(subNode MlgNodeKind)) {
	fn(&n.Id)
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

func (n *AxiomGroup) ForEach(fn func(subNode MlgNodeKind)) {
	maybeForIdItem(n.Id, fn)
	if n.Given != nil {
		forEachTarget(n.Given.Given, fn)
	}
	if n.Using != nil {
		forEachTarget(n.Using.Using, fn)
	}
	if n.Where != nil {
		forEachSpec(n.Where.Specs, fn)
	}
	if n.SuchThat != nil {
		forEach(n.SuchThat.Clauses, fn)
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
	maybeForIdItem(n.Id, fn)
	if n.Given != nil {
		forEachTarget(n.Given.Given, fn)
	}
	if n.Using != nil {
		forEachTarget(n.Using.Using, fn)
	}
	if n.Where != nil {
		forEachSpec(n.Where.Specs, fn)
	}
	if n.SuchThat != nil {
		forEach(n.SuchThat.Clauses, fn)
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
	maybeForIdItem(n.Id, fn)
	if n.Given != nil {
		forEachTarget(n.Given.Given, fn)
	}
	if n.Using != nil {
		forEachTarget(n.Using.Using, fn)
	}
	if n.Where != nil {
		forEachSpec(n.Where.Specs, fn)
	}
	if n.SuchThat != nil {
		forEach(n.SuchThat.Clauses, fn)
	}
	if n.If != nil {
		forEach(n.If.Clauses, fn)
	}
	if n.Iff != nil {
		forEach(n.Iff.Clauses, fn)
	}
	forEach(n.Then.Clauses, fn)
	if n.Proof != nil {
		forEachProofItem(n.Proof.Proof, fn)
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

func (n *CorollaryGroup) ForEach(fn func(subNode MlgNodeKind)) {
	maybeForIdItem(n.Id, fn)
	forEachTextItem(n.To.To, fn)
	if n.Given != nil {
		forEachTarget(n.Given.Given, fn)
	}
	if n.Using != nil {
		forEachTarget(n.Using.Using, fn)
	}
	if n.Where != nil {
		forEachSpec(n.Where.Specs, fn)
	}
	if n.SuchThat != nil {
		forEach(n.SuchThat.Clauses, fn)
	}
	if n.If != nil {
		forEach(n.If.Clauses, fn)
	}
	if n.Iff != nil {
		forEach(n.Iff.Clauses, fn)
	}
	forEach(n.Then.Clauses, fn)
	if n.Proof != nil {
		forEachProofItem(n.Proof.Proof, fn)
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

func (n *LemmaGroup) ForEach(fn func(subNode MlgNodeKind)) {
	maybeForIdItem(n.Id, fn)
	forEachTextItem(n.For.For, fn)
	if n.Given != nil {
		forEachTarget(n.Given.Given, fn)
	}
	if n.Using != nil {
		forEachTarget(n.Using.Using, fn)
	}
	if n.Where != nil {
		forEachSpec(n.Where.Specs, fn)
	}
	if n.SuchThat != nil {
		forEach(n.SuchThat.Clauses, fn)
	}
	if n.If != nil {
		forEach(n.If.Clauses, fn)
	}
	if n.Iff != nil {
		forEach(n.Iff.Clauses, fn)
	}
	forEach(n.Then.Clauses, fn)
	if n.Proof != nil {
		forEachProofItem(n.Proof.Proof, fn)
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

func (n *ZeroGroup) ForEach(fn func(subNode MlgNodeKind)) {
	fn(n.SingleMeans.Means)
}

func (n *PositiveIntGroup) ForEach(fn func(subNode MlgNodeKind)) {
	fn(n.SingleMeans.Means)
}

func (n *NegativeIntGroup) ForEach(fn func(subNode MlgNodeKind)) {
	fn(n.SingleMeans.Means)
}

func (n *PositiveFloatGroup) ForEach(fn func(subNode MlgNodeKind)) {
	fn(n.SingleMeans.Means)
}

func (n *NegativeFloatGroup) ForEach(fn func(subNode MlgNodeKind)) {
	fn(n.SingleMeans.Means)
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
	for i := range n.Items {
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

func (n *LabeledGrouping) ForEach(fn func(subNode MlgNodeKind)) {
	fn(n.Arg)
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

func (n *AlsoExpression) ForEach(fn func(subNode MlgNodeKind)) {
	forEach(n.Lhs, fn)
	forEach(n.Rhs, fn)
}

func (n *ExtendsExpression) ForEach(fn func(subNode MlgNodeKind)) {
	forEach(n.Lhs, fn)
	forEach(n.Rhs, fn)
}

func (n *AsExpression) ForEach(fn func(subNode MlgNodeKind)) {
	fn(n.Lhs)
	fn(n.Rhs)
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

func (n *SelectFromBuiltinExpression) ForEach(fn func(subNode MlgNodeKind)) {
	fn(&n.Target)
}

func (n *DefinitionBuiltinExpression) ForEach(fn func(subNode MlgNodeKind)) {
	if n.Of != nil {
		fn(n.Of)
	}
	if n.Satisfies != nil {
		fn(n.Satisfies)
	}
}

func (n *MapToElseBuiltinExpression) ForEach(fn func(subNode MlgNodeKind)) {
	fn(&n.Target)
	fn(n.To)
	if n.Else != nil {
		fn(n.Else)
	}
}

func (n *TypeMetaKind) ForEach(fn func(subNode MlgNodeKind)) {
	if n.Types != nil {
		forEachTypeKind(*n.Types, fn)
	}
}

func (n *FormulationMetaKind) ForEach(fn func(subNode MlgNodeKind)) {
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
	forEachExpressionKind(n.Rhs, fn)
}

func (n *EnclosedNonCommandOperatorTarget) ForEach(fn func(subNode MlgNodeKind)) {
	fn(n.Target)
}

func (n *NonEnclosedNonCommandOperatorTarget) ForEach(fn func(subNode MlgNodeKind)) {
	// this doesn't have any sub nodes
}

func (n *InfixCommandExpression) ForEach(fn func(subNode MlgNodeKind)) {
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

func (n *NamedTypeParam) ForEach(fn func(subNode MlgNodeKind)) {
	fn(&n.Name)
	if n.CurlyTypeParam != nil {
		fn(n.CurlyTypeParam)
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

func (n *FunctionLiteralForm) ForEach(fn func(subNode MlgNodeKind)) {
	fn(&n.Lhs)
	fn(n.Rhs)
}

func (n *CurlyParam) ForEach(fn func(subNode MlgNodeKind)) {
	if n.CurlyParams != nil {
		forEach(*n.CurlyParams, fn)
	}
	if n.Direction != nil {
		fn(n.Direction)
	}
}

func (n *CurlyTypeParam) ForEach(fn func(subNode MlgNodeKind)) {
	if n.CurlyTypeParams != nil {
		forEach(*n.CurlyTypeParams, fn)
	}
	if n.TypeDirection != nil {
		fn(n.TypeDirection)
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
	forEach(n.CurlyParams, fn)
}

func (n *DirectionalTypeParam) ForEach(fn func(subNode MlgNodeKind)) {
	if n.Name != nil {
		fn(n.Name)
	}
	forEachDirectionType(n.CurlyTypeParams, fn)
}

func (n *DirectionType) ForEach(fn func(subNode MlgNodeKind)) {
	// a DirectionType does not have any child nodes
}

func (n *ProofThenGroup) ForEach(fn func(subNode MlgNodeKind)) {
	forEachProofItem(n.Then.Then, fn)
	if n.By != nil {
		forEachProofItem(n.By.By, fn)
	}
	if n.Because != nil {
		forEachProofItem(n.Because.Because, fn)
	}
}

func (n *ProofThusGroup) ForEach(fn func(subNode MlgNodeKind)) {
	forEachProofItem(n.Thus.Thus, fn)
	if n.By != nil {
		forEachProofItem(n.By.By, fn)
	}
	if n.Because != nil {
		forEachProofItem(n.Because.Because, fn)
	}
}

func (n *ProofThereforeGroup) ForEach(fn func(subNode MlgNodeKind)) {
	forEachProofItem(n.Therefore.Therefore, fn)
	if n.By != nil {
		forEachProofItem(n.By.By, fn)
	}
	if n.Because != nil {
		forEachProofItem(n.Because.Because, fn)
	}
}

func (n *ProofHenceGroup) ForEach(fn func(subNode MlgNodeKind)) {
	forEachProofItem(n.Hence.Hence, fn)
	if n.By != nil {
		forEachProofItem(n.By.By, fn)
	}
	if n.Because != nil {
		forEachProofItem(n.Because.Because, fn)
	}
}

func (n *ProofNoticeGroup) ForEach(fn func(subNode MlgNodeKind)) {
	forEachProofItem(n.Notice.Notice, fn)
	if n.By != nil {
		forEachProofItem(n.By.By, fn)
	}
	if n.Because != nil {
		forEachProofItem(n.Because.Because, fn)
	}
}

func (n *ProofNextGroup) ForEach(fn func(subNode MlgNodeKind)) {
	forEachProofItem(n.Next.Next, fn)
	if n.By != nil {
		forEachProofItem(n.By.By, fn)
	}
	if n.Because != nil {
		forEachProofItem(n.Because.Because, fn)
	}
}

func (n *ProofByBecauseThenGroup) ForEach(fn func(subNode MlgNodeKind)) {
	if n.By != nil {
		forEachProofItem(n.By.By, fn)
	}
	if n.Because != nil {
		forEachProofItem(n.Because.Because, fn)
	}
	forEachProofItem(n.Then.Then, fn)
}

func (n *ProofBecauseThenGroup) ForEach(fn func(subNode MlgNodeKind)) {
	forEachProofItem(n.Because.Because, fn)
	forEachProofItem(n.Then.Then, fn)
}

func (n *ProofStepwiseGroup) ForEach(fn func(subNode MlgNodeKind)) {
	forEachProofItem(n.Stepwise.Stepwise, fn)
}

func (n *ProofSupposeGroup) ForEach(fn func(subNode MlgNodeKind)) {
	forEachProofItem(n.Suppose.Suppose, fn)
	forEachProofItem(n.Then.Then, fn)
}

func (n *ProofBlockGroup) ForEach(fn func(subNode MlgNodeKind)) {
	forEachProofItem(n.Block.Block, fn)
}

func (n *ProofCasewiseGroup) ForEach(fn func(subNode MlgNodeKind)) {
	for i := range n.Cases {
		forEachProofItem(n.Cases[i].Case, fn)
	}
	if n.Else != nil {
		forEachProofItem(n.Else.Else, fn)
	}
}

func (n *ProofWithoutLossOfGeneralityGroup) ForEach(fn func(subNode MlgNodeKind)) {
	forEachProofItem(n.WithoutLossOfGenerality.Items, fn)
}

func (n *ProofForContradictionGroup) ForEach(fn func(subNode MlgNodeKind)) {
	forEachProofItem(n.ForContradiction.Items, fn)
}

func (n *ProofClaimGroup) ForEach(fn func(subNode MlgNodeKind)) {
	if n.Given != nil {
		forEachTarget(n.Given.Given, fn)
	}
	if n.Using != nil {
		forEachTarget(n.Using.Using, fn)
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
		forEachProofItem(n.Proof.Proof, fn)
	}
}

func (n *CommandTypeForm) ForEach(fn func(subNode MlgNodeKind)) {
	forEachNameForm(n.Names, fn)
	if n.CurlyTypeParam != nil {
		fn(n.CurlyTypeParam)
	}
	if n.NamedTypeParams != nil {
		forEachNamedTypeParam(*n.NamedTypeParams, fn)
	}
	if n.ParenTypeParams != nil {
		forEach(*n.ParenTypeParams, fn)
	}
}

func (n *InfixCommandTypeForm) ForEach(fn func(subNode MlgNodeKind)) {
	forEachNameForm(n.Names, fn)
	if n.CurlyTypeParam != nil {
		fn(n.CurlyTypeParam)
	}
	if n.NamedTypeParams != nil {
		forEachNamedTypeParam(*n.NamedTypeParams, fn)
	}
	if n.ParenTypeParams != nil {
		forEach(*n.ParenTypeParams, fn)
	}
}

func (n *ProofRemarkGroup) ForEach(fn func(subNode MlgNodeKind)) {
	fn(&n.Remark.Remark)
}

func (n *ProofForContrapositiveGroup) ForEach(fn func(subNode MlgNodeKind)) {
	forEachProofItem(n.ForContrapositive.Items, fn)
}

func (n *ProofForInductionGroup) ForEach(fn func(subNode MlgNodeKind)) {
	forEachProofItem(n.ForInduction.Items, fn)
}

func (n *ProofPartwiseGroup) ForEach(fn func(subNode MlgNodeKind)) {
	for _, part := range n.Parts {
		forEachProofItem(part.Part, fn)
	}
}

func (n *ProofSufficesToShowGroup) ForEach(fn func(subNode MlgNodeKind)) {
	forEachProofItem(n.SufficesToShow.Items, fn)
}

func (n *ProofToShowGroup) ForEach(fn func(subNode MlgNodeKind)) {
	forEachProofItem(n.ToShow.Items, fn)
	forEachProofItem(n.Observe.Items, fn)
}

func (n *ProofQedGroup) ForEach(fn func(subNode MlgNodeKind)) {}

func (n *ProofAbsurdGroup) ForEach(fn func(subNode MlgNodeKind)) {}

func (n *ProofDoneGroup) ForEach(fn func(subNode MlgNodeKind)) {}

func (n *ProofContradictionGroup) ForEach(fn func(subNode MlgNodeKind)) {}

////////////////////////////////////////////////////////////////////////////////////////////////////

func maybeForIdItem(id *IdItem, fn func(n MlgNodeKind)) {
	if id != nil {
		fn(id)
	}
}

func forEach[T MlgNodeKind](items []T, fn func(n MlgNodeKind)) {
	for i := range items {
		fn(items[i])
	}
}

func forEachDirectionType(items []DirectionType, fn func(n MlgNodeKind)) {
	for i := range items {
		fn(&items[i])
	}
}

func forEachExpressionKind(items []ExpressionKind, fn func(n MlgNodeKind)) {
	for i := range items {
		fn(items[i])
	}
}

func forEachNameForm(items []NameForm, fn func(n MlgNodeKind)) {
	for i := range items {
		fn(&items[i])
	}
}

func forEachNamedArg(items []NamedArg, fn func(n MlgNodeKind)) {
	for i := range items {
		fn(&items[i])
	}
}

func forEachNamedTypeParam(items []NamedTypeParam, fn func(n MlgNodeKind)) {
	for i := range items {
		fn(&items[i])
	}
}

func forEachNamedParam(items []NamedParam, fn func(n MlgNodeKind)) {
	for i := range items {
		fn(&items[i])
	}
}

func forEachFormulation(items []Formulation[FormulationNodeKind], fn func(n MlgNodeKind)) {
	for i := range items {
		fn(&items[i])
	}
}

func forEachTarget(items []Target, fn func(n MlgNodeKind)) {
	for i := range items {
		fn(&items[i])
	}
}

func forEachTypeKind(types []TypeFormKind, fn func(n MlgNodeKind)) {
	for i := range types {
		fn(types[i])
	}
}

func forEachSpec(items []Spec, fn func(n MlgNodeKind)) {
	for i := range items {
		fn(&items[i])
	}
}

func forEachAlias(items []Alias, fn func(n MlgNodeKind)) {
	for i := range items {
		fn(&items[i])
	}
}

func forEachTextItem(items []TextItem, fn func(n MlgNodeKind)) {
	for i := range items {
		fn(&items[i])
	}
}

func forEachProofItem(items []ProofItemKind, fn func(n MlgNodeKind)) {
	for i := range items {
		fn(items[i])
	}
}

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
	"strings"
)

func StructuralNodeToCode(item StructuralNodeKind) string {
	return strings.Join(item.ToCode(0, false), "\n")
}

func (n *IdItem) ToCode(indent int, hasDot bool) []string {
	if n.Root == nil {
		return []string{n.RawText}
	}
	return buildIndentedLineSlice(indent, hasDot, "["+n.Root.ToCode(NoOp)+"]")
}

func (n *Target) ToCode(indent int, hasDot bool) []string {
	if n.Root == nil {
		return []string{n.RawText}
	}
	return buildIndentedLineSlice(indent, hasDot, n.Root.ToCode(NoOp))
}

func (n *Spec) ToCode(indent int, hasDot bool) []string {
	if n.Root == nil {
		return []string{n.RawText}
	}
	text := "'" + n.Root.ToCode(NoOp) + "'"
	if n.Label != nil {
		text += fmt.Sprintf("     (%s)", *n.Label)
	}
	return buildIndentedLineSlice(indent, hasDot, text)
}

func (n *Alias) ToCode(indent int, hasDot bool) []string {
	if n.Root == nil {
		return []string{n.RawText}
	}
	text := "'" + n.Root.ToCode(NoOp) + "'"
	if n.Label != nil {
		text += fmt.Sprintf("     (%s)", *n.Label)
	}
	return buildIndentedLineSlice(indent, hasDot, text)
}

func (n *Formulation[T]) ToCode(indent int, hasDot bool) []string {
	text := "'" + n.Root.ToCode(NoOp) + "'"
	if n.Label != nil {
		text += fmt.Sprintf("     (%s)", *n.Label)
	}
	return buildIndentedLineSlice(indent, hasDot, text)
}

func (n *TextItem) ToCode(indent int, hasDot bool) []string {
	return buildIndentedLineSlice(indent, hasDot, "\""+n.RawText+"\"")
}

func (n *LetGroup) ToCode(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.MaybeAppendGroupLabel(n.Label, indent, hasDot)
	db.MaybeAppendLetSection(&n.Let, indent, hasDot && n.Label == nil)
	db.MaybeAppendUsingSection(n.Using, indent, false)
	db.MaybeAppendWhereSection(n.Where, indent, false)
	db.MaybeAppendSuchThatSection(n.SuchThat, indent, false)
	db.MaybeAppendThenSection(&n.Then, indent, false)
	return db.Lines()
}

func (n *AllOfGroup) ToCode(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.MaybeAppendGroupLabel(n.Label, indent, hasDot)
	db.AppendClausesSection(LowerAllOfName, n.AllOf.Clauses, indent, hasDot && n.Label == nil)
	return db.Lines()
}

func (n *EquivalentlyGroup) ToCode(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.MaybeAppendGroupLabel(n.Label, indent, hasDot)
	db.AppendClausesSection(LowerEquivalentlyName, n.Equivalently.Clauses, indent,
		hasDot && n.Label == nil)
	return db.Lines()
}

func (n *NotGroup) ToCode(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.MaybeAppendGroupLabel(n.Label, indent, hasDot)
	db.AppendSection(LowerNotName, indent, hasDot && n.Label == nil)
	db.Append(n.Not.Clause, indent+2, true)
	return db.Lines()
}

func (n *AnyOfGroup) ToCode(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.MaybeAppendGroupLabel(n.Label, indent, hasDot)
	db.AppendClausesSection(LowerAnyOfName, n.AnyOf.Clauses, indent, hasDot && n.Label == nil)
	return db.Lines()
}

func (n *OneOfGroup) ToCode(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.MaybeAppendGroupLabel(n.Label, indent, hasDot)
	db.AppendClausesSection(LowerOneOfName, n.OneOf.Clauses, indent, hasDot && n.Label == nil)
	return db.Lines()
}

func (n *ExistsGroup) ToCode(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.MaybeAppendGroupLabel(n.Label, indent, hasDot)
	db.AppendTargetsSection(LowerExistsName, n.Exists.Targets, indent, hasDot && n.Label == nil)
	db.MaybeAppendUsingSection(n.Using, indent, false)
	db.MaybeAppendWhereSection(n.Where, indent, false)
	db.MaybeAppendSuchThatSection(n.SuchThat, indent, false)
	return db.Lines()
}

func (n *ExistsUniqueGroup) ToCode(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.MaybeAppendGroupLabel(n.Label, indent, hasDot)
	db.AppendTargetsSection(
		LowerExistsName, n.ExistsUnique.Targets, indent, hasDot && n.Label == nil)
	db.MaybeAppendUsingSection(n.Using, indent, false)
	db.MaybeAppendWhereSection(n.Where, indent, false)
	db.MaybeAppendSuchThatSection(&n.SuchThat, indent, false)
	return db.Lines()
}

func (n *ForAllGroup) ToCode(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.MaybeAppendGroupLabel(n.Label, indent, hasDot)
	db.AppendTargetsSection(LowerForAllName, n.ForAll.Targets, indent, hasDot && n.Label == nil)
	db.MaybeAppendUsingSection(n.Using, indent, false)
	db.MaybeAppendWhereSection(n.Where, indent, false)
	db.MaybeAppendSuchThatSection(n.SuchThat, indent, false)
	db.MaybeAppendThenSection(&n.Then, indent, false)
	return db.Lines()
}

func (n *IfGroup) ToCode(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.MaybeAppendGroupLabel(n.Label, indent, hasDot)
	db.MaybeAppendIfSection(&n.If, indent, hasDot && n.Label == nil)
	db.MaybeAppendThenSection(&n.Then, indent, false)
	return db.Lines()
}

func (n *IffGroup) ToCode(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.MaybeAppendGroupLabel(n.Label, indent, hasDot)
	db.MaybeAppendIffSection(&n.Iff, indent, hasDot && n.Label == nil)
	db.MaybeAppendThenSection(&n.Then, indent, false)
	return db.Lines()
}

func (n *PiecewiseGroup) ToCode(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.MaybeAppendGroupLabel(n.Label, indent, hasDot)
	db.AppendSection(LowerPiecewiseName, indent, hasDot && n.Label == nil)
	for _, ifThen := range n.IfThen {
		db.MaybeAppendIfSection(&ifThen.If, indent, false)
		db.MaybeAppendThenSection(&ifThen.Then, indent, false)
	}
	if n.Else != nil {
		db.AppendClausesSection(LowerElseName, n.Else.Items, indent, false)
	}
	return db.Lines()
}

func (n *WhenGroup) ToCode(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.MaybeAppendGroupLabel(n.Label, indent, hasDot)
	db.MaybeAppendWhenSection(&n.When, indent, hasDot && n.Label == nil)
	db.MaybeAppendThenSection(&n.Then, indent, false)
	return db.Lines()
}

func (n *SymbolWrittenGroup) ToCode(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.MaybeAppendGroupLabel(n.Label, indent, hasDot)
	db.AppendSection(LowerSymbolName, indent, hasDot && n.Label == nil)
	db.Append(&n.Symbol.Symbol, indent+2, true)
	if n.Written != nil {
		db.AppendTextItemsSection(LowerWrittenName, n.Written.Written, indent, true)
	}
	return db.Lines()
}

func (n *ViewGroup) ToCode(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.MaybeAppendGroupLabel(n.Label, indent, hasDot)
	db.AppendSection(LowerViewName, indent, hasDot && n.Label == nil)
	db.AppendTargetSection(LowerAsName, n.As.As, indent, hasDot)
	db.MaybeAppendUsingSection(n.Using, indent, true)
	db.MaybeAppendWhereSection(n.Where, indent, true)
	db.MaybeAppendThroughSection(n.Through, indent, true)
	if n.Signfies != nil {
		db.AppendSpecsSection(LowerSignifiesName, n.Signfies.Signifies, indent, true)
	}
	return db.Lines()
}

func (n *EncodingGroup) ToCode(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.MaybeAppendGroupLabel(n.Label, indent, hasDot)
	db.AppendSection(LowerEncodingName, indent, hasDot && n.Label == nil)
	db.AppendTargetSection(LowerAsName, n.As.As, indent, hasDot)
	db.MaybeAppendUsingSection(n.Using, indent, true)
	db.MaybeAppendWhereSection(n.Where, indent, true)
	db.MaybeAppendThroughSection(n.Through, indent, true)
	return db.Lines()
}

func (n *WrittenGroup) ToCode(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.MaybeAppendGroupLabel(n.Label, indent, hasDot)
	db.AppendTextItemsSection(
		LowerWrittenName, n.Written.Written, indent, hasDot && n.Label == nil)
	return db.Lines()
}

func (n *CalledGroup) ToCode(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.MaybeAppendGroupLabel(n.Label, indent, hasDot)
	db.AppendTextItemsSection(LowerCalledName, n.Called.Called, indent, hasDot && n.Label == nil)
	return db.Lines()
}

func (n *ExpressedGroup) ToCode(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.MaybeAppendGroupLabel(n.Label, indent, hasDot)
	db.AppendTextItemsSection(
		LowerExpressedName, n.Expressed.Expressed, indent, hasDot && n.Label == nil)
	return db.Lines()
}

func (n *OverviewGroup) ToCode(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.MaybeAppendGroupLabel(n.Label, indent, hasDot)
	db.AppendSingleTextItemSection(
		LowerOverviewName, n.Overview.Overview, indent, hasDot && n.Label == nil)
	return db.Lines()
}

func (n *RelatedGroup) ToCode(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.MaybeAppendGroupLabel(n.Label, indent, hasDot)
	db.AppendTextItemsSection(
		LowerRelatedName, n.Related.Related, indent, hasDot && n.Label == nil)
	return db.Lines()
}

func (n *LabelGroup) ToCode(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.AppendSingleTextItemSection(LowerLabelName, n.Label.Label, indent, hasDot)
	return db.Lines()
}

func (n *ByGroup) ToCode(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.AppendTextItemsSection(LowerByName, n.By.By, indent, hasDot)
	return db.Lines()
}

func (n *DescribesGroup) ToCode(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.MaybeAppendIdItem(&n.Id, indent, hasDot)
	db.AppendSection(UpperDescribesName, indent, false)
	db.Append(&n.Describes.Describes, indent+2, true)
	db.MaybeAppendUsingSection(n.Using, indent, false)
	db.MaybeAppendWhenSection(n.When, indent, false)
	db.MaybeAppendSuchThatSection(n.SuchThat, indent, false)
	if n.Extends != nil {
		db.AppendClausesSection(LowerExtendsName, n.Extends.Extends, indent, false)
	}
	if n.Satisfies != nil {
		db.AppendClausesSection(LowerSatisfiesName, n.Satisfies.Satisfies, indent, false)
	}
	if n.Provides != nil {
		db.AppendSection(UpperProvidesName, indent, false)
		for _, item := range n.Provides.Provides {
			db.Append(item, indent+2, true)
		}
	}
	if n.Justified != nil {
		db.AppendSection(UpperJustifiedName, indent, false)
		for _, item := range n.Justified.Justified {
			db.Append(item, indent+2, true)
		}
	}
	db.MaybeAppendDocumentedSection(n.Documented, indent, false)
	db.MaybeAppendReferencesSection(n.References, indent, false)
	db.MaybeAppendAliasesSection(n.Aliases, indent, false)
	db.MaybeAppendMetaIdSection(n.MetaId, indent, false)
	return db.Lines()
}

func (n *DefinesGroup) ToCode(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.MaybeAppendIdItem(&n.Id, indent, hasDot)
	db.AppendSection(UpperDefinesName, indent, false)
	db.Append(&n.Defines.Defines, indent+2, true)
	db.MaybeAppendUsingSection(n.Using, indent, false)
	db.MaybeAppendWhenSection(n.When, indent, false)
	db.MaybeAppendSuchThatSection(n.SuchThat, indent, false)
	if n.Means != nil {
		db.AppendClausesSection(LowerMeansName, n.Means.Means, indent, false)
	}
	if n.Specifies != nil {
		db.AppendClausesSection(LowerSpecifiesName, n.Specifies.Specifies, indent, false)
	}
	if n.Provides != nil {
		db.AppendSection(UpperProvidesName, indent, false)
		for _, item := range n.Provides.Provides {
			db.Append(item, indent+2, true)
		}
	}
	if n.Justified != nil {
		db.AppendSection(UpperJustifiedName, indent, false)
		for _, item := range n.Justified.Justified {
			db.Append(item, indent+2, true)
		}
	}
	db.MaybeAppendDocumentedSection(n.Documented, indent, false)
	db.MaybeAppendReferencesSection(n.References, indent, false)
	db.MaybeAppendAliasesSection(n.Aliases, indent, false)
	db.MaybeAppendMetaIdSection(n.MetaId, indent, false)
	return db.Lines()
}

func (n *LowerDefineGroup) ToCode(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.AppendSection(LowerDefineName, indent, false)
	db.Append(&n.Define.Define, indent+2, true)
	db.MaybeAppendUsingSection(n.Using, indent, false)
	db.MaybeAppendWhenSection(n.When, indent, false)
	db.MaybeAppendSuchThatSection(n.SuchThat, indent, false)
	if n.Means != nil {
		db.AppendClausesSection(LowerMeansName, n.Means.Means, indent, false)
	}
	db.AppendClausesSection(LowerAsName, n.As.As, indent, false)
	return db.Lines()
}

func (n *CapturesGroup) ToCode(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.MaybeAppendIdItem(&n.Id, indent, hasDot)
	db.AppendFormulationsSection(UpperCapturesName, n.Captures.Captures, indent, false)
	if n.Justified != nil {
		db.AppendSection(UpperJustifiedName, indent, false)
		for _, item := range n.Justified.Justified {
			db.Append(item, indent+2, true)
		}
	}
	db.MaybeAppendDocumentedSection(n.Documented, indent, false)
	db.MaybeAppendReferencesSection(n.References, indent, false)
	db.MaybeAppendMetaIdSection(n.MetaId, indent, false)
	return db.Lines()
}

func (n *StatesGroup) ToCode(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.MaybeAppendIdItem(&n.Id, indent, hasDot)
	db.AppendSection(UpperStatesName, indent, false)
	db.MaybeAppendUsingSection(n.Using, indent, false)
	db.MaybeAppendWhenSection(n.When, indent, false)
	db.MaybeAppendSuchThatSection(n.SuchThat, indent, false)
	db.AppendClausesSection(LowerThatName, n.That.That, indent, false)
	if n.Justified != nil {
		db.AppendSection(UpperJustifiedName, indent, false)
		for _, item := range n.Justified.Justified {
			db.Append(item, indent+2, true)
		}
	}
	db.MaybeAppendDocumentedSection(n.Documented, indent, false)
	db.MaybeAppendReferencesSection(n.References, indent, false)
	db.MaybeAppendAliasesSection(n.Aliases, indent, false)
	db.MaybeAppendMetaIdSection(n.MetaId, indent, false)
	return db.Lines()
}

func (n *ProofGroup) ToCode(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.MaybeAppendIdItem(&n.Id, indent, hasDot)
	db.AppendSection(UpperProofName, indent, false)
	db.AppendSingleTextItemSection(LowerOfName, n.Of.Of, indent, false)
	db.AppendProofItemsSection(LowerContentName, n.Content.Content, indent, false)
	db.MaybeAppendReferencesSection(n.References, indent, false)
	db.MaybeAppendMetaIdSection(n.MetaId, indent, false)
	return db.Lines()
}

func (n *AxiomGroup) ToCode(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.MaybeAppendIdItem(n.Id, indent, hasDot)
	db.AppendSection(UpperAxiomName, indent, hasDot && n.Id == nil)
	db.MaybeAppendGivenSection(n.Given, indent, false)
	db.MaybeAppendUsingSection(n.Using, indent, false)
	db.MaybeAppendWhereSection(n.Where, indent, false)
	db.MaybeAppendIfSection(n.If, indent, false)
	db.MaybeAppendIffSection(n.Iff, indent, false)
	db.MaybeAppendThenSection(&n.Then, indent, false)
	db.MaybeAppendDocumentedSection(n.Documented, indent, false)
	db.MaybeAppendReferencesSection(n.References, indent, false)
	db.MaybeAppendAliasesSection(n.Aliases, indent, false)
	db.MaybeAppendMetaIdSection(n.MetaId, indent, false)
	return db.Lines()
}

func (n *ConjectureGroup) ToCode(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.MaybeAppendIdItem(n.Id, indent, hasDot)
	db.AppendSection(UpperConjectureName, indent, hasDot && n.Id == nil)
	db.MaybeAppendGivenSection(n.Given, indent, false)
	db.MaybeAppendUsingSection(n.Using, indent, false)
	db.MaybeAppendWhereSection(n.Where, indent, false)
	db.MaybeAppendIfSection(n.If, indent, false)
	db.MaybeAppendIffSection(n.Iff, indent, false)
	db.MaybeAppendThenSection(&n.Then, indent, false)
	db.MaybeAppendDocumentedSection(n.Documented, indent, false)
	db.MaybeAppendReferencesSection(n.References, indent, false)
	db.MaybeAppendAliasesSection(n.Aliases, indent, false)
	db.MaybeAppendMetaIdSection(n.MetaId, indent, false)
	return db.Lines()
}

func (n *TheoremGroup) ToCode(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.MaybeAppendIdItem(n.Id, indent, hasDot)
	db.AppendSection(UpperTheoremName, indent, hasDot && n.Id == nil)
	db.MaybeAppendGivenSection(n.Given, indent, false)
	db.MaybeAppendUsingSection(n.Using, indent, false)
	db.MaybeAppendWhereSection(n.Where, indent, false)
	db.MaybeAppendIfSection(n.If, indent, false)
	db.MaybeAppendIffSection(n.Iff, indent, false)
	db.MaybeAppendThenSection(&n.Then, indent, false)
	if n.Proof != nil {
		db.AppendProofItemsSection(UpperProofName, n.Proof.Proof, indent, false)
	}
	db.MaybeAppendDocumentedSection(n.Documented, indent, false)
	db.MaybeAppendReferencesSection(n.References, indent, false)
	db.MaybeAppendAliasesSection(n.Aliases, indent, false)
	db.MaybeAppendMetaIdSection(n.MetaId, indent, false)
	return db.Lines()
}

func (n *CorollaryGroup) ToCode(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.MaybeAppendIdItem(n.Id, indent, hasDot)
	db.AppendSection(UpperCorollaryName, indent, hasDot && n.Id == nil)
	db.AppendTextItemsSection(LowerToName, n.To.To, indent, false)
	db.MaybeAppendGivenSection(n.Given, indent, false)
	db.MaybeAppendUsingSection(n.Using, indent, false)
	db.MaybeAppendWhereSection(n.Where, indent, false)
	db.MaybeAppendIfSection(n.If, indent, false)
	db.MaybeAppendIffSection(n.Iff, indent, false)
	db.MaybeAppendThenSection(&n.Then, indent, false)
	if n.Proof != nil {
		db.AppendProofItemsSection(UpperProofName, n.Proof.Proof, indent, false)
	}
	db.MaybeAppendDocumentedSection(n.Documented, indent, false)
	db.MaybeAppendReferencesSection(n.References, indent, false)
	db.MaybeAppendAliasesSection(n.Aliases, indent, false)
	db.MaybeAppendMetaIdSection(n.MetaId, indent, false)
	return db.Lines()
}

func (n *LemmaGroup) ToCode(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.MaybeAppendIdItem(n.Id, indent, hasDot)
	db.AppendSection(UpperLemmaName, indent, hasDot && n.Id == nil)
	db.AppendTextItemsSection(LowerForName, n.For.For, indent, false)
	db.MaybeAppendGivenSection(n.Given, indent, false)
	db.MaybeAppendUsingSection(n.Using, indent, false)
	db.MaybeAppendWhereSection(n.Where, indent, false)
	db.MaybeAppendIfSection(n.If, indent, false)
	db.MaybeAppendIffSection(n.Iff, indent, false)
	db.MaybeAppendThenSection(&n.Then, indent, false)
	if n.Proof != nil {
		db.AppendProofItemsSection(UpperProofName, n.Proof.Proof, indent, false)
	}
	db.MaybeAppendDocumentedSection(n.Documented, indent, false)
	db.MaybeAppendReferencesSection(n.References, indent, false)
	db.MaybeAppendAliasesSection(n.Aliases, indent, false)
	db.MaybeAppendMetaIdSection(n.MetaId, indent, false)
	return db.Lines()
}

func (n *ZeroGroup) ToCode(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.AppendSection(LowerZeroName, indent, hasDot)
	db.MaybeAppendSingleMeansSection(&n.SingleMeans, indent, false)
	return db.Lines()
}

func (n *PositiveIntGroup) ToCode(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.AppendSection(LowerPositiveIntName, indent, hasDot)
	db.MaybeAppendSingleMeansSection(&n.SingleMeans, indent, false)
	return db.Lines()
}

func (n *NegativeIntGroup) ToCode(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.AppendSection(LowerNegativeIntName, indent, hasDot)
	db.MaybeAppendSingleMeansSection(&n.SingleMeans, indent, false)
	return db.Lines()
}

func (n *PositiveFloatGroup) ToCode(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.AppendSection(LowerPositiveFloatName, indent, hasDot)
	db.MaybeAppendSingleMeansSection(&n.SingleMeans, indent, false)
	return db.Lines()
}

func (n *NegativeFloatGroup) ToCode(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.AppendSection(LowerNegativeFloatName, indent, hasDot)
	db.MaybeAppendSingleMeansSection(&n.SingleMeans, indent, false)
	return db.Lines()
}

func (n *SpecifyGroup) ToCode(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.AppendSection(UpperSpecifyName, indent, hasDot)
	for _, item := range n.Specify.Specify {
		db.Append(item, indent+2, true)
	}
	db.MaybeAppendMetaIdSection(n.MetaId, indent, false)
	return db.Lines()
}

func (n *PersonGroup) ToCode(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.AppendString(n.Id, indent, hasDot)
	db.AppendSection(UpperPersonName, indent, false)
	for _, item := range n.Person.Items {
		db.Append(item, indent+2, true)
	}
	db.MaybeAppendMetaIdSection(n.MetaId, indent, false)
	return db.Lines()
}

func (n *NameGroup) ToCode(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.AppendTextItemsSection(LowerNameName, n.Name.Name, indent, hasDot)
	return db.Lines()
}

func (n *BiographyGroup) ToCode(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.AppendSection(LowerBiographyName, indent, hasDot)
	db.Append(&n.Biography.Biography, indent+2, true)
	return db.Lines()
}

func (n *ResourceGroup) ToCode(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.AppendString(n.Id, indent, hasDot)
	db.AppendSection(UpperResourceName, indent, false)
	for _, item := range n.Resource.Items {
		db.Append(item, indent+2, true)
	}
	db.MaybeAppendMetaIdSection(n.MetaId, indent, false)
	return db.Lines()
}

func (n *TitleGroup) ToCode(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.AppendSection(LowerTitleName, indent, hasDot)
	db.Append(&n.Title.Title, indent+2, true)
	return db.Lines()
}

func (n *AuthorGroup) ToCode(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.AppendTextItemsSection(LowerAuthorName, n.Author.Author, indent, hasDot)
	return db.Lines()
}

func (n *OffsetGroup) ToCode(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.AppendSection(LowerOffsetName, indent, hasDot)
	db.Append(&n.Offset.Offset, indent+2, true)
	return db.Lines()
}

func (n *UrlGroup) ToCode(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.AppendSection(LowerUrlName, indent, hasDot)
	db.Append(&n.Url.Url, indent+2, true)
	return db.Lines()
}

func (n *HomepageGroup) ToCode(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.AppendSection(LowerHomepageName, indent, hasDot)
	db.Append(&n.Homepage.Homepage, indent+2, true)
	return db.Lines()
}

func (n *TypeGroup) ToCode(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.AppendSection(LowerTypeName, indent, hasDot)
	db.Append(&n.Type.Type, indent+2, true)
	return db.Lines()
}

func (n *EditorGroup) ToCode(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.AppendTextItemsSection(LowerEditorName, n.Editor.Editor, indent, hasDot)
	return db.Lines()
}

func (n *EditionGroup) ToCode(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.AppendSection(LowerEditionName, indent, hasDot)
	db.Append(&n.Edition.Edition, indent+2, true)
	return db.Lines()
}

func (n *InstitutionGroup) ToCode(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.AppendTextItemsSection(LowerInstitutionName, n.Institution.Institution, indent, hasDot)
	return db.Lines()
}

func (n *JournalGroup) ToCode(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.AppendTextItemsSection(LowerJournalName, n.Journal.Journal, indent, hasDot)
	return db.Lines()
}

func (n *PublisherGroup) ToCode(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.AppendTextItemsSection(LowerPublisherName, n.Publisher.Publisher, indent, hasDot)
	return db.Lines()
}

func (n *VolumeGroup) ToCode(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.AppendSection(LowerVolumeName, indent, hasDot)
	db.Append(&n.Volume.Volume, indent+2, true)
	return db.Lines()
}

func (n *MonthGroup) ToCode(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.AppendSection(LowerMonthName, indent, hasDot)
	db.Append(&n.Month.Month, indent+2, true)
	return db.Lines()
}

func (n *YearGroup) ToCode(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.AppendSection(LowerYearName, indent, hasDot)
	db.Append(&n.Year.Year, indent+2, true)
	return db.Lines()
}

func (n *DescriptionGroup) ToCode(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.AppendSection(LowerDescriptionName, indent, hasDot)
	db.Append(&n.Description.Description, indent+2, true)
	return db.Lines()
}

func (n *Document) ToCode(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	for _, item := range n.Items {
		db.Append(item, indent, hasDot)
		db.AppendString("", indent, false)
		db.AppendString("", indent, false)
	}
	return db.Lines()
}

func (n *TextBlockItem) ToCode(indent int, hasDot bool) []string {
	return buildIndentedLineSlice(indent, hasDot, "::"+n.Text+"::")
}

func (n *ProofThenGroup) ToCode(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.MaybeAppendGroupLabel(n.Label, indent, hasDot)
	db.AppendProofItemsSection(LowerThenName, n.Then.Then, indent, hasDot && n.Label == nil)
	return db.Lines()
}

func (n *ProofThusGroup) ToCode(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.MaybeAppendGroupLabel(n.Label, indent, hasDot)
	db.AppendProofItemsSection(LowerThusName, n.Thus.Thus, indent, hasDot && n.Label == nil)
	return db.Lines()
}

func (n *ProofThereforeGroup) ToCode(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.MaybeAppendGroupLabel(n.Label, indent, hasDot)
	db.AppendProofItemsSection(LowerThereforeName, n.Therefore.Therefore,
		indent, hasDot && n.Label == nil)
	return db.Lines()
}

func (n *ProofHenceGroup) ToCode(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.MaybeAppendGroupLabel(n.Label, indent, hasDot)
	db.AppendProofItemsSection(LowerHenceName, n.Hence.Hence, indent, hasDot && n.Label == nil)
	return db.Lines()
}

func (n *ProofNoticeGroup) ToCode(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.MaybeAppendGroupLabel(n.Label, indent, hasDot)
	db.AppendProofItemsSection(LowerNoticeName, n.Notice.Notice, indent, hasDot && n.Label == nil)
	return db.Lines()
}

func (n *ProofNextGroup) ToCode(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.MaybeAppendGroupLabel(n.Label, indent, hasDot)
	db.AppendProofItemsSection(LowerNextName, n.Next.Next, indent, hasDot && n.Label == nil)
	return db.Lines()
}

func (n *ProofThenByGroup) ToCode(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.MaybeAppendGroupLabel(n.Label, indent, hasDot)
	db.AppendProofItemsSection(LowerThenName, n.Then.Then, indent, hasDot && n.Label == nil)
	db.AppendTextItemsSection(LowerByName, n.By.Items, indent, false)
	return db.Lines()
}

func (n *ProofThusByGroup) ToCode(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.MaybeAppendGroupLabel(n.Label, indent, hasDot)
	db.AppendProofItemsSection(LowerThusName, n.Thus.Thus, indent, hasDot && n.Label == nil)
	db.AppendTextItemsSection(LowerByName, n.By.Items, indent, false)
	return db.Lines()
}

func (n *ProofThereforeByGroup) ToCode(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.MaybeAppendGroupLabel(n.Label, indent, hasDot)
	db.AppendProofItemsSection(LowerThereforeName, n.Therefore.Therefore,
		indent, hasDot && n.Label == nil)
	db.AppendTextItemsSection(LowerByName, n.By.Items, indent, false)
	return db.Lines()
}

func (n *ProofHenceByGroup) ToCode(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.MaybeAppendGroupLabel(n.Label, indent, hasDot)
	db.AppendProofItemsSection(LowerHenceName, n.Hence.Hence, indent, hasDot && n.Label == nil)
	db.AppendTextItemsSection(LowerByName, n.By.Items, indent, false)
	return db.Lines()
}

func (n *ProofNoticeByGroup) ToCode(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.MaybeAppendGroupLabel(n.Label, indent, hasDot)
	db.AppendProofItemsSection(LowerNoticeName, n.Notice.Notice, indent, hasDot && n.Label == nil)
	db.AppendTextItemsSection(LowerByName, n.By.Items, indent, false)
	return db.Lines()
}

func (n *ProofNextByGroup) ToCode(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.MaybeAppendGroupLabel(n.Label, indent, hasDot)
	db.AppendProofItemsSection(LowerNextName, n.Next.Next, indent, hasDot && n.Label == nil)
	db.AppendTextItemsSection(LowerByName, n.By.Items, indent, false)
	return db.Lines()
}

func (n *ProofThenBecauseGroup) ToCode(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.MaybeAppendGroupLabel(n.Label, indent, hasDot)
	db.AppendProofItemsSection(LowerThenName, n.Then.Then, indent, hasDot && n.Label == nil)
	db.AppendProofItemsSection(LowerBecauseName, n.Because.Because, indent, false)
	return db.Lines()
}

func (n *ProofThusBecauseGroup) ToCode(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.MaybeAppendGroupLabel(n.Label, indent, hasDot)
	db.AppendProofItemsSection(LowerThusName, n.Thus.Thus, indent, hasDot && n.Label == nil)
	db.AppendProofItemsSection(LowerBecauseName, n.Because.Because, indent, false)
	return db.Lines()
}

func (n *ProofThereforeBecauseGroup) ToCode(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.MaybeAppendGroupLabel(n.Label, indent, hasDot)
	db.AppendProofItemsSection(LowerThereforeName, n.Therefore.Therefore,
		indent, hasDot && n.Label == nil)
	db.AppendProofItemsSection(LowerBecauseName, n.Because.Because, indent, false)
	return db.Lines()
}

func (n *ProofHenceBecauseGroup) ToCode(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.MaybeAppendGroupLabel(n.Label, indent, hasDot)
	db.AppendProofItemsSection(LowerHenceName, n.Hence.Hence, indent, hasDot && n.Label == nil)
	db.AppendProofItemsSection(LowerBecauseName, n.Because.Because, indent, false)
	return db.Lines()
}

func (n *ProofNoticeBecauseGroup) ToCode(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.MaybeAppendGroupLabel(n.Label, indent, hasDot)
	db.AppendProofItemsSection(LowerNoticeName, n.Notice.Notice, indent, hasDot && n.Label == nil)
	db.AppendProofItemsSection(LowerBecauseName, n.Because.Because, indent, false)
	return db.Lines()
}

func (n *ProofNextBecauseGroup) ToCode(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.MaybeAppendGroupLabel(n.Label, indent, hasDot)
	db.AppendProofItemsSection(LowerNextName, n.Next.Next, indent, hasDot && n.Label == nil)
	db.AppendProofItemsSection(LowerBecauseName, n.Because.Because, indent, false)
	return db.Lines()
}

func (n *ProofByThenGroup) ToCode(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.MaybeAppendGroupLabel(n.Label, indent, hasDot)
	db.AppendTextItemsSection(LowerByName, n.By.Items, indent, hasDot && n.Label == nil)
	db.AppendProofItemsSection(LowerThenName, n.Then.Then, indent, false)
	return db.Lines()
}

func (n *ProofBecauseThenGroup) ToCode(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.MaybeAppendGroupLabel(n.Label, indent, hasDot)
	db.AppendProofItemsSection(LowerBecauseName, n.Because.Because, indent, hasDot && n.Label == nil)
	db.AppendProofItemsSection(LowerThenName, n.Then.Then, indent, false)
	return db.Lines()
}

func (n *ProofIndependentlyGroup) ToCode(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.MaybeAppendGroupLabel(n.Label, indent, hasDot)
	db.AppendProofItemsSection(
		LowerIndependentlyName, n.Independently.Independently, indent, hasDot && n.Label == nil)
	return db.Lines()
}

func (n *ProofStepwiseGroup) ToCode(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.MaybeAppendGroupLabel(n.Label, indent, hasDot)
	db.AppendProofItemsSection(
		LowerStepwiseName, n.Stepwise.Stepwise, indent, hasDot && n.Label == nil)
	return db.Lines()
}

func (n *ProofSupposeGroup) ToCode(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.MaybeAppendGroupLabel(n.Label, indent, hasDot)
	db.AppendProofItemsSection(LowerSupposeName, n.Suppose.Suppose, indent, hasDot && n.Label == nil)
	db.AppendProofItemsSection(LowerThenName, n.Then.Then, indent, false)
	return db.Lines()
}

func (n *ProofBlockGroup) ToCode(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.MaybeAppendGroupLabel(n.Label, indent, hasDot)
	db.AppendProofItemsSection(LowerBlockName, n.Block.Block, indent, hasDot && n.Label == nil)
	return db.Lines()
}

func (n *ProofCasewiseGroup) ToCode(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.MaybeAppendGroupLabel(n.Label, indent, hasDot)
	db.AppendSection(LowerCasewiseName, indent, hasDot && n.Label == nil)
	for _, caseSection := range n.Cases {
		db.AppendProofItemsSection(LowerCaseName, caseSection.Case, indent, false)
	}
	return db.Lines()
}

func (n *ProofWithoutLossOfGeneralityGroup) ToCode(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.MaybeAppendGroupLabel(n.Label, indent, hasDot)
	db.AppendProofItemsSection(
		LowerWithoutLossOfGeneralityName, n.WithoutLossOfGenerality.Items,
		indent, hasDot && n.Label == nil)
	return db.Lines()
}

func (n *ProofContradictingGroup) ToCode(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.MaybeAppendGroupLabel(n.Label, indent, hasDot)
	db.AppendTextItemsSection(LowerContradictingName, n.Contradicting.Contradicting, indent, hasDot)
	return db.Lines()
}

func (n *ProofForContradictionGroup) ToCode(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.MaybeAppendGroupLabel(n.Label, indent, hasDot)
	db.AppendSection(LowerForContradictionName, indent, hasDot && n.Label == nil)
	db.AppendProofItemsSection(LowerSupposeName, n.Suppose.Suppose, indent, false)
	db.AppendProofItemsSection(LowerThenName, n.Then.Then, indent, false)
	return db.Lines()
}

func (n *ProofForInductionGroup) ToCode(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.MaybeAppendGroupLabel(n.Label, indent, hasDot)
	db.AppendSection(LowerForInductionName, indent, hasDot && n.Label == nil)
	db.AppendProofItemsSection(LowerBaseCaseName, n.BaseCase.BaseCase, indent, false)
	db.AppendProofItemsSection(LowerGenerallyName, n.Generally.Generally, indent, false)
	return db.Lines()
}

func (n *ProofClaimGroup) ToCode(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.MaybeAppendGroupLabel(n.Label, indent, hasDot)
	db.AppendSection(LowerClaimName, indent, hasDot && n.Label == nil)
	db.MaybeAppendGivenSection(n.Given, indent, false)
	db.MaybeAppendUsingSection(n.Using, indent, false)
	db.MaybeAppendWhereSection(n.Where, indent, false)
	db.MaybeAppendIfSection(n.If, indent, false)
	db.MaybeAppendIffSection(n.Iff, indent, false)
	db.MaybeAppendThenSection(&n.Then, indent, false)
	if n.Proof != nil {
		db.AppendProofItemsSection(UpperProofName, n.Proof.Proof, indent, false)
	}
	return db.Lines()
}

func (n *ProofLetGroup) ToCode(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.MaybeAppendGroupLabel(n.Label, indent, hasDot)
	db.MaybeAppendLetSection(&n.Let, indent, hasDot && n.Label == nil)
	db.MaybeAppendUsingSection(n.Using, indent, false)
	db.MaybeAppendWhereSection(n.Where, indent, false)
	db.MaybeAppendProofSuchThatSection(n.SuchThat, indent, false)
	db.MaybeAppendProofThenSection(&n.Then, indent, false)
	return db.Lines()
}

func (n *ProofAllOfGroup) ToCode(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.MaybeAppendGroupLabel(n.Label, indent, hasDot)
	db.AppendProofItemsSection(LowerAllOfName, n.AllOf.Items, indent, hasDot && n.Label == nil)
	return db.Lines()
}

func (n *ProofEquivalentlyGroup) ToCode(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.MaybeAppendGroupLabel(n.Label, indent, hasDot)
	db.AppendProofItemsSection(LowerEquivalentlyName, n.Equivalently.Items, indent,
		hasDot && n.Label == nil)
	return db.Lines()
}

func (n *ProofNotGroup) ToCode(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.MaybeAppendGroupLabel(n.Label, indent, hasDot)
	db.AppendSection(LowerNotName, indent, hasDot && n.Label == nil)
	db.Append(n.Not.Item, indent+2, true)
	return db.Lines()
}

func (n *ProofAnyOfGroup) ToCode(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.MaybeAppendGroupLabel(n.Label, indent, hasDot)
	db.AppendProofItemsSection(LowerAnyOfName, n.AnyOf.Items, indent, hasDot && n.Label == nil)
	return db.Lines()
}

func (n *ProofOneOfGroup) ToCode(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.MaybeAppendGroupLabel(n.Label, indent, hasDot)
	db.AppendProofItemsSection(LowerOneOfName, n.OneOf.Items, indent, hasDot && n.Label == nil)
	return db.Lines()
}

func (n *ProofExistsGroup) ToCode(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.MaybeAppendGroupLabel(n.Label, indent, hasDot)
	db.AppendTargetsSection(LowerExistsName, n.Exists.Targets, indent, hasDot && n.Label == nil)
	db.MaybeAppendUsingSection(n.Using, indent, false)
	db.MaybeAppendWhereSection(n.Where, indent, false)
	db.MaybeAppendProofSuchThatSection(n.SuchThat, indent, false)
	return db.Lines()
}

func (n *ProofExistsUniqueGroup) ToCode(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.MaybeAppendGroupLabel(n.Label, indent, hasDot)
	db.AppendTargetsSection(
		LowerExistsName, n.ExistsUnique.Targets, indent, hasDot && n.Label == nil)
	db.MaybeAppendUsingSection(n.Using, indent, false)
	db.MaybeAppendWhereSection(n.Where, indent, false)
	db.MaybeAppendProofSuchThatSection(&n.SuchThat, indent, false)
	return db.Lines()
}

func (n *ProofForAllGroup) ToCode(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.MaybeAppendGroupLabel(n.Label, indent, hasDot)
	db.AppendTargetsSection(LowerForAllName, n.ForAll.Targets, indent, hasDot && n.Label == nil)
	db.MaybeAppendUsingSection(n.Using, indent, false)
	db.MaybeAppendWhereSection(n.Where, indent, false)
	db.MaybeAppendProofSuchThatSection(n.SuchThat, indent, false)
	db.MaybeAppendProofThenSection(&n.Then, indent, false)
	return db.Lines()
}

func (n *ProofIfGroup) ToCode(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.MaybeAppendGroupLabel(n.Label, indent, hasDot)
	db.MaybeAppendProofIfSection(&n.If, indent, hasDot && n.Label == nil)
	db.MaybeAppendProofThenSection(&n.Then, indent, false)
	return db.Lines()
}

func (n *ProofIffGroup) ToCode(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.MaybeAppendGroupLabel(n.Label, indent, hasDot)
	db.MaybeAppendProofIffSection(&n.Iff, indent, hasDot && n.Label == nil)
	db.MaybeAppendProofThenSection(&n.Then, indent, false)
	return db.Lines()
}

////////////////////////////////////////////////////////////////////////////////////////////////////

// Note: This file contains methods that look similar because a non-generic struct in Go cannot
//       have a generic method and slices in Go do not support variances (in particular, they are
//       not covariant).  Thus, a specialized version of each method is needed for each type.

type debugBuilder struct {
	lines []string
}

func newDebugBuilder() debugBuilder {
	return debugBuilder{
		lines: make([]string, 0),
	}
}

func (db *debugBuilder) Lines() []string {
	return db.lines
}

func (db *debugBuilder) AppendString(str string, indent int, hasDot bool) {
	db.lines = append(db.lines, buildIndent(indent, hasDot)+str)
}

func (db *debugBuilder) Append(node StructuralNodeKind, indent int, hasDot bool) {
	db.lines = append(db.lines, node.ToCode(indent, hasDot)...)
}

func (db *debugBuilder) AppendSection(name string, indent int, hasDot bool) {
	db.lines = append(db.lines, buildIndentedLine(indent, hasDot, name+":"))
}

func (db *debugBuilder) AppendTargets(targets []Target, indent int, hasDot bool) {
	for _, target := range targets {
		db.Append(&target, indent, hasDot)
	}
}

func (db *debugBuilder) AppendSpecs(specs []Spec, indent int, hasDot bool) {
	for _, spec := range specs {
		db.Append(&spec, indent, hasDot)
	}
}

func (db *debugBuilder) AppendClauses(clauses []ClauseKind, indent int, hasDot bool) {
	for _, clause := range clauses {
		db.Append(clause, indent, hasDot)
	}
}

func (db *debugBuilder) AppendProofItems(proofItems []ProofItemKind, indent int, hasDot bool) {
	for _, item := range proofItems {
		db.Append(item, indent, hasDot)
	}
}

func (db *debugBuilder) AppendTextItems(items []TextItem, indent int, hasDot bool) {
	for _, item := range items {
		db.Append(&item, indent, hasDot)
	}
}

func (db *debugBuilder) AppendFormulations(
	items []Formulation[FormulationNodeKind], indent int, hasDot bool,
) {
	for _, item := range items {
		db.Append(&item, indent, hasDot)
	}
}

func (db *debugBuilder) AppendFormulationsSection(
	name string, items []Formulation[FormulationNodeKind], indent int, hasDot bool,
) {
	db.AppendSection(name, indent, hasDot)
	db.AppendFormulations(items, indent+2, true)
}

func (db *debugBuilder) AppendSpecsSection(name string, specs []Spec, indent int, hasDot bool) {
	db.AppendSection(name, indent, hasDot)
	db.AppendSpecs(specs, indent+2, true)
}

func (db *debugBuilder) AppendClausesSection(
	name string, clauses []ClauseKind, indent int, hasDot bool) {
	db.AppendSection(name, indent, hasDot)
	db.AppendClauses(clauses, indent+2, true)
}

func (db *debugBuilder) AppendProofItemsSection(
	name string, proofItems []ProofItemKind, indent int, hasDot bool) {
	db.AppendSection(name, indent, hasDot)
	db.AppendProofItems(proofItems, indent+2, true)
}

func (db *debugBuilder) AppendTargetsSection(
	name string, targets []Target, indent int, hasDot bool) {
	db.AppendSection(name, indent, hasDot)
	db.AppendTargets(targets, indent+2, true)
}

func (db *debugBuilder) AppendTargetSection(
	name string, target Target, indent int, hasDot bool) {
	db.AppendSection(name, indent, hasDot)
	db.Append(&target, indent+2, true)
}

func (db *debugBuilder) AppendSingleTextItemSection(
	name string, item TextItem, indent int, hasDot bool) {
	db.AppendSection(name, indent, hasDot)
	db.Append(&item, indent+2, true)
}

func (db *debugBuilder) AppendTextItemsSection(
	name string, items []TextItem, indent int, hasDot bool) {
	db.AppendSection(name, indent, hasDot)
	db.AppendTextItems(items, indent+2, true)
}

func (db *debugBuilder) MaybeAppendIdItem(item *IdItem, indent int, hasDot bool) {
	if item != nil {
		db.Append(item, indent, hasDot)
	}
}

func (db *debugBuilder) MaybeAppendMetaIdSection(sec *MetaIdSection, indent int, hasDot bool) {
	if sec != nil {
		db.AppendSection(UpperIdName, indent, hasDot)
		db.Append(&sec.Id, indent+2, true)
	}
}

func (db *debugBuilder) MaybeAppendDocumentedSection(
	sec *DocumentedSection, indent int, hasDot bool) {
	if sec != nil {
		db.AppendSection(UpperDocumentedName, indent, hasDot)
		for _, item := range sec.Documented {
			db.Append(item, indent+2, true)
		}
	}
}

func (db *debugBuilder) MaybeAppendReferencesSection(
	sec *ReferencesSection, indent int, hasDot bool) {
	if sec != nil {
		db.AppendSection(UpperReferencesName, indent, hasDot)
		for _, item := range sec.References {
			db.Append(&item, indent+2, true)
		}
	}
}

func (db *debugBuilder) MaybeAppendAliasesSection(sec *AliasesSection, indent int, hasDot bool) {
	if sec != nil {
		db.AppendSection(UpperAliasesName, indent, hasDot)
		for _, item := range sec.Aliases {
			db.Append(&item, indent+2, true)
		}
	}
}

func (db *debugBuilder) MaybeAppendLetSection(sec *LetSection, indent int, hasDot bool) {
	if sec != nil {
		db.AppendSection(LowerLetName, indent, hasDot)
		for _, item := range sec.Let {
			db.Append(&item, indent+2, true)
		}
	}
}

func (db *debugBuilder) MaybeAppendGivenSection(sec *GivenSection, indent int, hasDot bool) {
	if sec != nil {
		db.AppendSection(LowerGivenName, indent, hasDot)
		for _, item := range sec.Given {
			db.Append(&item, indent+2, true)
		}
	}
}

func (db *debugBuilder) MaybeAppendThroughSection(
	sec *ThroughSection, indent int, hasDot bool,
) {
	if sec != nil {
		db.AppendSection(LowerThroughName, indent, hasDot)
		for _, item := range sec.Through {
			db.Append(&item, indent+2, true)
		}
	}
}

func (db *debugBuilder) MaybeAppendWhereSection(sec *WhereSection, indent int, hasDot bool) {
	if sec != nil {
		db.AppendSection(LowerWhereName, indent, hasDot)
		for _, item := range sec.Specs {
			db.Append(&item, indent+2, true)
		}
	}
}

func (db *debugBuilder) MaybeAppendWhenSection(sec *WhenSection, indent int, hasDot bool) {
	if sec != nil {
		db.AppendSection(LowerWhenName, indent, hasDot)
		for _, item := range sec.When {
			db.Append(item, indent+2, true)
		}
	}
}

func (db *debugBuilder) MaybeAppendSuchThatSection(sec *SuchThatSection, indent int, hasDot bool) {
	if sec != nil {
		db.AppendSection(LowerSuchThatName, indent, hasDot)
		for _, item := range sec.Clauses {
			db.Append(item, indent+2, true)
		}
	}
}

func (db *debugBuilder) MaybeAppendProofSuchThatSection(
	sec *ProofSuchThatSection, indent int, hasDot bool,
) {
	if sec != nil {
		db.AppendSection(LowerSuchThatName, indent, hasDot)
		for _, item := range sec.Items {
			db.Append(item, indent+2, true)
		}
	}
}

func (db *debugBuilder) MaybeAppendThenSection(sec *ThenSection, indent int, hasDot bool) {
	if sec != nil {
		db.AppendSection(LowerThenName, indent, hasDot)
		for _, item := range sec.Clauses {
			db.Append(item, indent+2, true)
		}
	}
}

func (db *debugBuilder) MaybeAppendProofThenSection(
	sec *ProofThenSection, indent int, hasDot bool,
) {
	if sec != nil {
		db.AppendSection(LowerThenName, indent, hasDot)
		for _, item := range sec.Then {
			db.Append(item, indent+2, true)
		}
	}
}

func (db *debugBuilder) MaybeAppendUsingSection(sec *UsingSection, indent int, hasDot bool) {
	if sec != nil {
		db.AppendSection(LowerUsingName, indent, hasDot)
		for _, item := range sec.Using {
			db.Append(&item, indent+2, true)
		}
	}
}

func (db *debugBuilder) MaybeAppendIfSection(sec *IfSection, indent int, hasDot bool) {
	if sec != nil {
		db.AppendSection(LowerIfName, indent, hasDot)
		for _, item := range sec.Clauses {
			db.Append(item, indent+2, true)
		}
	}
}

func (db *debugBuilder) MaybeAppendProofIfSection(sec *ProofIfSection, indent int, hasDot bool) {
	if sec != nil {
		db.AppendSection(LowerIfName, indent, hasDot)
		for _, item := range sec.Items {
			db.Append(item, indent+2, true)
		}
	}
}

func (db *debugBuilder) MaybeAppendIffSection(sec *IffSection, indent int, hasDot bool) {
	if sec != nil {
		db.AppendSection(LowerIfName, indent, hasDot)
		for _, item := range sec.Clauses {
			db.Append(item, indent+2, true)
		}
	}
}

func (db *debugBuilder) MaybeAppendProofIffSection(sec *ProofIffSection, indent int, hasDot bool) {
	if sec != nil {
		db.AppendSection(LowerIfName, indent, hasDot)
		for _, item := range sec.Items {
			db.Append(item, indent+2, true)
		}
	}
}

func (db *debugBuilder) MaybeAppendSingleMeansSection(
	sec *SingleMeansSection, indent int, hasDot bool,
) {
	if sec != nil {
		db.AppendSection(LowerMeansName, indent, hasDot)
		db.Append(sec.Means, indent+2, true)
	}
}

func (db *debugBuilder) MaybeAppendGroupLabel(label *GroupLabel, indent int, hasDot bool) {
	if label != nil {
		db.AppendString(fmt.Sprintf("[%s]", label.Label), indent, hasDot)
	}
}

////////////////////////////////////////////////////////////////////////////////////////////////////

func buildIndent(indent int, hasDot bool) string {
	if hasDot {
		return strings.Repeat(" ", indent-2) + ". "
	} else {
		return strings.Repeat(" ", indent)
	}
}

func buildIndentedLine(indent int, hasDot bool, text string) string {
	return buildIndent(indent, hasDot) + text
}

func buildIndentedLineSlice(indent int, hasDot bool, text string) []string {
	return []string{
		buildIndentedLine(indent, hasDot, text),
	}
}

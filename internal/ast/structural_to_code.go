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
	"strings"
)

type IStructuralToCode interface {
	Debug(indent int, hasDot bool) []string
}

func StructuralNodeToCode(item IStructuralToCode) string {
	return strings.Join(item.Debug(0, false), "\n")
}

func (n IdItem) Debug(indent int, hasDot bool) []string {
	if n.Root == nil {
		return []string{n.RawText}
	}
	return buildIndentedLineSlice(indent, hasDot, "["+n.Root.ToCode()+"]")
}

func (n Target) Debug(indent int, hasDot bool) []string {
	if n.Root == nil {
		return []string{n.RawText}
	}
	return buildIndentedLineSlice(indent, hasDot, n.Root.ToCode())
}

func (n Spec) Debug(indent int, hasDot bool) []string {
	if n.Root == nil {
		return []string{n.RawText}
	}
	return buildIndentedLineSlice(indent, hasDot, "'"+n.Root.ToCode()+"'")
}

func (n Alias) Debug(indent int, hasDot bool) []string {
	if n.Root == nil {
		return []string{n.RawText}
	}
	return buildIndentedLineSlice(indent, hasDot, "'"+n.Root.ToCode()+"'")
}

func (n Formulation[T]) Debug(indent int, hasDot bool) []string {
	return buildIndentedLineSlice(indent, hasDot, "'"+n.Root.ToCode()+"'")
}

func (n TextItem) Debug(indent int, hasDot bool) []string {
	return buildIndentedLineSlice(indent, hasDot, "\""+n.RawText+"\"")
}

func (n GivenGroup) Debug(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.MaybeAppendGivenSection(&n.Given, indent, hasDot)
	db.MaybeAppendWhereSection(n.Where, indent, false)
	db.MaybeAppendSuchThatSection(n.SuchThat, indent, false)
	db.MaybeAppendThenSection(&n.Then, indent, false)
	return db.Lines()
}

func (n AllOfGroup) Debug(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.AppendClausesSection(LowerAllOfName, n.AllOf.Clauses, indent, hasDot)
	return db.Lines()
}

func (n NotGroup) Debug(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.AppendSection(LowerNotName, indent, hasDot)
	db.Append(n.Not.Clause, indent+2, true)
	return db.Lines()
}

func (n AnyOfGroup) Debug(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.AppendClausesSection(LowerAnyOfName, n.AnyOf.Clauses, indent, hasDot)
	return db.Lines()
}

func (n OneOfGroup) Debug(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.AppendClausesSection(LowerOneOfName, n.OneOf.Clauses, indent, hasDot)
	return db.Lines()
}

func (n ExistsGroup) Debug(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.AppendTargetsSection(LowerExistsName, n.Exists.Targets, indent, hasDot)
	db.MaybeAppendWhereSection(n.Where, indent, false)
	db.MaybeAppendSuchThatSection(n.SuchThat, indent, false)
	return db.Lines()
}

func (n ExistsUniqueGroup) Debug(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.AppendTargetsSection(LowerExistsName, n.ExistsUnique.Targets, indent, hasDot)
	db.MaybeAppendWhereSection(n.Where, indent, false)
	db.MaybeAppendSuchThatSection(&n.SuchThat, indent, false)
	return db.Lines()
}

func (n ForAllGroup) Debug(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.AppendTargetsSection(LowerForAllName, n.ForAll.Targets, indent, hasDot)
	db.MaybeAppendWhereSection(n.Where, indent, false)
	db.MaybeAppendSuchThatSection(n.SuchThat, indent, false)
	db.MaybeAppendThenSection(&n.Then, indent, false)
	return db.Lines()
}

func (n IfGroup) Debug(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.MaybeAppendIfSection(&n.If, indent, hasDot)
	db.MaybeAppendThenSection(&n.Then, indent, false)
	return db.Lines()
}

func (n IffGroup) Debug(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.MaybeAppendIffSection(&n.Iff, indent, hasDot)
	db.MaybeAppendThenSection(&n.Then, indent, false)
	return db.Lines()
}

func (n PiecewiseGroup) Debug(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.AppendSection(LowerPiecewiseName, indent, hasDot)
	for _, ifThen := range n.IfThen {
		db.MaybeAppendIfSection(&ifThen.If, indent, false)
		db.MaybeAppendThenSection(&ifThen.Then, indent, false)
	}
	if n.Else != nil {
		db.AppendClausesSection(LowerElseName, n.Else.Items, indent, false)
	}
	return db.Lines()
}

func (n WhenGroup) Debug(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.MaybeAppendWhenSection(&n.When, indent, hasDot)
	db.MaybeAppendThenSection(&n.Then, indent, false)
	return db.Lines()
}

func (n SymbolWrittenGroup) Debug(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.AppendSection(LowerSymbolName, indent, hasDot)
	db.Append(&n.Symbol.Symbol, indent+2, true)
	if n.Written != nil {
		db.AppendTextItemsSection(LowerWrittenName, n.Written.Written, indent, true)
	}
	return db.Lines()
}

func (n ConnectionGroup) Debug(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.AppendSection(LowerConnectionName, indent, hasDot)
	db.MaybeAppendUsingSection(n.Using, indent, true)
	return db.Lines()
}

func (n WrittenGroup) Debug(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.AppendTextItemsSection(LowerWrittenName, n.Written.Written, indent, hasDot)
	return db.Lines()
}

func (n CalledGroup) Debug(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.AppendTextItemsSection(LowerCalledName, n.Called.Called, indent, hasDot)
	return db.Lines()
}

func (n WritingGroup) Debug(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.AppendSection(LowerWritingName, indent, hasDot)
	db.Append(&n.Writing.Writing, indent, false)
	db.AppendTextItemsSection(LowerAsName, n.As.As, indent, false)
	return db.Lines()
}

func (n OverviewGroup) Debug(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.AppendSingleTextItemSection(LowerOverviewName, n.Overview.Overview, indent, hasDot)
	return db.Lines()
}

func (n MotivationGroup) Debug(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.AppendSingleTextItemSection(LowerMotivationName, n.Motivation.Motivation, indent, hasDot)
	return db.Lines()
}

func (n HistoryGroup) Debug(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.AppendSingleTextItemSection(LowerHistoryName, n.History.History, indent, hasDot)
	return db.Lines()
}

func (n ExampleGroup) Debug(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.AppendTextItemsSection(LowerExampleName, n.Examples.Examples, indent, hasDot)
	return db.Lines()
}

func (n RelatedGroup) Debug(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.AppendTextItemsSection(LowerRelatedName, n.Related.Related, indent, hasDot)
	return db.Lines()
}

func (n DiscovererGroup) Debug(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.AppendTextItemsSection(LowerDiscovererName, n.Discoverer.Discoverer, indent, hasDot)
	return db.Lines()
}

func (n NoteGroup) Debug(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.AppendSection(LowerNoteName, indent, hasDot)
	for _, item := range n.Note.Note {
		db.Append(item, indent+2, true)
	}
	return db.Lines()
}

func (n DescribingGroup) Debug(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.AppendSingleTextItemSection(LowerDescribingName, n.Describing.Describing, indent, hasDot)
	db.AppendSingleTextItemSection(LowerContentName, n.Content.Content, indent, hasDot)
	return db.Lines()
}

func (n LabelGroup) Debug(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.AppendSingleTextItemSection(LowerLabelName, n.Label.Label, indent, hasDot)
	return db.Lines()
}

func (n ByGroup) Debug(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.AppendTextItemsSection(LowerByName, n.By.By, indent, hasDot)
	return db.Lines()
}

func (n DescribesGroup) Debug(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.MaybeAppendIdItem(&n.Id, indent, hasDot)
	db.AppendSection(UpperDescribesName, indent, false)
	db.Append(&n.Describes.Describes, indent+2, true)
	db.MaybeAppendWithSection(n.With, indent, false)
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

func (n DefinesGroup) Debug(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.MaybeAppendIdItem(&n.Id, indent, hasDot)
	db.AppendSection(UpperDefinesName, indent, false)
	db.Append(&n.Defines.Defines, indent+2, true)
	db.MaybeAppendWithSection(n.With, indent, false)
	db.MaybeAppendUsingSection(n.Using, indent, false)
	db.MaybeAppendWhenSection(n.When, indent, false)
	db.MaybeAppendSuchThatSection(n.SuchThat, indent, false)
	if n.Generalizes != nil {
		db.AppendSection(LowerGeneralizesName, indent, false)
		db.AppendFormulations(n.Generalizes.Generalizes, indent+2, true)
	}
	if n.Means != nil {
		db.AppendSection(LowerMeansName, indent, false)
		db.Append(n.Means.Means, indent+2, true)
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

func (n StatesGroup) Debug(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.MaybeAppendIdItem(&n.Id, indent, hasDot)
	db.AppendSection(UpperStatesName, indent, false)
	db.MaybeAppendWithSection(n.With, indent, false)
	db.MaybeAppendUsingSection(n.Using, indent, false)
	db.MaybeAppendWhenSection(n.When, indent, false)
	db.MaybeAppendSuchThatSection(n.SuchThat, indent, false)
	db.AppendClausesSection(LowerThatName, n.That.That, indent, false)
	db.MaybeAppendDocumentedSection(n.Documented, indent, false)
	if n.Justified != nil {
		db.AppendSection(UpperJustifiedName, indent, false)
		for _, item := range n.Justified.Justified {
			db.Append(item, indent+2, true)
		}
	}
	db.MaybeAppendReferencesSection(n.References, indent, false)
	db.MaybeAppendAliasesSection(n.Aliases, indent, false)
	db.MaybeAppendMetaIdSection(n.MetaId, indent, false)
	return db.Lines()
}

func (n ProofGroup) Debug(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.MaybeAppendIdItem(&n.Id, indent, hasDot)
	db.AppendSection(UpperProofName, indent, false)
	db.AppendSingleTextItemSection(LowerOfName, n.Of.Of, indent, false)
	db.AppendSingleTextItemSection(LowerContentName, n.Content.Content, indent, false)
	db.MaybeAppendReferencesSection(n.References, indent, false)
	db.MaybeAppendMetaIdSection(n.MetaId, indent, false)
	return db.Lines()
}

func (n AxiomGroup) Debug(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.MaybeAppendIdItem(n.Id, indent, hasDot)
	db.AppendSection(UpperAxiomName, indent, hasDot && n.Id == nil)
	db.MaybeAppendGivenSection(n.Given, indent, false)
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

func (n ConjectureGroup) Debug(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.MaybeAppendIdItem(n.Id, indent, hasDot)
	db.AppendSection(UpperConjectureName, indent, hasDot && n.Id == nil)
	db.MaybeAppendGivenSection(n.Given, indent, false)
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

func (n TheoremGroup) Debug(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.MaybeAppendIdItem(n.Id, indent, hasDot)
	db.AppendSection(UpperTheoremName, indent, hasDot && n.Id == nil)
	db.MaybeAppendGivenSection(n.Given, indent, false)
	db.MaybeAppendWhereSection(n.Where, indent, false)
	db.MaybeAppendIfSection(n.If, indent, false)
	db.MaybeAppendIffSection(n.Iff, indent, false)
	db.MaybeAppendThenSection(&n.Then, indent, false)
	if n.Proof != nil {
		db.AppendSection(UpperProofName, indent, false)
		db.Append(&n.Proof.Proof, indent+2, true)
	}
	db.MaybeAppendDocumentedSection(n.Documented, indent, false)
	db.MaybeAppendReferencesSection(n.References, indent, false)
	db.MaybeAppendAliasesSection(n.Aliases, indent, false)
	db.MaybeAppendMetaIdSection(n.MetaId, indent, false)
	return db.Lines()
}

func (n TopicGroup) Debug(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.MaybeAppendIdItem(&n.Id, indent, hasDot)
	db.AppendSection(UpperTopicName, indent, false)
	db.AppendSingleTextItemSection(LowerContentName, n.Content.Content, indent, false)
	db.MaybeAppendReferencesSection(n.References, indent, false)
	db.MaybeAppendMetaIdSection(n.MetaId, indent, false)
	return db.Lines()
}

func (n ZeroGroup) Debug(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.AppendSection(LowerZeroName, indent, hasDot)
	db.MaybeAppendMeansSection(&n.Means, indent, false)
	return db.Lines()
}

func (n PositiveIntGroup) Debug(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.AppendSection(LowerPositiveIntName, indent, hasDot)
	db.MaybeAppendMeansSection(&n.Means, indent, false)
	return db.Lines()
}

func (n NegativeIntGroup) Debug(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.AppendSection(LowerNegativeIntName, indent, hasDot)
	db.MaybeAppendMeansSection(&n.Means, indent, false)
	return db.Lines()
}

func (n PositiveFloatGroup) Debug(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.AppendSection(LowerPositiveFloatName, indent, hasDot)
	db.MaybeAppendMeansSection(&n.Means, indent, false)
	return db.Lines()
}

func (n NegativeFloatGroup) Debug(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.AppendSection(LowerNegativeFloatName, indent, hasDot)
	db.MaybeAppendMeansSection(&n.Means, indent, false)
	return db.Lines()
}

func (n SpecifyGroup) Debug(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.AppendSection(UpperSpecifyName, indent, hasDot)
	for _, item := range n.Specify.Specify {
		db.Append(item, indent+2, true)
	}
	db.MaybeAppendMetaIdSection(n.MetaId, indent, false)
	return db.Lines()
}

func (n PersonGroup) Debug(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.AppendString(n.Id, indent, hasDot)
	db.AppendSection(UpperPersonName, indent, false)
	for _, item := range n.Person.Items {
		db.Append(item, indent+2, true)
	}
	db.MaybeAppendMetaIdSection(n.MetaId, indent, false)
	return db.Lines()
}

func (n NameGroup) Debug(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.AppendTextItemsSection(LowerNameName, n.Name.Name, indent, hasDot)
	return db.Lines()
}

func (n BiographyGroup) Debug(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.AppendSection(LowerBiographyName, indent, hasDot)
	db.Append(&n.Biography.Biography, indent+2, true)
	return db.Lines()
}

func (n ResourceGroup) Debug(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.AppendString(n.Id, indent, hasDot)
	db.AppendSection(UpperResourceName, indent, false)
	for _, item := range n.Resource.Items {
		db.Append(item, indent+2, true)
	}
	db.MaybeAppendMetaIdSection(n.MetaId, indent, false)
	return db.Lines()
}

func (n TitleGroup) Debug(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.AppendSection(LowerTitleName, indent, hasDot)
	db.Append(&n.Title.Title, indent+2, true)
	return db.Lines()
}

func (n AuthorGroup) Debug(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.AppendTextItemsSection(LowerAuthorName, n.Author.Author, indent, hasDot)
	return db.Lines()
}

func (n OffsetGroup) Debug(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.AppendSection(LowerOffsetName, indent, hasDot)
	db.Append(&n.Offset.Offset, indent+2, true)
	return db.Lines()
}

func (n UrlGroup) Debug(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.AppendSection(LowerUrlName, indent, hasDot)
	db.Append(&n.Url.Url, indent+2, true)
	return db.Lines()
}

func (n HomepageGroup) Debug(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.AppendSection(LowerHomepageName, indent, hasDot)
	db.Append(&n.Homepage.Homepage, indent+2, true)
	return db.Lines()
}

func (n TypeGroup) Debug(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.AppendSection(LowerTypeName, indent, hasDot)
	db.Append(&n.Type.Type, indent+2, true)
	return db.Lines()
}

func (n EditorGroup) Debug(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.AppendTextItemsSection(LowerEditorName, n.Editor.Editor, indent, hasDot)
	return db.Lines()
}

func (n EditionGroup) Debug(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.AppendSection(LowerEditionName, indent, hasDot)
	db.Append(&n.Edition.Edition, indent+2, true)
	return db.Lines()
}

func (n InstitutionGroup) Debug(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.AppendTextItemsSection(LowerInstitutionName, n.Institution.Institution, indent, hasDot)
	return db.Lines()
}

func (n JournalGroup) Debug(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.AppendTextItemsSection(LowerJournalName, n.Journal.Journal, indent, hasDot)
	return db.Lines()
}

func (n PublisherGroup) Debug(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.AppendTextItemsSection(LowerPublisherName, n.Publisher.Publisher, indent, hasDot)
	return db.Lines()
}

func (n VolumeGroup) Debug(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.AppendSection(LowerVolumeName, indent, hasDot)
	db.Append(&n.Volume.Volume, indent+2, true)
	return db.Lines()
}

func (n MonthGroup) Debug(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.AppendSection(LowerMonthName, indent, hasDot)
	db.Append(&n.Month.Month, indent+2, true)
	return db.Lines()
}

func (n YearGroup) Debug(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.AppendSection(LowerYearName, indent, hasDot)
	db.Append(&n.Year.Year, indent+2, true)
	return db.Lines()
}

func (n DescriptionGroup) Debug(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	db.AppendSection(LowerDescriptionName, indent, hasDot)
	db.Append(&n.Description.Description, indent+2, true)
	return db.Lines()
}

func (n Document) Debug(indent int, hasDot bool) []string {
	db := newDebugBuilder()
	for _, item := range n.Items {
		db.Append(item, indent, hasDot)
		db.AppendString("", indent, false)
		db.AppendString("", indent, false)
	}
	return db.Lines()
}

func (n TextBlockItem) Debug(indent int, hasDot bool) []string {
	return buildIndentedLineSlice(indent, hasDot, "::"+n.Text+"::")
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

func (db *debugBuilder) Append(node StructuralNodeType, indent int, hasDot bool) {
	db.lines = append(db.lines, node.Debug(indent, hasDot)...)
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

func (db *debugBuilder) AppendClauses(clauses []ClauseType, indent int, hasDot bool) {
	for _, clause := range clauses {
		db.Append(clause, indent, hasDot)
	}
}

func (db *debugBuilder) AppendTextItems(items []TextItem, indent int, hasDot bool) {
	for _, item := range items {
		db.Append(&item, indent, hasDot)
	}
}

func (db *debugBuilder) AppendFormulations(
	items []Formulation[FormulationNodeType], indent int, hasDot bool) {
	for _, item := range items {
		db.Append(&item, indent, hasDot)
	}
}

func (db *debugBuilder) AppendSpecsSection(name string, specs []Spec, indent int, hasDot bool) {
	db.AppendSection(name, indent, hasDot)
	db.AppendSpecs(specs, indent+2, true)
}

func (db *debugBuilder) AppendClausesSection(
	name string, clauses []ClauseType, indent int, hasDot bool) {
	db.AppendSection(name, indent, hasDot)
	db.AppendClauses(clauses, indent+2, true)
}

func (db *debugBuilder) AppendTargetsSection(
	name string, targets []Target, indent int, hasDot bool) {
	db.AppendSection(name, indent, hasDot)
	db.AppendTargets(targets, indent+2, true)
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

func (db *debugBuilder) MaybeAppendGivenSection(sec *GivenSection, indent int, hasDot bool) {
	if sec != nil {
		db.AppendSection(LowerGivenName, indent, hasDot)
		for _, item := range sec.Given {
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

func (db *debugBuilder) MaybeAppendThenSection(sec *ThenSection, indent int, hasDot bool) {
	if sec != nil {
		db.AppendSection(LowerThenName, indent, hasDot)
		for _, item := range sec.Clauses {
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

func (db *debugBuilder) MaybeAppendWithSection(sec *WithSection, indent int, hasDot bool) {
	if sec != nil {
		db.AppendSection(LowerWithName, indent, hasDot)
		for _, item := range sec.With {
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

func (db *debugBuilder) MaybeAppendIffSection(sec *IffSection, indent int, hasDot bool) {
	if sec != nil {
		db.AppendSection(LowerIfName, indent, hasDot)
		for _, item := range sec.Clauses {
			db.Append(item, indent+2, true)
		}
	}
}

func (db *debugBuilder) MaybeAppendMeansSection(sec *MeansSection, indent int, hasDot bool) {
	if sec != nil {
		db.AppendSection(LowerMeansName, indent, hasDot)
		db.Append(sec.Means, indent+2, true)
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

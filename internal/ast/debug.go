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

func (n IdItem) Debug() []string {
	return []string{
		"[" + n.RawText + "]",
	}
}

func (n Target) Debug() []string {
	return []string{
		n.RawText,
	}
}

func (n Spec) Debug() []string {
	return []string{
		"'" + n.RawText + "'",
	}
}

func (n Alias) Debug() []string {
	return []string{
		"'" + n.RawText + "'",
	}
}

func (n Formulation[T]) Debug() []string {
	return []string{
		"'" + n.RawText + "'",
	}
}

func (n TextItem) Debug() []string {
	return []string{
		"\"" + n.RawText + "\"",
	}
}

func (n GivenGroup) Debug() []string {
	result := make([]string, 0)
	result = appendSectionName(result, LowerGivenName)
	result = appendTargets(result, n.Given.Given)
	result = maybeAppendWhereSection(result, n.Where)
	result = maybeAppendSuchThatSection(result, n.SuchThat)
	result = maybeAppendThenSection(result, &n.Then)
	return result
}

func maybeAppendSuchThatSection(lines []string, suchThat *SuchThatSection) []string {
	if suchThat != nil {
		lines = appendSectionName(lines, LowerSuchThatName)
		lines = appendClauses(lines, suchThat.Clauses)
	}
	return lines
}

func (n AllOfGroup) Debug() []string {
	result := make([]string, 0)
	result = appendSectionName(result, LowerAllOfName)
	result = appendClauses(result, n.AllOf.Clauses)
	return result
}

func (n NotGroup) Debug() []string {
	result := make([]string, 0)
	result = appendSectionName(result, LowerNotName)
	result = append(result, n.Not.Clause.Debug()...)
	return result
}

func (n AnyOfGroup) Debug() []string {
	result := make([]string, 0)
	result = appendSectionName(result, LowerAllOfName)
	result = appendClauses(result, n.AnyOf.Clauses)
	return result
}

func (n OneOfGroup) Debug() []string {
	result := make([]string, 0)
	result = appendSectionName(result, LowerAllOfName)
	result = appendClauses(result, n.OneOf.Clauses)
	return result
}

func (n ExistsGroup) Debug() []string {
	result := make([]string, 0)
	result = appendSectionName(result, LowerExistsName)
	result = appendTargets(result, n.Exists.Targets)
	result = maybeAppendWhereSection(result, n.Where)
	result = maybeAppendSuchThatSection(result, n.SuchThat)
	return result
}

func (n ExistsUniqueGroup) Debug() []string {
	result := make([]string, 0)
	result = appendSectionName(result, LowerExistsUniqueName)
	result = appendTargets(result, n.ExistsUnique.Targets)
	result = maybeAppendWhereSection(result, n.Where)
	result = maybeAppendSuchThatSection(result, &n.SuchThat)
	return result
}

func (n ForAllGroup) Debug() []string {
	result := make([]string, 0)
	result = appendSectionName(result, LowerForAllName)
	result = appendTargets(result, n.ForAll.Targets)
	result = maybeAppendWhereSection(result, n.Where)
	result = maybeAppendSuchThatSection(result, n.SuchThat)
	result = maybeAppendThenSection(result, &n.Then)
	return result
}

func (n IfGroup) Debug() []string { return []string{} }

func (n IffGroup) Debug() []string { return []string{} }

func (n PiecewiseGroup) Debug() []string { return []string{} }

func (n WhenGroup) Debug() []string { return []string{} }

func (n SymbolWrittenGroup) Debug() []string { return []string{} }

func (n ConnectionGroup) Debug() []string { return []string{} }

func (n WrittenGroup) Debug() []string { return []string{} }

func (n CalledGroup) Debug() []string { return []string{} }

func (n WritingGroup) Debug() []string { return []string{} }

func (n OverviewGroup) Debug() []string { return []string{} }

func (n MotivationGroup) Debug() []string { return []string{} }

func (n HistoryGroup) Debug() []string { return []string{} }

func (n ExampleGroup) Debug() []string { return []string{} }

func (n RelatedGroup) Debug() []string { return []string{} }

func (n DiscovererGroup) Debug() []string { return []string{} }

func (n NoteGroup) Debug() []string { return []string{} }

func (n DescribingGroup) Debug() []string { return []string{} }

func (n LabelGroup) Debug() []string { return []string{} }

func (n ByGroup) Debug() []string { return []string{} }

func (n DescribesGroup) Debug() []string { return []string{} }

func (n DefinesGroup) Debug() []string { return []string{} }

func (n StatesGroup) Debug() []string { return []string{} }

func (n ProofGroup) Debug() []string { return []string{} }

func (n AxiomGroup) Debug() []string {
	result := make([]string, 0)
	result = maybeAppendIdItem(result, n.Id)
	result = appendSectionName(result, UpperTheoremName)
	result = maybeAppendGivenSection(result, n.Given)
	result = maybeAppendWhereSection(result, n.Where)
	result = maybeAppendIfSection(result, n.If)
	result = maybeAppendIffSection(result, n.Iff)
	result = maybeAppendThenSection(result, &n.Then)
	result = maybeAppendDocumentedSection(result, n.Documented)
	result = maybeAppendReferencesSection(result, n.References)
	result = maybeAppendAliasesSection(result, n.Aliases)
	return result
}

func (n ConjectureGroup) Debug() []string {
	result := make([]string, 0)
	result = maybeAppendIdItem(result, n.Id)
	result = appendSectionName(result, UpperTheoremName)
	result = maybeAppendGivenSection(result, n.Given)
	result = maybeAppendWhereSection(result, n.Where)
	result = maybeAppendIfSection(result, n.If)
	result = maybeAppendIffSection(result, n.Iff)
	result = maybeAppendThenSection(result, &n.Then)
	result = maybeAppendDocumentedSection(result, n.Documented)
	result = maybeAppendReferencesSection(result, n.References)
	result = maybeAppendAliasesSection(result, n.Aliases)
	return result
}

func (n TheoremGroup) Debug() []string {
	result := make([]string, 0)
	result = maybeAppendIdItem(result, n.Id)
	result = appendSectionName(result, UpperTheoremName)
	result = maybeAppendGivenSection(result, n.Given)
	result = maybeAppendWhereSection(result, n.Where)
	result = maybeAppendIfSection(result, n.If)
	result = maybeAppendIffSection(result, n.Iff)
	result = maybeAppendThenSection(result, &n.Then)
	if n.Proof != nil {
	}
	result = maybeAppendDocumentedSection(result, n.Documented)
	result = maybeAppendReferencesSection(result, n.References)
	result = maybeAppendAliasesSection(result, n.Aliases)
	return result
}

func maybeAppendGivenSection(lines []string, given *GivenSection) []string {
	if given != nil {
		lines = appendSectionName(lines, LowerGivenName)
		lines = appendTargets(lines, given.Given)
	}
	return lines
}

func maybeAppendWhereSection(lines []string, where *WhereSection) []string {
	if where != nil {
		lines = appendSectionName(lines, LowerWhereName)
		lines = appendSpecs(lines, where.Specs)
	}
	return lines
}

func maybeAppendIfSection(lines []string, ifSec *IfSection) []string {
	if ifSec != nil {
		lines = appendSectionName(lines, LowerIfName)
		lines = appendClauses(lines, ifSec.Clauses)
	}
	return lines
}

func maybeAppendIffSection(lines []string, iffSec *IffSection) []string {
	if iffSec != nil {
		lines = appendSectionName(lines, LowerIffName)
		lines = appendClauses(lines, iffSec.Clauses)
	}
	return lines
}

func maybeAppendThenSection(lines []string, then *ThenSection) []string {
	if then != nil {
		lines = appendSectionName(lines, LowerThenName)
		lines = appendClauses(lines, then.Clauses)
	}
	return lines
}

func (n TopicGroup) Debug() []string { return []string{} }

func (n ZeroGroup) Debug() []string { return []string{} }

func (n PositiveIntGroup) Debug() []string { return []string{} }

func (n NegativeIntGroup) Debug() []string { return []string{} }

func (n PositiveFloatGroup) Debug() []string { return []string{} }

func (n NegativeFloatGroup) Debug() []string { return []string{} }

func (n SpecifyGroup) Debug() []string { return []string{} }

func (n PersonGroup) Debug() []string { return []string{} }

func (n NameGroup) Debug() []string { return []string{} }

func (n BiographyGroup) Debug() []string { return []string{} }

func (n ResourceGroup) Debug() []string { return []string{} }

func (n TitleGroup) Debug() []string { return []string{} }

func (n AuthorGroup) Debug() []string { return []string{} }

func (n OffsetGroup) Debug() []string { return []string{} }

func (n UrlGroup) Debug() []string { return []string{} }

func (n HomepageGroup) Debug() []string { return []string{} }

func (n TypeGroup) Debug() []string { return []string{} }

func (n EditorGroup) Debug() []string { return []string{} }

func (n EditionGroup) Debug() []string { return []string{} }

func (n InstitutionGroup) Debug() []string { return []string{} }

func (n JournalGroup) Debug() []string { return []string{} }

func (n PublisherGroup) Debug() []string { return []string{} }

func (n VolumeGroup) Debug() []string { return []string{} }

func (n MonthGroup) Debug() []string { return []string{} }

func (n YearGroup) Debug() []string { return []string{} }

func (n DescriptionGroup) Debug() []string { return []string{} }

func (n Document) Debug() []string {
	result := make([]string, 0)
	for _, item := range n.Items {
		result = append(result, item.Debug()...)
	}
	return result
}

func (n TextBlockItem) Debug() []string { return []string{} }

func Debug(node StructuralNodeType) string {
	return strings.Join(indent(node.Debug(), 0, false), "\n")
}

func maybeAppendDocumentedSection(lines []string, sec *DocumentedSection) []string {
	if sec != nil {
		lines = appendSectionName(lines, UpperDocumentedName)
		for _, item := range sec.Documented {
			lines = append(lines, appendDotSpaceToNode(item)...)
		}
	}
	return lines
}

func maybeAppendReferencesSection(lines []string, sec *ReferencesSection) []string {
	if sec != nil {
		lines = appendSectionName(lines, UpperReferencesName)
		for _, item := range sec.References {
			lines = append(lines, appendDotSpaceToNode(item)...)
		}
	}
	return lines
}

func maybeAppendAliasesSection(lines []string, sec *AliasesSection) []string {
	if sec != nil {
		lines = appendSectionName(lines, UpperAliasesName)
		for _, item := range sec.Aliases {
			lines = append(lines, appendDotSpaceToNode(item)...)
		}
	}
	return lines
}

func maybeAppendMetadataIdSection(lines []string, sec *MetaIdSection) []string {
	if sec != nil {
		lines = appendSectionName(lines, UpperIdName)
		lines = append(lines, appendDotSpaceToNode(sec.Id)...)
	}
	return lines
}

func maybeAppendIdItem(lines []string, id *IdItem) []string {
	if id != nil {
		lines = append(lines, id.Debug()...)
	}
	return lines
}

func appendNode[T StructuralNodeType](lines []string, node T) []string {
	return append(lines, node.Debug()...)
}

func appendDotSpaceNodes[T StructuralNodeType](lines []string, nodes []T) []string {
	for _, n := range nodes {
		lines = append(lines, appendDotSpaceToNode(n)...)
	}
	return lines
}

func appendTargets(lines []string, targets []Target) []string {
	return appendDotSpaceNodes(lines, targets)
}

func appendSpecs(lines []string, specs []Spec) []string {
	return appendDotSpaceNodes(lines, specs)
}

func appendClauses(lines []string, clauses []Clause) []string {
	return appendDotSpaceNodes(lines, clauses)
}

func appendSectionName(lines []string, name string) []string {
	return append(lines, name+":")
}

func appendDotSpaceToNode(node StructuralNodeType) []string {
	return indent(node.Debug(), 2, true)
}

func indent(lines []string, indent int, dot bool) []string {
	indentStr := strings.Repeat(" ", indent)
	result := make([]string, 0)
	for i, line := range lines {
		if i == 0 && dot {
			result = append(result, strings.Repeat(" ", indent-2)+". "+line)
		} else {
			result = append(result, indentStr+line)
		}
	}
	return result
}

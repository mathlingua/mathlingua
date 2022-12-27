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

import "strings"

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
		n.RawText,
	}
}

func (n Alias) Debug() []string {
	return []string{
		n.RawText,
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
	result = appendSection(result, LowerGivenName)
	for _, target := range n.Given.Given {
		result = append(result, appendDotSpaceToNode(target)...)
	}
	if n.Where != nil {
		result = appendSection(result, LowerWhereName)
		result = appendSpecs(result, n.Where.Specs)
	}
	if n.SuchThat != nil {
		result = appendSection(result, LowerSuchThatName)
		result = appendClauses(result, n.SuchThat.Clauses)
	}
	result = appendSection(result, LowerThenName)
	result = appendClauses(result, n.Then.Clauses)
	return result
}

func (n AllOfGroup) Debug() []string {
	result := make([]string, 0)
	result = appendSection(result, LowerAllOfName)
	result = appendClauses(result, n.AllOf.Clauses)
	return result
}

func (n NotGroup) Debug() []string {
	result := make([]string, 0)
	result = appendSection(result, LowerNotName)
	result = append(result, n.Not.Clause.Debug()...)
	return result
}

func (n AnyOfGroup) Debug() []string {
	result := make([]string, 0)
	result = appendSection(result, LowerAllOfName)
	result = appendClauses(result, n.AnyOf.Clauses)
	return result
}

func (n OneOfGroup) Debug() []string {
	result := make([]string, 0)
	result = appendSection(result, LowerAllOfName)
	result = appendClauses(result, n.OneOf.Clauses)
	return result
}

func (n ExistsGroup) Debug() []string {
	result := make([]string, 0)
	result = appendSection(result, LowerExistsName)
	for _, target := range n.Exists.Targets {
		result = append(result, appendDotSpaceToNode(target)...)
	}
	if n.Where != nil {
		result = appendSection(result, LowerWhereName)
		result = appendSpecs(result, n.Where.Specs)
	}
	if n.SuchThat != nil {
		result = appendSection(result, LowerSuchThatName)
		result = appendClauses(result, n.SuchThat.Clauses)
	}
	return result
}

func (n ExistsUniqueGroup) Debug() []string {
	result := make([]string, 0)
	result = appendSection(result, LowerExistsUniqueName)
	for _, target := range n.ExistsUnique.Targets {
		result = append(result, appendDotSpaceToNode(target)...)
	}
	if n.Where != nil {
		result = appendSection(result, LowerWhereName)
		result = appendSpecs(result, n.Where.Specs)
	}
	result = appendSection(result, LowerSuchThatName)
	result = appendClauses(result, n.SuchThat.Clauses)
	return result
}

func (n ForAllGroup) Debug() []string {
	result := make([]string, 0)
	result = appendSection(result, LowerForAllName)
	for _, target := range n.ForAll.Targets {
		result = append(result, appendDotSpaceToNode(target)...)
	}
	if n.Where != nil {
		result = appendSection(result, LowerWhereName)
		result = appendSpecs(result, n.Where.Specs)
	}
	if n.SuchThat != nil {
		result = appendSection(result, LowerSuchThatName)
		result = appendClauses(result, n.SuchThat.Clauses)
	}
	result = appendSection(result, LowerThenName)
	result = appendClauses(result, n.Then.Clauses)
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

func (n AxiomGroup) Debug() []string { return []string{} }

func (n ConjectureGroup) Debug() []string { return []string{} }

func (n TheoremGroup) Debug() []string { return []string{} }

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

func (n Document) Debug() []string { return []string{} }

func (n TextBlockItem) Debug() []string { return []string{} }

func Debug(node StructuralNodeType) string {
	return strings.Join(indent(node.Debug(), 0, false), "\n")
}

func appendSpecs(lines []string, specs []Spec) []string {
	for _, spec := range specs {
		lines = append(lines, appendDotSpaceToNode(spec)...)
	}
	return lines
}

func appendClauses(lines []string, clauses []Clause) []string {
	for _, clause := range clauses {
		lines = append(lines, appendDotSpaceToNode(clause)...)
	}
	return lines
}

func appendSection(lines []string, name string) []string {
	return append(lines, name+":")
}

func appendDotSpaceToNode(node StructuralNodeType) []string {
	return appendDotSpaceToLines(node.Debug())
}

func appendDotSpaceToLines(lines []string) []string {
	result := make([]string, 0)
	for _, line := range lines {
		result = append(result, ". "+line)
	}
	return result
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

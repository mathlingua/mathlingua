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

package phase5

import (
	"mathlingua/internal/ast"
	"mathlingua/internal/frontend"
	"mathlingua/internal/frontend/formulation"
	"mathlingua/internal/frontend/phase4"
)

func NewPhase5Parser(tracker frontend.DiagnosticTracker) parser {
	return parser{
		tracker: tracker,
	}
}

//////////////////////////////////////////////////////////////////////////

type parser struct {
	tracker frontend.DiagnosticTracker
}

/////////////////////////// allOf ///////////////////////////////////////

func (p *parser) toAllOfGroup(group phase4.Group) (ast.AllOfGroup, bool) {
	sections, ok := IdentifySections(group.Sections, p.tracker, "allOf")
	if !ok {
		return ast.AllOfGroup{}, false
	}
	return ast.AllOfGroup{
		AllOf: *p.toAllOfSection(sections["allOf"]),
	}, true
}

func (p *parser) toAllOfSection(section phase4.Section) *ast.AllOfSection {
	return &ast.AllOfSection{
		Clauses: p.oneOrMoreClauses(section),
	}
}

/////////////////////////////// not //////////////////////////////////////

func (p *parser) toNotGroup(group phase4.Group) (ast.NotGroup, bool) {
	sections, ok := IdentifySections(group.Sections, p.tracker, "not")
	if !ok {
		return ast.NotGroup{}, false
	}
	return ast.NotGroup{
		Not: *p.toNotSection(sections["not"]),
	}, true
}

func (p *parser) toNotSection(section phase4.Section) *ast.NotSection {
	return &ast.NotSection{
		Clause: p.exactlyOneClause(section),
	}
}

//////////////////////////////////// anyOf ////////////////////////////////

func (p *parser) toAnyOfGroup(group phase4.Group) (ast.AnyOfGroup, bool) {
	sections, ok := IdentifySections(group.Sections, p.tracker, "anyOf")
	if !ok {
		return ast.AnyOfGroup{}, false
	}
	return ast.AnyOfGroup{
		AnyOf: *p.toAnyOfSection(sections["anyOf"]),
	}, true
}

func (p *parser) toAnyOfSection(section phase4.Section) *ast.AnyOfSection {
	return &ast.AnyOfSection{
		Clauses: p.oneOrMoreClauses(section),
	}
}

////////////////////////////////////// oneOf /////////////////////////////

func (p *parser) toOneOfGroup(group phase4.Group) (ast.OneOfGroup, bool) {
	sections, ok := IdentifySections(group.Sections, p.tracker, "oneOf")
	if !ok {
		return ast.OneOfGroup{}, false
	}
	return ast.OneOfGroup{
		OneOf: *p.toOneOfSection(sections["oneOf"]),
	}, true
}

func (p *parser) toOneOfSection(section phase4.Section) *ast.OneOfSection {
	return &ast.OneOfSection{
		Clauses: p.oneOrMoreClauses(section),
	}
}

/////////////////////////////// exists //////////////////////////////////

func (p *parser) ToExistsGroup(group phase4.Group) (ast.ExistsGroup, bool) {
	sections, ok := IdentifySections(group.Sections, p.tracker,
		"exists",
		"where?",
		"suchThat?")
	if !ok {
		return ast.ExistsGroup{}, false
	}
	exists := *p.toExistsSection(sections["exists"])
	var where *ast.WhereSection
	if sect, ok := sections["where"]; ok {
		where = p.toWhereSection(sect)
	}
	var suchThat *ast.SuchThatSection
	if sect, ok := sections["suchThat"]; ok {
		suchThat = p.toSuchThatSection(sect)
	}
	return ast.ExistsGroup{
		Exists:   exists,
		Where:    where,
		SuchThat: suchThat,
	}, true
}

func (p *parser) toExistsSection(section phase4.Section) *ast.ExistsSection {
	return &ast.ExistsSection{
		Targets: p.oneOrMoreTargets(section),
	}
}

func (p *parser) toWhereSection(section phase4.Section) *ast.WhereSection {
	return &ast.WhereSection{
		Specs: p.oneOrMoreSpecs(section),
	}
}

func (p *parser) toSuchThatSection(section phase4.Section) *ast.SuchThatSection {
	return &ast.SuchThatSection{
		Clauses: p.oneOrMoreClauses(section),
	}
}

/////////////////////////////// exists //////////////////////////////////

func (p *parser) ToExistsUniqueGroup(group phase4.Group) (ast.ExistsUniqueGroup, bool) {
	sections, ok := IdentifySections(group.Sections, p.tracker,
		"existsUnique",
		"where?",
		"suchThat?")
	if !ok {
		return ast.ExistsUniqueGroup{}, false
	}
	existsUnique := *p.toExistsUniqueSection(sections["existsUnique"])
	var where *ast.WhereSection
	if sect, ok := sections["where"]; ok {
		where = p.toWhereSection(sect)
	}
	var suchThat *ast.SuchThatSection
	if sect, ok := sections["suchThat"]; ok {
		suchThat = p.toSuchThatSection(sect)
	}
	return ast.ExistsUniqueGroup{
		ExistsUnique: existsUnique,
		Where:        where,
		SuchThat:     suchThat,
	}, true
}

func (p *parser) toExistsUniqueSection(section phase4.Section) *ast.ExistsUniqueSection {
	return &ast.ExistsUniqueSection{
		Targets: p.oneOrMoreTargets(section),
	}
}

///////////////////////// forAll /////////////////////////////////////////

func (p *parser) ToForAllGroup(group phase4.Group) (ast.ForAllGroup, bool) {
	sections, ok := IdentifySections(group.Sections, p.tracker,
		"forAll",
		"where?",
		"suchThat?",
		"then")
	if !ok {
		return ast.ForAllGroup{}, false
	}
	forAll := *p.toForAllSection(sections["forAll"])
	var where *ast.WhereSection
	if sec, ok := sections["where"]; ok {
		where = p.toWhereSection(sec)
	}
	var suchThat *ast.SuchThatSection
	if sec, ok := sections["suchThat"]; ok {
		suchThat = p.toSuchThatSection(sec)
	}
	then := *p.toThenSection(sections["then"])
	return ast.ForAllGroup{
		ForAll:   forAll,
		Where:    where,
		SuchThat: suchThat,
		Then:     then,
	}, true
}

func (p *parser) toForAllSection(section phase4.Section) *ast.ForAllSection {
	return &ast.ForAllSection{
		Targets: p.oneOrMoreTargets(section),
	}
}

func (p *parser) toThenSection(section phase4.Section) *ast.ThenSection {
	return &ast.ThenSection{
		Clauses: p.oneOrMoreClauses(section),
	}
}

/////////////////////////////// if ///////////////////////////////////////

func (p *parser) ToIfGroup(group phase4.Group) (ast.IfGroup, bool) {
	sections, ok := IdentifySections(group.Sections, p.tracker,
		"if",
		"then")
	if !ok {
		return ast.IfGroup{}, false
	}
	return ast.IfGroup{
		If:   *p.toIfSection(sections["if"]),
		Then: *p.toThenSection(sections["then"]),
	}, true
}

func (p *parser) toIfSection(section phase4.Section) *ast.IfSection {
	return &ast.IfSection{
		Clauses: p.oneOrMoreClauses(section),
	}
}

/////////////////////////////// iff ///////////////////////////////////////

func (p *parser) ToIffGroup(group phase4.Group) (ast.IffGroup, bool) {
	sections, ok := IdentifySections(group.Sections, p.tracker,
		"iff",
		"then")
	if !ok {
		return ast.IffGroup{}, false
	}
	return ast.IffGroup{
		Iff:  *p.toIffSection(sections["iff"]),
		Then: *p.toThenSection(sections["then"]),
	}, true
}

func (p *parser) toIffSection(section phase4.Section) *ast.IffSection {
	return &ast.IffSection{
		Clauses: p.oneOrMoreClauses(section),
	}
}

////////////////////////// arguments ////////////////////////////////////

func (p *parser) toClause(arg phase4.Argument) ast.Clause {
	switch data := arg.Arg.(type) {
	case phase4.FormulationArgumentData:
		if node, ok := formulation.ParseExpression(data.Text, p.tracker); ok {
			return ast.Formulation[ast.NodeType]{
				RawText: data.Text,
				Root:    node,
				Label:   nil,
			}
		} else {
			return ast.Formulation[ast.NodeType]{}
		}
	case phase4.Group:
		if grp, ok := p.toAllOfGroup(data); ok {
			return grp
		} else if grp, ok := p.toNotGroup(data); ok {
			return grp
		} else if grp, ok := p.toAnyOfGroup(data); ok {
			return grp
		} else if grp, ok := p.toOneOfGroup(data); ok {
			return grp
		} else if grp, ok := p.ToExistsGroup(data); ok {
			return grp
		}
	}

	p.tracker.Append(newError("Expected a clause", arg.MetaData.Start))
	return ast.Formulation[ast.NodeType]{}
}

func (p *parser) toSpec(arg phase4.Argument) ast.Spec {
	switch data := arg.Arg.(type) {
	case phase4.FormulationArgumentData:
		if node, ok := formulation.ParseExpression(data.Text, p.tracker); ok {
			return ast.Spec{
				RawText: data.Text,
				Root:    node,
				Label:   nil,
			}
		} else {
			return ast.Spec{}
		}
	default:
		p.tracker.Append(newError("Expected a specification", arg.MetaData.Start))
		return ast.Spec{}
	}
}

func (p *parser) toTarget(arg phase4.Argument) ast.Target {
	switch data := arg.Arg.(type) {
	case phase4.ArgumentTextArgumentData:
		if node, ok := formulation.ParseForm(data.Text, p.tracker); ok {
			return ast.Target{
				RawText: data.Text,
				Root:    node,
				Label:   nil,
			}
		} else {
			return ast.Target{}
		}
	default:
		p.tracker.Append(newError("Expected a target", arg.MetaData.Start))
		return ast.Target{}
	}
}

func (p *parser) toIdItem(arg phase4.Argument) ast.IdItem {
	switch data := arg.Arg.(type) {
	case phase4.FormulationArgumentData:
		if node, ok := formulation.ParseId(data.Text, p.tracker); ok {
			return ast.IdItem{
				RawText: data.Text,
				Root:    node,
				Label:   nil,
			}
		} else {
			return ast.IdItem{}
		}
	default:
		p.tracker.Append(newError("Expected an id", arg.MetaData.Start))
		return ast.IdItem{}
	}
}

func (p *parser) toTextItem(arg phase4.Argument) ast.TextItem {
	switch data := arg.Arg.(type) {
	case phase4.TextArgumentData:
		return ast.TextItem{
			RawText: data.Text,
		}
	default:
		p.tracker.Append(newError("Expected a text item", arg.MetaData.Start))
		return ast.TextItem{}
	}
}

//////////////////////// argument lists /////////////////////////////////

func (p *parser) toClauses(args []phase4.Argument) []ast.Clause {
	result := make([]ast.Clause, 0)
	for _, arg := range args {
		result = append(result, p.toClause(arg))
	}
	return result
}

func (p *parser) toSpecs(args []phase4.Argument) []ast.Spec {
	result := make([]ast.Spec, 0)
	for _, arg := range args {
		result = append(result, p.toSpec(arg))
	}
	return result
}

func (p *parser) toTargets(args []phase4.Argument) []ast.Target {
	result := make([]ast.Target, 0)
	for _, arg := range args {
		result = append(result, p.toTarget(arg))
	}
	return result
}

func (p *parser) toTextItems(args []phase4.Argument) []ast.TextItem {
	result := make([]ast.TextItem, 0)
	for _, arg := range args {
		result = append(result, p.toTextItem(arg))
	}
	return result
}

/////////////////////////////////////////////////////////////////////////

func (p *parser) oneOrMoreClauses(section phase4.Section) []ast.Clause {
	return oneOrMore(p.toClauses(section.Args), section.MetaData.Start, p.tracker)
}

func (p *parser) exactlyOneClause(section phase4.Section) ast.Clause {
	var def ast.Clause = ast.Formulation[ast.NodeType]{}
	return exactlyOne(p.toClauses(section.Args), def, section.MetaData.Start, p.tracker)
}

func (p *parser) oneOrMoreSpecs(section phase4.Section) []ast.Spec {
	return oneOrMore(p.toSpecs(section.Args), section.MetaData.Start, p.tracker)
}

func (p *parser) exactlyOneSpec(section phase4.Section) ast.Spec {
	var def ast.Spec = ast.Formulation[ast.NodeType]{}
	return exactlyOne(p.toSpecs(section.Args), def, section.MetaData.Start, p.tracker)
}

func (p *parser) oneOrMoreTargets(section phase4.Section) []ast.Target {
	return oneOrMore(p.toTargets(section.Args), section.MetaData.Start, p.tracker)
}

func (p *parser) exactlyOneTarget(section phase4.Section) ast.Target {
	var def ast.Target = ast.Formulation[ast.NodeType]{}
	return exactlyOne(p.toTargets(section.Args), def, section.MetaData.Start, p.tracker)
}

////////////////////////// support functions ////////////////////////////

func oneOrMore[T any](items []T, position ast.Position, tracker frontend.DiagnosticTracker) []T {
	if len(items) == 0 {
		tracker.Append(newError("Expected at least one item", position))
		return []T{}
	}
	return items
}

func exactlyOne[T any](items []T, defaultItem T, position ast.Position, tracker frontend.DiagnosticTracker) T {
	if len(items) != 1 {
		tracker.Append(newError("Expected at exactly one item", position))
	}
	if len(items) == 0 {
		return defaultItem
	}
	return items[0]
}

/*
func hasSections(group phase4.Group, names ...string) bool {
	if len(names) > len(group.Sections) {
		return false
	}

	for i, name := range names {
		if group.Sections[i].Name != name {
			return false
		}
	}

	return true
}
*/

func newError(message string, position ast.Position) frontend.Diagnostic {
	return frontend.Diagnostic{
		Type:     frontend.Error,
		Origin:   frontend.Phase5ParserOrigin,
		Message:  message,
		Position: position,
	}
}

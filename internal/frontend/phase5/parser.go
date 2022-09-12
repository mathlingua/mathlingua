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

func (p *parser) ToAllOfGroup(group phase4.Group) (ast.AllOfGroup, bool) {
	sections, ok := IdentifySections(group.Sections, p.tracker, "allOf")
	if !ok {
		return ast.AllOfGroup{}, false
	}
	return ast.AllOfGroup{
		AllOf: required(p.toAllOfSection(getSection(sections, "allOf"))),
	}, true
}

func (p *parser) toAllOfSection(section phase4.Section, ok bool) (ast.AllOfSection, bool) {
	if !ok {
		return ast.AllOfSection{}, false
	}
	return ast.AllOfSection{
		Clauses: required(p.oneOrMoreClause(section, ok)),
	}, true
}

/////////////////////////////// not //////////////////////////////////////

func (p *parser) toNotGroup(group phase4.Group) (ast.NotGroup, bool) {
	sections, ok := IdentifySections(group.Sections, p.tracker, "not")
	if !ok {
		return ast.NotGroup{}, false
	}
	return ast.NotGroup{
		Not: required(p.toNotSection(getSection(sections, "not"))),
	}, true
}

func (p *parser) toNotSection(section phase4.Section, ok bool) (ast.NotSection, bool) {
	if !ok {
		return ast.NotSection{}, false
	}
	return ast.NotSection{
		Clause: required(p.exactlyOneClause(section, ok)),
	}, true
}

//////////////////////////////////// anyOf ////////////////////////////////

func (p *parser) toAnyOfGroup(group phase4.Group) (ast.AnyOfGroup, bool) {
	sections, ok := IdentifySections(group.Sections, p.tracker, "anyOf")
	if !ok {
		return ast.AnyOfGroup{}, false
	}
	return ast.AnyOfGroup{
		AnyOf: required(p.toAnyOfSection(getSection(sections, "anyOf"))),
	}, true
}

func (p *parser) toAnyOfSection(section phase4.Section, ok bool) (ast.AnyOfSection, bool) {
	if !ok {
		return ast.AnyOfSection{}, false
	}
	return ast.AnyOfSection{
		Clauses: required(p.oneOrMoreClause(section, ok)),
	}, true
}

////////////////////////////////////// oneOf /////////////////////////////

func (p *parser) toOneOfGroup(group phase4.Group) (ast.OneOfGroup, bool) {
	sections, ok := IdentifySections(group.Sections, p.tracker, "oneOf")
	if !ok {
		return ast.OneOfGroup{}, false
	}
	return ast.OneOfGroup{
		OneOf: required(p.toOneOfSection(getSection(sections, "oneOf"))),
	}, true
}

func (p *parser) toOneOfSection(section phase4.Section, ok bool) (ast.OneOfSection, bool) {
	if !ok {
		return ast.OneOfSection{}, false
	}
	return ast.OneOfSection{
		Clauses: required(p.oneOrMoreClause(section, ok)),
	}, true
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
		if grp, ok := p.ToAllOfGroup(data); ok {
			return grp
		} else if grp, ok := p.toNotGroup(data); ok {
			return grp
		} else if grp, ok := p.toAnyOfGroup(data); ok {
			return grp
		} else if grp, ok := p.toOneOfGroup(data); ok {
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

func (p *parser) oneOrMoreClause(section phase4.Section, ok bool) ([]ast.Clause, bool) {
	return oneOrMore(p.toClauses(section.Args), ok, section.MetaData.Start, p.tracker)
}

func (p *parser) exactlyOneClause(section phase4.Section, ok bool) (ast.Clause, bool) {
	var def ast.Clause = ast.Formulation[ast.NodeType]{}
	return exactlyOne(p.toClauses(section.Args), ok, def, section.MetaData.Start, p.tracker)
}

func (p *parser) oneOrMoreSpecs(section phase4.Section, ok bool) ([]ast.Spec, bool) {
	return oneOrMore(p.toSpecs(section.Args), ok, section.MetaData.Start, p.tracker)
}

func (p *parser) exactlyOneSpec(section phase4.Section, ok bool) (ast.Spec, bool) {
	var def ast.Spec = ast.Formulation[ast.NodeType]{}
	return exactlyOne(p.toSpecs(section.Args), ok, def, section.MetaData.Start, p.tracker)
}

func (p *parser) oneOrMoreTargets(section phase4.Section, ok bool) ([]ast.Target, bool) {
	return oneOrMore(p.toTargets(section.Args), ok, section.MetaData.Start, p.tracker)
}

func (p *parser) exactlyOneTarget(section phase4.Section, ok bool) (ast.Target, bool) {
	var def ast.Target = ast.Formulation[ast.NodeType]{}
	return exactlyOne(p.toTargets(section.Args), ok, def, section.MetaData.Start, p.tracker)
}

////////////////////////// support functions ////////////////////////////

func required[T any](value T, ok bool) T {
	return value
}

func optional[T any](value T, ok bool) *T {
	if ok {
		return &value
	} else {
		return nil
	}
}

func oneOrMore[T any](items []T, ok bool, position ast.Position, tracker frontend.DiagnosticTracker) ([]T, bool) {
	if !ok {
		return []T{}, false
	}
	if len(items) == 0 {
		tracker.Append(newError("Expected at least one item", position))
		return []T{}, false
	}
	return items, true
}

func exactlyOne[T any](items []T, ok bool, defaultItem T, position ast.Position, tracker frontend.DiagnosticTracker) (T, bool) {
	if !ok {
		return defaultItem, false
	}
	if len(items) != 0 {
		tracker.Append(newError("Expected at exactly one item", position))
		return defaultItem, false
	}
	if len(items) == 0 {
		return defaultItem, true
	}
	return items[0], true
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

func getSection(sections map[string]phase4.Section, name string) (phase4.Section, bool) {
	sec, ok := sections[name]
	return sec, ok
}

func newError(message string, position ast.Position) frontend.Diagnostic {
	return frontend.Diagnostic{
		Type:     frontend.Error,
		Origin:   frontend.Phase5ParserOrigin,
		Message:  message,
		Position: position,
	}
}

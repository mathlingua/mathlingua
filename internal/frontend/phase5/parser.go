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

/////////////////////////// allOf ///////////////////////////////////////

func ToAllOfGroup(group phase4.Group, tracker frontend.DiagnosticTracker) (ast.AllOfGroup, bool) {
	sections, ok := IdentifySections(group.Sections, tracker, "allOf")
	if !ok {
		return ast.AllOfGroup{}, false
	}
	return ast.AllOfGroup{
		AllOf: toAllOfSection(sections["allOf"], tracker),
	}, true
}

func toAllOfSection(section phase4.Section, tracker frontend.DiagnosticTracker) ast.AllOfSection {
	return ast.AllOfSection{
		Clauses: oneOrMoreClause(section, tracker),
	}
}

/////////////////////////////// not //////////////////////////////////////

func toNotGroup(group phase4.Group, tracker frontend.DiagnosticTracker) (ast.NotGroup, bool) {
	sections, ok := IdentifySections(group.Sections, tracker, "not")
	if !ok {
		return ast.NotGroup{}, false
	}
	return ast.NotGroup{
		Not: toNotSection(sections["not"], tracker),
	}, true
}

func toNotSection(section phase4.Section, tracker frontend.DiagnosticTracker) ast.NotSection {
	return ast.NotSection{
		Clause: exactlyOneClause(section, tracker),
	}
}

//////////////////////////////////// anyOf ////////////////////////////////

func toAnyOfGroup(group phase4.Group, tracker frontend.DiagnosticTracker) (ast.AnyOfGroup, bool) {
	sections, ok := IdentifySections(group.Sections, tracker, "anyOf")
	if !ok {
		return ast.AnyOfGroup{}, false
	}
	return ast.AnyOfGroup{
		AnyOf: toAnyOfSection(sections["anyOf"], tracker),
	}, true
}

func toAnyOfSection(section phase4.Section, tracker frontend.DiagnosticTracker) ast.AnyOfSection {
	return ast.AnyOfSection{
		Clauses: oneOrMoreClause(section, tracker),
	}
}

////////////////////////////////////// oneOf /////////////////////////////

func toOneOfGroup(group phase4.Group, tracker frontend.DiagnosticTracker) (ast.OneOfGroup, bool) {
	sections, ok := IdentifySections(group.Sections, tracker, "oneOf")
	if !ok {
		return ast.OneOfGroup{}, false
	}
	return ast.OneOfGroup{
		OneOf: toOneOfSection(sections["oneOf"], tracker),
	}, true
}

func toOneOfSection(section phase4.Section, tracker frontend.DiagnosticTracker) ast.OneOfSection {
	return ast.OneOfSection{
		Clauses: oneOrMoreClause(section, tracker),
	}
}

////////////////////////// arguments ////////////////////////////////////

func toClause(arg phase4.Argument, tracker frontend.DiagnosticTracker) ast.Clause {
	switch data := arg.Arg.(type) {
	case phase4.FormulationArgumentData:
		if node, ok := formulation.ParseExpression(data.Text, tracker); ok {
			return ast.Clause{
				RawText: data.Text,
				Root:    node,
				Label:   nil,
			}
		} else {
			return ast.Clause{}
		}
	default:
		tracker.Append(newError("Expected a clause", arg.MetaData.Start))
		return ast.Clause{}
	}
}

func toSpec(arg phase4.Argument, tracker frontend.DiagnosticTracker) ast.Spec {
	switch data := arg.Arg.(type) {
	case phase4.FormulationArgumentData:
		if node, ok := formulation.ParseExpression(data.Text, tracker); ok {
			return ast.Spec{
				RawText: data.Text,
				Root:    node,
				Label:   nil,
			}
		} else {
			return ast.Spec{}
		}
	default:
		tracker.Append(newError("Expected a specification", arg.MetaData.Start))
		return ast.Spec{}
	}
}

func toTarget(arg phase4.Argument, tracker frontend.DiagnosticTracker) ast.Target {
	switch data := arg.Arg.(type) {
	case phase4.ArgumentTextArgumentData:
		if node, ok := formulation.ParseForm(data.Text, tracker); ok {
			return ast.Target{
				RawText: data.Text,
				Root:    node,
				Label:   nil,
			}
		} else {
			return ast.Target{}
		}
	default:
		tracker.Append(newError("Expected a target", arg.MetaData.Start))
		return ast.Target{}
	}
}

func toIdItem(arg phase4.Argument, tracker frontend.DiagnosticTracker) ast.IdItem {
	switch data := arg.Arg.(type) {
	case phase4.FormulationArgumentData:
		if node, ok := formulation.ParseId(data.Text, tracker); ok {
			return ast.IdItem{
				RawText: data.Text,
				Root:    node,
				Label:   nil,
			}
		} else {
			return ast.IdItem{}
		}
	default:
		tracker.Append(newError("Expected an id", arg.MetaData.Start))
		return ast.IdItem{}
	}
}

func toTextItem(arg phase4.Argument, tracker frontend.DiagnosticTracker) ast.TextItem {
	switch data := arg.Arg.(type) {
	case phase4.TextArgumentData:
		return ast.TextItem{
			RawText: data.Text,
		}
	default:
		tracker.Append(newError("Expected a text item", arg.MetaData.Start))
		return ast.TextItem{}
	}
}

//////////////////////// argument lists /////////////////////////////////

func toClauses(args []phase4.Argument, tracker frontend.DiagnosticTracker) []ast.Clause {
	result := make([]ast.Clause, 0)
	for _, arg := range args {
		result = append(result, toClause(arg, tracker))
	}
	return result
}

func toSpecs(args []phase4.Argument, tracker frontend.DiagnosticTracker) []ast.Spec {
	result := make([]ast.Clause, 0)
	for _, arg := range args {
		result = append(result, toSpec(arg, tracker))
	}
	return result
}

func toTargets(args []phase4.Argument, tracker frontend.DiagnosticTracker) []ast.Target {
	result := make([]ast.Target, 0)
	for _, arg := range args {
		result = append(result, toTarget(arg, tracker))
	}
	return result
}

func toTextItems(args []phase4.Argument, tracker frontend.DiagnosticTracker) []ast.TextItem {
	result := make([]ast.TextItem, 0)
	for _, arg := range args {
		result = append(result, toTextItem(arg, tracker))
	}
	return result
}

/////////////////////////////////////////////////////////////////////////

func oneOrMoreClause(section phase4.Section, tracker frontend.DiagnosticTracker) []ast.Clause {
	return oneOrMore(toClauses(section.Args, tracker), tracker, section.MetaData.Start)
}

func exactlyOneClause(section phase4.Section, tracker frontend.DiagnosticTracker) ast.Clause {
	return exactlyOne(toClauses(section.Args, tracker), tracker, ast.Clause{}, section.MetaData.Start)
}

func oneOrMoreSpecs(section phase4.Section, tracker frontend.DiagnosticTracker) []ast.Spec {
	return oneOrMore(toSpecs(section.Args, tracker), tracker, section.MetaData.Start)
}

func exactlyOneSpec(section phase4.Section, tracker frontend.DiagnosticTracker) ast.Spec {
	var def ast.Spec = ast.Formulation[ast.NodeType]{}
	return exactlyOne(toSpecs(section.Args, tracker), tracker, def, section.MetaData.Start)
}

func oneOrMoreTargets(section phase4.Section, tracker frontend.DiagnosticTracker) []ast.Target {
	return oneOrMore(toTargets(section.Args, tracker), tracker, section.MetaData.Start)
}

func exactlyOneTarget(section phase4.Section, tracker frontend.DiagnosticTracker) ast.Target {
	var def ast.Target = ast.Formulation[ast.NodeType]{}
	return exactlyOne(toTargets(section.Args, tracker), tracker, def, section.MetaData.Start)
}

////////////////////////// support functions ////////////////////////////

func required[T any](value T, ok bool) T {
	return value
}

func optional[T any](value T, ok bool) T {
	return value
}

func oneOrMore[T any](items []T, tracker frontend.DiagnosticTracker, position ast.Position) []T {
	if len(items) == 0 {
		tracker.Append(newError("Expected at least one item", position))
	}
	return items
}

func exactlyOne[T any](items []T, tracker frontend.DiagnosticTracker, defaultItem T, position ast.Position) T {
	if len(items) != 0 {
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

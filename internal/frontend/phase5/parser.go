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

func toAllOfSection(section phase4.Section) (ast.AllOfSection, bool) {
	if section.Name != "allOf" {
		return ast.AllOfSection{}, false
	}
	return ast.AllOfSection{
		//		Clauses: toClauses(section.Args, tracker),
	}, true
}

////////////////////////// arguments ////////////////////////////////////

func toClause(arg phase4.Argument, tracker frontend.DiagnosticTracker) (ast.Clause, bool) {
	switch data := arg.Arg.(type) {
	case phase4.FormulationArgumentData:
		if node, ok := formulation.ParseExpression(data.Text, tracker); ok {
			return ast.Clause{
				RawText: data.Text,
				Root:    node,
				Label:   nil,
			}, true
		} else {
			return ast.Clause{}, false
		}
	default:
		tracker.Append(newError("Expected a clause", arg.MetaData.Start))
		return ast.Clause{}, false
	}
}

func toSpec(arg phase4.Argument, tracker frontend.DiagnosticTracker) (ast.Spec, bool) {
	switch data := arg.Arg.(type) {
	case phase4.FormulationArgumentData:
		if node, ok := formulation.ParseExpression(data.Text, tracker); ok {
			return ast.Clause{
				RawText: data.Text,
				Root:    node,
				Label:   nil,
			}, true
		} else {
			return ast.Clause{}, false
		}
	default:
		tracker.Append(newError("Expected a specification", arg.MetaData.Start))
		return ast.Clause{}, false
	}
}

func toTarget(arg phase4.Argument, tracker frontend.DiagnosticTracker) (ast.Clause, bool) {
	switch data := arg.Arg.(type) {
	case phase4.ArgumentTextArgumentData:
		if node, ok := formulation.ParseForm(data.Text, tracker); ok {
			return ast.Target{
				RawText: data.Text,
				Root:    node,
				Label:   nil,
			}, true
		} else {
			return ast.Clause{}, false
		}
	default:
		tracker.Append(newError("Expected a target", arg.MetaData.Start))
		return ast.Clause{}, false
	}
}

func toIdItem(arg phase4.Argument, tracker frontend.DiagnosticTracker) (ast.IdItem, bool) {
	switch data := arg.Arg.(type) {
	case phase4.FormulationArgumentData:
		if node, ok := formulation.ParseId(data.Text, tracker); ok {
			return ast.IdItem{
				RawText: data.Text,
				Root:    node,
				Label:   nil,
			}, true
		} else {
			return ast.Clause{}, false
		}
	default:
		tracker.Append(newError("Expected an id", arg.MetaData.Start))
		return ast.Clause{}, false
	}
}

func toTextItem(arg phase4.Argument, tracker frontend.DiagnosticTracker) (ast.TextItem, bool) {
	switch data := arg.Arg.(type) {
	case phase4.TextArgumentData:
		return ast.TextItem{
			RawText: data.Text,
		}, true
	default:
		tracker.Append(newError("Expected a text item", arg.MetaData.Start))
		return ast.TextItem{}, false
	}
}

//////////////////////// argument lists /////////////////////////////////

func toClauses(args []phase4.Argument, tracker frontend.DiagnosticTracker) ([]ast.Clause, bool) {
	result := make([]ast.Clause, 0)
	allOk := true
	for _, arg := range args {
		clause, ok := toClause(arg, tracker)
		allOk = allOk && ok
		if ok {
			result = append(result, clause)
		}
	}
	return result, allOk
}

func toSpecs(args []phase4.Argument, tracker frontend.DiagnosticTracker) ([]ast.Spec, bool) {
	result := make([]ast.Clause, 0)
	allOk := true
	for _, arg := range args {
		spec, ok := toSpec(arg, tracker)
		allOk = allOk && ok
		if ok {
			result = append(result, spec)
		}
	}
	return result, allOk
}

func toTargets(args []phase4.Argument, tracker frontend.DiagnosticTracker) ([]ast.Target, bool) {
	result := make([]ast.Target, 0)
	allOk := true
	for _, arg := range args {
		target, ok := toTarget(arg, tracker)
		allOk = allOk && ok
		if ok {
			result = append(result, target)
		}
	}
	return result, allOk
}

func toTextItems(args []phase4.Argument, tracker frontend.DiagnosticTracker) ([]ast.TextItem, bool) {
	result := make([]ast.TextItem, 0)
	allOk := true
	for _, arg := range args {
		textItem, ok := toTextItem(arg, tracker)
		allOk = allOk && ok
		if ok {
			result = append(result, textItem)
		}
	}
	return result, allOk
}

////////////////////////// support functions ////////////////////////////

func required[T any](value T, ok bool) T {
	return value
}

func optional[T any](value T, ok bool) T {
	return value
}

func newError(message string, position ast.Position) frontend.Diagnostic {
	return frontend.Diagnostic{
		Type:     frontend.Error,
		Origin:   frontend.Phase5ParserOrigin,
		Message:  message,
		Position: position,
	}
}

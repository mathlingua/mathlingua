/*
 * Copyright 2024 Dominic Kramer
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

package backend

import (
	"mathlingua/internal/ast"
	"mathlingua/internal/frontend"
)

func GetInputSummary(
	node ast.TopLevelItemKind,
	tracker *frontend.DiagnosticTracker,
) (*ast.InputSummary, bool) {
	switch entry := node.(type) {
	case *ast.DescribesGroup:
		return GetDescribesInputSummary(entry), true
	case *ast.DefinesGroup:
		return GetDefinesInputSummary(entry), true
	case *ast.StatesGroup:
		return GetStatesInputSummary(entry), true
	case *ast.CapturesGroup:
		return GetCapturesInputSummary(entry), true
	case *ast.AxiomGroup:
		return GetAxiomInputSummary(entry), true
	case *ast.ConjectureGroup:
		return GetConjectureInputSummary(entry), true
	case *ast.CorollaryGroup:
		return GetCorollaryInputSummary(entry), true
	case *ast.LemmaGroup:
		return GetLemmaInputSummary(entry), true
	default:
		return nil, false
	}
}

func GetDescribesInputSummary(describes *ast.DescribesGroup) *ast.InputSummary {
	if describes == nil {
		return nil
	}
	input, _ := toCommandPatternFromId(describes.Id)
	return &ast.InputSummary{
		Input: input,
	}
}

func GetDefinesInputSummary(defines *ast.DefinesGroup) *ast.InputSummary {
	if defines == nil {
		return nil
	}
	input, _ := toCommandPatternFromId(defines.Id)
	return &ast.InputSummary{
		Input: input,
	}
}

func GetStatesInputSummary(states *ast.StatesGroup) *ast.InputSummary {
	if states == nil {
		return nil
	}
	input, _ := toCommandPatternFromId(states.Id)
	return &ast.InputSummary{
		Input: input,
	}
}

func GetCapturesInputSummary(captures *ast.CapturesGroup) *ast.InputSummary {
	if captures == nil {
		return nil
	}
	input, _ := toCommandPatternFromId(captures.Id)
	return &ast.InputSummary{
		Input: input,
	}
}

func GetAxiomInputSummary(axiom *ast.AxiomGroup) *ast.InputSummary {
	if axiom == nil {
		return nil
	}
	var input ast.PatternKind
	if axiom.Id != nil {
		pattern, _ := toCommandPatternFromId(*axiom.Id)
		input = pattern
	}
	return &ast.InputSummary{
		Input: input,
	}
}

func GetConjectureInputSummary(conjecture *ast.ConjectureGroup) *ast.InputSummary {
	if conjecture == nil {
		return nil
	}
	var input ast.PatternKind
	if conjecture.Id != nil {
		pattern, _ := toCommandPatternFromId(*conjecture.Id)
		input = pattern
	}
	return &ast.InputSummary{
		Input: input,
	}
}

func GetTheoremInputSummary(theorem *ast.TheoremGroup) *ast.InputSummary {
	if theorem == nil {
		return nil
	}
	var input ast.PatternKind
	if theorem.Id != nil {
		pattern, _ := toCommandPatternFromId(*theorem.Id)
		input = pattern
	}
	return &ast.InputSummary{
		Input: input,
	}
}

func GetCorollaryInputSummary(corollary *ast.CorollaryGroup) *ast.InputSummary {
	if corollary == nil {
		return nil
	}
	var input ast.PatternKind
	if corollary.Id != nil {
		pattern, _ := toCommandPatternFromId(*corollary.Id)
		input = pattern
	}
	return &ast.InputSummary{
		Input: input,
	}
}

func GetLemmaInputSummary(lemma *ast.LemmaGroup) *ast.InputSummary {
	if lemma == nil {
		return nil
	}
	var input ast.PatternKind
	if lemma.Id != nil {
		pattern, _ := toCommandPatternFromId(*lemma.Id)
		input = pattern
	}
	return &ast.InputSummary{
		Input: input,
	}
}

////////////////////////////////////////////////////////////////////////////////////////////////////

func toCommandPatternFromId(id ast.IdItem) (ast.PatternKind, bool) {
	switch root := id.Root.(type) {
	case *ast.CommandId:
		pattern := ast.ToCommandPattern(*root)
		return &pattern, true
	case *ast.InfixCommandOperatorId:
		pattern := ast.ToInfixCommandPattern(*root)
		return &pattern, true
	default:
		return &ast.CommandPattern{}, false
	}
}

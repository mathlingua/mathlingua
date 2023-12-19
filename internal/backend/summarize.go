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

package backend

import (
	"mathlingua/internal/ast"
	"mathlingua/internal/frontend"
)

func Summarize(
	node ast.TopLevelItemKind,
	tracker frontend.IDiagnosticTracker,
) (SummaryKind, bool) {
	switch entry := node.(type) {
	case *ast.DescribesGroup:
		return SummarizeDescribes(entry), true
	case *ast.DefinesGroup:
		return SummarizeDefines(entry), true
	case *ast.StatesGroup:
		return SummarizeStates(entry), true
	case *ast.CapturesGroup:
		return SummarizeCaptures(entry), true
	case *ast.AxiomGroup:
		return SummarizeAxiom(entry), true
	case *ast.ConjectureGroup:
		return SummarizeConjecture(entry), true
	case *ast.CorollaryGroup:
		return SummarizeCorollary(entry), true
	case *ast.LemmaGroup:
		return SummarizeLemma(entry), true
	default:
		return nil, false
	}
}

func SummarizeDescribes(describes *ast.DescribesGroup) *DescribesSummary {
	if describes == nil {
		return nil
	}
	input, _ := toCommandPatternFromId(describes.Id)
	return &DescribesSummary{
		Input:   input,
		Written: GetWrittenSummaries(describes.Documented),
		Writing: GetWritingSummaries(describes.Documented),
		Called:  GetCalledSummaries(describes.Documented),
	}
}

func SummarizeDefines(defines *ast.DefinesGroup) *DefinesSummary {
	if defines == nil {
		return nil
	}
	input, _ := toCommandPatternFromId(defines.Id)
	return &DefinesSummary{
		Input:   input,
		Written: GetWrittenSummaries(defines.Documented),
		Writing: GetWritingSummaries(defines.Documented),
		Called:  GetCalledSummaries(defines.Documented),
	}
}

func SummarizeStates(states *ast.StatesGroup) *StatesSummary {
	if states == nil {
		return nil
	}
	input, _ := toCommandPatternFromId(states.Id)
	return &StatesSummary{
		Input:   input,
		Written: GetWrittenSummaries(states.Documented),
		Writing: GetWritingSummaries(states.Documented),
		Called:  GetCalledSummaries(states.Documented),
	}
}

func SummarizeCaptures(captures *ast.CapturesGroup) *CapturesSummary {
	if captures == nil {
		return nil
	}
	input, _ := toCommandPatternFromId(captures.Id)
	return &CapturesSummary{
		Input:   input,
		Written: GetWrittenSummaries(captures.Documented),
		Called:  GetCalledSummaries(captures.Documented),
	}
}

func SummarizeAxiom(axiom *ast.AxiomGroup) *AxiomSummary {
	if axiom == nil {
		return nil
	}
	var input *PatternKind
	if axiom.Id != nil {
		pattern, _ := toCommandPatternFromId(*axiom.Id)
		input = &pattern
	}
	return &AxiomSummary{
		Input:  input,
		Called: GetCalledSummaries(axiom.Documented),
	}
}

func SummarizeConjecture(conjecture *ast.ConjectureGroup) *ConjectureSummary {
	if conjecture == nil {
		return nil
	}
	var input *PatternKind
	if conjecture.Id != nil {
		pattern, _ := toCommandPatternFromId(*conjecture.Id)
		input = &pattern
	}
	return &ConjectureSummary{
		Input:  input,
		Called: GetCalledSummaries(conjecture.Documented),
	}
}

func SummarizeTheorem(theorem *ast.TheoremGroup) *TheoremSummary {
	if theorem == nil {
		return nil
	}
	var input *PatternKind
	if theorem.Id != nil {
		pattern, _ := toCommandPatternFromId(*theorem.Id)
		input = &pattern
	}
	return &TheoremSummary{
		Input:  input,
		Called: GetCalledSummaries(theorem.Documented),
	}
}

func SummarizeCorollary(corollary *ast.CorollaryGroup) *CorollarySummary {
	if corollary == nil {
		return nil
	}
	var input *PatternKind
	if corollary.Id != nil {
		pattern, _ := toCommandPatternFromId(*corollary.Id)
		input = &pattern
	}
	return &CorollarySummary{
		Input:  input,
		Called: GetCalledSummaries(corollary.Documented),
	}
}

func SummarizeLemma(lemma *ast.LemmaGroup) *LemmaSummary {
	if lemma == nil {
		return nil
	}
	var input *PatternKind
	if lemma.Id != nil {
		pattern, _ := toCommandPatternFromId(*lemma.Id)
		input = &pattern
	}
	return &LemmaSummary{
		Input:  input,
		Called: GetCalledSummaries(lemma.Documented),
	}
}

////////////////////////////////////////////////////////////////////////////////////////////////////

func toCommandPatternFromId(id ast.IdItem) (PatternKind, bool) {
	switch root := id.Root.(type) {
	case *ast.CommandId:
		pattern := ToCommandPattern(*root)
		return &pattern, true
	case *ast.InfixCommandOperatorId:
		pattern := ToInfixCommandPattern(*root)
		return &pattern, true
	default:
		return &CommandPattern{}, false
	}
}

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

/*
func Summarize(
	node ast.TopLevelItemKind,
	tracker *frontend.DiagnosticTracker,
) (ast.SummaryKind, bool) {
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

func SummarizeDescribes(describes *ast.DescribesGroup) *ast.DescribesSummary {
	if describes == nil {
		return nil
	}
	input, _ := toCommandPatternFromId(describes.Id)
	return &ast.DescribesSummary{
		Input:   input,
		Written: ast.GetWrittenSummaries(describes.Documented),
		Writing: ast.GetWritingSummaries(describes.Documented),
		Called:  ast.GetCalledSummaries(describes.Documented),
	}
}

func SummarizeDefines(defines *ast.DefinesGroup) *ast.DefinesSummary {
	if defines == nil {
		return nil
	}
	input, _ := toCommandPatternFromId(defines.Id)
	return &ast.DefinesSummary{
		Input:   input,
		Written: ast.GetWrittenSummaries(defines.Documented),
		Writing: ast.GetWritingSummaries(defines.Documented),
		Called:  ast.GetCalledSummaries(defines.Documented),
	}
}

func SummarizeStates(states *ast.StatesGroup) *ast.StatesSummary {
	if states == nil {
		return nil
	}
	input, _ := toCommandPatternFromId(states.Id)
	return &ast.StatesSummary{
		Input:   input,
		Written: ast.GetWrittenSummaries(states.Documented),
		Writing: ast.GetWritingSummaries(states.Documented),
		Called:  ast.GetCalledSummaries(states.Documented),
	}
}

func SummarizeCaptures(captures *ast.CapturesGroup) *ast.CapturesSummary {
	if captures == nil {
		return nil
	}
	input, _ := toCommandPatternFromId(captures.Id)
	return &ast.CapturesSummary{
		Input:   input,
		Written: ast.GetWrittenSummaries(captures.Documented),
		Called:  ast.GetCalledSummaries(captures.Documented),
	}
}

func SummarizeAxiom(axiom *ast.AxiomGroup) *ast.AxiomSummary {
	if axiom == nil {
		return nil
	}
	var input *ast.PatternKind
	if axiom.Id != nil {
		pattern, _ := toCommandPatternFromId(*axiom.Id)
		input = &pattern
	}
	return &ast.AxiomSummary{
		Input:  input,
		Called: ast.GetCalledSummaries(axiom.Documented),
	}
}

func SummarizeConjecture(conjecture *ast.ConjectureGroup) *ast.ConjectureSummary {
	if conjecture == nil {
		return nil
	}
	var input *ast.PatternKind
	if conjecture.Id != nil {
		pattern, _ := toCommandPatternFromId(*conjecture.Id)
		input = &pattern
	}
	return &ast.ConjectureSummary{
		Input:  input,
		Called: ast.GetCalledSummaries(conjecture.Documented),
	}
}

func SummarizeTheorem(theorem *ast.TheoremGroup) *ast.TheoremSummary {
	if theorem == nil {
		return nil
	}
	var input *ast.PatternKind
	if theorem.Id != nil {
		pattern, _ := toCommandPatternFromId(*theorem.Id)
		input = &pattern
	}
	return &ast.TheoremSummary{
		Input:  input,
		Called: ast.GetCalledSummaries(theorem.Documented),
	}
}

func SummarizeCorollary(corollary *ast.CorollaryGroup) *ast.CorollarySummary {
	if corollary == nil {
		return nil
	}
	var input *ast.PatternKind
	if corollary.Id != nil {
		pattern, _ := toCommandPatternFromId(*corollary.Id)
		input = &pattern
	}
	return &ast.CorollarySummary{
		Input:  input,
		Called: ast.GetCalledSummaries(corollary.Documented),
	}
}

func SummarizeLemma(lemma *ast.LemmaGroup) *ast.LemmaSummary {
	if lemma == nil {
		return nil
	}
	var input *ast.PatternKind
	if lemma.Id != nil {
		pattern, _ := toCommandPatternFromId(*lemma.Id)
		input = &pattern
	}
	return &ast.LemmaSummary{
		Input:  input,
		Called: ast.GetCalledSummaries(lemma.Documented),
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
*/

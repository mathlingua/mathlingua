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

func GetDocumentedSummary(
	node ast.TopLevelItemKind,
	tracker *frontend.DiagnosticTracker,
) (*ast.DocumentedSummary, bool) {
	switch entry := node.(type) {
	case *ast.DescribesGroup:
		return GetDescribesDocumentedSummary(entry), true
	case *ast.DefinesGroup:
		return GetDefinesDocumentedSummary(entry), true
	case *ast.StatesGroup:
		return GetStatesDocumentedSummary(entry), true
	case *ast.CapturesGroup:
		return GetCapturesDocumentedSummary(entry), true
	case *ast.AxiomGroup:
		return GetAxiomDocumentedSummary(entry), true
	case *ast.ConjectureGroup:
		return GetConjectureDocumentedSummary(entry), true
	case *ast.CorollaryGroup:
		return GetCorollaryDocumentedSummary(entry), true
	case *ast.LemmaGroup:
		return GetLemmaDocumentedSummary(entry), true
	default:
		return nil, false
	}
}

func GetDescribesDocumentedSummary(describes *ast.DescribesGroup) *ast.DocumentedSummary {
	if describes == nil {
		return nil
	}
	return &ast.DocumentedSummary{
		Written: ast.GetWrittenSummaries(describes.Documented),
		Writing: ast.GetWritingSummaries(describes.Documented),
		Called:  ast.GetCalledSummaries(describes.Documented),
	}
}

func GetDefinesDocumentedSummary(defines *ast.DefinesGroup) *ast.DocumentedSummary {
	if defines == nil {
		return nil
	}
	return &ast.DocumentedSummary{
		Written: ast.GetWrittenSummaries(defines.Documented),
		Writing: ast.GetWritingSummaries(defines.Documented),
		Called:  ast.GetCalledSummaries(defines.Documented),
	}
}

func GetStatesDocumentedSummary(states *ast.StatesGroup) *ast.DocumentedSummary {
	if states == nil {
		return nil
	}
	return &ast.DocumentedSummary{
		Written: ast.GetWrittenSummaries(states.Documented),
		Writing: ast.GetWritingSummaries(states.Documented),
		Called:  ast.GetCalledSummaries(states.Documented),
	}
}

func GetCapturesDocumentedSummary(captures *ast.CapturesGroup) *ast.DocumentedSummary {
	if captures == nil {
		return nil
	}
	return &ast.DocumentedSummary{
		Written: ast.GetWrittenSummaries(captures.Documented),
		Called:  ast.GetCalledSummaries(captures.Documented),
	}
}

func GetAxiomDocumentedSummary(axiom *ast.AxiomGroup) *ast.DocumentedSummary {
	if axiom == nil {
		return nil
	}
	return &ast.DocumentedSummary{
		Called: ast.GetCalledSummaries(axiom.Documented),
	}
}

func GetConjectureDocumentedSummary(conjecture *ast.ConjectureGroup) *ast.DocumentedSummary {
	if conjecture == nil {
		return nil
	}
	return &ast.DocumentedSummary{
		Called: ast.GetCalledSummaries(conjecture.Documented),
	}
}

func GetTheoremDocumentedSummary(theorem *ast.TheoremGroup) *ast.DocumentedSummary {
	if theorem == nil {
		return nil
	}
	return &ast.DocumentedSummary{
		Called: ast.GetCalledSummaries(theorem.Documented),
	}
}

func GetCorollaryDocumentedSummary(corollary *ast.CorollaryGroup) *ast.DocumentedSummary {
	if corollary == nil {
		return nil
	}
	return &ast.DocumentedSummary{
		Called: ast.GetCalledSummaries(corollary.Documented),
	}
}

func GetLemmaDocumentedSummary(lemma *ast.LemmaGroup) *ast.DocumentedSummary {
	if lemma == nil {
		return nil
	}
	return &ast.DocumentedSummary{
		Called: ast.GetCalledSummaries(lemma.Documented),
	}
}

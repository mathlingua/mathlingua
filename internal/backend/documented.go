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
	"fmt"
	"mathlingua/internal/ast"
	"mathlingua/internal/frontend"
	"mathlingua/internal/mlglib"
	"regexp"
	"strings"
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
		Written: GetWrittenSummaries(describes.Documented),
		Writing: GetWritingSummaries(describes.Documented),
		Called:  GetCalledSummaries(describes.Documented),
	}
}

func GetDefinesDocumentedSummary(defines *ast.DefinesGroup) *ast.DocumentedSummary {
	if defines == nil {
		return nil
	}
	return &ast.DocumentedSummary{
		Written: GetWrittenSummaries(defines.Documented),
		Writing: GetWritingSummaries(defines.Documented),
		Called:  GetCalledSummaries(defines.Documented),
	}
}

func GetStatesDocumentedSummary(states *ast.StatesGroup) *ast.DocumentedSummary {
	if states == nil {
		return nil
	}
	return &ast.DocumentedSummary{
		Written: GetWrittenSummaries(states.Documented),
		Writing: GetWritingSummaries(states.Documented),
		Called:  GetCalledSummaries(states.Documented),
	}
}

func GetCapturesDocumentedSummary(captures *ast.CapturesGroup) *ast.DocumentedSummary {
	if captures == nil {
		return nil
	}
	return &ast.DocumentedSummary{
		Written: GetWrittenSummaries(captures.Documented),
		Called:  GetCalledSummaries(captures.Documented),
	}
}

func GetAxiomDocumentedSummary(axiom *ast.AxiomGroup) *ast.DocumentedSummary {
	if axiom == nil {
		return nil
	}
	return &ast.DocumentedSummary{
		Called: GetCalledSummaries(axiom.Documented),
	}
}

func GetConjectureDocumentedSummary(conjecture *ast.ConjectureGroup) *ast.DocumentedSummary {
	if conjecture == nil {
		return nil
	}
	return &ast.DocumentedSummary{
		Called: GetCalledSummaries(conjecture.Documented),
	}
}

func GetTheoremDocumentedSummary(theorem *ast.TheoremGroup) *ast.DocumentedSummary {
	if theorem == nil {
		return nil
	}
	return &ast.DocumentedSummary{
		Called: GetCalledSummaries(theorem.Documented),
	}
}

func GetCorollaryDocumentedSummary(corollary *ast.CorollaryGroup) *ast.DocumentedSummary {
	if corollary == nil {
		return nil
	}
	return &ast.DocumentedSummary{
		Called: GetCalledSummaries(corollary.Documented),
	}
}

func GetLemmaDocumentedSummary(lemma *ast.LemmaGroup) *ast.DocumentedSummary {
	if lemma == nil {
		return nil
	}
	return &ast.DocumentedSummary{
		Called: GetCalledSummaries(lemma.Documented),
	}
}

func ToCalledSummaries(node ast.CalledGroup) []ast.CalledSummary {
	result := make([]ast.CalledSummary, 0)
	for _, item := range node.Called.Called {
		raw := item.RawText
		parsed, err := ParseCalledWritten(raw)
		result = append(result, ast.CalledSummary{
			RawCalled:    raw,
			ParsedCalled: parsed,
			Errors:       errorToString(err),
		})
	}
	return result
}

func ToWrittenSummaries(node ast.WrittenGroup) []ast.WrittenSummary {
	result := make([]ast.WrittenSummary, 0)
	for _, item := range node.Written.Written {
		raw := item.RawText
		parsed, err := ParseCalledWritten(raw)
		result = append(result, ast.WrittenSummary{
			RawWritten:    raw,
			ParsedWritten: parsed,
			Errors:        errorToString(err),
		})
	}
	return result
}

func ToWritingSummaries(node ast.WritingGroup) []ast.WritingSummary {
	result := make([]ast.WritingSummary, 0)
	for _, item := range node.Writing.Writing {
		raw := item.RawText
		parsed, err := ParseCalledWritten(raw)
		result = append(result, ast.WritingSummary{
			RawWritten:    raw,
			ParsedWriting: parsed,
			Errors:        errorToString(err),
		})
	}
	return result
}

func GetWrittenSummaries(documented *ast.DocumentedSection) []ast.WrittenSummary {
	summaries := make([]ast.WrittenSummary, 0)
	if documented == nil {
		return summaries
	}
	for _, docItem := range documented.Documented {
		switch item := docItem.(type) {
		case *ast.WrittenGroup:
			for _, text := range item.Written.Written {
				raw := text.RawText
				parsed, err := ParseCalledWritten(raw)
				summaries = append(summaries, ast.WrittenSummary{
					RawWritten:    raw,
					ParsedWritten: parsed,
					Errors:        errorToString(err),
				})
			}
		}
	}
	return summaries
}

func GetWritingSummaries(documented *ast.DocumentedSection) []ast.WritingSummary {
	summaries := make([]ast.WritingSummary, 0)
	if documented == nil {
		return summaries
	}
	for _, docItem := range documented.Documented {
		switch item := docItem.(type) {
		case *ast.WritingGroup:
			for _, as := range item.Writing.Writing {
				raw := as.RawText
				parsed, err := ParseCalledWritten(raw)
				summaries = append(summaries, ast.WritingSummary{
					RawWritten:    raw,
					ParsedWriting: parsed,
					Errors:        errorToString(err),
				})
			}
		}
	}
	return summaries
}

func GetCalledSummaries(documented *ast.DocumentedSection) []ast.CalledSummary {
	summaries := make([]ast.CalledSummary, 0)
	if documented == nil {
		return summaries
	}
	for _, docItem := range documented.Documented {
		switch item := docItem.(type) {
		case *ast.CalledGroup:
			for _, text := range item.Called.Called {
				raw := text.RawText
				parsed, err := ParseCalledWritten(raw)
				summaries = append(summaries, ast.CalledSummary{
					RawCalled:    raw,
					ParsedCalled: parsed,
					Errors:       errorToString(err),
				})
			}
		}
	}
	return summaries
}

func ParseCalledWritten(text string) ([]ast.TextItemKind, error) {
	nameMatch := regexp.MustCompile(`[a-zA-Z0-9]+(\+|-|=)?\?`)
	result := make([]ast.TextItemKind, 0)
	for len(text) > 0 {
		indices := nameMatch.FindStringIndex(text)
		if indices == nil {
			result = append(result, &ast.StringItem{
				Text: text,
			})
			break
		}
		prefix := text[0:indices[0]]

		if len(prefix) > 0 {
			result = append(result, &ast.StringItem{
				Text: prefix,
			})
		}

		// use indices[1]-1 to ignore the trailing ?
		match := text[indices[0] : indices[1]-1]
		name := ""
		nameSuffix := ""
		if strings.HasSuffix(match, "+") || strings.HasSuffix(match, "-") ||
			strings.HasSuffix(match, "=") {
			name = match[0 : len(match)-1]
			nameSuffix = match[len(match)-1:]
		} else {
			name = match
		}

		suffix := text[indices[1]:]

		text = suffix

		inner := ""
		if len(text) > 0 && text[0] == '{' {
			inner += "{"
			stack := mlglib.NewStack[byte]()
			stack.Push('{')
			i := 1
			for i < len(text) {
				c := text[i]
				i += 1
				if c == '{' {
					stack.Push(c)
				} else if c == '}' {
					if !stack.IsEmpty() && stack.Peek() == '{' {
						stack.Pop()
					}
					if stack.IsEmpty() {
						inner += string(c)
						break
					}
				}
				inner += string(c)
			}
			text = text[len(inner):]

			// inner is of the form {...}, so remove the leading { and trailing }
			inner = inner[1 : len(inner)-1]

			innerPrefix := ""
			hasDotDotDot1 := false
			innerInfix := ""
			hasDotDotDot2 := false
			innerSuffix := ""

			j := 0
			for j < len(inner) && inner[j] != '.' {
				innerPrefix += string(inner[j])
				j += 1
			}

			if j+2 < len(inner) && inner[j] == '.' && inner[j+1] == '.' && inner[j+2] == '.' {
				hasDotDotDot1 = true
				j += 3
			}

			for j < len(inner) && inner[j] != '.' {
				innerInfix += string(inner[j])
				j += 1
			}

			if j+2 < len(inner) && inner[j] == '.' && inner[j+1] == '.' && inner[j+2] == '.' {
				hasDotDotDot2 = true
				j += 3
			}

			for j < len(inner) {
				innerSuffix += string(inner[j])
				j += 1
			}

			if !hasDotDotDot1 && !hasDotDotDot2 {
				return nil, fmt.Errorf("At least one .... expected")
			}

			numNonEmpty := 0
			if innerPrefix != "" {
				numNonEmpty++
			}

			if innerInfix != "" {
				numNonEmpty++
			}

			if innerSuffix != "" {
				numNonEmpty++
			}

			if numNonEmpty != 1 {
				return nil, fmt.Errorf("Expected one of the forms x..., ...x, or ...x...")
			}

			result = append(result, &ast.SubstitutionItem{
				Name:       name,
				NameSuffix: nameSuffix,
				IsVarArg:   true,
				Prefix:     innerPrefix,
				Infix:      innerInfix,
				Suffix:     innerSuffix,
			})
		} else {
			result = append(result, &ast.SubstitutionItem{
				Name:       name,
				NameSuffix: nameSuffix,
				IsVarArg:   false,
			})
		}
	}
	return result, nil
}

func GetResolvedWritten(summary ast.DocumentedSummary) ([]ast.TextItemKind, bool) {
	called := getSingleCalled(summary.Called)
	written := getSingleWritten(summary.Written)

	if written != nil {
		return *written, true
	}

	if called != nil {
		result := make([]ast.TextItemKind, 0)
		result = append(result, &ast.StringItem{
			Text: "\\textrm{",
		})
		result = append(result, *called...)
		result = append(result, &ast.StringItem{
			Text: "}",
		})
		return result, true
	}

	return nil, false
}

////////////////////////////////////////////////////////////////////////////////////////////////////

func getSingleCalled(called []ast.CalledSummary) *[]ast.TextItemKind {
	if len(called) == 0 {
		return nil
	}
	text := called[0].ParsedCalled
	return &text
}

func getSingleWritten(written []ast.WrittenSummary) *[]ast.TextItemKind {
	if len(written) == 0 {
		return nil
	}
	text := written[0].ParsedWritten
	return &text
}

func errorToString(err error) []string {
	if err == nil {
		return []string{}
	}
	return []string{err.Error()}
}

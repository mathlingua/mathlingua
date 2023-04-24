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

import "mathlingua/internal/ast"

type CalledSummary struct {
	Called string
}

type WrittenSummary struct {
	Written string
}

type WritingSummary struct {
	Form    PatternType
	Written string
}

func ToCalledSummaries(node ast.CalledGroup) []CalledSummary {
	result := make([]CalledSummary, 0)
	for _, item := range node.Called.Called {
		result = append(result, CalledSummary{
			Called: item.RawText,
		})
	}
	return result
}

func ToWrittenSummaries(node ast.WrittenGroup) []WrittenSummary {
	result := make([]WrittenSummary, 0)
	for _, item := range node.Written.Written {
		result = append(result, WrittenSummary{
			Written: item.RawText,
		})
	}
	return result
}

func ToWritingSummaries(node ast.WritingGroup) []WritingSummary {
	result := make([]WritingSummary, 0)
	for _, item := range node.As.As {
		result = append(result, WritingSummary{
			Form:    ToPatternFromTarget(node.Writing.Writing),
			Written: item.RawText,
		})
	}
	return result
}

func GetWrittenSummaries(documented *ast.DocumentedSection) []WrittenSummary {
	summaries := make([]WrittenSummary, 0)
	if documented == nil {
		return summaries
	}
	for _, docItem := range documented.Documented {
		switch item := docItem.(type) {
		case *ast.WrittenGroup:
			for _, text := range item.Written.Written {
				summaries = append(summaries, WrittenSummary{
					Written: text.RawText,
				})
			}
		}
	}
	return summaries
}

func GetWritingSummaries(documented *ast.DocumentedSection) []WritingSummary {
	summaries := make([]WritingSummary, 0)
	if documented == nil {
		return summaries
	}
	for _, docItem := range documented.Documented {
		switch item := docItem.(type) {
		case *ast.WritingGroup:
			for _, as := range item.As.As {
				summaries = append(summaries, WritingSummary{
					Form:    ToPatternFromTarget(item.Writing.Writing),
					Written: as.RawText,
				})
			}
		}
	}
	return summaries
}

func GetCalledSummaries(documented *ast.DocumentedSection) []CalledSummary {
	summaries := make([]CalledSummary, 0)
	if documented == nil {
		return summaries
	}
	for _, docItem := range documented.Documented {
		switch item := docItem.(type) {
		case *ast.CalledGroup:
			for _, text := range item.Called.Called {
				summaries = append(summaries, CalledSummary{
					Called: text.RawText,
				})
			}
		}
	}
	return summaries
}

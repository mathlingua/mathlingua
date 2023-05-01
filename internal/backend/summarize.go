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

func Summarize(node ast.TopLevelItemType, tracker *frontend.DiagnosticTracker) (SummaryType, bool) {
	switch entry := node.(type) {
	case *ast.DescribesGroup:
		return SummarizeDescribes(entry), true
	case *ast.DefinesGroup:
		return SummarizeDefines(entry), true
	case *ast.StatesGroup:
		return SummarizeStates(entry), true
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
		Input:   &input,
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
		Input:   &input,
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
		Input:   &input,
		Written: GetWrittenSummaries(states.Documented),
		Writing: GetWritingSummaries(states.Documented),
		Called:  GetCalledSummaries(states.Documented),
	}
}

////////////////////////////////////////////////////////////////////////////////////////////////////

func toCommandPatternFromId(id ast.IdItem) (CommandPattern, bool) {
	switch root := id.Root.(type) {
	case *ast.CommandId:
		return ToCommandPattern(*root), true
	default:
		return CommandPattern{}, false
	}
}

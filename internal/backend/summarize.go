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
	return nil
}

func SummarizeDefines(defines *ast.DefinesGroup) *DefinesSummary {
	return nil
}

func SummarizeStates(states *ast.StatesGroup) *StatesSummary {
	return nil
}

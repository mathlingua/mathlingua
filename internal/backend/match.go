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

// There are a couple scenarios where matching needs to occur:
// 1. Given a command expression, one needs to match the command's signature to a Describes
//    or Defines, determine if the expression matches the Describes/Defines input pattern,
//    and map the expression's argument's expressions to string names in the input pattern.
//    (This will be used for performing resolution of summaries to match their exact usage.)
// 2. Given an expression alias or a spec alias and an expression (assuming it has already been
//    determined, using type information, that the alias is what  should be used for the expression)
//    determine if the alias's  input pattern matches the expression, and map parts
//    of the expression to the names in the input pattern.
// 3. Given the left-hand-side, input  pattern, of an expression or spec alias, map the
//    names in the pattern to expression in the right-hand-side of the alias.
// Note: 2 and 3 will be used together to expand aliases inline in expressions.

type MatchResult struct {
	Mapping         map[string]ast.ExpressionType
	Diagnostics     []frontend.Diagnostic
	MatchMakesSense bool
}

func MatchExpToNames(pattern PatternType, exp ast.ExpressionType) (MatchResult, bool) {
	return MatchResult{}, false
}

func ExpandAliasInline(node ast.MlgNodeType, aliasSummary AliasSummaryType) bool {
	return false
}

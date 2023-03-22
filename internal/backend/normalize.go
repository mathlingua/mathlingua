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

package backend

import (
	"fmt"
	"mathlingua/internal/ast"
	"mathlingua/internal/frontend"
	"strings"

	"github.com/google/uuid"
)

func AppendMetaIds(startText string) (string, error) {
	endText := ""
	lines := strings.Split(startText, "\n")
	i := 0
	isInTopLevel := false
	hasComment := false
	hasMetaId := false
	for i < len(lines) {
		cur := lines[i]
		i++

		trimmedCur := strings.TrimSpace(cur)
		if strings.HasPrefix(trimmedCur, "::") {
			endText += cur
			endText += "\n"

			isInTopLevel = false
			hasComment = false
			hasMetaId = false

			if trimmedCur == "::" || !strings.HasSuffix(trimmedCur, "::") {
				// it is a textblock
				for i < len(lines) {
					next := lines[i]
					i++
					endText += next
					endText += "\n"
					if strings.HasSuffix(strings.TrimSpace(next), "::") {
						break
					}
				}
			}
		} else if isInTopLevel && strings.HasPrefix(cur, "--") {
			hasComment = true
			endText += cur
			endText += "\n"
		} else if isInTopLevel && strings.HasPrefix(cur, "Id:") {
			hasMetaId = true
			endText += cur
			endText += "\n"
		} else if strings.HasPrefix(cur, "Defines:") ||
			strings.HasPrefix(cur, "Describes:") ||
			strings.HasPrefix(cur, "States:") ||
			strings.HasPrefix(cur, "Axiom:") ||
			strings.HasPrefix(cur, "Conjecture:") ||
			strings.HasPrefix(cur, "Theorem:") ||
			strings.HasPrefix(cur, "Topic:") ||
			strings.HasPrefix(cur, "Resource:") ||
			strings.HasPrefix(cur, "Person:") ||
			strings.HasPrefix(cur, "Specify:") ||
			strings.HasPrefix(cur, "Proof:") {
			isInTopLevel = true
			endText += cur
			endText += "\n"
		} else if cur == "" {
			if isInTopLevel {
				if !hasComment {
					endText += "------------------------------------------\n"
				}
				if !hasMetaId {
					newId, _ := uuid.NewRandom()
					endText += fmt.Sprintf("Id: \"%s\"\n", newId)
				}
			}
			isInTopLevel = false
			hasComment = false
			hasMetaId = false

			endText += cur
			endText += "\n"
		} else {
			endText += cur
			endText += "\n"
		}
	}

	if strings.HasSuffix(endText, "\n") {
		endText = string(endText[0 : len(endText)-1])
	}

	return endText, nil
}

// Normalizes the given AST node in-place, which includes:
//   - update any place where an identifier is introduced to ensure it has any input and
//     output identifiers specified.  That is if `f(x)` is introduced, it is replaced with
//     something like `f(x) := y` where the output has an identifier.  Also, if `(a, b, c)`
//     is introduced, then it replaced with something like `X := (a, b, c)` where an
//     identifier `X` for the tuple itself is introduced.
//   - Any alias in formulations are expanded so that aliases are not needed anymore.
func Normalize[T ast.MlgNodeType](node T, tracker *frontend.DiagnosticTracker) {
}

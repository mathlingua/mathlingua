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

package phase5

import (
	"fmt"
	"mathlingua/internal/ast"
	"mathlingua/internal/frontend"
	"mathlingua/internal/frontend/phase4"
	"mathlingua/internal/mlglib"
	"strings"
)

func IdentifySections(
	sections []phase4.Section,
	tracker frontend.DiagnosticTracker,
	expected ...string) (map[string]phase4.Section, bool) {
	// the pattern message to use in diagnostic messages
	pattern := ""
	for i, name := range expected {
		if i > 0 {
			pattern += "\n"
		}
		pattern += name + ":"
	}

	sectionQueue := mlglib.NewQueue[phase4.Section]()
	for _, section := range sections {
		sectionQueue.Push(section)
	}

	expectedQueue := mlglib.NewQueue[string]()
	for _, e := range expected {
		expectedQueue.Push(e)
	}

	usedSectionNames := make(map[string]int)
	result := make(map[string]phase4.Section)

	for !sectionQueue.IsEmpty() && !expectedQueue.IsEmpty() {
		nextSection := sectionQueue.Peek()
		maybeName := expectedQueue.Peek()

		isOptional := strings.HasSuffix(maybeName, "?")
		trueName := maybeName
		if isOptional {
			trueName = strings.TrimSuffix(maybeName, "?")
		}

		key := trueName
		if count, ok := usedSectionNames[trueName]; ok {
			key = fmt.Sprintf("%s%d", trueName, count)
			usedSectionNames[trueName] = count + 1
		} else {
			usedSectionNames[trueName] = 0
		}

		if nextSection.Name == trueName {
			result[key] = nextSection
			// the expected name and Section have both been used so move past them
			sectionQueue.Pop()
			expectedQueue.Pop()
		} else if isOptional {
			// The Section found doesn't match the expected name but the expected name is optional.
			// So move past the expected name but don't move past the Section so it can be processed
			// again in the next run of the loop.
			expectedQueue.Pop()
		} else {
			tracker.Append(frontend.Diagnostic{
				Type:   frontend.Error,
				Origin: frontend.Phase5ParserOrigin,
				Message: "For pattern:\n\n" +
					pattern +
					"\n\nExpected '" +
					trueName +
					"' but found '" +
					nextSection.Name +
					"'",
				Position: nextSection.MetaData.Start,
			})
			return nil, false
		}
	}

	if !sectionQueue.IsEmpty() {
		peek := sectionQueue.Peek()
		tracker.Append(frontend.Diagnostic{
			Type:   frontend.Error,
			Origin: frontend.Phase5ParserOrigin,
			Message: "For pattern:\n\n" + pattern +
				"\n\nUnexpected section '" + peek.Name + "'",
			Position: peek.MetaData.Start,
		})
	}

	nextExpected := ""
	for !expectedQueue.IsEmpty() {
		exp := expectedQueue.Pop()
		if !strings.HasSuffix(exp, "?") {
			// trim the ?
			nextExpected = strings.TrimSuffix(exp, "?")
			break
		}
	}

	start := ast.Position{}
	if len(sections) > 0 {
		sect := sections[0]
		start = sect.MetaData.Start
	}

	if len(nextExpected) > 0 {
		tracker.Append(frontend.Diagnostic{
			Type:   frontend.Error,
			Origin: frontend.Phase5ParserOrigin,
			Message: "For pattern:\n\n" + pattern +
				"\n\nExpected a section '" + nextExpected + "'",
			Position: start,
		})
		return nil, false
	}

	return result, true
}

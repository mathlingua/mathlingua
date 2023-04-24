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

package phase2

import (
	"mathlingua/internal/frontend"
	"mathlingua/internal/frontend/phase1"
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestPhase2LexerIndent(t *testing.T) {
	tracker := frontend.NewDiagnosticTracker(false)
	lexer1 := phase1.NewLexer(`
a:
. b:
  . c:
d:
`, "", tracker)
	lexer2 := NewLexer(lexer1, "", tracker)

	actual := "\n"
	for lexer2.HasNext() {
		actual += lexer2.Next().Text + "\n"
	}

	expected := `
<Newline>
a
:
<Newline>
<Indent>
<DotSpace>
b
:
<Newline>
<Indent>
<DotSpace>
c
:
<Newline>
<UnIndent>
<UnIndent>
d
:
<Newline>
<LineBreak>
`

	assert.Equal(t, expected, actual)
	assert.Equal(t, []frontend.Diagnostic{}, tracker.Diagnostics())
}

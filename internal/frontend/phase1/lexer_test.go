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

package phase1

import (
	"mathlingua/internal/frontend"
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestPhase1Lexer(t *testing.T) {
	tracker := frontend.NewDiagnosticTracker(false)
	lexer1 := NewLexer(`
a: x, y
. b: 123
  . c:
xyz: "abc",'123'
`, "", tracker)

	actual := "\n"
	for lexer1.HasNext() {
		actual += lexer1.Next().Text + "\n"
	}

	expected := `
<Newline>
a
:
x
,
y
<Newline>
<DotSpace>
b
:
123
<Newline>
<Space>
<Space>
<DotSpace>
c
:
<Newline>
xyz
:
abc
,
123
<Newline>
<Newline>
<Newline>
<Newline>
`

	assert.Equal(t, expected, actual)
	assert.Equal(t, []frontend.Diagnostic{}, tracker.Diagnostics())
}

func TestPhase1LexerParsesId(t *testing.T) {
	tracker := frontend.NewDiagnosticTracker(false)
	lexer1 := NewLexer("[some[id[x]]]", "", tracker)

	actual := "\n"
	for lexer1.HasNext() {
		actual += lexer1.Next().Text + "\n"
	}

	expected := "\nsome[id[x]]\n<Newline>\n<Newline>\n<Newline>\n"

	assert.Equal(t, expected, actual)
	assert.Equal(t, []frontend.Diagnostic{}, tracker.Diagnostics())
}

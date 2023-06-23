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

package phase4

import (
	"mathlingua/internal/ast"
	"mathlingua/internal/frontend"
	"mathlingua/internal/frontend/phase1"
	"mathlingua/internal/frontend/phase2"
	"mathlingua/internal/frontend/phase3"
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestParsesGroupWithLabel(t *testing.T) {
	text := `
[some.label]
given: x
then: 'y'
`
	path := ast.ToPath("/")
	tracker := frontend.NewDiagnosticTracker()

	lexer1 := phase1.NewLexer(text, path, tracker)
	lexer2 := phase2.NewLexer(lexer1, path, tracker)
	lexer3 := phase3.NewLexer(lexer2, path, tracker)
	doc := Parse(lexer3, path, tracker)

	writer := NewTextCodeWriter()
	doc.ToCode(writer)

	expected := `[some.label]
given: x
then: 'y'


`
	actual := writer.String()

	assert.Equal(t, expected, actual)
}

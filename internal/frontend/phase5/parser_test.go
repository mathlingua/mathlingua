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
	"mathlingua/internal/ast"
	"mathlingua/internal/frontend"
	"mathlingua/internal/frontend/phase1"
	"mathlingua/internal/frontend/phase2"
	"mathlingua/internal/frontend/phase3"
	"mathlingua/internal/frontend/phase4"
	"mathlingua/internal/mlglib"
	"testing"

	"github.com/stretchr/testify/assert"
)

func parse(text string) (ast.Document, frontend.DiagnosticTracker) {
	tracker := frontend.NewDiagnosticTracker()

	lexer1 := phase1.NewLexer(text, tracker)
	lexer2 := phase2.NewLexer(lexer1, tracker)
	lexer3 := phase3.NewLexer(lexer2, tracker)

	root := phase4.Parse(lexer3, tracker)
	doc, _ := Parse(root, tracker)

	return doc, tracker
}

func runTest(t *testing.T, input string, expected string) {
	doc, tracker := parse(input)
	actual := mlglib.PrettyPrint(doc)

	assert.Equal(t, expected, actual)
	assert.Equal(t, 0, len(tracker.Diagnostics()))
}

func TestBasicTheorem(t *testing.T) {
	input := `Theorem:
given: x
then: 'x'`
	expected := `ast.Document{
  Items: []ast.TopLevelItemType{
    ast.TheoremGroup{
      Id: nil,
      Theorem: ast.TheoremSection{
        Theorem: []ast.TextItem{},
      },
      Given: &ast.GivenSection{
        Given: []ast.Formulation[ast.NodeType]{
          ast.Formulation[ast.NodeType]{
            RawText: "x",
            Root: ast.NameForm{
              Text: "x",
              IsStropped: false,
              HasQuestionMark: false,
              VarArg: ast.VarArgData{
                IsVarArg: false,
                VarArgCount: nil,
              },
            },
            Label: nil,
          },
        },
      },
      Where: nil,
      SuchThat: nil,
      Then: ast.ThenSection{
        Clauses: []ast.Clause{
          ast.Formulation[ast.NodeType]{
            RawText: "x",
            Root: ast.NameForm{
              Text: "x",
              IsStropped: false,
              HasQuestionMark: false,
              VarArg: ast.VarArgData{
                IsVarArg: false,
                VarArgCount: nil,
              },
            },
            Label: nil,
          },
        },
      },
      Iff: nil,
      Using: nil,
      Proof: nil,
      Documented: nil,
      References: nil,
      Metadata: nil,
    },
  },
}`
	runTest(t, input, expected)
}

/*
 * Copyright 2024 Dominic Kramer
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

package formulation

import (
	"fmt"
	"mathlingua/internal/ast"
	"mathlingua/internal/frontend"
	"mathlingua/internal/mlglib"
	"os"
	"path"
	"strings"
	"testing"

	"github.com/stretchr/testify/assert"
)

func runParserSmokeTest(
	t *testing.T,
	name string,
	parse func(text string) (string, *frontend.DiagnosticTracker, bool),
) {
	inputTextData, err := os.ReadFile(
		path.Join("..", "..", "..", "testdata", fmt.Sprintf("formulation_%s.txt", name)))
	assert.Nil(t, err)
	inputLines := strings.Split(string(inputTextData), "\n")

	expectedOutputData, err := os.ReadFile(
		path.Join("..", "..", "..", "testdata", fmt.Sprintf("formulation_%s_expected.txt", name)))
	assert.Nil(t, err)
	exptedOutputLines := strings.Split(string(expectedOutputData), "\n")

	for i, line := range inputLines {
		actualOutput, tracker, ok := parse(line)

		diagnostics := ""
		for _, diag := range tracker.Diagnostics() {
			diagnostics += fmt.Sprintf("Line %d: %s (%d, %d): %s [%s]\n", i+1, diag.Type, diag.Position.Row,
				diag.Position.Column, diag.Message, diag.Origin)
		}

		assert.Equal(t, "", diagnostics)
		assert.True(t, ok)
		assert.Equal(t, 0, len(tracker.Diagnostics()))

		assert.True(t, i < len(exptedOutputLines),
			fmt.Sprintf("Expected line %d to exist in formulation_%s_expected.txt", i+1, name))
		expectedOutput := fmt.Sprintf("Line %d: %s", i+1, exptedOutputLines[i])

		assert.Equal(t, expectedOutput, fmt.Sprintf("Line %d: %s", i+1, actualOutput))
	}
}

func TestExpressionParserSmoke(t *testing.T) {
	runParserSmokeTest(
		t,
		"expression",
		func(text string) (string, *frontend.DiagnosticTracker, bool) {
			path := ast.ToPath("/")
			tracker := frontend.NewDiagnosticTracker()
			start := ast.Position{
				Offset: 0,
				Row:    0,
				Column: 0,
			}
			keyGenerator := mlglib.NewKeyGenerator()
			exp, ok := ParseExpression(path, text, start, tracker, keyGenerator)
			output := ""
			if ok {
				output = exp.ToCode(ast.NoOp)
			}
			return output, tracker, ok
		})
}

func TestFormParserSmoke(t *testing.T) {
	runParserSmokeTest(
		t,
		"form",
		func(text string) (string, *frontend.DiagnosticTracker, bool) {
			path := ast.ToPath("/")
			tracker := frontend.NewDiagnosticTracker()
			start := ast.Position{
				Offset: 0,
				Row:    0,
				Column: 0,
			}
			keyGenerator := mlglib.NewKeyGenerator()
			form, ok := ParseForm(path, text, start, tracker, keyGenerator)
			output := ""
			if ok {
				output = form.ToCode(ast.NoOp)
			}
			return output, tracker, ok
		})
}

func TestIdParserSmoke(t *testing.T) {
	runParserSmokeTest(
		t,
		"id",
		func(text string) (string, *frontend.DiagnosticTracker, bool) {
			path := ast.ToPath("/")
			tracker := frontend.NewDiagnosticTracker()
			start := ast.Position{
				Offset: 0,
				Row:    0,
				Column: 0,
			}
			keyGenerator := mlglib.NewKeyGenerator()
			id, ok := ParseId(path, text, start, tracker, keyGenerator)
			output := ""
			if ok {
				output = id.ToCode(ast.NoOp)
			}
			return output, tracker, ok
		})
}

func TestSignatureParserSmoke(t *testing.T) {
	runParserSmokeTest(
		t,
		"signature",
		func(text string) (string, *frontend.DiagnosticTracker, bool) {
			path := ast.ToPath("/")
			tracker := frontend.NewDiagnosticTracker()
			start := ast.Position{
				Offset: 0,
				Row:    0,
				Column: 0,
			}
			keyGenerator := mlglib.NewKeyGenerator()
			signature, ok := ParseSignature(path, text, start, tracker, keyGenerator)
			output := ""
			if ok {
				output = signature.ToCode(ast.NoOp)
			}
			return output, tracker, ok
		})
}

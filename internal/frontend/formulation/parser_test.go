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

package formulation

import (
	"mathlingua/internal/ast"
	"mathlingua/internal/frontend"
	"mathlingua/internal/mlglib"
	"testing"

	"github.com/stretchr/testify/assert"
)

func parseExpression(text string) (ast.FormulationNodeKind, *frontend.DiagnosticTracker) {
	tracker := frontend.NewDiagnosticTracker()
	node, _ := ParseExpression("/some/path", text, ast.Position{}, tracker, mlglib.NewKeyGenerator())
	return node, tracker
}

func runExpressionTest(t *testing.T, input string, expected string) {
	doc, tracker := parseExpression(input)
	actual := ""
	if doc == nil {
		actual = "<nil>"
	} else {
		actual = doc.ToCode(func(node ast.MlgNodeKind) (string, bool) { return "", false })
	}

	messages := ""
	for _, diag := range tracker.Diagnostics() {
		messages += diag.String() + "\n"
	}

	assert.Equal(t, "", messages)
	assert.Equal(t, 0, len(tracker.Diagnostics()))
	assert.Equal(t, expected, actual)
}

func parseIdForm(text string) (ast.FormulationNodeKind, *frontend.DiagnosticTracker) {
	tracker := frontend.NewDiagnosticTracker()
	node, _ := ParseId("/some/path", text, ast.Position{}, tracker, mlglib.NewKeyGenerator())
	return node, tracker
}

func runIdFormTest(t *testing.T, input string, expected string) {
	doc, tracker := parseIdForm(input)
	assert.NotNil(t, doc)
	actual := doc.ToCode(func(node ast.MlgNodeKind) (string, bool) { return "", false })

	messages := ""
	for _, diag := range tracker.Diagnostics() {
		messages += diag.String() + "\n"
	}

	assert.Equal(t, "", messages)
	assert.Equal(t, 0, len(tracker.Diagnostics()))
	assert.Equal(t, expected, actual)
}

func parseForm(text string) (ast.FormulationNodeKind, *frontend.DiagnosticTracker) {
	tracker := frontend.NewDiagnosticTracker()
	node, _ := ParseForm("/some/path", text, ast.Position{}, tracker, mlglib.NewKeyGenerator())
	return node, tracker
}

func runFormTest(t *testing.T, input string, expected string) {
	doc, tracker := parseForm(input)
	assert.NotNil(t, doc)
	if doc == nil {
		return
	}
	actual := doc.ToCode(func(node ast.MlgNodeKind) (string, bool) { return "", false })

	messages := ""
	for _, diag := range tracker.Diagnostics() {
		messages += diag.String() + "\n"
	}

	assert.Equal(t, "", messages)
	assert.Equal(t, 0, len(tracker.Diagnostics()))
	assert.Equal(t, expected, actual)
}

func TestIdentifier(t *testing.T) {
	runExpressionTest(t, "x", "x")
}

func TestMultiCharIdentifier(t *testing.T) {
	runExpressionTest(t, "abc", "abc")
}

func TestIdentifierQuestion(t *testing.T) {
	runExpressionTest(t, "x?", "x")
}

func TestStroppedIdentifier(t *testing.T) {
	runExpressionTest(t, `"ab c"`, "ab c")
}

func TestStroppedIdentifierQuestion(t *testing.T) {
	runExpressionTest(t, `"ab c"?`, "ab c")
}

func TestVarArgIdentifier(t *testing.T) {
	runExpressionTest(t, "abc...", "abc...")
}

func TestStroppedVarArgIdentifier(t *testing.T) {
	runExpressionTest(t, `"ab c"?`, "ab c")
}

func TestStroppedVarArgIdentifierQuestion(t *testing.T) {
	runExpressionTest(t, `"ab c"...?`, "ab c...")
}

func TestChainExpressionWithNames(t *testing.T) {
	runExpressionTest(t, "a.b.c", "a.b.c")
}

func TestConditionalSetForm(t *testing.T) {
	runFormTest(t, "[x]{x : s(x)}", "[x]{x : s(x)}")
	runFormTest(t, "[x]{x : s(x) | p(x)}", "[x]{x : s(x) | p(x)}")
	runFormTest(t, "[x]{f(x) : s(x)}", "[x]{f(x) : s(x)}")
	runFormTest(t, "[x]{f(x) : s(x) | p(x)}", "[x]{f(x) : s(x) | p(x)}")
	runFormTest(t, "[x, y]{f(x, y) : s(x)}", "[x, y]{f(x, y) : s(x)}")
	runFormTest(t, "[x, y]{f(x, y) : s(x) | p(x)}", "[x, y]{f(x, y) : s(x) | p(x)}")
	runFormTest(t, "[x, y]{(f(x, y), a) : s(x)}", "[x, y]{(f(x, y), a) : s(x)}")
	runFormTest(t, "[x, y]{(f(x, y), a, [z]{z : s(x)}) : s(x)}", "[x, y]{(f(x, y), a, [z]{z : s(x)}) : s(x)}")
	runFormTest(t,
		"[x, y]{(f(x, y), a, [z]{z : s(x) | p(x)}) : s(x) | p(x)}",
		"[x, y]{(f(x, y), a, [z]{z : s(x) | p(x)}) : s(x) | p(x)}")
}

func TestConditionalSetExpression(t *testing.T) {
	runExpressionTest(t, "[x]{x : x}", "[x]{x : x}")
	runExpressionTest(t, "[x]{x | x > 0}", "[x]{x | x > 0}")
	runExpressionTest(t, "[x, y]{x | x > 0 \\.and./ y = 0}", "[x, y]{x | x > 0 \\.and./ y = 0}")
	runExpressionTest(t, "[x, f(y)]{x | x > 0 \\.and./ y = 0}", "[x, f(y)]{x | x > 0 \\.and./ y = 0}")
	runExpressionTest(t, "[x, y]{(x, y) | x > 0 \\.and./ y = 0}", "[x, y]{(x, y) | x > 0 \\.and./ y = 0}")
	runExpressionTest(t, "[x, y]{(f(x), y) | x > 0 \\.and./ y = 0}", "[x, y]{(f(x), y) | x > 0 \\.and./ y = 0}")
	runExpressionTest(t,
		"[x, y]{(f(x), [z]{z | g(y)}) | x > 0 \\.and./ y = 0}",
		"[x, y]{(f(x), [z]{z | g(y)}) | x > 0 \\.and./ y = 0}")
}

func TestConditionalSetIdForm(t *testing.T) {
	runIdFormTest(t, "\\set[x]{x : s(x)}", "\\set{[x]{x : s(x)}}")
	runIdFormTest(t, "\\set[x]{x | p(x)}", "\\set{[x]{x | p(x)}}")
	runIdFormTest(t, "\\set[x]{x : s(x) | p(x)}", "\\set{[x]{x : s(x) | p(x)}}")
}

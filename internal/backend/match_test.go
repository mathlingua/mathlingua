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
	"fmt"
	"mathlingua/internal/ast"
	"mathlingua/internal/frontend"
	"mathlingua/internal/frontend/formulation"
	"mathlingua/internal/mlglib"
	"sort"
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestName(t *testing.T) {
	runTest(t, "x", "y", map[string]string{
		"x": "y",
	}, map[string][]string{})
}

func TestNameAdvanced(t *testing.T) {
	runTest(t, "\\x", "X", map[string]string{
		"X": "\\x",
	}, map[string][]string{})
}

func TestNameVarArg(t *testing.T) {
	runTest(t, "x", "X...", map[string]string{
		"x": "X",
	}, map[string][]string{})
}

func TestFunction(t *testing.T) {
	runTest(t, "f(x)", "g(y)", map[string]string{
		"f": "g",
		"x": "y",
	}, map[string][]string{})
	runTest(t, "f(a, b, c)", "g(x, y, z)", map[string]string{
		"f": "g",
		"a": "x",
		"b": "y",
		"c": "z",
	}, map[string][]string{})
}

func TestFunctionVarArg(t *testing.T) {
	runTest(t, "f(x, y, z)", "F(X...)", map[string]string{
		"F": "f",
	}, map[string][]string{
		"X": {"x", "y", "z"},
	})
}

func TestNestedFunction(t *testing.T) {
	runTest(t, "f(x, g(y, z))", "F(X, G(Y, Z))", map[string]string{
		"f": "F",
		"x": "X",
		"g": "G",
		"y": "Y",
		"z": "Z",
	}, map[string][]string{})
}

func TestTuple(t *testing.T) {
	runTest(t, "(x, y, z)", "(X, Y, Z)", map[string]string{
		"x": "X",
		"y": "Y",
		"z": "Z",
	}, map[string][]string{})
}

func runTest(t *testing.T, expText string, patternText string,
	expectedSingle map[string]string,
	expectedVarArg map[string][]string) {
	expNode := parseNode(t, expText)
	patternNode := parseForm(t, patternText)
	pattern := ToFormPattern(patternNode)
	assert.NotNil(t, pattern,
		fmt.Sprintf("Could not determine the pattern for: %s\n%s",
			patternText,
			mlglib.PrettyPrint(patternNode)))
	match := Match(expNode, pattern)
	assert.Equal(t, true, match.MatchMakesSense)
	messages := ""
	for _, msg := range match.Messages {
		messages += msg
		messages += "\n"
	}
	assert.Equal(t, "", messages)
	assert.Equal(t, stringMapToString(expectedSingle), nodeMapToString(match.Mapping))
	assert.Equal(t, stringMapToStringSlice(expectedVarArg), nodeMapToStringSlice(match.VarArgMapping))
}

func nodeMapToString(mapping map[string]ast.MlgNodeType) string {
	keys := make([]string, 0)
	sort.Strings(keys)

	result := ""
	for _, key := range keys {
		result += key + " -> " + ast.Debug(mapping[key], noOp) + "\n"
	}
	return result
}

func nodeMapToStringSlice(mapping map[string][]ast.MlgNodeType) string {
	keys := make([]string, 0)
	sort.Strings(keys)

	result := ""
	for _, key := range keys {
		values := ""
		for i, v := range mapping[key] {
			if i > 0 {
				values += ","
			}
			values += ast.Debug(v, noOp)
		}
		result += key + " -> " + values + "\n"
	}
	return result
}

func stringMapToString(mapping map[string]string) string {
	keys := make([]string, 0)
	sort.Strings(keys)

	result := ""
	for _, key := range keys {
		result += key + " -> " + mapping[key] + "\n"
	}
	return result
}

func stringMapToStringSlice(mapping map[string][]string) string {
	keys := make([]string, 0)
	sort.Strings(keys)

	result := ""
	for _, key := range keys {
		values := ""
		for i, v := range mapping[key] {
			if i > 0 {
				values += ","
			}
			values += v
		}
		result += key + " -> " + values + "\n"
	}
	return result
}

func parseNode(t *testing.T, exp string) ast.MlgNodeType {
	return parseImpl(t, exp, func(text string) (ast.MlgNodeType, *frontend.DiagnosticTracker, bool) {
		tracker := frontend.NewDiagnosticTracker()
		keyGen := mlglib.NewKeyGenerator()
		root, ok := formulation.ParseExpression(ast.ToPath("/"), exp, ast.Position{
			Offset: 0,
			Row:    0,
			Column: 0,
		}, tracker, keyGen)
		return root, tracker, ok
	})
}

func parseForm(t *testing.T, exp string) ast.StructuralFormType {
	node := parseImpl(t, exp, func(text string) (ast.MlgNodeType, *frontend.DiagnosticTracker, bool) {
		tracker := frontend.NewDiagnosticTracker()
		keyGen := mlglib.NewKeyGenerator()
		root, ok := formulation.ParseForm(ast.ToPath("/"), exp, ast.Position{
			Offset: 0,
			Row:    0,
			Column: 0,
		}, tracker, keyGen)
		return root, tracker, ok
	})
	switch n := node.(type) {
	case ast.StructuralFormType:
		return n
	default:
		t.Fatalf("Expected a StructuralFormType but found: %s", mlglib.PrettyPrint(n))
		return &ast.NameForm{}
	}
}

func parseImpl(t *testing.T, exp string,
	parse func(text string) (ast.MlgNodeType, *frontend.DiagnosticTracker, bool)) ast.MlgNodeType {
	root, tracker, ok := parse(exp)
	messages := ""
	for _, diag := range tracker.Diagnostics() {
		messages += diag.ToString()
		messages += "\n"
	}
	assert.Equal(t, "", messages)
	assert.Equal(t, true, ok)
	return root
}

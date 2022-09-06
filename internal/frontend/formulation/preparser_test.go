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
	"testing"

	"github.com/kr/pretty"
	"github.com/stretchr/testify/assert"
)

func runTest(t *testing.T, text string, expected ast.NodeType) {
	actual, diagnostics, ok := PreParseExpression(text)
	assert.True(t, ok)
	assert.Equal(t, 0, len(diagnostics))

	actualStr := pretty.Sprintf("%# v", actual)
	expectedStr := pretty.Sprintf("%# v", expected)

	assert.Equal(t, expectedStr, actualStr)
}

func TestIdentifier(t *testing.T) {
	runTest(t, "x", ast.PseudoExpression{
		Children: []ast.NodeType{
			ast.NameForm{
				Text:            "x",
				IsStropped:      false,
				HasQuestionMark: false,
				VarArg:          ast.VarArgData{},
			},
		},
	})
}

func TestMultiCharIdentifier(t *testing.T) {
	runTest(t, "abc", ast.PseudoExpression{
		Children: []ast.NodeType{
			ast.NameForm{
				Text:            "abc",
				IsStropped:      false,
				HasQuestionMark: false,
				VarArg:          ast.VarArgData{},
			},
		},
	})
}

func TestIdentifierQuestion(t *testing.T) {
	runTest(t, "x?", ast.PseudoExpression{
		Children: []ast.NodeType{
			ast.NameForm{
				Text:            "x",
				IsStropped:      false,
				HasQuestionMark: true,
				VarArg:          ast.VarArgData{},
			},
		},
	})
}

func TestStroppedIdentifier(t *testing.T) {
	runTest(t, "\"ab c\"", ast.PseudoExpression{
		Children: []ast.NodeType{
			ast.NameForm{
				Text:            "ab c",
				IsStropped:      true,
				HasQuestionMark: false,
				VarArg:          ast.VarArgData{},
			},
		},
	})
}

func TestStroppedIdentifierQuestion(t *testing.T) {
	runTest(t, "\"ab c\"?", ast.PseudoExpression{
		Children: []ast.NodeType{
			ast.NameForm{
				Text:            "ab c",
				IsStropped:      true,
				HasQuestionMark: true,
				VarArg:          ast.VarArgData{},
			},
		},
	})
}

func TestVarArgIdentifier(t *testing.T) {
	runTest(t, "abc...", ast.PseudoExpression{
		Children: []ast.NodeType{
			ast.NameForm{
				Text:            "abc",
				IsStropped:      false,
				HasQuestionMark: false,
				VarArg: ast.VarArgData{
					IsVarArg:    true,
					VarArgCount: nil,
				},
			},
		},
	})
}

func TestStroppedVarArgIdentifier(t *testing.T) {
	runTest(t, "\"ab c\"...", ast.PseudoExpression{
		Children: []ast.NodeType{
			ast.NameForm{
				Text:            "ab c",
				IsStropped:      true,
				HasQuestionMark: false,
				VarArg: ast.VarArgData{
					IsVarArg:    true,
					VarArgCount: nil,
				},
			},
		},
	})
}

func TestStroppedVarArgIdentifierQuestion(t *testing.T) {
	runTest(t, "\"ab c\"...?", ast.PseudoExpression{
		Children: []ast.NodeType{
			ast.NameForm{
				Text:            "ab c",
				IsStropped:      true,
				HasQuestionMark: true,
				VarArg: ast.VarArgData{
					IsVarArg:    true,
					VarArgCount: nil,
				},
			},
		},
	})
}

func TestChainExpressionWithNames(t *testing.T) {
	runTest(t, "a.b.c", ast.PseudoExpression{
		Children: []ast.NodeType{
			ast.ChainExpression{
				Parts: []ast.ExpressionType{
					ast.NameForm{
						Text:            "a",
						IsStropped:      false,
						HasQuestionMark: false,
						VarArg:          ast.VarArgData{},
					},
					ast.NameForm{
						Text:            "b",
						IsStropped:      false,
						HasQuestionMark: false,
						VarArg:          ast.VarArgData{},
					},
					ast.NameForm{
						Text:            "c",
						IsStropped:      false,
						HasQuestionMark: false,
						VarArg:          ast.VarArgData{},
					},
				},
			},
		},
	})
}

func TestChainExpressionWithFunctionCall(t *testing.T) {
	runTest(t, "a.f(b).c", ast.PseudoExpression{
		Children: []ast.NodeType{
			ast.ChainExpression{
				Parts: []ast.ExpressionType{
					ast.NameForm{
						Text:            "a",
						IsStropped:      false,
						HasQuestionMark: false,
						VarArg:          ast.VarArgData{},
					},
					ast.FunctionCallExpression{
						Target: ast.NameForm{
							Text:            "f",
							IsStropped:      false,
							HasQuestionMark: false,
							VarArg:          ast.VarArgData{},
						},
						Args: []ast.ExpressionType{
							ast.PseudoExpression{
								Children: []ast.NodeType{
									ast.NameForm{
										Text:            "b",
										IsStropped:      false,
										HasQuestionMark: false,
										VarArg:          ast.VarArgData{},
									},
								},
							},
						},
					},
					ast.NameForm{
						Text:            "c",
						IsStropped:      false,
						HasQuestionMark: false,
						VarArg:          ast.VarArgData{},
					},
				},
			},
		},
	})
}

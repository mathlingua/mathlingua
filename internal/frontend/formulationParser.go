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

package frontend

import (
	"fmt"
	"mathlingua/internal/ast"
	"strings"
)

func ParseFormulation(text string) (ast.NodeType, []Diagnostic, bool) {
	lexer := NewFormulationLexer(text)
	parser := formulationParser{
		lexer:       lexer,
		diagnostics: make([]Diagnostic, 0),
	}
	node, _ := parser.structuralForm()
	return node, parser.diagnostics, len(parser.diagnostics) == 0
}

type formulationParser struct {
	lexer       Lexer
	diagnostics []Diagnostic
}

func (fp *formulationParser) has(tokenType TokenType) bool {
	return fp.lexer.HasNext() && fp.lexer.Peek().Type == tokenType
}

func (fp *formulationParser) hasHas(tokenType1 TokenType, tokenType2 TokenType) bool {
	return fp.lexer.HasNextNext() && fp.lexer.Peek().Type == tokenType1 && fp.lexer.PeekPeek().Type == tokenType2
}

func (fp *formulationParser) next() Token {
	return fp.lexer.Next()
}

func (fp *formulationParser) error(message string) {
	fp.diagnostics = append(fp.diagnostics, Diagnostic{
		Type:     Error,
		Origin:   FormulationParserOrigin,
		Message:  message,
		Position: fp.lexer.Position(),
	})
}

func (fp *formulationParser) expect(tokenType TokenType) (Token, bool) {
	if !fp.has(tokenType) {
		fp.error(fmt.Sprintf("Expected a token of type %s", tokenType))
		return Token{}, false
	}
	return fp.next(), true
}

func (fp *formulationParser) structuralForm() (ast.StructuralFormType, bool) {
	fun, ok := fp.functionForm()
	if ok {
		return fun, true
	}

	name, ok := fp.nameForm()
	if ok {
		return name, true
	}

	return nil, false
}

func (fp *formulationParser) nameForm() (ast.NameForm, bool) {
	if !fp.has(Name) {
		return ast.NameForm{}, false
	}

	name := fp.next().Text
	isStropped := false
	if strings.HasPrefix(name, "\"") && strings.HasSuffix(name, "\"") {
		isStropped = true
		name = strings.TrimPrefix(strings.TrimSuffix(name, "\""), "\"")
	}

	isVarArg := false
	var varArgCount *string = nil
	hasQuestionMark := false
	if fp.has(DotDotDot) {
		fp.next() // skip the ...
		isVarArg = true

		if fp.hasHas(Operator, Name) && fp.lexer.Peek().Text == "#" {
			fp.next() // skip the #
			text := fp.next().Text
			varArgCount = &text
		}
	}

	if fp.has(QuestionMark) {
		fp.next() // skip the ?
		hasQuestionMark = true
	}

	return ast.NameForm{
		Text:            name,
		IsStropped:      isStropped,
		HasQuestionMark: hasQuestionMark,
		VarArg: ast.VarArgData{
			IsVarArg:    isVarArg,
			VarArgCount: varArgCount,
		},
	}, true
}

func (fp *formulationParser) varArgData() (ast.VarArgData, bool) {
	if !fp.has(DotDotDot) {
		return ast.VarArgData{
			IsVarArg:    false,
			VarArgCount: nil,
		}, false
	}
	fp.expect(DotDotDot)
	var varArgCount *string = nil
	if fp.hasHas(Operator, Name) && fp.lexer.Peek().Text == "#" {
		text := fp.next().Text
		varArgCount = &text
	}
	return ast.VarArgData{
		IsVarArg:    true,
		VarArgCount: varArgCount,
	}, true
}

func (fp *formulationParser) functionForm() (ast.FunctionForm, bool) {
	if !fp.hasHas(Name, LParen) {
		return ast.FunctionForm{}, false
	}

	target, ok := fp.nameForm()
	if !ok {
		return ast.FunctionForm{}, false
	}

	params := make([]ast.NameForm, 0)
	fp.expect(LParen)
	for fp.lexer.HasNext() {
		if fp.has(RParen) {
			break
		}

		if len(params) > 0 {
			fp.expect(Comma)
		}

		param, ok := fp.nameForm()
		if !ok {
			fp.error("Expected a name")
			// move past the unexpected token
			fp.lexer.Next()
		} else {
			params = append(params, param)
		}
	}
	fp.expect(RParen)

	varArgData, _ := fp.varArgData()
	return ast.FunctionForm{
		Target: target,
		Params: params,
		VarArg: varArgData,
	}, true
}

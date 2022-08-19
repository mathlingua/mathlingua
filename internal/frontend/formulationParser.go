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

func ParseFormulation(text string) {
	lexer := NewFormulationLexer(text)
	parser := formulationParser{
		lexer:       lexer,
		diagnostics: make([]Diagnostic, 0),
	}
	form, ok := parser.NameForm()
	fmt.Printf("%+v, ok=%v\n", form, ok)
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

func (fp *formulationParser) NameForm() (ast.NameForm, bool) {
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

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
	"mathlingua/internal/mlglib"
)

type Phase3Lexer interface {
	HasNext() bool
	Next() Token
	Peek() Token
	Diagnostics() []Diagnostic
}

func NewPhase3Lexer(lexer Phase2Lexer) Phase3Lexer {
	result := phase3Lexer{
		index:       0,
		tokens:      make([]Token, 0),
		diagnostics: make([]Diagnostic, 0),
	}
	result.init(lexer)
	return &result
}

////////////////////////////////////////////////////////////

type phase3Lexer struct {
	index       int
	tokens      []Token
	diagnostics []Diagnostic
}

func (lexer *phase3Lexer) init(lexer2 Phase2Lexer) {
	lexer.diagnostics = append(lexer.diagnostics, lexer2.Diagnostics()...)
	stack := mlglib.NewStack[TokenType]()

	appendEndSection := func() {
		lexer.tokens = append(lexer.tokens, Token{
			Type:   EndSection,
			Text:   "<EndSection>",
			Offset: lexer2.Offset(),
			Row:    lexer2.Row(),
			Column: lexer2.Column(),
		})
	}

	appendEndArgumentGroup := func() {
		lexer.tokens = append(lexer.tokens, Token{
			Type:   EndArgumentGroup,
			Text:   "<EndArgumentGroup>",
			Offset: lexer2.Offset(),
			Row:    lexer2.Row(),
			Column: lexer2.Column(),
		})
	}

	appendEndTopLevelGroup := func() {
		lexer.tokens = append(lexer.tokens, Token{
			Type:   EndTopLevelGroup,
			Text:   "<EndTopLevelGroup>",
			Offset: lexer2.Offset(),
			Row:    lexer2.Row(),
			Column: lexer2.Column(),
		})
	}

	appendEndArgument := func() {
		lexer.tokens = append(lexer.tokens, Token{
			Type:   EndArgument,
			Text:   "<EndArgument>",
			Offset: lexer2.Offset(),
			Row:    lexer2.Row(),
			Column: lexer2.Column(),
		})
	}

	beginSection := func() {
		lexer.tokens = append(lexer.tokens, Token{
			Type:   BeginSection,
			Text:   "<BeginSection>",
			Offset: lexer2.Offset(),
			Row:    lexer2.Row(),
			Column: lexer2.Column(),
		})
		stack.Push(BeginSection)
	}

	beginTopLevelGroup := func() {
		lexer.tokens = append(lexer.tokens, Token{
			Type:   BeginTopLevelGroup,
			Text:   "<BeginTopLevelGroup>",
			Offset: lexer2.Offset(),
			Row:    lexer2.Row(),
			Column: lexer2.Column(),
		})
		stack.Push(BeginTopLevelGroup)
	}

	beginArgumentGroup := func() {
		lexer.tokens = append(lexer.tokens, Token{
			Type:   BeginArgumentGroup,
			Text:   "<BeginArgumentGroup>",
			Offset: lexer2.Offset(),
			Row:    lexer2.Row(),
			Column: lexer2.Column(),
		})
		stack.Push(BeginArgumentGroup)
	}

	beginArgument := func() {
		lexer.tokens = append(lexer.tokens, Token{
			Type:   BeginArgument,
			Text:   "<BeginArgument>",
			Offset: lexer2.Offset(),
			Row:    lexer2.Row(),
			Column: lexer2.Column(),
		})
		stack.Push(BeginArgument)
	}

	maybeEndSection := func() {
		if !stack.IsEmpty() && stack.Peek() == BeginSection {
			stack.Pop()
			appendEndSection()
		}
	}

	maybeEndArgumentGroup := func() {
		if !stack.IsEmpty() && stack.Peek() == BeginArgumentGroup {
			stack.Pop()
			appendEndArgumentGroup()
		}
	}

	maybeEndArgument := func() {
		if !stack.IsEmpty() && stack.Peek() == BeginArgument {
			stack.Pop()
			appendEndArgument()
		}
	}

	appendToken := func(tok Token) {
		lexer.tokens = append(lexer.tokens, tok)
	}

	appendNext := func() {
		appendToken(lexer2.Next())
	}

	skipNext := func() {
		lexer2.Next()
	}

	appendDiagnostic := func(message string, row int, column int) {
		lexer.diagnostics = append(lexer.diagnostics, Diagnostic{
			Type:    Error,
			Origin:  Phase3LexerOrigin,
			Message: message,
			Row:     row,
			Column:  column,
		})
	}

	hasNameColon := func() bool {
		return lexer2.HasNextNext() &&
			lexer2.Peek().Type == Name && lexer2.PeekPeek().Type == Colon
	}

	has := func(tokenType TokenType) bool {
		return lexer2.HasNext() && lexer2.Peek().Type == tokenType
	}

	hasHas := func(type1 TokenType, type2 TokenType) bool {
		return lexer2.HasNextNext() &&
			lexer2.Peek().Type == type1 &&
			lexer2.PeekPeek().Type == type2
	}

	for lexer2.HasNext() {
		if hasNameColon() {
			if stack.IsEmpty() {
				beginTopLevelGroup()
			}
			maybeEndSection()
			beginSection()
			appendNext()
			appendNext()
		} else if hasHas(Indent, DotSpace) {
			skipNext()
			skipNext()
			beginArgument()
			if hasNameColon() {
				beginArgumentGroup()
			}
		} else if has(DotSpace) {
			skipNext()
			maybeEndSection()
			maybeEndArgumentGroup()
			maybeEndArgument()
			beginArgument()
			if hasNameColon() {
				beginArgumentGroup()
			}
		} else if has(UnIndent) {
			skipNext()
			maybeEndSection()
			maybeEndArgumentGroup()
			maybeEndArgument()
		} else if has(LineBreak) {
			for !stack.IsEmpty() {
				top := stack.Pop()
				if top == BeginSection {
					appendEndSection()
				} else if top == BeginArgumentGroup {
					appendEndArgumentGroup()
				} else if top == BeginTopLevelGroup {
					appendEndTopLevelGroup()
				} else if top == BeginArgument {
					appendEndArgument()
				} else {
					panic(fmt.Sprintf("Unexpected structural type %s", top))
				}
			}
			skipNext()
		} else if has(Indent) {
			if !has(DotSpace) {
				row := -1
				column := -1
				if lexer2.HasNext() {
					peek := lexer2.Peek()
					row = peek.Row
					column = peek.Column
				}
				appendDiagnostic("Unexpected indent", row, column)
			}
			skipNext()
		} else if has(Newline) {
			skipNext()
		} else {
			appendNext()
		}
	}
}

func (lexer *phase3Lexer) HasNext() bool {
	return lexer.index < len(lexer.tokens)
}

func (lexer *phase3Lexer) Next() Token {
	peek := lexer.Peek()
	lexer.index++
	return peek
}

func (lexer *phase3Lexer) Peek() Token {
	return lexer.tokens[lexer.index]
}

func (lexer *phase3Lexer) Diagnostics() []Diagnostic {
	return lexer.diagnostics
}

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
			Type:     EndSection,
			Text:     "<EndSection>",
			Position: lexer2.Position(),
		})
	}

	appendEndArgumentGroup := func() {
		lexer.tokens = append(lexer.tokens, Token{
			Type:     EndArgumentGroup,
			Text:     "<EndArgumentGroup>",
			Position: lexer2.Position(),
		})
	}

	appendEndTopLevelGroup := func() {
		lexer.tokens = append(lexer.tokens, Token{
			Type:     EndTopLevelGroup,
			Text:     "<EndTopLevelGroup>",
			Position: lexer2.Position(),
		})
	}

	appendEndDotSpaceArgument := func() {
		lexer.tokens = append(lexer.tokens, Token{
			Type:     EndDotSpaceArgument,
			Text:     "<EndDotSpaceArgument>",
			Position: lexer2.Position(),
		})
	}

	beginSection := func() {
		lexer.tokens = append(lexer.tokens, Token{
			Type:     BeginSection,
			Text:     "<BeginSection>",
			Position: lexer2.Position(),
		})
		stack.Push(BeginSection)
	}

	beginTopLevelGroup := func() {
		lexer.tokens = append(lexer.tokens, Token{
			Type:     BeginTopLevelGroup,
			Text:     "<BeginTopLevelGroup>",
			Position: lexer2.Position(),
		})
		stack.Push(BeginTopLevelGroup)
	}

	beginArgumentGroup := func() {
		lexer.tokens = append(lexer.tokens, Token{
			Type:     BeginArgumentGroup,
			Text:     "<BeginArgumentGroup>",
			Position: lexer2.Position(),
		})
		stack.Push(BeginArgumentGroup)
	}

	beginDotSpaceArgument := func() {
		lexer.tokens = append(lexer.tokens, Token{
			Type:     BeginDotSpaceArgument,
			Text:     "<BeginDotSpaceArgument>",
			Position: lexer2.Position(),
		})
		stack.Push(BeginDotSpaceArgument)
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

	maybeEndDotSpaceArgument := func() {
		if !stack.IsEmpty() && stack.Peek() == BeginDotSpaceArgument {
			stack.Pop()
			appendEndDotSpaceArgument()
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

	appendDiagnostic := func(message string, position Position) {
		lexer.diagnostics = append(lexer.diagnostics, Diagnostic{
			Type:     Error,
			Origin:   Phase3LexerOrigin,
			Message:  message,
			Position: position,
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
			beginDotSpaceArgument()
			if hasNameColon() {
				beginArgumentGroup()
			}
		} else if has(DotSpace) {
			skipNext()
			maybeEndSection()
			maybeEndArgumentGroup()
			maybeEndDotSpaceArgument()
			beginDotSpaceArgument()
			if hasNameColon() {
				beginArgumentGroup()
			}
		} else if has(UnIndent) {
			skipNext()
			maybeEndSection()
			maybeEndArgumentGroup()
			maybeEndDotSpaceArgument()
		} else if has(LineBreak) {
			for !stack.IsEmpty() {
				top := stack.Pop()
				if top == BeginSection {
					appendEndSection()
				} else if top == BeginArgumentGroup {
					appendEndArgumentGroup()
				} else if top == BeginTopLevelGroup {
					appendEndTopLevelGroup()
				} else if top == BeginDotSpaceArgument {
					appendEndDotSpaceArgument()
				} else {
					panic(fmt.Sprintf("Unexpected structural type %s", top))
				}
			}
			skipNext()
		} else if has(Indent) {
			if !has(DotSpace) {
				appendDiagnostic("Unexpected indent", lexer2.Position())
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

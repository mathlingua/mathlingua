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

package phase3

import (
	"fmt"
	"mathlingua/internal/ast"
	"mathlingua/internal/frontend"
	"mathlingua/internal/frontend/shared"
	"mathlingua/internal/mlglib"
)

func NewLexer(phase2Lexer shared.Lexer) shared.Lexer {
	return shared.NewLexer(getTokens(phase2Lexer))
}

////////////////////////////////////////////////////////////

func getTokens(lexer2 shared.Lexer) ([]shared.Token, []frontend.Diagnostic) {
	tokens := make([]shared.Token, 0)
	diagnostics := make([]frontend.Diagnostic, 0)

	diagnostics = append(diagnostics, lexer2.Diagnostics()...)
	stack := mlglib.NewStack[shared.TokenType]()

	appendEndSection := func() {
		tokens = append(tokens, shared.Token{
			Type:     shared.EndSection,
			Text:     "<EndSection>",
			Position: lexer2.Position(),
		})
	}

	appendEndGroup := func() {
		tokens = append(tokens, shared.Token{
			Type:     shared.EndGroup,
			Text:     "<EndGroup>",
			Position: lexer2.Position(),
		})
	}

	appendEndDotSpaceArgument := func() {
		tokens = append(tokens, shared.Token{
			Type:     shared.EndDotSpaceArgument,
			Text:     "<EndDotSpaceArgument>",
			Position: lexer2.Position(),
		})
	}

	appendEndInlineArgument := func() {
		tokens = append(tokens, shared.Token{
			Type:     shared.EndInlineArgument,
			Text:     "<EndInlineArgument>",
			Position: lexer2.Position(),
		})
	}

	beginSection := func() {
		tokens = append(tokens, shared.Token{
			Type:     shared.BeginSection,
			Text:     "<BeginSection>",
			Position: lexer2.Position(),
		})
		stack.Push(shared.BeginSection)
	}

	beginGroup := func() {
		tokens = append(tokens, shared.Token{
			Type:     shared.BeginGroup,
			Text:     "<BeginGroup>",
			Position: lexer2.Position(),
		})
		stack.Push(shared.BeginGroup)
	}

	beginDotSpaceArgument := func() {
		tokens = append(tokens, shared.Token{
			Type:     shared.BeginDotSpaceArgument,
			Text:     "<BeginDotSpaceArgument>",
			Position: lexer2.Position(),
		})
		stack.Push(shared.BeginDotSpaceArgument)
	}

	beginInlineArgument := func() {
		tokens = append(tokens, shared.Token{
			Type:     shared.BeginInlineArgument,
			Text:     "<BeginInlineArgument>",
			Position: lexer2.Position(),
		})
		stack.Push(shared.BeginInlineArgument)
	}

	maybeEndSection := func() {
		if !stack.IsEmpty() && stack.Peek() == shared.BeginSection {
			stack.Pop()
			appendEndSection()
		}
	}

	maybeEndGroup := func() {
		if !stack.IsEmpty() && stack.Peek() == shared.BeginGroup {
			stack.Pop()
			appendEndGroup()
		}
	}

	maybeEndDotSpaceArgument := func() {
		if !stack.IsEmpty() && stack.Peek() == shared.BeginDotSpaceArgument {
			stack.Pop()
			appendEndDotSpaceArgument()
		}
	}

	maybeEndInlineArgument := func() {
		if !stack.IsEmpty() && stack.Peek() == shared.BeginInlineArgument {
			stack.Pop()
			appendEndInlineArgument()
		}
	}

	appendToken := func(tok shared.Token) {
		tokens = append(tokens, tok)
	}

	appendNext := func() {
		appendToken(lexer2.Next())
	}

	skipNext := func() {
		lexer2.Next()
	}

	appendDiagnostic := func(message string, position ast.Position) {
		diagnostics = append(diagnostics, frontend.Diagnostic{
			Type:     frontend.Error,
			Origin:   frontend.Phase3LexerOrigin,
			Message:  message,
			Position: position,
		})
	}

	hasNameColon := func() bool {
		return lexer2.HasNextNext() &&
			lexer2.Peek().Type == shared.Name && lexer2.PeekPeek().Type == shared.Colon
	}

	has := func(tokenType shared.TokenType) bool {
		return lexer2.HasNext() && lexer2.Peek().Type == tokenType
	}

	hasHas := func(type1 shared.TokenType, type2 shared.TokenType) bool {
		return lexer2.HasNextNext() &&
			lexer2.Peek().Type == type1 &&
			lexer2.PeekPeek().Type == type2
	}

	for lexer2.HasNext() {
		if hasNameColon() {
			if stack.IsEmpty() {
				beginGroup()
			}
			maybeEndSection()
			beginSection()
			appendNext() // append the name
			skipNext()   // but skip the :
			beginInlineArgument()
		} else if hasHas(shared.Indent, shared.DotSpace) {
			skipNext()
			skipNext()
			beginDotSpaceArgument()
			if hasNameColon() {
				beginGroup()
			}
		} else if has(shared.DotSpace) {
			skipNext()
			maybeEndSection()
			maybeEndGroup()
			maybeEndDotSpaceArgument()
			beginDotSpaceArgument()
			if hasNameColon() {
				beginGroup()
			}
		} else if has(shared.UnIndent) {
			skipNext()
			maybeEndSection()
			maybeEndGroup()
			maybeEndDotSpaceArgument()
		} else if has(shared.LineBreak) {
			for !stack.IsEmpty() {
				top := stack.Pop()
				if top == shared.BeginSection {
					appendEndSection()
				} else if top == shared.BeginGroup {
					appendEndGroup()
				} else if top == shared.BeginDotSpaceArgument {
					appendEndDotSpaceArgument()
				} else if top == shared.BeginInlineArgument {
					appendEndInlineArgument()
				} else {
					panic(fmt.Sprintf("Unexpected structural type %s", top))
				}
			}
			skipNext()
		} else if has(shared.Indent) {
			if !has(shared.DotSpace) {
				appendDiagnostic("Unexpected indent", lexer2.Position())
			}
			skipNext()
		} else if has(shared.Newline) {
			maybeEndInlineArgument()
			skipNext()
		} else if has(shared.Comma) {
			skipNext()
			maybeEndInlineArgument()
			beginInlineArgument()
		} else {
			appendNext()
		}
	}

	cleanedTokens := make([]shared.Token, 0)
	j := 0

	for j < len(tokens) {
		cur := tokens[j]
		j++
		if cur.Type == shared.BeginInlineArgument && j < len(tokens) && tokens[j].Type == shared.EndInlineArgument {
			j++
			// skip the begin and end inline argument tokens because the argument is empty
		} else {
			cleanedTokens = append(cleanedTokens, cur)
		}
	}

	return cleanedTokens, diagnostics
}

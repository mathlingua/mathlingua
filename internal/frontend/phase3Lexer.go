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

func NewPhase3Lexer(phase2Lexer Lexer) Lexer {
	return NewLexer(getPhase3Tokens(phase2Lexer))
}

////////////////////////////////////////////////////////////

func getPhase3Tokens(lexer2 Lexer) ([]Token, []Diagnostic) {
	tokens := make([]Token, 0)
	diagnostics := make([]Diagnostic, 0)

	diagnostics = append(diagnostics, lexer2.Diagnostics()...)
	stack := mlglib.NewStack[TokenType]()

	appendEndSection := func() {
		tokens = append(tokens, Token{
			Type:     EndSection,
			Text:     "<EndSection>",
			Position: lexer2.Position(),
		})
	}

	appendEndArgumentGroup := func() {
		tokens = append(tokens, Token{
			Type:     EndArgumentGroup,
			Text:     "<EndArgumentGroup>",
			Position: lexer2.Position(),
		})
	}

	appendEndTopLevelGroup := func() {
		tokens = append(tokens, Token{
			Type:     EndTopLevelGroup,
			Text:     "<EndTopLevelGroup>",
			Position: lexer2.Position(),
		})
	}

	appendEndDotSpaceArgument := func() {
		tokens = append(tokens, Token{
			Type:     EndDotSpaceArgument,
			Text:     "<EndDotSpaceArgument>",
			Position: lexer2.Position(),
		})
	}

	appendEndInlineArgument := func() {
		tokens = append(tokens, Token{
			Type:     EndInlineArgument,
			Text:     "<EndInlineArgument>",
			Position: lexer2.Position(),
		})
	}

	beginSection := func() {
		tokens = append(tokens, Token{
			Type:     BeginSection,
			Text:     "<BeginSection>",
			Position: lexer2.Position(),
		})
		stack.Push(BeginSection)
	}

	beginTopLevelGroup := func() {
		tokens = append(tokens, Token{
			Type:     BeginTopLevelGroup,
			Text:     "<BeginTopLevelGroup>",
			Position: lexer2.Position(),
		})
		stack.Push(BeginTopLevelGroup)
	}

	beginArgumentGroup := func() {
		tokens = append(tokens, Token{
			Type:     BeginArgumentGroup,
			Text:     "<BeginArgumentGroup>",
			Position: lexer2.Position(),
		})
		stack.Push(BeginArgumentGroup)
	}

	beginDotSpaceArgument := func() {
		tokens = append(tokens, Token{
			Type:     BeginDotSpaceArgument,
			Text:     "<BeginDotSpaceArgument>",
			Position: lexer2.Position(),
		})
		stack.Push(BeginDotSpaceArgument)
	}

	beginInlineArgument := func() {
		tokens = append(tokens, Token{
			Type:     BeginInlineArgument,
			Text:     "<BeginInlineArgument>",
			Position: lexer2.Position(),
		})
		stack.Push(BeginInlineArgument)
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

	maybeEndInlineArgument := func() {
		if !stack.IsEmpty() && stack.Peek() == BeginInlineArgument {
			stack.Pop()
			appendEndInlineArgument()
		}
	}

	appendToken := func(tok Token) {
		tokens = append(tokens, tok)
	}

	appendNext := func() {
		appendToken(lexer2.Next())
	}

	skipNext := func() {
		lexer2.Next()
	}

	appendDiagnostic := func(message string, position Position) {
		diagnostics = append(diagnostics, Diagnostic{
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
			appendNext() // append the name
			skipNext()   // but skip the :
			beginInlineArgument()
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
				} else if top == BeginInlineArgument {
					appendEndInlineArgument()
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
			maybeEndInlineArgument()
			skipNext()
		} else if has(Comma) {
			skipNext()
			maybeEndInlineArgument()
			beginInlineArgument()
		} else {
			appendNext()
		}
	}

	cleanedTokens := make([]Token, 0)
	j := 0

	for j < len(tokens) {
		cur := tokens[j]
		j++
		if cur.Type == BeginInlineArgument && j < len(tokens) && tokens[j].Type == EndInlineArgument {
			j++
			// skip the begin and end inline argument tokens because the argument is empty
		} else {
			cleanedTokens = append(cleanedTokens, cur)
		}
	}

	return cleanedTokens, diagnostics
}

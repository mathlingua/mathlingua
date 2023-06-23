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
	"mathlingua/internal/mlglib"
)

func NewLexer(
	phase2Lexer frontend.ILexer,
	path ast.Path,
	tracker frontend.IDiagnosticTracker,
) frontend.ILexer {
	return frontend.NewLexer(getTokens(phase2Lexer, path, tracker))
}

////////////////////////////////////////////////////////////////////////////////////////////////////

func getTokens(
	phase2Lexer frontend.ILexer,
	path ast.Path,
	tracker frontend.IDiagnosticTracker,
) []ast.Token {
	tokens := make([]ast.Token, 0)
	stack := mlglib.NewStack[ast.TokenType]()

	appendEndSection := func() {
		tokens = append(tokens, ast.Token{
			Type:     ast.EndSection,
			Text:     "<EndSection>",
			Position: phase2Lexer.Position(),
		})
	}

	appendEndGroup := func() {
		tokens = append(tokens, ast.Token{
			Type:     ast.EndGroup,
			Text:     "<EndGroup>",
			Position: phase2Lexer.Position(),
		})
	}

	appendEndDotSpaceArgument := func() {
		tokens = append(tokens, ast.Token{
			Type:     ast.EndDotSpaceArgument,
			Text:     "<EndDotSpaceArgument>",
			Position: phase2Lexer.Position(),
		})
	}

	appendEndInlineArgument := func() {
		tokens = append(tokens, ast.Token{
			Type:     ast.EndInlineArgument,
			Text:     "<EndInlineArgument>",
			Position: phase2Lexer.Position(),
		})
	}

	beginSection := func() {
		tokens = append(tokens, ast.Token{
			Type:     ast.BeginSection,
			Text:     "<BeginSection>",
			Position: phase2Lexer.Position(),
		})
		stack.Push(ast.BeginSection)
	}

	beginGroup := func() {
		tokens = append(tokens, ast.Token{
			Type:     ast.BeginGroup,
			Text:     "<BeginGroup>",
			Position: phase2Lexer.Position(),
		})
		stack.Push(ast.BeginGroup)
	}

	beginDotSpaceArgument := func() {
		tokens = append(tokens, ast.Token{
			Type:     ast.BeginDotSpaceArgument,
			Text:     "<BeginDotSpaceArgument>",
			Position: phase2Lexer.Position(),
		})
		stack.Push(ast.BeginDotSpaceArgument)
	}

	beginInlineArgument := func() {
		tokens = append(tokens, ast.Token{
			Type:     ast.BeginInlineArgument,
			Text:     "<BeginInlineArgument>",
			Position: phase2Lexer.Position(),
		})
		stack.Push(ast.BeginInlineArgument)
	}

	maybeEndSection := func() {
		if !stack.IsEmpty() && stack.Peek() == ast.BeginSection {
			stack.Pop()
			appendEndSection()
		}
	}

	maybeEndGroup := func() {
		if !stack.IsEmpty() && stack.Peek() == ast.BeginGroup {
			stack.Pop()
			appendEndGroup()
		}
	}

	maybeEndDotSpaceArgument := func() {
		if !stack.IsEmpty() && stack.Peek() == ast.BeginDotSpaceArgument {
			stack.Pop()
			appendEndDotSpaceArgument()
		}
	}

	maybeEndInlineArgument := func() {
		if !stack.IsEmpty() && stack.Peek() == ast.BeginInlineArgument {
			stack.Pop()
			appendEndInlineArgument()
		}
	}

	appendToken := func(tok ast.Token) {
		tokens = append(tokens, tok)
	}

	appendNext := func() {
		appendToken(phase2Lexer.Next())
	}

	skipNext := func() {
		phase2Lexer.Next()
	}

	appendDiagnostic := func(message string, position ast.Position) {
		tracker.Append(frontend.Diagnostic{
			Path:     path,
			Type:     frontend.Error,
			Origin:   frontend.Phase3LexerOrigin,
			Message:  message,
			Position: position,
		})
	}

	hasNameColon := func() bool {
		return phase2Lexer.HasNextNext() &&
			phase2Lexer.Peek().Type == ast.Name && phase2Lexer.PeekPeek().Type == ast.Colon
	}

	has := func(tokenType ast.TokenType) bool {
		return phase2Lexer.HasNext() && phase2Lexer.Peek().Type == tokenType
	}

	hasHas := func(type1 ast.TokenType, type2 ast.TokenType) bool {
		return phase2Lexer.HasNextNext() &&
			phase2Lexer.Peek().Type == type1 &&
			phase2Lexer.PeekPeek().Type == type2
	}

	for phase2Lexer.HasNext() {
		if hasNameColon() {
			if stack.IsEmpty() {
				beginGroup()
			}
			maybeEndSection()
			beginSection()
			appendNext() // append the name
			skipNext()   // but skip the :
			beginInlineArgument()
		} else if hasHas(ast.Indent, ast.DotSpace) {
			skipNext()
			skipNext()
			beginDotSpaceArgument()
			if hasNameColon() {
				beginGroup()
			}
		} else if has(ast.DotSpace) {
			skipNext()
			maybeEndSection()
			maybeEndGroup()
			maybeEndDotSpaceArgument()
			beginDotSpaceArgument()
			if hasNameColon() {
				beginGroup()
			}
		} else if has(ast.UnIndent) {
			skipNext()
			maybeEndSection()
			maybeEndGroup()
			maybeEndDotSpaceArgument()
		} else if has(ast.LineBreak) {
			for !stack.IsEmpty() {
				top := stack.Pop()
				if top == ast.BeginSection {
					appendEndSection()
				} else if top == ast.BeginGroup {
					appendEndGroup()
				} else if top == ast.BeginDotSpaceArgument {
					appendEndDotSpaceArgument()
				} else if top == ast.BeginInlineArgument {
					appendEndInlineArgument()
				} else {
					panic(fmt.Sprintf("Unexpected structural type %s", top))
				}
			}
			skipNext()
		} else if has(ast.Indent) {
			if !has(ast.DotSpace) {
				appendDiagnostic("Unexpected indent", phase2Lexer.Position())
			}
			skipNext()
		} else if has(ast.Newline) {
			maybeEndInlineArgument()
			skipNext()
		} else if has(ast.Comma) {
			skipNext()
			maybeEndInlineArgument()
			beginInlineArgument()
		} else if has(ast.Id) {
			appendNext()
			beginGroup()
		} else {
			appendNext()
		}
	}

	cleanedTokens := make([]ast.Token, 0)
	j := 0

	for j < len(tokens) {
		cur := tokens[j]
		j++
		if cur.Type == ast.BeginInlineArgument && j < len(tokens) &&
			tokens[j].Type == ast.EndInlineArgument {
			j++
			// skip the begin and end inline argument tokens because the argument is empty
		} else {
			cleanedTokens = append(cleanedTokens, cur)
		}
	}

	return cleanedTokens
}

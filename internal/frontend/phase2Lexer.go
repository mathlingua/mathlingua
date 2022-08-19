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
)

func NewPhase2Lexer(phase1Lexer Lexer) Lexer {
	return NewLexer(getPhase2Tokens(phase1Lexer))
}

//////////////////////////////////////////////////////////////////

func getPhase2Tokens(lexer1 Lexer) ([]Token, []Diagnostic) {
	tokens := make([]Token, 0)
	diagnostics := make([]Diagnostic, 0)

	diagnostics = append(diagnostics, lexer1.Diagnostics()...)
	prevIndent := 0

	appendToken := func(tok Token) {
		tokens = append(tokens, tok)
	}

	handleIndentsOrUnIndents := func(curIndent int, position ast.Position) {
		if curIndent > prevIndent {
			for j := 0; j < curIndent-prevIndent; j++ {
				appendToken(Token{
					Type:     Indent,
					Text:     "<Indent>",
					Position: position,
				})
			}
		} else {
			for j := 0; j < prevIndent-curIndent; j++ {
				appendToken(Token{
					Type:     UnIndent,
					Text:     "<UnIndent>",
					Position: position,
				})
			}
		}
	}

	for lexer1.HasNext() {
		cur := lexer1.Next()

		if cur.Type == Newline {
			appendToken(cur)
			// if the Newline is followed by at least one more Newline
			// also record a LineBreak
			if lexer1.HasNext() && lexer1.Peek().Type == Newline {
				for lexer1.HasNext() && lexer1.Peek().Type == Newline {
					lexer1.Next()
				}
				appendToken(Token{
					Type:     LineBreak,
					Text:     "<LineBreak>",
					Position: cur.Position,
				})
			}
			if lexer1.HasNext() && lexer1.Peek().Type != Space && lexer1.Peek().Type != DotSpace {
				// since there isn't a space handle any unindents
				handleIndentsOrUnIndents(0, cur.Position)
				prevIndent = 0
			}
		} else if cur.Type == Space {
			numSpaces := 1
			for lexer1.HasNext() && lexer1.Peek().Type == Space {
				lexer1.Next()
				numSpaces++
			}
			if numSpaces%2 == 1 {
				diagnostics = append(diagnostics, Diagnostic{
					Type:     Error,
					Origin:   Phase2LexerOrigin,
					Message:  fmt.Sprintf("Expected an even indent but found %d", numSpaces),
					Position: cur.Position,
				})
			}
			curIndent := numSpaces / 2
			if lexer1.HasNext() && lexer1.Peek().Type == DotSpace {
				curIndent++
			}
			handleIndentsOrUnIndents(curIndent, cur.Position)
			if lexer1.HasNext() && lexer1.Peek().Type == DotSpace {
				appendToken(lexer1.Next())
			}
			prevIndent = curIndent
		} else if cur.Type == DotSpace {
			curIndent := 1
			handleIndentsOrUnIndents(curIndent, cur.Position)
			appendToken(cur)
			prevIndent = curIndent
		} else {
			appendToken(cur)
		}
	}

	return tokens, diagnostics
}

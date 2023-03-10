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

package phase2

import (
	"fmt"
	"mathlingua/internal/ast"
	"mathlingua/internal/frontend"
)

func NewLexer(phase1Lexer frontend.Lexer, path ast.Path,
	tracker frontend.DiagnosticTracker) frontend.Lexer {
	return frontend.NewLexer(getTokens(phase1Lexer, path, tracker))
}

//////////////////////////////////////////////////////////////////

func getTokens(lexer1 frontend.Lexer, path ast.Path,
	tracker frontend.DiagnosticTracker) []ast.Token {
	tokens := make([]ast.Token, 0)
	prevIndent := 0

	appendToken := func(tok ast.Token) {
		tokens = append(tokens, tok)
	}

	handleIndentsOrUnIndents := func(curIndent int, position ast.Position) {
		if curIndent > prevIndent {
			for j := 0; j < curIndent-prevIndent; j++ {
				appendToken(ast.Token{
					Type:     ast.Indent,
					Text:     "<Indent>",
					Position: position,
				})
			}
		} else {
			for j := 0; j < prevIndent-curIndent; j++ {
				appendToken(ast.Token{
					Type:     ast.UnIndent,
					Text:     "<UnIndent>",
					Position: position,
				})
			}
		}
	}

	for lexer1.HasNext() {
		cur := lexer1.Next()

		if cur.Type == ast.Newline {
			appendToken(cur)
			// if the ast.Newline is followed by at least one more ast.Newline
			// also record a ast.LineBreak
			if lexer1.HasNext() && lexer1.Peek().Type == ast.Newline {
				for lexer1.HasNext() && lexer1.Peek().Type == ast.Newline {
					lexer1.Next()
				}
				appendToken(ast.Token{
					Type:     ast.LineBreak,
					Text:     "<LineBreak>",
					Position: cur.Position,
				})
			}
			if lexer1.HasNext() && lexer1.Peek().Type != ast.Space && lexer1.Peek().Type != ast.DotSpace {
				// since there isn't a space handle any unindents
				handleIndentsOrUnIndents(0, cur.Position)
				prevIndent = 0
			}
		} else if cur.Type == ast.Space {
			numSpaces := 1
			for lexer1.HasNext() && lexer1.Peek().Type == ast.Space {
				lexer1.Next()
				numSpaces++
			}
			if numSpaces%2 == 1 {
				tracker.Append(frontend.Diagnostic{
					Path:     path,
					Type:     frontend.Error,
					Origin:   frontend.Phase2LexerOrigin,
					Message:  fmt.Sprintf("Expected an even indent but found %d", numSpaces),
					Position: cur.Position,
				})
			}
			curIndent := numSpaces / 2
			if lexer1.HasNext() && lexer1.Peek().Type == ast.DotSpace {
				curIndent++
			}
			handleIndentsOrUnIndents(curIndent, cur.Position)
			if lexer1.HasNext() && lexer1.Peek().Type == ast.DotSpace {
				appendToken(lexer1.Next())
			}
			prevIndent = curIndent
		} else if cur.Type == ast.DotSpace {
			curIndent := 1
			handleIndentsOrUnIndents(curIndent, cur.Position)
			appendToken(cur)
			prevIndent = curIndent
		} else {
			appendToken(cur)
		}
	}

	return tokens
}

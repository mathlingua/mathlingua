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

import "fmt"

type Phase2Lexer interface {
	HasNext() bool
	HasNextNext() bool
	Next() Token
	Peek() Token
	PeekPeek() Token
	Row() int
	Column() int
	Offset() int
	Diagnostics() []Diagnostic
}

func NewPhase2Lexer(lexer Phase1Lexer) Phase2Lexer {
	result := phase2Lexer{
		index:       0,
		tokens:      make([]Token, 0),
		diagnostics: make([]Diagnostic, 0),
	}
	result.init(lexer)
	return &result
}

//////////////////////////////////////////////////////////////////

type phase2Lexer struct {
	index       int
	tokens      []Token
	diagnostics []Diagnostic
}

func (lexer *phase2Lexer) init(lexer1 Phase1Lexer) {
	lexer.diagnostics = append(lexer.diagnostics, lexer1.Diagnostics()...)
	prevIndent := 0

	appendToken := func(tok Token) {
		lexer.tokens = append(lexer.tokens, tok)
	}

	handleIndentsOrUnIndents := func(curIndent int, offset int, row int, column int) {
		if curIndent > prevIndent {
			for j := 0; j < curIndent-prevIndent; j++ {
				appendToken(Token{
					Type:   Indent,
					Text:   "<Indent>",
					Offset: offset,
					Row:    row,
					Column: column,
				})
			}
		} else {
			for j := 0; j < prevIndent-curIndent; j++ {
				appendToken(Token{
					Type:   UnIndent,
					Text:   "<UnIndent>",
					Offset: offset,
					Row:    row,
					Column: column,
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
					Type:   LineBreak,
					Text:   "<LineBreak>",
					Offset: cur.Offset,
					Row:    cur.Row,
					Column: cur.Column,
				})
			}
			if lexer1.HasNext() && lexer1.Peek().Type != Space && lexer1.Peek().Type != DotSpace {
				// since there isn't a space handle any unindents
				handleIndentsOrUnIndents(0, cur.Offset, cur.Row, cur.Column)
				prevIndent = 0
			}
		} else if cur.Type == Space {
			numSpaces := 1
			for lexer1.HasNext() && lexer1.Peek().Type == Space {
				lexer1.Next()
				numSpaces++
			}
			if numSpaces%2 == 1 {
				lexer.diagnostics = append(lexer.diagnostics, Diagnostic{
					Type:    Error,
					Origin:  Phase2LexerOrigin,
					Message: fmt.Sprintf("Expected an even indent but found %d", numSpaces),
					Row:     cur.Row,
					Column:  cur.Column,
				})
			}
			curIndent := numSpaces / 2
			if lexer1.HasNext() && lexer1.Peek().Type == DotSpace {
				curIndent++
			}
			handleIndentsOrUnIndents(curIndent, cur.Offset, cur.Row, cur.Column)
			if lexer1.HasNext() && lexer1.Peek().Type == DotSpace {
				appendToken(lexer1.Next())
			}
			prevIndent = curIndent
		} else if cur.Type == DotSpace {
			curIndent := 1
			handleIndentsOrUnIndents(curIndent, cur.Offset, cur.Row, cur.Column)
			appendToken(cur)
			prevIndent = curIndent
		} else {
			appendToken(cur)
		}
	}
}

func (lexer *phase2Lexer) HasNext() bool {
	return lexer.index < len(lexer.tokens)
}

func (lexer *phase2Lexer) HasNextNext() bool {
	return lexer.index+1 < len(lexer.tokens)
}

func (lexer *phase2Lexer) Next() Token {
	peek := lexer.Peek()
	lexer.index++
	return peek
}

func (lexer *phase2Lexer) Peek() Token {
	return lexer.tokens[lexer.index]
}

func (lexer *phase2Lexer) PeekPeek() Token {
	return lexer.tokens[lexer.index+1]
}

func (lexer *phase2Lexer) Row() int {
	if lexer.HasNext() {
		return lexer.Peek().Row
	} else {
		return -1
	}
}

func (lexer *phase2Lexer) Column() int {
	if lexer.HasNext() {
		return lexer.Peek().Column
	} else {
		return -1
	}
}

func (lexer *phase2Lexer) Offset() int {
	if lexer.HasNext() {
		return lexer.Peek().Offset
	} else {
		return -1
	}
}

func (lexer *phase2Lexer) Diagnostics() []Diagnostic {
	return lexer.diagnostics
}

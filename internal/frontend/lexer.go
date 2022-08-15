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

type Lexer interface {
	HasNext() bool
	HasNextNext() bool
	Next() Token
	Peek() Token
	PeekPeek() Token
	Position() Position
	Diagnostics() []Diagnostic
}

func NewLexer(tokens []Token, diagnostics []Diagnostic) Lexer {
	return &lexer{
		index:       0,
		tokens:      tokens,
		diagnostics: diagnostics,
	}
}

///////////////////////////////////////////////////////////////

type lexer struct {
	index       int
	tokens      []Token
	diagnostics []Diagnostic
}

func (lex *lexer) HasNext() bool {
	return lex.index < len(lex.tokens)
}

func (lex *lexer) HasNextNext() bool {
	return lex.index+1 < len(lex.tokens)
}

func (lex *lexer) Next() Token {
	peek := lex.Peek()
	lex.index++
	return peek
}

func (lex *lexer) Peek() Token {
	return lex.tokens[lex.index]
}

func (lex *lexer) PeekPeek() Token {
	return lex.tokens[lex.index+1]
}

func (lex *lexer) Position() Position {
	if lex.HasNext() {
		return lex.Peek().Position
	} else {
		return Position{
			Offset: -1,
			Row:    -1,
			Column: -1,
		}
	}
}

func (lex *lexer) Diagnostics() []Diagnostic {
	return lex.diagnostics
}

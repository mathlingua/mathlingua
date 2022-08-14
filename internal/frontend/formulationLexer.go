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
	"unicode"
)

type FormulationLexer interface {
	HasNext() bool
	Next() Token
	Peek() Token
	Diagnostics() []Diagnostic
}

func NewFormulationLexer(text string) FormulationLexer {
	lexer := formulationLexer{
		index:       0,
		tokens:      make([]Token, 0),
		diagnostics: make([]Diagnostic, 0),
	}
	lexer.init(text)
	return &lexer
}

//////////////////////////////////////////////////////////////

type formulationLexer struct {
	index       int
	tokens      []Token
	diagnostics []Diagnostic
}

func (lexer *formulationLexer) init(text string) {
	chars := GetChars(text)
	i := 0

	appendToken := func(token Token) {
		lexer.tokens = append(lexer.tokens, token)
	}

	appendDiagnostic := func(message string, position Position) {
		lexer.diagnostics = append(lexer.diagnostics, Diagnostic{
			Type:     Error,
			Origin:   FormulationLexerOrigin,
			Message:  message,
			Position: position,
		})
	}

	absorbWhitespace := func() {
		for i < len(chars) && isWhitespace(chars[i].Symbol) {
			i++
		}
	}

	getName := func(start Char) (string, bool) {
		if i >= len(chars) || !isNameSymbol(start.Symbol) {
			return "", false
		}
		result := string(start.Symbol)
		for i < len(chars) && isNameSymbol(chars[i].Symbol) {
			result += string(chars[i].Symbol)
			i++
		}
		return result, true
	}

	getOperator := func(start Char) (string, bool) {
		if i >= len(chars) || !isOperatorSymbol(start.Symbol) {
			return "", false
		}
		result := string(start.Symbol)
		for i < len(chars) && isOperatorSymbol(chars[i].Symbol) {
			result += string(chars[i].Symbol)
			i++
		}
		return result, true
	}

	for i < len(chars) {
		absorbWhitespace()
		if i >= len(chars) {
			break
		}

		cur := chars[i]
		i++
		switch {
		case cur.Symbol == ',':
			appendToken(Token{
				Type:     Comma,
				Text:     ",",
				Position: cur.Position,
			})
		case cur.Symbol == '(':
			appendToken(Token{
				Type:     LParen,
				Text:     "(",
				Position: cur.Position,
			})
		case cur.Symbol == ')':
			appendToken(Token{
				Type:     RParen,
				Text:     ")",
				Position: cur.Position,
			})
		case cur.Symbol == '[':
			appendToken(Token{
				Type:     LSquare,
				Text:     "[",
				Position: cur.Position,
			})
		case cur.Symbol == ']':
			appendToken(Token{
				Type:     RSquare,
				Text:     "]",
				Position: cur.Position,
			})
		case cur.Symbol == '{':
			appendToken(Token{
				Type:     LCurly,
				Text:     "{",
				Position: cur.Position,
			})
		case cur.Symbol == '}':
			appendToken(Token{
				Type:     RCurly,
				Text:     "}",
				Position: cur.Position,
			})
		case cur.Symbol == '_':
			appendToken(Token{
				Type:     Underscore,
				Text:     "_",
				Position: cur.Position,
			})
		case cur.Symbol == '|':
			appendToken(Token{
				Type:     Bar,
				Text:     "|",
				Position: cur.Position,
			})
		case cur.Symbol == '/':
			appendToken(Token{
				Type:     Slash,
				Text:     "/",
				Position: cur.Position,
			})
		case cur.Symbol == '\\':
			appendToken(Token{
				Type:     BackSlash,
				Text:     "\\",
				Position: cur.Position,
			})
		case cur.Symbol == ';':
			appendToken(Token{
				Type:     Semicolon,
				Text:     ";",
				Position: cur.Position,
			})
		case cur.Symbol == '?':
			appendToken(Token{
				Type:     QuestionMark,
				Text:     "?",
				Position: cur.Position,
			})
		case cur.Symbol == '^':
			appendToken(Token{
				Type:     Caret,
				Text:     "^",
				Position: cur.Position,
			})
		case cur.Symbol == '.':
			if i+1 < len(chars) && chars[i].Symbol == '.' && chars[i+1].Symbol == '.' {
				i += 2 // skip the ..
				appendToken(Token{
					Type:     DotDotDot,
					Text:     "...",
					Position: cur.Position,
				})
			} else {
				appendToken(Token{
					Type:     Dot,
					Text:     ".",
					Position: cur.Position,
				})
			}
		case cur.Symbol == ':':
			if i < len(chars) && chars[i].Symbol == '=' {
				i++ // move past the =
				appendToken(Token{
					Type:     ColonEquals,
					Text:     ":=",
					Position: cur.Position,
				})
			} else {
				appendToken(Token{
					Type:     Colon,
					Text:     ":",
					Position: cur.Position,
				})
			}
		default:
			if name, ok := getName(cur); ok {
				if name == "is" {
					appendToken(Token{
						Type:     Is,
						Text:     "is",
						Position: cur.Position,
					})
				} else if name == "as" {
					appendToken(Token{
						Type:     As,
						Text:     "as",
						Position: cur.Position,
					})
				} else {
					appendToken(Token{
						Type:     Name,
						Text:     name,
						Position: cur.Position,
					})
				}
			} else if op, ok := getOperator(cur); ok {
				appendToken(Token{
					Type:     Operator,
					Text:     op,
					Position: cur.Position,
				})
			} else {
				appendDiagnostic(fmt.Sprintf("Unexpected token '%c'", cur.Symbol), cur.Position)
			}
		}
	}
}

func (lexer *formulationLexer) HasNext() bool {
	return lexer.index < len(lexer.tokens)
}

func (lexer *formulationLexer) Next() Token {
	peek := lexer.Peek()
	lexer.index++
	return peek
}

func (lexer *formulationLexer) Peek() Token {
	return lexer.tokens[lexer.index]
}

func (lexer *formulationLexer) Diagnostics() []Diagnostic {
	return lexer.diagnostics
}

////////////////////////////////////////////////////////////////////////

func isWhitespace(c rune) bool {
	return c == ' ' || c == '\t' || c == '\n' || c == '\r'
}

func isNameSymbol(c rune) bool {
	return unicode.IsLetter(c) || unicode.IsDigit(c)
}

func isOperatorSymbol(c rune) bool {
	return c == '~' ||
		c == '!' ||
		c == '@' ||
		c == '#' ||
		c == '$' ||
		c == '%' ||
		c == '^' ||
		c == '&' ||
		c == '*' ||
		c == '-' ||
		c == '+' ||
		c == '=' ||
		c == '|' ||
		c == '<' ||
		c == '>'
}

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

package formulation

import (
	"fmt"
	"mathlingua/internal/ast"
	"mathlingua/internal/frontend"
	"mathlingua/internal/frontend/shared"
	"unicode"
)

func NewLexer(text string, tracker frontend.DiagnosticTracker) shared.Lexer {
	return shared.NewLexer(getTokens(text, tracker))
}

//////////////////////////////////////////////////////////////

func getTokens(text string, tracker frontend.DiagnosticTracker) []ast.Token {
	tokens := make([]ast.Token, 0)
	chars := ast.GetChars(text)
	i := 0

	appendToken := func(token ast.Token) {
		tokens = append(tokens, token)
	}

	appendDiagnostic := func(message string, position ast.Position) {
		tracker.Append(frontend.Diagnostic{
			Type:     frontend.Error,
			Origin:   frontend.FormulationLexerOrigin,
			Message:  message,
			Position: position,
		})
	}

	absorbWhitespace := func() {
		for i < len(chars) && isWhitespace(chars[i].Symbol) {
			i++
		}
	}

	getName := func(start ast.Char) (string, bool) {
		if i > len(chars) || !isNameSymbol(start.Symbol) {
			return "", false
		}
		result := string(start.Symbol)
		for i < len(chars) && isNameSymbol(chars[i].Symbol) {
			result += string(chars[i].Symbol)
			i++
		}
		return result, true
	}

	getOperator := func(start ast.Char) (string, bool) {
		if i > len(chars) || !isOperatorSymbol(start.Symbol) {
			return "", false
		}
		result := string(start.Symbol)
		for i < len(chars) && isOperatorSymbol(chars[i].Symbol) {
			result += string(chars[i].Symbol)
			i++
		}
		return result, true
	}

	getStroppedName := func(start ast.Char) string {
		result := ""
		for i < len(chars) && chars[i].Symbol != '"' {
			result += string(chars[i].Symbol)
			i++
		}
		if i < len(chars) && chars[i].Symbol == '"' {
			i++ // move past the "
		} else {
			appendDiagnostic("Unterminated \"", start.Position)
		}
		return fmt.Sprintf("\"%s\"", result)
	}

	for i < len(chars) {
		absorbWhitespace()
		if i >= len(chars) {
			break
		}

		cur := chars[i]
		i++
		switch {
		case cur.Symbol == '@':
			appendToken(ast.Token{
				Type:     ast.At,
				Text:     "@",
				Position: cur.Position,
			})
		case cur.Symbol == ',':
			appendToken(ast.Token{
				Type:     ast.Comma,
				Text:     ",",
				Position: cur.Position,
			})
		case cur.Symbol == '(':
			appendToken(ast.Token{
				Type:     ast.LParen,
				Text:     "(",
				Position: cur.Position,
			})
		case cur.Symbol == ')':
			appendToken(ast.Token{
				Type:     ast.RParen,
				Text:     ")",
				Position: cur.Position,
			})
		case cur.Symbol == '[':
			if i < len(chars) && chars[i].Symbol == ':' {
				i++ // move past the :
				appendToken(ast.Token{
					Type:     ast.LSquareColon,
					Text:     "[:",
					Position: cur.Position,
				})
			} else {
				appendToken(ast.Token{
					Type:     ast.LSquare,
					Text:     "[",
					Position: cur.Position,
				})
			}
		case cur.Symbol == ']':
			appendToken(ast.Token{
				Type:     ast.RSquare,
				Text:     "]",
				Position: cur.Position,
			})
		case cur.Symbol == '{':
			appendToken(ast.Token{
				Type:     ast.LCurly,
				Text:     "{",
				Position: cur.Position,
			})
		case cur.Symbol == '}':
			appendToken(ast.Token{
				Type:     ast.RCurly,
				Text:     "}",
				Position: cur.Position,
			})
		case cur.Symbol == '_':
			appendToken(ast.Token{
				Type:     ast.Underscore,
				Text:     "_",
				Position: cur.Position,
			})
		case cur.Symbol == '|':
			appendToken(ast.Token{
				Type:     ast.Bar,
				Text:     "|",
				Position: cur.Position,
			})
		case cur.Symbol == '/':
			appendToken(ast.Token{
				Type:     ast.Slash,
				Text:     "/",
				Position: cur.Position,
			})
		case cur.Symbol == '\\':
			appendToken(ast.Token{
				Type:     ast.BackSlash,
				Text:     "\\",
				Position: cur.Position,
			})
		case cur.Symbol == ';':
			appendToken(ast.Token{
				Type:     ast.Semicolon,
				Text:     ";",
				Position: cur.Position,
			})
		case cur.Symbol == '?':
			appendToken(ast.Token{
				Type:     ast.QuestionMark,
				Text:     "?",
				Position: cur.Position,
			})
		case cur.Symbol == '"':
			name := getStroppedName(cur)
			appendToken(ast.Token{
				Type:     ast.Name,
				Text:     name,
				Position: cur.Position,
			})
		case cur.Symbol == '.':
			if i+1 < len(chars) && chars[i].Symbol == '.' && chars[i+1].Symbol == '.' {
				i += 2 // skip the ..
				appendToken(ast.Token{
					Type:     ast.DotDotDot,
					Text:     "...",
					Position: cur.Position,
				})
			} else {
				appendToken(ast.Token{
					Type:     ast.Dot,
					Text:     ".",
					Position: cur.Position,
				})
			}
		case cur.Symbol == '=':
			if i < len(chars) && chars[i].Symbol == '>' {
				i++ // move past the >
				appendToken(ast.Token{
					Type:     ast.RightArrow,
					Text:     "=>",
					Position: cur.Position,
				})
			} else {
				appendToken(ast.Token{
					Type:     ast.Operator,
					Text:     "=",
					Position: cur.Position,
				})
			}
		case cur.Symbol == ':':
			if i < len(chars) && chars[i].Symbol == '=' {
				i++ // move past the =
				if i < len(chars) && chars[i].Symbol == '>' {
					i++ // move past the >
					appendToken(ast.Token{
						Type:     ast.ColonArrow,
						Text:     ":=>",
						Position: cur.Position,
					})
				} else {
					appendToken(ast.Token{
						Type:     ast.ColonEquals,
						Text:     ":=",
						Position: cur.Position,
					})
				}
			} else if i < len(chars) && chars[i].Symbol == ']' {
				i++ // move past the ]
				appendToken(ast.Token{
					Type:     ast.ColonRSquare,
					Text:     ":]",
					Position: cur.Position,
				})
			} else {
				appendToken(ast.Token{
					Type:     ast.Colon,
					Text:     ":",
					Position: cur.Position,
				})
			}
		default:
			if name, ok := getName(cur); ok {
				if i+1 < len(chars) && chars[i].Symbol == '_' && isNameSymbol(chars[i+1].Symbol) {
					name += "_"
					i++ // move past the _
					firstChar := chars[i]
					i++ // move past the first char
					if subName, ok := getName(firstChar); ok {
						name += subName
					} else {
						name += string(firstChar.Symbol)
					}
				}
				if name == "is" {
					appendToken(ast.Token{
						Type:     ast.Is,
						Text:     "is",
						Position: cur.Position,
					})
				} else if name == "extends" {
					appendToken(ast.Token{
						Type:     ast.Extends,
						Text:     "extends",
						Position: cur.Position,
					})
				} else if name == "as" {
					appendToken(ast.Token{
						Type:     ast.As,
						Text:     "as",
						Position: cur.Position,
					})
				} else {
					appendToken(ast.Token{
						Type:     ast.Name,
						Text:     name,
						Position: cur.Position,
					})
				}
			} else if op, ok := getOperator(cur); ok {
				if i+1 < len(chars) && chars[i].Symbol == '_' && isNameSymbol(chars[i+1].Symbol) {
					op += "_"
					i++ // move past the _
					firstChar := chars[i]
					i++ // move past the first char
					if subName, ok := getName(firstChar); ok {
						op += subName
					} else {
						op += string(firstChar.Symbol)
					}
				}
				appendToken(ast.Token{
					Type:     ast.Operator,
					Text:     op,
					Position: cur.Position,
				})
			} else {
				appendDiagnostic(fmt.Sprintf("Unexpected token '%c'", cur.Symbol), cur.Position)
			}
		}
	}

	return tokens
}

////////////////////////////////////////////////////////////////////////

func isWhitespace(c rune) bool {
	return c == ' ' || c == '\t' || c == '\n' || c == '\r'
}

func isNameSymbol(c rune) bool {
	return unicode.IsLetter(c) || unicode.IsDigit(c) || c == '`' || c == '\''
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

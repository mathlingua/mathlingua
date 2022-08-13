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
	"unicode"
)

type Phase1Lexer interface {
	HasNext() bool
	Next() Token
	Peek() Token
	Diagnostics() []Diagnostic
}

func NewPhase1Lexer(text string) Phase1Lexer {
	lexer := phase1Lexer{
		index:       0,
		tokens:      make([]Token, 0),
		diagnostics: make([]Diagnostic, 0),
	}
	// ensure the text ends with enough newlines so that it
	// terminates any sections and groups.  This makes parsing
	// easier to implement.
	lexer.init(text + "\n\n\n")
	return &lexer
}

////////////////////////////////////////////////////

type phase1Lexer struct {
	index       int
	tokens      []Token
	diagnostics []Diagnostic
}

func (lexer *phase1Lexer) init(text string) {
	chars := GetChars(text)
	i := 0

	appendToken := func(token Token) {
		lexer.tokens = append(lexer.tokens, token)
	}

	appendDiagnostic := func(message string, row int, column int) {
		lexer.diagnostics = append(lexer.diagnostics, Diagnostic{
			Type:    Error,
			Origin:  Phase1LexerOrigin,
			Message: message,
			Row:     row,
			Column:  column,
		})
	}

	collectUntil := func(start Char, terminator rune, allowsEscape bool) string {
		result := ""
		terminatorFound := false
		for i < len(chars) {
			c := chars[i]
			i++
			if allowsEscape && c.symbol == '\\' && i < len(chars) && chars[i].symbol == terminator {
				result += string(c.symbol) + string(chars[i].symbol)
				i++
			} else if c.symbol == terminator {
				terminatorFound = true
				break
			} else {
				result += string(c.symbol)
			}
		}
		if !terminatorFound {
			appendDiagnostic(fmt.Sprintf("Unterminated %c", terminator), start.row, start.column)
		}
		return result
	}

	absorbNewlines := func() {
		for i < len(chars) && chars[i].symbol == '\n' {
			c := chars[i]
			i++
			appendToken(Token{
				Type:   Newline,
				Text:   "<Newline>",
				Offset: c.offset,
				Column: c.column,
				Row:    c.row,
			})
			for i < len(chars) && chars[i].symbol == ' ' {
				c := chars[i]
				i++
				appendToken(Token{
					Type:   Space,
					Text:   "<Space>",
					Offset: c.offset,
					Column: c.column,
					Row:    c.row,
				})
			}
		}
	}

	collectEnclosedArgument := func() (Token, bool) {
		for i < len(chars) && chars[i].symbol == ' ' {
			i++
		}

		if i >= len(chars) {
			return Token{}, false
		}

		c := chars[i]
		if c.symbol == '"' {
			i++
			return Token{
				Type:   Text,
				Text:   collectUntil(c, '"', true),
				Offset: c.offset,
				Row:    c.row,
				Column: c.column,
			}, true
		} else if c.symbol == '\'' {
			i++
			return Token{
				Type:   Formulation,
				Text:   collectUntil(c, '\'', false),
				Offset: c.offset,
				Row:    c.row,
				Column: c.column,
			}, true
		} else if c.symbol == '`' {
			i++
			return Token{
				Type:   Formulation,
				Text:   collectUntil(c, '`', false),
				Offset: c.offset,
				Row:    c.row,
				Column: c.column,
			}, true
		} else {
			return Token{}, false
		}
	}

	collectNonEnclosedArgument := func() (Token, bool) {
		for i < len(chars) && chars[i].symbol == ' ' {
			i++
		}

		if i >= len(chars) {
			return Token{}, false
		}

		start := chars[i]
		result := ""
		stack := mlglib.NewStack[rune]()
		for i < len(chars) {
			c := chars[i]
			if stack.IsEmpty() && (c.symbol == '\n' || c.symbol == ',') {
				break
			}
			i++
			result += string(c.symbol)
			if c.symbol == '(' || c.symbol == '[' || c.symbol == '{' {
				stack.Push(c.symbol)
			} else if c.symbol == ')' {
				if !stack.IsEmpty() && stack.Peek() == '(' {
					stack.Pop()
				}
			} else if c.symbol == ']' {
				if !stack.IsEmpty() && stack.Peek() == '[' {
					stack.Pop()
				}
			} else if c.symbol == '}' {
				if !stack.IsEmpty() && stack.Peek() == '{' {
					stack.Pop()
				}
			}
		}

		if len(result) == 0 {
			return Token{}, false
		}

		return Token{
			Type:   ArgumentText,
			Text:   result,
			Offset: start.offset,
			Row:    start.row,
			Column: start.column,
		}, true
	}

	collectArgument := func() (Token, bool) {
		arg, ok := collectEnclosedArgument()
		if ok {
			return arg, true
		}
		return collectNonEnclosedArgument()
	}

	collectAllArguments := func() {
		for i < len(chars) {
			arg, ok := collectArgument()
			if !ok {
				break
			}
			appendToken(arg)
			if i >= len(chars) || chars[i].symbol == '\n' {
				break
			} else if chars[i].symbol == ',' {
				start := chars[i]
				i++
				appendToken(Token{
					Type:   Comma,
					Text:   ",",
					Offset: start.offset,
					Row:    start.row,
					Column: start.column,
				})
			} else if chars[i].symbol == ' ' {
				appendDiagnostic("Unnecessary trailing whitespace", arg.Row, arg.Column)
			} else {
				appendDiagnostic("Expected a , to follow this argument", arg.Row, arg.Column)
			}
		}
	}

	collectName := func(start Char) {
		result := string(start.symbol)
		for i < len(chars) && unicode.IsLetter(chars[i].symbol) {
			result += string(chars[i].symbol)
			i++
		}
		appendToken(Token{
			Type:   Name,
			Text:   result,
			Offset: start.offset,
			Row:    start.row,
			Column: start.column,
		})
	}

	collectId := func(start Char) {
		result := ""
		stack := mlglib.NewStack[rune]()
		for i < len(chars) && chars[i].symbol != '\n' {
			c := chars[i]
			i++
			if c.symbol == '[' {
				stack.Push(c.symbol)
				result += string(c.symbol)
			} else if c.symbol == ']' {
				if !stack.IsEmpty() && stack.Peek() == '[' {
					stack.Pop()
				}
				if stack.IsEmpty() {
					break
				} else {
					result += string(c.symbol)
				}
			} else {
				result += string(c.symbol)
			}
		}
		appendToken(Token{
			Type:   Id,
			Text:   result,
			Offset: start.offset,
			Row:    start.row,
			Column: start.column,
		})
	}

	collectTexBlock := func(start Char) {
		result := ""
		for i < len(chars) {
			c := chars[i]
			i++
			if c.symbol == ':' && i < len(chars) && chars[i].symbol == ':' {
				// move past the second :
				i++
				break
			} else {
				result += string(c.symbol)
			}
		}
		appendToken(Token{
			Type:   TextBlock,
			Text:   result,
			Offset: start.offset,
			Row:    start.row,
			Column: start.column,
		})
	}

	isNextNameColon := func() bool {
		hasName := false
		j := i
		for j < len(chars) && unicode.IsLetter(chars[j].symbol) {
			j++
			hasName = true
		}
		return hasName && j < len(chars) && chars[j].symbol == ':'
	}

	for i < len(chars) {
		absorbNewlines()
		if i >= len(chars) {
			break
		}

		c := chars[i]
		i++

		if unicode.IsLetter(c.symbol) {
			collectName(c)
		} else if c.symbol == ':' {
			if i < len(chars) && chars[i].symbol == ':' {
				// process :: text blocks
				// move past the second :
				i++
				collectTexBlock(c)
			} else {
				// the : is processed as being at the end of <name>:
				appendToken(Token{
					Type:   Colon,
					Text:   ":",
					Offset: c.offset,
					Row:    c.row,
					Column: c.column,
				})
				collectAllArguments()
			}
		} else if c.symbol == '.' {
			if i < len(chars) && chars[i].symbol == ' ' {
				// move past the space
				i++
				appendToken(Token{
					Type:   DotSpace,
					Text:   "<DotSpace>",
					Offset: c.offset,
					Row:    c.row,
					Column: c.column,
				})
				if !isNextNameColon() {
					collectAllArguments()
				}
			} else {
				appendDiagnostic(fmt.Sprintf("Unexpected character '%c'", c.symbol), c.row, c.column)
			}
		} else if c.symbol == '[' {
			collectId(c)
		} else {
			appendDiagnostic(fmt.Sprintf("Unrecognized character '%c'", c.symbol), c.row, c.column)
		}
	}
}

func (lexer *phase1Lexer) HasNext() bool {
	return lexer.index < len(lexer.tokens)
}

func (lexer *phase1Lexer) Next() Token {
	peek := lexer.Peek()
	lexer.index++
	return peek
}

func (lexer *phase1Lexer) Peek() Token {
	return lexer.tokens[lexer.index]
}

func (lexer *phase1Lexer) Diagnostics() []Diagnostic {
	return lexer.diagnostics
}

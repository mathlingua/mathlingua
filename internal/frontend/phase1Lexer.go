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
	"mathlingua/internal/mlglib"
	"unicode"
)

func NewPhase1Lexer(text string) Lexer {
	// ensure the text ends with enough newlines so that it
	// terminates any sections and groups.  This makes parsing
	// easier to implement.
	return NewLexer(getPhase1Tokens(text + "\n\n\n"))
}

////////////////////////////////////////////////////

func getPhase1Tokens(text string) ([]Token, []Diagnostic) {
	chars := GetChars(text)
	i := 0

	tokens := make([]Token, 0)
	diagnostics := make([]Diagnostic, 0)

	appendToken := func(token Token) {
		tokens = append(tokens, token)
	}

	appendDiagnostic := func(message string, position ast.Position) {
		diagnostics = append(diagnostics, Diagnostic{
			Type:     Error,
			Origin:   Phase1LexerOrigin,
			Message:  message,
			Position: position,
		})
	}

	collectUntil := func(start Char, terminator rune, allowsEscape bool) string {
		result := ""
		terminatorFound := false
		for i < len(chars) {
			c := chars[i]
			i++
			if allowsEscape && c.Symbol == '\\' && i < len(chars) && chars[i].Symbol == terminator {
				result += string(c.Symbol) + string(chars[i].Symbol)
				i++
			} else if c.Symbol == terminator {
				terminatorFound = true
				break
			} else {
				result += string(c.Symbol)
			}
		}
		if !terminatorFound {
			appendDiagnostic(fmt.Sprintf("Unterminated %c", terminator), start.Position)
		}
		return result
	}

	absorbNewlines := func() {
		for i < len(chars) && chars[i].Symbol == '\n' {
			c := chars[i]
			i++
			appendToken(Token{
				Type:     Newline,
				Text:     "<Newline>",
				Position: c.Position,
			})
			for i < len(chars) && chars[i].Symbol == ' ' {
				c := chars[i]
				i++
				appendToken(Token{
					Type:     Space,
					Text:     "<Space>",
					Position: c.Position,
				})
			}
		}
	}

	collectEnclosedArgument := func() (Token, bool) {
		for i < len(chars) && chars[i].Symbol == ' ' {
			i++
		}

		if i >= len(chars) {
			return Token{}, false
		}

		c := chars[i]
		if c.Symbol == '"' {
			i++
			return Token{
				Type:     Text,
				Text:     collectUntil(c, '"', true),
				Position: c.Position,
			}, true
		} else if c.Symbol == '\'' {
			i++
			return Token{
				Type:     Formulation,
				Text:     collectUntil(c, '\'', false),
				Position: c.Position,
			}, true
		} else if c.Symbol == '`' {
			i++
			return Token{
				Type:     Formulation,
				Text:     collectUntil(c, '`', false),
				Position: c.Position,
			}, true
		} else {
			return Token{}, false
		}
	}

	collectNonEnclosedArgument := func() (Token, bool) {
		for i < len(chars) && chars[i].Symbol == ' ' {
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
			if stack.IsEmpty() && (c.Symbol == '\n' || c.Symbol == ',') {
				break
			}
			i++
			result += string(c.Symbol)
			if c.Symbol == '(' || c.Symbol == '[' || c.Symbol == '{' {
				stack.Push(c.Symbol)
			} else if c.Symbol == ')' {
				if !stack.IsEmpty() && stack.Peek() == '(' {
					stack.Pop()
				}
			} else if c.Symbol == ']' {
				if !stack.IsEmpty() && stack.Peek() == '[' {
					stack.Pop()
				}
			} else if c.Symbol == '}' {
				if !stack.IsEmpty() && stack.Peek() == '{' {
					stack.Pop()
				}
			}
		}

		if len(result) == 0 {
			return Token{}, false
		}

		return Token{
			Type:     ArgumentText,
			Text:     result,
			Position: start.Position,
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
			if i >= len(chars) || chars[i].Symbol == '\n' {
				break
			} else if chars[i].Symbol == ',' {
				start := chars[i]
				i++
				appendToken(Token{
					Type:     Comma,
					Text:     ",",
					Position: start.Position,
				})
			} else if chars[i].Symbol == ' ' {
				appendDiagnostic("Unnecessary trailing whitespace", arg.Position)
			} else {
				appendDiagnostic("Expected a , to follow this argument", arg.Position)
			}
		}
	}

	collectName := func(start Char) {
		result := string(start.Symbol)
		for i < len(chars) && unicode.IsLetter(chars[i].Symbol) {
			result += string(chars[i].Symbol)
			i++
		}
		appendToken(Token{
			Type:     Name,
			Text:     result,
			Position: start.Position,
		})
	}

	collectId := func(start Char) {
		result := ""
		stack := mlglib.NewStack[rune]()
		for i < len(chars) && chars[i].Symbol != '\n' {
			c := chars[i]
			i++
			if c.Symbol == '[' {
				stack.Push(c.Symbol)
				result += string(c.Symbol)
			} else if c.Symbol == ']' {
				if !stack.IsEmpty() && stack.Peek() == '[' {
					stack.Pop()
				}
				if stack.IsEmpty() {
					break
				} else {
					result += string(c.Symbol)
				}
			} else {
				result += string(c.Symbol)
			}
		}
		appendToken(Token{
			Type:     Id,
			Text:     result,
			Position: start.Position,
		})
	}

	collectTexBlock := func(start Char) {
		result := ""
		for i < len(chars) {
			c := chars[i]
			i++
			if c.Symbol == ':' && i < len(chars) && chars[i].Symbol == ':' {
				// move past the second :
				i++
				break
			} else {
				result += string(c.Symbol)
			}
		}
		appendToken(Token{
			Type:     TextBlock,
			Text:     result,
			Position: start.Position,
		})
	}

	isNextNameColon := func() bool {
		hasName := false
		j := i
		for j < len(chars) && unicode.IsLetter(chars[j].Symbol) {
			j++
			hasName = true
		}
		return hasName && j < len(chars) && chars[j].Symbol == ':'
	}

	for i < len(chars) {
		absorbNewlines()
		if i >= len(chars) {
			break
		}

		c := chars[i]
		i++

		if unicode.IsLetter(c.Symbol) {
			collectName(c)
		} else if c.Symbol == ':' {
			if i < len(chars) && chars[i].Symbol == ':' {
				// process :: text blocks
				// move past the second :
				i++
				collectTexBlock(c)
			} else {
				// the : is processed as being at the end of <name>:
				appendToken(Token{
					Type:     Colon,
					Text:     ":",
					Position: c.Position,
				})
				collectAllArguments()
			}
		} else if c.Symbol == '.' {
			if i < len(chars) && chars[i].Symbol == ' ' {
				// move past the space
				i++
				appendToken(Token{
					Type:     DotSpace,
					Text:     "<DotSpace>",
					Position: c.Position,
				})
				if !isNextNameColon() {
					collectAllArguments()
				}
			} else {
				appendDiagnostic(fmt.Sprintf("Unexpected character '%c'", c.Symbol), c.Position)
			}
		} else if c.Symbol == '[' {
			collectId(c)
		} else {
			appendDiagnostic(fmt.Sprintf("Unrecognized character '%c'", c.Symbol), c.Position)
		}
	}

	return tokens, diagnostics
}

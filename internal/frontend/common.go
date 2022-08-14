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

type DiagnosticType string

const (
	Error DiagnosticType = "Error"
)

type DiagnosticOrigin string

const (
	Phase1LexerOrigin      DiagnosticOrigin = "Phase1LexerOrigin"
	Phase2LexerOrigin      DiagnosticOrigin = "Phase2LexerOrigin"
	Phase3LexerOrigin      DiagnosticOrigin = "Phase3LexerOrigin"
	FormulationLexerOrigin DiagnosticOrigin = "FormulationLexerOrigin"
)

type Position struct {
	Offset int
	Row    int
	Column int
}

type Diagnostic struct {
	Type     DiagnosticType
	Origin   DiagnosticOrigin
	Message  string
	Position Position
}

type TokenType string

const (
	Name                  TokenType = "Name"
	Colon                 TokenType = "Colon"
	Text                  TokenType = "Text"
	Formulation           TokenType = "Formulation"
	TextBlock             TokenType = "TextBlock"
	Indent                TokenType = "Indent"
	UnIndent              TokenType = "Unindent"
	SameIndent            TokenType = "SameIndent"
	DotSpace              TokenType = "DotSpace"
	LineBreak             TokenType = "LineBreak"
	Id                    TokenType = "Id"
	Newline               TokenType = "Newline"
	ArgumentText          TokenType = "ArgumentText"
	Comma                 TokenType = "Comma"
	Space                 TokenType = "Space"
	BeginArgumentGroup    TokenType = "BeginArgumentGroup"
	EndArgumentGroup      TokenType = "EndArgumentGroup"
	BeginTopLevelGroup    TokenType = "BeginTopLevelGroup"
	EndTopLevelGroup      TokenType = "EndTopLevelGroup"
	BeginSection          TokenType = "BeginSection"
	EndSection            TokenType = "EndSection"
	BeginDotSpaceArgument TokenType = "BeginDotSpaceArgument"
	EndDotSpaceArgument   TokenType = "EndDotSpaceArgument"
	BeginInlineArgument   TokenType = "BeginInlineArgument"
	EndInlineArgument     TokenType = "EndInlineArgument"
	Operator              TokenType = "Operator"
	LParen                TokenType = "LParen"
	RParen                TokenType = "RParen"
	LSquare               TokenType = "LSquare"
	RSquare               TokenType = "RSquare"
	LCurly                TokenType = "LCurly"
	RCurly                TokenType = "RCurly"
	Underscore            TokenType = "Underscore"
	Bar                   TokenType = "Bar"
	Dot                   TokenType = "Dot"
	Slash                 TokenType = "Slash"
	BackSlash             TokenType = "BackSlash"
	Semicolon             TokenType = "Semicolon"
	ColonEquals           TokenType = "ColonEquals"
	DotDotDot             TokenType = "DotDotDot"
	QuestionMark          TokenType = "QuestionMark"
	Is                    TokenType = "Is"
	As                    TokenType = "As"
	Caret                 TokenType = "Caret"
)

type Token struct {
	Type     TokenType
	Text     string
	Position Position
}

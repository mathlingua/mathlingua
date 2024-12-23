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
	"mathlingua/internal/frontend"
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestFormulationLexer(t *testing.T) {
	tracker := frontend.NewDiagnosticTracker()
	lexer := NewLexer("/some/path", `
xyzABC123 +*-? f(x, y, z) [x]{(a, b) | a ; b} f(x...) \command[x]_{a}^{b}:f{x}(y) x.y x is `+
		`\something/ x as \[something] "*+" name @ extends (. .)|->abc=:->....[..] `+
		"{..}[||]{::}:=: name` *+`$",
		tracker)

	actual := "\n"
	for lexer.HasNext() {
		next := lexer.Next()
		actual += fmt.Sprintf("%s %s\n", next.Text, next.Type)
	}

	expected := `
xyzABC123 Name
+*- Operator
? QuestionMark
f Name
( LParen
x Name
, Comma
y Name
, Comma
z Name
) RParen
[ LSquare
x Name
] RSquare
{ LCurly
( LParen
a Name
, Comma
b Name
) RParen
| Bar
a Name
; Semicolon
b Name
} RCurly
f Name
( LParen
x Name
... DotDotDot
) RParen
\ BackSlash
command Name
[ LSquare
x Name
] RSquare
_ Underscore
{ LCurly
a Name
} RCurly
^ Operator
{ LCurly
b Name
} RCurly
: Colon
f Name
{ LCurly
x Name
} RCurly
( LParen
y Name
) RParen
x Name
. Dot
y Name
x Name
is Is
\ BackSlash
something Name
/ Slash
x Name
as As
\ BackSlash
[ LSquare
something Name
] RSquare
"*+" Name
name Name
@ At
extends Extends
(. LParenDot
.) DotRParen
|-> BarRightDashArrow
abc Name
= Operator
:-> ColonDashArrow
.... Name
[. LSquareDot
.] DotRSquare
{. LCurlyDot
.} DotRCurly
[| LSquareBar
|] BarRSquare
{: LCurlyColon
:} ColonRCurly
:=: ColonEqualsColon
` + "name` Name\n*+` Operator\n$ DollarSign\n"

	assert.Equal(t, expected, actual)
	assert.Equal(t, []frontend.Diagnostic{}, tracker.Diagnostics())
}

func TestFormulationLexerMultiNames(t *testing.T) {
	tracker := frontend.NewDiagnosticTracker()
	lexer := NewLexer("/some/path", "a b c", tracker)

	actual := "\n"
	for lexer.HasNext() {
		next := lexer.Next()
		actual += fmt.Sprintf("%s %s\n", next.Text, next.Type)
	}

	expected := `
a Name
b Name
c Name
`

	assert.Equal(t, expected, actual)
	assert.Equal(t, []frontend.Diagnostic{}, tracker.Diagnostics())
}

func TestFormulationLexerFunctionVarArg(t *testing.T) {
	tracker := frontend.NewDiagnosticTracker()
	lexer := NewLexer("/some/path", "f(x)...", tracker)

	actual := "\n"
	for lexer.HasNext() {
		next := lexer.Next()
		actual += fmt.Sprintf("%s %s\n", next.Text, next.Type)
	}

	expected := `
f Name
( LParen
x Name
) RParen
... DotDotDot
`

	assert.Equal(t, expected, actual)
	assert.Equal(t, []frontend.Diagnostic{}, tracker.Diagnostics())
}

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
	"strings"
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestPhase3LexerIndentWithDoubleUnindent(t *testing.T) {
	lexer1 := NewPhase1Lexer(`
a:
. b:
  . c:
d:
`)
	lexer2 := NewPhase2Lexer(lexer1)
	lexer3 := NewPhase3Lexer(lexer2)

	actual := "\n"
	for lexer3.HasNext() {
		actual += lexer3.Next().Text + "\n"
	}

	expected := strings.ReplaceAll(`
<BeginTopLevelGroup>
	<BeginSection>
		a
		:
		<BeginArgument>
			<BeginArgumentGroup>
				<BeginSection>
					b
					:
					<BeginArgument>
						<BeginArgumentGroup>
							<BeginSection>
								c
								:
							<EndSection>
						<EndArgumentGroup>
					<EndArgument>
				<EndSection>
			<EndArgumentGroup>
		<EndArgument>
	<EndSection>
	<BeginSection>
		d
		:
	<EndSection>
<EndTopLevelGroup>
`, "\t", "")

	assert.Equal(t, expected, actual)
	assert.Equal(t, []Diagnostic{}, lexer3.Diagnostics())
}

func TestPhase3LexerIndentWithDoubleUnindentMultiSections(t *testing.T) {
	lexer1 := NewPhase1Lexer(`
a:
. b:
  . c:
d:
e:
`)
	lexer2 := NewPhase2Lexer(lexer1)
	lexer3 := NewPhase3Lexer(lexer2)

	actual := "\n"
	for lexer3.HasNext() {
		actual += lexer3.Next().Text + "\n"
	}

	expected := strings.ReplaceAll(`
<BeginTopLevelGroup>
	<BeginSection>
		a
		:
		<BeginArgument>
			<BeginArgumentGroup>
				<BeginSection>
					b
					:
					<BeginArgument>
						<BeginArgumentGroup>
							<BeginSection>
								c
								:
							<EndSection>
						<EndArgumentGroup>
					<EndArgument>
				<EndSection>
			<EndArgumentGroup>
		<EndArgument>
	<EndSection>
	<BeginSection>
		d
		:
	<EndSection>
	<BeginSection>
		e
		:
	<EndSection>
<EndTopLevelGroup>
`, "\t", "")

	assert.Equal(t, expected, actual)
	assert.Equal(t, []Diagnostic{}, lexer3.Diagnostics())
}

func TestPhase3LexerMultiIndent(t *testing.T) {
	lexer1 := NewPhase1Lexer(`
a:
. b:
  . c:
`)
	lexer2 := NewPhase2Lexer(lexer1)
	lexer3 := NewPhase3Lexer(lexer2)

	actual := "\n"
	for lexer3.HasNext() {
		actual += lexer3.Next().Text + "\n"
	}

	expected := strings.ReplaceAll(`
<BeginTopLevelGroup>
	<BeginSection>
		a
		:
		<BeginArgument>
			<BeginArgumentGroup>
				<BeginSection>
					b
					:
					<BeginArgument>
						<BeginArgumentGroup>
							<BeginSection>
								c
								:
							<EndSection>
						<EndArgumentGroup>
					<EndArgument>
				<EndSection>
			<EndArgumentGroup>
		<EndArgument>
	<EndSection>
<EndTopLevelGroup>
`, "\t", "")

	assert.Equal(t, expected, actual)
	assert.Equal(t, []Diagnostic{}, lexer3.Diagnostics())
}

func TestPhase3LexerSingleSection(t *testing.T) {
	lexer1 := NewPhase1Lexer(`
a:
`)
	lexer2 := NewPhase2Lexer(lexer1)
	lexer3 := NewPhase3Lexer(lexer2)

	actual := "\n"
	for lexer3.HasNext() {
		actual += lexer3.Next().Text + "\n"
	}

	expected := strings.ReplaceAll(`
<BeginTopLevelGroup>
	<BeginSection>
		a
		:
	<EndSection>
<EndTopLevelGroup>
`, "\t", "")

	assert.Equal(t, expected, actual)
	assert.Equal(t, []Diagnostic{}, lexer3.Diagnostics())
}

func TestPhase3LexerSingleSectionWithSingleArg(t *testing.T) {
	lexer1 := NewPhase1Lexer(`
a: xyz
`)
	lexer2 := NewPhase2Lexer(lexer1)
	lexer3 := NewPhase3Lexer(lexer2)

	actual := "\n"
	for lexer3.HasNext() {
		actual += lexer3.Next().Text + "\n"
	}

	expected := strings.ReplaceAll(`
<BeginTopLevelGroup>
	<BeginSection>
		a
		:
		xyz
	<EndSection>
<EndTopLevelGroup>
`, "\t", "")

	assert.Equal(t, expected, actual)
	assert.Equal(t, []Diagnostic{}, lexer3.Diagnostics())
}

func TestPhase3LexerSingleSectionWithMultiArgs(t *testing.T) {
	lexer1 := NewPhase1Lexer(`
a: xyz, abc
`)
	lexer2 := NewPhase2Lexer(lexer1)
	lexer3 := NewPhase3Lexer(lexer2)

	actual := "\n"
	for lexer3.HasNext() {
		actual += lexer3.Next().Text + "\n"
	}

	expected := strings.ReplaceAll(`
<BeginTopLevelGroup>
	<BeginSection>
		a
		:
		xyz
		,
		abc
	<EndSection>
<EndTopLevelGroup>
`, "\t", "")

	assert.Equal(t, expected, actual)
	assert.Equal(t, []Diagnostic{}, lexer3.Diagnostics())
}

func TestPhase3LexerSingleSectionWithMultiLineNonGroupArgs(t *testing.T) {
	lexer1 := NewPhase1Lexer(`
a:
. b
. c
`)
	lexer2 := NewPhase2Lexer(lexer1)
	lexer3 := NewPhase3Lexer(lexer2)

	actual := "\n"
	for lexer3.HasNext() {
		actual += lexer3.Next().Text + "\n"
	}

	expected := strings.ReplaceAll(`
<BeginTopLevelGroup>
	<BeginSection>
		a
		:
		<BeginArgument>
			b
		<EndArgument>
		<BeginArgument>
			c
		<EndArgument>
	<EndSection>
<EndTopLevelGroup>
`, "\t", "")

	assert.Equal(t, expected, actual)
	assert.Equal(t, []Diagnostic{}, lexer3.Diagnostics())
}

func TestPhase3LexerSingleSectionWithMultiGroupArgs(t *testing.T) {
	lexer1 := NewPhase1Lexer(`
a:
. b:
. c:
`)
	lexer2 := NewPhase2Lexer(lexer1)
	lexer3 := NewPhase3Lexer(lexer2)

	actual := "\n"
	for lexer3.HasNext() {
		actual += lexer3.Next().Text + "\n"
	}

	expected := strings.ReplaceAll(`
<BeginTopLevelGroup>
	<BeginSection>
		a
		:
		<BeginArgument>
			<BeginArgumentGroup>
				<BeginSection>
					b
					:
				<EndSection>
			<EndArgumentGroup>
		<EndArgument>
		<BeginArgument>
			<BeginArgumentGroup>
				<BeginSection>
					c
					:
				<EndSection>
			<EndArgumentGroup>
		<EndArgument>
	<EndSection>
<EndTopLevelGroup>
`, "\t", "")

	assert.Equal(t, expected, actual)
	assert.Equal(t, []Diagnostic{}, lexer3.Diagnostics())
}

func TestPhase3LexerMultiSections(t *testing.T) {
	lexer1 := NewPhase1Lexer(`
a:
b:
c:
`)
	lexer2 := NewPhase2Lexer(lexer1)
	lexer3 := NewPhase3Lexer(lexer2)

	actual := "\n"
	for lexer3.HasNext() {
		actual += lexer3.Next().Text + "\n"
	}

	expected := strings.ReplaceAll(`
<BeginTopLevelGroup>
	<BeginSection>
		a
		:
	<EndSection>
	<BeginSection>
		b
		:
	<EndSection>
	<BeginSection>
		c
		:
	<EndSection>
<EndTopLevelGroup>
`, "\t", "")

	assert.Equal(t, expected, actual)
	assert.Equal(t, []Diagnostic{}, lexer3.Diagnostics())
}

func TestPhase3LexerMultiSectionsWithGroupArgs(t *testing.T) {
	lexer1 := NewPhase1Lexer(`
a:
. x:
  y:
b:
. A:
c:
. P:
  Q:
  R:
`)
	lexer2 := NewPhase2Lexer(lexer1)
	lexer3 := NewPhase3Lexer(lexer2)

	actual := "\n"
	for lexer3.HasNext() {
		actual += lexer3.Next().Text + "\n"
	}

	expected := strings.ReplaceAll(`
<BeginTopLevelGroup>
	<BeginSection>
		a
		:
		<BeginArgument>
			<BeginArgumentGroup>
				<BeginSection>
					x
					:
				<EndSection>
				<BeginSection>
					y
					:
				<EndSection>
			<EndArgumentGroup>
		<EndArgument>
	<EndSection>
	<BeginSection>
		b
		:
		<BeginArgument>
			<BeginArgumentGroup>
				<BeginSection>
					A
					:
				<EndSection>
			<EndArgumentGroup>
		<EndArgument>
	<EndSection>
	<BeginSection>
		c
		:
		<BeginArgument>
			<BeginArgumentGroup>
				<BeginSection>
					P
					:
				<EndSection>
				<BeginSection>
					Q
					:
				<EndSection>
				<BeginSection>
					R
					:
				<EndSection>
			<EndArgumentGroup>
		<EndArgument>
	<EndSection>
<EndTopLevelGroup>
`, "\t", "")

	assert.Equal(t, expected, actual)
	assert.Equal(t, []Diagnostic{}, lexer3.Diagnostics())
}

func TestPhase3LexerMultiSectionsWithGroupArgsAndNonGroupArgs(t *testing.T) {
	lexer1 := NewPhase1Lexer(`
a:
. x:x1,x2,x3
  y:y1
b:
. A:A1,A2
c:
. P:
  Q:Q1
  R:
`)
	lexer2 := NewPhase2Lexer(lexer1)
	lexer3 := NewPhase3Lexer(lexer2)

	actual := "\n"
	for lexer3.HasNext() {
		actual += lexer3.Next().Text + "\n"
	}

	expected := strings.ReplaceAll(`
<BeginTopLevelGroup>
	<BeginSection>
		a
		:
		<BeginArgument>
			<BeginArgumentGroup>
				<BeginSection>
					x
					:
					x1
					,
					x2
					,
					x3
				<EndSection>
				<BeginSection>
					y
					:
					y1
				<EndSection>
			<EndArgumentGroup>
		<EndArgument>
	<EndSection>
	<BeginSection>
		b
		:
		<BeginArgument>
			<BeginArgumentGroup>
				<BeginSection>
					A
					:
					A1
					,
					A2
				<EndSection>
			<EndArgumentGroup>
		<EndArgument>
	<EndSection>
	<BeginSection>
		c
		:
		<BeginArgument>
			<BeginArgumentGroup>
				<BeginSection>
					P
					:
				<EndSection>
				<BeginSection>
					Q
					:
					Q1
				<EndSection>
				<BeginSection>
					R
					:
				<EndSection>
			<EndArgumentGroup>
		<EndArgument>
	<EndSection>
<EndTopLevelGroup>
`, "\t", "")

	assert.Equal(t, expected, actual)
	assert.Equal(t, []Diagnostic{}, lexer3.Diagnostics())
}

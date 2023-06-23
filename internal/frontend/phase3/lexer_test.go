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

package phase3

import (
	"mathlingua/internal/frontend"
	"mathlingua/internal/frontend/phase1"
	"mathlingua/internal/frontend/phase2"
	"strings"
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestPhase3LexerSingleSection(t *testing.T) {
	tracker := frontend.NewDiagnosticTracker()
	lexer1 := phase1.NewLexer(`
a:
`, "", tracker)
	lexer2 := phase2.NewLexer(lexer1, "", tracker)
	lexer3 := NewLexer(lexer2, "", tracker)

	actual := "\n"
	for lexer3.HasNext() {
		actual += lexer3.Next().Text + "\n"
	}

	expected := strings.ReplaceAll(`
<BeginGroup>
	<BeginSection>
		a
	<EndSection>
<EndGroup>
`, "\t", "")

	assert.Equal(t, expected, actual)
	assert.Equal(t, []frontend.Diagnostic{}, tracker.Diagnostics())
}

func TestPhase3LexerGroupLabel(t *testing.T) {
	tracker := frontend.NewDiagnosticTracker()
	lexer1 := phase1.NewLexer(`
[some.label]
a:
`, "", tracker)
	lexer2 := phase2.NewLexer(lexer1, "", tracker)
	lexer3 := NewLexer(lexer2, "", tracker)

	actualText := "\n"
	actualTypes := "\n"
	for lexer3.HasNext() {
		next := lexer3.Next()
		actualText += next.Text + "\n"
		actualTypes += string(next.Type) + "\n"
	}

	expectedText := strings.ReplaceAll(`
		some.label
		<BeginGroup>
			<BeginSection>
				a
			<EndSection>
		<EndGroup>
		`, "\t", "")

	expectedTypes := strings.ReplaceAll(`
		Id
		BeginGroup
		BeginSection
		Name
		EndSection
		EndGroup
	`, "\t", "")

	assert.Equal(t, expectedText, actualText)
	assert.Equal(t, expectedTypes, actualTypes)
	assert.Equal(t, []frontend.Diagnostic{}, tracker.Diagnostics())
}

func TestPhase3LexerIndentedGroupLabel(t *testing.T) {
	tracker := frontend.NewDiagnosticTracker()
	lexer1 := phase1.NewLexer(`
a:
. [some.label]
  b:
`, "", tracker)
	lexer2 := phase2.NewLexer(lexer1, "", tracker)
	lexer3 := NewLexer(lexer2, "", tracker)

	actualText := "\n"
	actualTypes := "\n"
	for lexer3.HasNext() {
		next := lexer3.Next()
		actualText += next.Text + "\n"
		actualTypes += string(next.Type) + "\n"
	}

	expectedText := strings.ReplaceAll(`
		<BeginGroup>
			<BeginSection>
				a
				<BeginDotSpaceArgument>
					some.label
					<BeginGroup>
						<BeginSection>
							b
						<EndSection>
					<EndGroup>
				<EndDotSpaceArgument>
			<EndSection>
		<EndGroup>
		`, "\t", "")

	expectedTypes := strings.ReplaceAll(`
		BeginGroup
			BeginSection
				Name
				BeginDotSpaceArgument
					Id
					BeginGroup
						BeginSection
							Name
						EndSection
					EndGroup
				EndDotSpaceArgument
			EndSection
		EndGroup
		`, "\t", "")

	assert.Equal(t, expectedText, actualText)
	assert.Equal(t, expectedTypes, actualTypes)
	assert.Equal(t, []frontend.Diagnostic{}, tracker.Diagnostics())
}

func TestPhase3LexerSingleSectionWithSingleArg(t *testing.T) {
	tracker := frontend.NewDiagnosticTracker()
	lexer1 := phase1.NewLexer(`
a: xyz
`, "", tracker)
	lexer2 := phase2.NewLexer(lexer1, "", tracker)
	lexer3 := NewLexer(lexer2, "", tracker)

	actual := "\n"
	for lexer3.HasNext() {
		actual += lexer3.Next().Text + "\n"
	}

	expected := strings.ReplaceAll(`
<BeginGroup>
	<BeginSection>
		a
		<BeginInlineArgument>
			xyz
		<EndInlineArgument>
	<EndSection>
<EndGroup>
`, "\t", "")

	assert.Equal(t, expected, actual)
	assert.Equal(t, []frontend.Diagnostic{}, tracker.Diagnostics())
}

func TestPhase3LexerSingleSectionWithMultiArgs(t *testing.T) {
	tracker := frontend.NewDiagnosticTracker()
	lexer1 := phase1.NewLexer(`
a: xyz, abc
`, "", tracker)
	lexer2 := phase2.NewLexer(lexer1, "", tracker)
	lexer3 := NewLexer(lexer2, "", tracker)

	actual := "\n"
	for lexer3.HasNext() {
		actual += lexer3.Next().Text + "\n"
	}

	expected := strings.ReplaceAll(`
<BeginGroup>
	<BeginSection>
		a
		<BeginInlineArgument>
		xyz
		<EndInlineArgument>
		<BeginInlineArgument>
		abc
		<EndInlineArgument>
	<EndSection>
<EndGroup>
`, "\t", "")

	assert.Equal(t, expected, actual)
	assert.Equal(t, []frontend.Diagnostic{}, tracker.Diagnostics())
}

func TestPhase3LexerIndentWithDoubleUnindent(t *testing.T) {
	tracker := frontend.NewDiagnosticTracker()
	lexer1 := phase1.NewLexer(`
a:
. b:
  . c:
d:
`, "", tracker)
	lexer2 := phase2.NewLexer(lexer1, "", tracker)
	lexer3 := NewLexer(lexer2, "", tracker)

	actual := "\n"
	for lexer3.HasNext() {
		actual += lexer3.Next().Text + "\n"
	}

	expected := strings.ReplaceAll(`
<BeginGroup>
	<BeginSection>
		a
		<BeginDotSpaceArgument>
			<BeginGroup>
				<BeginSection>
					b
					<BeginDotSpaceArgument>
						<BeginGroup>
							<BeginSection>
								c
							<EndSection>
						<EndGroup>
					<EndDotSpaceArgument>
				<EndSection>
			<EndGroup>
		<EndDotSpaceArgument>
	<EndSection>
	<BeginSection>
		d
	<EndSection>
<EndGroup>
`, "\t", "")

	assert.Equal(t, expected, actual)
	assert.Equal(t, []frontend.Diagnostic{}, tracker.Diagnostics())
}

func TestPhase3LexerIndentWithDoubleUnindentMultiSections(t *testing.T) {
	tracker := frontend.NewDiagnosticTracker()
	lexer1 := phase1.NewLexer(`
a:
. b:
  . c:
d:
e:
`, "", tracker)
	lexer2 := phase2.NewLexer(lexer1, "", tracker)
	lexer3 := NewLexer(lexer2, "", tracker)

	actual := "\n"
	for lexer3.HasNext() {
		actual += lexer3.Next().Text + "\n"
	}

	expected := strings.ReplaceAll(`
<BeginGroup>
	<BeginSection>
		a
		<BeginDotSpaceArgument>
			<BeginGroup>
				<BeginSection>
					b
					<BeginDotSpaceArgument>
						<BeginGroup>
							<BeginSection>
								c
							<EndSection>
						<EndGroup>
					<EndDotSpaceArgument>
				<EndSection>
			<EndGroup>
		<EndDotSpaceArgument>
	<EndSection>
	<BeginSection>
		d
	<EndSection>
	<BeginSection>
		e
	<EndSection>
<EndGroup>
`, "\t", "")

	assert.Equal(t, expected, actual)
	assert.Equal(t, []frontend.Diagnostic{}, tracker.Diagnostics())
}

func TestPhase3LexerMultiIndent(t *testing.T) {
	tracker := frontend.NewDiagnosticTracker()
	lexer1 := phase1.NewLexer(`
a:
. b:
  . c:
`, "", tracker)
	lexer2 := phase2.NewLexer(lexer1, "", tracker)
	lexer3 := NewLexer(lexer2, "", tracker)

	actual := "\n"
	for lexer3.HasNext() {
		actual += lexer3.Next().Text + "\n"
	}

	expected := strings.ReplaceAll(`
<BeginGroup>
	<BeginSection>
		a
		<BeginDotSpaceArgument>
			<BeginGroup>
				<BeginSection>
					b
					<BeginDotSpaceArgument>
						<BeginGroup>
							<BeginSection>
								c
							<EndSection>
						<EndGroup>
					<EndDotSpaceArgument>
				<EndSection>
			<EndGroup>
		<EndDotSpaceArgument>
	<EndSection>
<EndGroup>
`, "\t", "")

	assert.Equal(t, expected, actual)
	assert.Equal(t, []frontend.Diagnostic{}, tracker.Diagnostics())
}

func TestPhase3LexerSingleSectionWithMultiLineNonGroupArgs(t *testing.T) {
	tracker := frontend.NewDiagnosticTracker()
	lexer1 := phase1.NewLexer(`
a:
. b
. c
`, "", tracker)
	lexer2 := phase2.NewLexer(lexer1, "", tracker)
	lexer3 := NewLexer(lexer2, "", tracker)

	actual := "\n"
	for lexer3.HasNext() {
		actual += lexer3.Next().Text + "\n"
	}

	expected := strings.ReplaceAll(`
<BeginGroup>
	<BeginSection>
		a
		<BeginDotSpaceArgument>
			b
		<EndDotSpaceArgument>
		<BeginDotSpaceArgument>
			c
		<EndDotSpaceArgument>
	<EndSection>
<EndGroup>
`, "\t", "")

	assert.Equal(t, expected, actual)
	assert.Equal(t, []frontend.Diagnostic{}, tracker.Diagnostics())
}

func TestPhase3LexerSingleSectionWithMultiGroupArgs(t *testing.T) {
	tracker := frontend.NewDiagnosticTracker()
	lexer1 := phase1.NewLexer(`
a:
. b:
. c:
`, "", tracker)
	lexer2 := phase2.NewLexer(lexer1, "", tracker)
	lexer3 := NewLexer(lexer2, "", tracker)

	actual := "\n"
	for lexer3.HasNext() {
		actual += lexer3.Next().Text + "\n"
	}

	expected := strings.ReplaceAll(`
<BeginGroup>
	<BeginSection>
		a
		<BeginDotSpaceArgument>
			<BeginGroup>
				<BeginSection>
					b
				<EndSection>
			<EndGroup>
		<EndDotSpaceArgument>
		<BeginDotSpaceArgument>
			<BeginGroup>
				<BeginSection>
					c
				<EndSection>
			<EndGroup>
		<EndDotSpaceArgument>
	<EndSection>
<EndGroup>
`, "\t", "")

	assert.Equal(t, expected, actual)
	assert.Equal(t, []frontend.Diagnostic{}, tracker.Diagnostics())
}

func TestPhase3LexerMultiSections(t *testing.T) {
	tracker := frontend.NewDiagnosticTracker()
	lexer1 := phase1.NewLexer(`
a:
b:
c:
`, "", tracker)
	lexer2 := phase2.NewLexer(lexer1, "", tracker)
	lexer3 := NewLexer(lexer2, "", tracker)

	actual := "\n"
	for lexer3.HasNext() {
		actual += lexer3.Next().Text + "\n"
	}

	expected := strings.ReplaceAll(`
<BeginGroup>
	<BeginSection>
		a
	<EndSection>
	<BeginSection>
		b
	<EndSection>
	<BeginSection>
		c
	<EndSection>
<EndGroup>
`, "\t", "")

	assert.Equal(t, expected, actual)
	assert.Equal(t, []frontend.Diagnostic{}, tracker.Diagnostics())
}

func TestPhase3LexerMultiSectionsWithGroupArgs(t *testing.T) {
	tracker := frontend.NewDiagnosticTracker()
	lexer1 := phase1.NewLexer(`
a:
. x:
  y:
b:
. A:
c:
. P:
  Q:
  R:
`, "", tracker)
	lexer2 := phase2.NewLexer(lexer1, "", tracker)
	lexer3 := NewLexer(lexer2, "", tracker)

	actual := "\n"
	for lexer3.HasNext() {
		actual += lexer3.Next().Text + "\n"
	}

	expected := strings.ReplaceAll(`
<BeginGroup>
	<BeginSection>
		a
		<BeginDotSpaceArgument>
			<BeginGroup>
				<BeginSection>
					x
				<EndSection>
				<BeginSection>
					y
				<EndSection>
			<EndGroup>
		<EndDotSpaceArgument>
	<EndSection>
	<BeginSection>
		b
		<BeginDotSpaceArgument>
			<BeginGroup>
				<BeginSection>
					A
				<EndSection>
			<EndGroup>
		<EndDotSpaceArgument>
	<EndSection>
	<BeginSection>
		c
		<BeginDotSpaceArgument>
			<BeginGroup>
				<BeginSection>
					P
				<EndSection>
				<BeginSection>
					Q
				<EndSection>
				<BeginSection>
					R
				<EndSection>
			<EndGroup>
		<EndDotSpaceArgument>
	<EndSection>
<EndGroup>
`, "\t", "")

	assert.Equal(t, expected, actual)
	assert.Equal(t, []frontend.Diagnostic{}, tracker.Diagnostics())
}

func TestPhase3LexerMultiSectionsWithGroupArgsAndNonGroupArgs(t *testing.T) {
	tracker := frontend.NewDiagnosticTracker()
	lexer1 := phase1.NewLexer(`
a:
. x:x1,x2,x3
  y:y1
b:
. A:A1,A2
c:
. P:
  Q:Q1
  R:
`, "", tracker)
	lexer2 := phase2.NewLexer(lexer1, "", tracker)
	lexer3 := NewLexer(lexer2, "", tracker)

	actual := "\n"
	for lexer3.HasNext() {
		actual += lexer3.Next().Text + "\n"
	}

	expected := strings.ReplaceAll(`
<BeginGroup>
	<BeginSection>
		a
		<BeginDotSpaceArgument>
			<BeginGroup>
				<BeginSection>
					x
					<BeginInlineArgument>
						x1
					<EndInlineArgument>
					<BeginInlineArgument>
						x2
					<EndInlineArgument>
					<BeginInlineArgument>
						x3
					<EndInlineArgument>
				<EndSection>
				<BeginSection>
					y
					<BeginInlineArgument>
						y1
					<EndInlineArgument>
				<EndSection>
			<EndGroup>
		<EndDotSpaceArgument>
	<EndSection>
	<BeginSection>
		b
		<BeginDotSpaceArgument>
			<BeginGroup>
				<BeginSection>
					A
					<BeginInlineArgument>
						A1
					<EndInlineArgument>
					<BeginInlineArgument>
						A2
					<EndInlineArgument>
				<EndSection>
			<EndGroup>
		<EndDotSpaceArgument>
	<EndSection>
	<BeginSection>
		c
		<BeginDotSpaceArgument>
			<BeginGroup>
				<BeginSection>
					P
				<EndSection>
				<BeginSection>
					Q
					<BeginInlineArgument>
						Q1
					<EndInlineArgument>
				<EndSection>
				<BeginSection>
					R
				<EndSection>
			<EndGroup>
		<EndDotSpaceArgument>
	<EndSection>
<EndGroup>
`, "\t", "")

	assert.Equal(t, expected, actual)
	assert.Equal(t, []frontend.Diagnostic{}, tracker.Diagnostics())
}

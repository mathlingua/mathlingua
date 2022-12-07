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

package internal

import (
	"fmt"
	"mathlingua/internal/ast"
	"testing"

	"github.com/stretchr/testify/assert"
)

type testCase struct {
	actual   []string
	expected []string
}

var all_group_test_cases = []testCase{
	{
		actual: ast.AllOfSections,
		expected: []string{
			"allOf",
		},
	},
	{
		actual: ast.NotSections,
		expected: []string{
			"not",
		},
	},
	{
		actual: ast.AnyOfSections,
		expected: []string{
			"anyOf",
		},
	},
	{
		actual: ast.OneOfSections,
		expected: []string{
			"oneOf",
		},
	},
	{
		actual: ast.ExistsSections,
		expected: []string{
			"exists",
			"where?",
			"suchThat",
		},
	},
	{
		actual: ast.ExistsUniqueSections,
		expected: []string{
			"existsUnique",
			"where?",
			"suchThat",
		},
	},
	{
		actual: ast.ForAllSections,
		expected: []string{
			"forAll",
			"where?",
			"suchThat?",
			"then",
		},
	},
	{
		actual: ast.IfSections,
		expected: []string{
			"if",
			"then",
		},
	},
	{
		actual: ast.IffSections,
		expected: []string{
			"iff",
			"then",
		},
	},
	{
		actual: ast.PiecewiseSections,
		expected: []string{
			"piecewise",
			"if",
			"then",
			"else?",
		},
	},
	{
		actual: ast.WhenSections,
		expected: []string{
			"when",
			"then",
		},
	},
	{
		actual: ast.ExpressedSections,
		expected: []string{
			"expressed",
		},
	},
	{
		actual: ast.CalledSections,
		expected: []string{
			"called",
		},
	},
	{
		actual: ast.ExpressingSections,
		expected: []string{
			"expressing",
			"as",
		},
	},
	{
		actual: ast.DetailsSections,
		expected: []string{
			"details",
		},
	},
	{
		actual: ast.OverviewSections,
		expected: []string{
			"overview",
		},
	},
	{
		actual: ast.MotivationSections,
		expected: []string{
			"motivation",
		},
	},
	{
		actual: ast.HistorySections,
		expected: []string{
			"history",
		},
	},
	{
		actual: ast.ExamplesSections,
		expected: []string{
			"examples",
		},
	},
	{
		actual: ast.RelatedSections,
		expected: []string{
			"related",
		},
	},
	{
		actual: ast.DiscoveredSections,
		expected: []string{
			"discovered",
		},
	},
	{
		actual: ast.NotesSections,
		expected: []string{
			"notes",
		},
	},
	{
		actual: ast.LabelSections,
		expected: []string{
			"label",
			"by",
		},
	},
	{
		actual: ast.BySections,
		expected: []string{
			"by",
		},
	},
	{
		actual: ast.IdSections,
		expected: []string{
			"id",
		},
	},
	{
		actual: ast.DescribesSections,
		expected: []string{
			"Describes",
			"with?",
			"using?",
			"when?",
			"suchThat?",
			"extends?",
			"satisfies?",
			"Provides?",
			"Justified?",
			"Documented?",
			"References?",
			"Aliases?",
			"Metadata?",
		},
	},
	{
		actual: ast.DefinesSections,
		expected: []string{
			"Defines",
			"with?",
			"using?",
			"when?",
			"suchThat?",
			"means?",
			"specifies",
			"Provides?",
			"Justified?",
			"Documented?",
			"References?",
			"Aliases?",
			"Metadata?",
		},
	},
	{
		actual: ast.StatesSections,
		expected: []string{
			"States",
			"with?",
			"using?",
			"when?",
			"suchThat?",
			"that",
			"Documented?",
			"Justified?",
			"References?",
			"Aliases?",
			"Metadata?",
		},
	},
	{
		actual: ast.ProofSections,
		expected: []string{
			"Proof",
			"of",
			"content",
			"References?",
			"Metadata?",
		},
	},
	{
		actual: ast.AxiomSections,
		expected: []string{
			"Axiom",
			"given?",
			"where?",
			"if?",
			"iff?",
			"then",
			"Documented?",
			"References?",
			"Aliases?",
			"Metadata?",
		},
	},
	{
		actual: ast.ConjectureSections,
		expected: []string{
			"Conjecture",
			"given?",
			"where?",
			"if?",
			"iff?",
			"then",
			"Documented?",
			"References?",
			"Aliases?",
			"Metadata?",
		},
	},
	{
		actual: ast.TheoremSections,
		expected: []string{
			"Theorem",
			"given?",
			"where?",
			"if?",
			"iff?",
			"then",
			"Proof?",
			"Documented?",
			"References?",
			"Aliases?",
			"Metadata?",
		},
	},
	{
		actual: ast.TopicSections,
		expected: []string{
			"Topic",
			"content",
			"References?",
			"Metadata?",
		},
	},
	{
		actual: ast.PositiveIntSections,
		expected: []string{
			"positiveInt",
			"is",
		},
	},
	{
		actual: ast.NegativeIntSections,
		expected: []string{
			"negativeInt",
			"is",
		},
	},
	{
		actual: ast.ZeroSections,
		expected: []string{
			"zero",
			"is",
		},
	},
	{
		actual: ast.PositiveFloatSections,
		expected: []string{
			"positiveFloat",
			"is",
		},
	},
	{
		actual: ast.NegativeFloatSections,
		expected: []string{
			"negativeFloat",
			"is",
		},
	},
	{
		actual: ast.SpecifySections,
		expected: []string{
			"Specify",
		},
	},
}

func TestSpec(t *testing.T) {
	for _, testCase := range all_group_test_cases {
		actual := "\n"
		expected := "\n"

		for _, section := range testCase.actual {
			actual += fmt.Sprintf("%s:\n", section)
		}

		for _, section := range testCase.expected {
			expected += fmt.Sprintf("%s:\n", section)
		}

		assert.Equal(t, expected, actual)
	}
}

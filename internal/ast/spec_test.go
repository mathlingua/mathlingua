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

package ast

import (
	"fmt"
	"testing"

	"github.com/stretchr/testify/assert"
)

type testCase struct {
	actual   []string
	expected []string
}

var all_group_test_cases = []testCase{
	{
		actual: AllOfSections,
		expected: []string{
			"allOf",
		},
	},
	{
		actual: NotSections,
		expected: []string{
			"not",
		},
	},
	{
		actual: AnyOfSections,
		expected: []string{
			"anyOf",
		},
	},
	{
		actual: OneOfSections,
		expected: []string{
			"oneOf",
		},
	},
	{
		actual: ExistsSections,
		expected: []string{
			"exists",
			"where?",
			"suchThat",
		},
	},
	{
		actual: ExistsUniqueSections,
		expected: []string{
			"existsUnique",
			"where?",
			"suchThat",
		},
	},
	{
		actual: ForAllSections,
		expected: []string{
			"forAll",
			"where?",
			"suchThat?",
			"then",
		},
	},
	{
		actual: GivenSections,
		expected: []string{
			"given",
			"where?",
			"suchThat?",
			"then",
		},
	},
	{
		actual: IfSections,
		expected: []string{
			"if",
			"then",
		},
	},
	{
		actual: IffSections,
		expected: []string{
			"iff",
			"then",
		},
	},
	{
		actual: PiecewiseSections,
		expected: []string{
			"piecewise",
			"if",
			"then",
			"else?",
		},
	},
	{
		actual: WhenSections,
		expected: []string{
			"when",
			"then",
		},
	},
	{
		actual: WrittenSections,
		expected: []string{
			"written",
		},
	},
	{
		actual: CalledSections,
		expected: []string{
			"called",
		},
	},
	{
		actual: WritingSections,
		expected: []string{
			"writing",
			"as",
		},
	},
	{
		actual: DetailsSections,
		expected: []string{
			"details",
		},
	},
	{
		actual: OverviewSections,
		expected: []string{
			"overview",
		},
	},
	{
		actual: MotivationSections,
		expected: []string{
			"motivation",
		},
	},
	{
		actual: HistorySections,
		expected: []string{
			"history",
		},
	},
	{
		actual: ExampleSections,
		expected: []string{
			"example",
		},
	},
	{
		actual: RelatedSections,
		expected: []string{
			"related",
		},
	},
	{
		actual: DiscovererSections,
		expected: []string{
			"discoverer",
		},
	},
	{
		actual: NoteSections,
		expected: []string{
			"note",
		},
	},
	{
		actual: LabelSections,
		expected: []string{
			"label",
			"by",
		},
	},
	{
		actual: BySections,
		expected: []string{
			"by",
		},
	},
	{
		actual: DescribesSections,
		expected: []string{
			"Describes",
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
			"Id?",
		},
	},
	{
		actual: DefinesSections,
		expected: []string{
			"Defines",
			"using?",
			"when?",
			"suchThat?",
			"means?",
			"specifies?",
			"Provides?",
			"Justified?",
			"Documented?",
			"References?",
			"Aliases?",
			"Id?",
		},
	},
	{
		actual: CapturesSections,
		expected: []string{
			"Captures",
			"Justified?",
			"Documented?",
			"References?",
			"Id?",
		},
	},
	{
		actual: StatesSections,
		expected: []string{
			"States",
			"using?",
			"when?",
			"suchThat?",
			"that",
			"Justified?",
			"Documented?",
			"References?",
			"Aliases?",
			"Id?",
		},
	},
	{
		actual: ProofSections,
		expected: []string{
			"Proof",
			"of",
			"content",
			"References?",
			"Id?",
		},
	},
	{
		actual: AxiomSections,
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
			"Id?",
		},
	},
	{
		actual: ConjectureSections,
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
			"Id?",
		},
	},
	{
		actual: TheoremSections,
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
			"Id?",
		},
	},
	{
		actual: TopicSections,
		expected: []string{
			"Topic",
			"content",
			"References?",
			"Id?",
		},
	},
	{
		actual: PositiveIntSections,
		expected: []string{
			"positiveInt",
			"means",
		},
	},
	{
		actual: NegativeIntSections,
		expected: []string{
			"negativeInt",
			"means",
		},
	},
	{
		actual: ZeroSections,
		expected: []string{
			"zero",
			"means",
		},
	},
	{
		actual: PositiveFloatSections,
		expected: []string{
			"positiveFloat",
			"means",
		},
	},
	{
		actual: NegativeFloatSections,
		expected: []string{
			"negativeFloat",
			"means",
		},
	},
	{
		actual: SpecifySections,
		expected: []string{
			"Specify",
			"Id?",
		},
	},
	{
		actual: NameSections,
		expected: []string{
			"name",
		},
	},
	{
		actual: BiographySections,
		expected: []string{
			"biography",
		},
	},
	{
		actual: PersonSections,
		expected: []string{
			"Person",
			"Id?",
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

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
			"using?",
			"where?",
			"suchThat",
		},
	},
	{
		actual: ExistsUniqueSections,
		expected: []string{
			"existsUnique",
			"using?",
			"where?",
			"suchThat",
		},
	},
	{
		actual: ForAllSections,
		expected: []string{
			"forAll",
			"using?",
			"where?",
			"suchThat?",
			"then",
		},
	},
	{
		actual: GivenSections,
		expected: []string{
			"given",
			"using?",
			"where?",
			"suchThat?",
			"then",
		},
	},
	{
		actual: LowerDefineSections,
		expected: []string{
			"define",
			"using?",
			"when?",
			"suchThat?",
			"means?",
			"as",
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
		actual: ExpressedSections,
		expected: []string{
			"expressed",
		},
	},
	{
		actual: OverviewSections,
		expected: []string{
			"overview",
		},
	},
	{
		actual: RelatedSections,
		expected: []string{
			"related",
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
		actual: LowerDefineSections,
		expected: []string{
			"define",
			"using?",
			"when?",
			"suchThat?",
			"means?",
			"as",
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
			"using?",
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
			"using?",
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
			"using?",
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
		actual: CorollarySections,
		expected: []string{
			"Corollary",
			"to",
			"given?",
			"using?",
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
		actual: LemmaSections,
		expected: []string{
			"Lemma",
			"for",
			"given?",
			"using?",
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
	{
		actual: ResourceSections,
		expected: []string{
			"Resource",
			"Id?",
		},
	},
	{
		actual: ViewSections,
		expected: []string{
			"view",
			"as",
			"using?",
			"where?",
			"through?",
			"signifies?",
		},
	},
	{
		actual: EncodingSections,
		expected: []string{
			"encoding",
			"as",
			"using?",
			"where?",
			"through?",
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

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
		actual: AnyOfSections,
		expected: []string{
			"anyOf",
		},
	},
	{
		actual: AuthorSections,
		expected: []string{
			"author",
		},
	},
	{
		actual: AxiomSections,
		expected: []string{
			"Axiom",
			"given?",
			"declaring?",
			"using?",
			"where?",
			"suchThat?",
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
		actual: BiographySections,
		expected: []string{
			"biography",
		},
	},
	{
		actual: BySections,
		expected: []string{
			"by",
		},
	},
	{
		actual: CalledSections,
		expected: []string{
			"called",
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
		actual: ConjectureSections,
		expected: []string{
			"Conjecture",
			"given?",
			"declaring?",
			"using?",
			"where?",
			"suchThat?",
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
		actual: CorollarySections,
		expected: []string{
			"Corollary",
			"to",
			"given?",
			"declaring?",
			"using?",
			"where?",
			"suchThat?",
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
		actual: DefinesSections,
		expected: []string{
			"Defines",
			"using?",
			"when?",
			"suchThat?",
			"means?",
			"equivalentTo?",
			"specifies?",
			"expressing?",
			"Provides?",
			"Justified?",
			"Documented?",
			"References?",
			"Aliases?",
			"Id?",
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
			"equivalentTo?",
			"specifies?",
			"satisfying?",
			"Provides?",
			"Justified?",
			"Documented?",
			"References?",
			"Aliases?",
			"Id?",
		},
	},
	{
		actual: DescriptionSections,
		expected: []string{
			"description",
		},
	},
	{
		actual: EditionSections,
		expected: []string{
			"edition",
		},
	},
	{
		actual: EditorSections,
		expected: []string{
			"editor",
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
	{
		actual: EquivalentlySections,
		expected: []string{
			"equivalently",
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
		actual: WritingSections,
		expected: []string{
			"writing",
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
		actual: HomepageSections,
		expected: []string{
			"homepage",
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
		actual: InstitutionSections,
		expected: []string{
			"institution",
		},
	},
	{
		actual: JournalSections,
		expected: []string{
			"journal",
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
		actual: LemmaSections,
		expected: []string{
			"Lemma",
			"for",
			"given?",
			"declaring?",
			"using?",
			"where?",
			"suchThat?",
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
		actual: DeclareSections,
		expected: []string{
			"declare",
			"using?",
			"where?",
			"suchThat?",
			"then",
		},
	},
	{
		actual: MonthSections,
		expected: []string{
			"month",
		},
	},
	{
		actual: NameSections,
		expected: []string{
			"name",
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
		actual: NegativeIntSections,
		expected: []string{
			"negativeInt",
			"means",
		},
	},
	{
		actual: NotSections,
		expected: []string{
			"not",
		},
	},
	{
		actual: OffsetSections,
		expected: []string{
			"offset",
		},
	},
	{
		actual: OneOfSections,
		expected: []string{
			"oneOf",
		},
	},
	{
		actual: OverviewSections,
		expected: []string{
			"overview",
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
		actual: PiecewiseSections,
		expected: []string{
			"piecewise",
			"if",
			"then",
			"elseIf?",
			"then?",
			"else?",
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
		actual: PositiveIntSections,
		expected: []string{
			"positiveInt",
			"means",
		},
	},
	{
		actual: ProofAllOfSections,
		expected: []string{
			"allOf",
		},
	},
	{
		actual: ProofAnyOfSections,
		expected: []string{
			"anyOf",
		},
	},
	{
		actual: ProofBecauseThenSections,
		expected: []string{
			"because",
			"then",
		},
	},
	{
		actual: ProofBlockSections,
		expected: []string{
			"block",
		},
	},
	{
		actual: ProofByBecauseThenSections,
		expected: []string{
			"by",
			"because?",
			"then",
		},
	},
	{
		actual: ProofCasewiseSections,
		expected: []string{
			"casewise",
			"case",
			"else?",
		},
	},
	{
		actual: ProofClaimSections,
		expected: []string{
			"claim",
			"given?",
			"using?",
			"where?",
			"suchThat?",
			"if?",
			"iff?",
			"then",
			"Proof?",
		},
	},
	{
		actual: ProofContradictionSections,
		expected: []string{
			"contradiction",
		},
	},
	{
		actual: ProofEquivalentlySections,
		expected: []string{
			"equivalently",
		},
	},
	{
		actual: ProofExistsSections,
		expected: []string{
			"exists",
			"using?",
			"where?",
			"suchThat",
		},
	},
	{
		actual: ProofExistsUniqueSections,
		expected: []string{
			"existsUnique",
			"using?",
			"where?",
			"suchThat",
		},
	},
	{
		actual: ProofForAllSections,
		expected: []string{
			"forAll",
			"using?",
			"where?",
			"suchThat?",
			"then",
		},
	},
	{
		actual: ProofForContradictionSections,
		expected: []string{
			"forContradiction",
		},
	},
	{
		actual: ProofForInductionSections,
		expected: []string{
			"forInduction",
		},
	},
	{
		actual: ProofIfSections,
		expected: []string{
			"if",
			"then",
		},
	},
	{
		actual: ProofIffSections,
		expected: []string{
			"iff",
			"then",
		},
	},
	{
		actual: ProofDeclareSections,
		expected: []string{
			"declare",
			"using?",
			"where?",
			"suchThat?",
			"then",
		},
	},
	{
		actual: ProofNotSections,
		expected: []string{
			"not",
		},
	},
	{
		actual: ProofOneOfSections,
		expected: []string{
			"oneOf",
		},
	},
	{
		actual: ProofStepwiseSections,
		expected: []string{
			"stepwise",
		},
	},
	{
		actual: ProofSupposeSections,
		expected: []string{
			"suppose",
			"then",
		},
	},
	{
		actual: ProofWithoutLossOfGeneralitySections,
		expected: []string{
			"withoutLossOfGenerality",
		},
	},
	{
		actual: PublisherSections,
		expected: []string{
			"publisher",
		},
	},
	{
		actual: RelatedSections,
		expected: []string{
			"related",
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
		actual: SpecifySections,
		expected: []string{
			"Specify",
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
			"specifies?",
			"that?",
			"Justified?",
			"Documented?",
			"References?",
			"Aliases?",
			"Id?",
		},
	},
	{
		actual: ComparisonSections,
		expected: []string{
			"comparison",
			"provided",
		},
	},
	{
		actual: SymbolWrittenSections,
		expected: []string{
			"symbol",
			"tracks?",
			"replaces?",
			"written?",
		},
	},
	{
		actual: TheoremSections,
		expected: []string{
			"Theorem",
			"given?",
			"declaring?",
			"using?",
			"where?",
			"suchThat?",
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
		actual: TitleSections,
		expected: []string{
			"title",
		},
	},
	{
		actual: TypeSections,
		expected: []string{
			"type",
		},
	},
	{
		actual: UrlSections,
		expected: []string{
			"url",
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
		actual: VolumeSections,
		expected: []string{
			"volume",
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
		actual: YearSections,
		expected: []string{
			"year",
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
		actual: ProofThenSections,
		expected: []string{
			"then",
			"by?",
			"because?",
		},
	},
	{
		actual: ProofThusSections,
		expected: []string{
			"thus",
			"by?",
			"because?",
		},
	},
	{
		actual: ProofThereforeSections,
		expected: []string{
			"therefore",
			"by?",
			"because?",
		},
	},
	{
		actual: ProofHenceSections,
		expected: []string{
			"hence",
			"by?",
			"because?",
		},
	},
	{
		actual: ProofNoticeSections,
		expected: []string{
			"notice",
			"by?",
			"because?",
		},
	},
	{
		actual: ProofNextSections,
		expected: []string{
			"next",
			"by?",
			"because?",
		},
	},
	{
		actual: ProofForContrapositiveSections,
		expected: []string{
			"forContrapositive",
		},
	},
	{
		actual: ProofQedSections,
		expected: []string{
			"qed",
		},
	},
	{
		actual: ProofAbsurdSections,
		expected: []string{
			"absurd",
		},
	},
	{
		actual: ProofDoneSections,
		expected: []string{
			"done",
		},
	},
	{
		actual: ProofPartwiseSections,
		expected: []string{
			"partwise",
			"part",
		},
	},
	{
		actual: ProofSufficesToShowSections,
		expected: []string{
			"sufficesToShow",
		},
	},
	{
		actual: ProofToShowSections,
		expected: []string{
			"toShow",
			"observe",
		},
	},
	{
		actual: ProofRemarkSections,
		expected: []string{
			"remark",
		},
	},
	{
		actual: InductivelySections,
		expected: []string{
			"inductively",
			"oneOf",
		},
	},
	{
		actual: InductivelyCaseSections,
		expected: []string{
			"case",
			"using?",
		},
	},
	{
		actual: MatchingCaseSections,
		expected: []string{
			"case",
			"using?",
			"then",
		},
	},
	{
		actual: MatchingSections,
		expected: []string{
			"matching",
			"as?",
			"against",
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

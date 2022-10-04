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

var all_groups = [][]string{
	ast.AllOfSections,
	ast.NotSections,
	ast.AnyOfSections,
	ast.OneOfSections,
	ast.ExistsSections,
	ast.ExistsUniqueSections,
	ast.ForAllSections,
	ast.IfSections,
	ast.IffSections,
	ast.GeneratedSections,
	ast.PiecewiseSections,
	ast.AsViaSections,
	ast.AsThroughStatesSections,
	ast.OperationsSections,
	ast.MembersSections,
	ast.ExpressedSections,
	ast.CalledSections,
	ast.ExpressingSections,
	ast.LooselySections,
	ast.OverviewSections,
	ast.MotivationSections,
	ast.HistorySections,
	ast.ExamplesSections,
	ast.RelatedSections,
	ast.DiscoveredSections,
	ast.NotesSections,
	ast.LabelSections,
	ast.BySections,
	ast.IdSections,
	ast.DescribesSections,
	ast.DeclaresSections,
	ast.StatesSections,
	ast.ProofSections,
	ast.AxiomSections,
	ast.ConjectureSections,
	ast.TheoremSections,
	ast.TopicSections,
	ast.NoteSections,
	ast.ZeroSections,
	ast.PositiveIntSections,
	ast.NegativeIntSections,
	ast.PositiveFloatSections,
	ast.NegativeFloatSections,
	ast.SpecifySections,
}

func TestSpec(t *testing.T) {
	t.Skip("Skipping since the code is not aligned with the spec")

	actual := "\n"
	for _, sections := range all_groups {
		for _, section := range sections {
			actual += fmt.Sprintf("%s:\n", section)
		}
		actual += "\n"
	}

	expected := `
allOf:

not:

anyOf:

oneOf:

exists:
where?:
suchThat:

existsUnique:
where?:
suchThat?:

forAll:
where?:
suchThat?:
then:

if:
then:

iff:
then:

generated:
from:
when?:

piecewise:
if:
then:
else?:

as:
via:

as:
through:
as?:
states?:

operations:
on?:
using?:
when?:
specify:

members:

expressed:

called:

expressing:
as:

loosely:

overview:

motivation:

history:

examples:

related:

discovered:

notes:

label:
by:

by:

id:

Describes:
with?:
using?:
provided?:
suchThat?:
extends?:
satisfies?:
Provides?:
Viewable?:
Justified?:
Documented?:
References?:
Aliases?:
Metadata?:

Declares:
with?:
using?:
provided?:
suchThat?:
means?:
defines?:
Provides?:
Viewable?:
Justified?:
Documented?:
References?:
Aliases?:
Metadata?:

States:
using?:
provided?:
suchThat?:
that:
Documented?:
Justified?:
References?:
Aliases?:
Metadata?:

Proof:
of:
content:
References?:
Metadata?:

Axiom:
given?:
where?:
suchThat?:
then:
iff?:
Documented?:
References?:
Aliases?:
Metadata?:

Conjecture:
given?:
where?:
suchThat?:
then:
iff?:
Documented?:
References?:
Aliases?:
Metadata?:

Theorem:
given?:
where?:
suchThat?:
then:
iff?:
Proof?:
Documented?:
References?:
Aliases?:
Metadata?:

Topic:
content:
References?:
Metadata?:

Note:
content:
Metadata?:

zero:
is:

positiveInt:
is:

negativeInt:
is:

positiveFloat:
is:

negativeFloat:
is:

Specify:

when:
then:

`

	assert.Equal(t, expected, actual)
}

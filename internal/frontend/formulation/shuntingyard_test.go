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
	"testing"

	"github.com/kr/pretty"
	"github.com/stretchr/testify/assert"
)

func TestPlusMultiply(t *testing.T) {
	items := []ShuntingYardItem[string]{
		{
			Item:     "x",
			ItemType: OperandType,
		},
		{
			Item:          "+",
			ItemType:      InfixOperatorType,
			Associativity: LeftAssociative,
			Precedence:    1,
		},
		{
			Item:     "y",
			ItemType: OperandType,
		},
		{
			Item:          "*",
			ItemType:      InfixOperatorType,
			Associativity: LeftAssociative,
			Precedence:    2,
		},
		{
			Item:     "z",
			ItemType: OperandType,
		},
	}
	result := ShuntingYard(items)

	actual := "\n"
	for _, res := range result {
		actual += pretty.Sprintf("%# v\n", res)
	}

	expected := `
formulation.ShuntingYardItem[string]{Item:"x", ItemType:"OperandType", Precedence:0, Associativity:""}
formulation.ShuntingYardItem[string]{Item:"y", ItemType:"OperandType", Precedence:0, Associativity:""}
formulation.ShuntingYardItem[string]{Item:"z", ItemType:"OperandType", Precedence:0, Associativity:""}
formulation.ShuntingYardItem[string]{Item:"*", ItemType:"InfixOperatorType", Precedence:2, Associativity:"LeftAssociative"}
formulation.ShuntingYardItem[string]{Item:"+", ItemType:"InfixOperatorType", Precedence:1, Associativity:"LeftAssociative"}
`

	assert.Equal(t, expected, actual)
}

func TestXSquared(t *testing.T) {
	items := []ShuntingYardItem[string]{
		{
			Item:          "-",
			ItemType:      PrefixOperatorType,
			Associativity: RightAssociative,
			Precedence:    1,
		},
		{
			Item:     "x",
			ItemType: OperandType,
		},
		{
			Item:          "^",
			ItemType:      InfixOperatorType,
			Associativity: RightAssociative,
			Precedence:    2,
		},
		{
			Item:     "2",
			ItemType: OperandType,
		},
	}
	result := ShuntingYard(items)

	actual := "\n"
	for _, res := range result {
		actual += pretty.Sprintf("%# v\n", res)
	}

	expected := `
formulation.ShuntingYardItem[string]{Item:"x", ItemType:"OperandType", Precedence:0, Associativity:""}
formulation.ShuntingYardItem[string]{Item:"2", ItemType:"OperandType", Precedence:0, Associativity:""}
formulation.ShuntingYardItem[string]{Item:"^", ItemType:"InfixOperatorType", Precedence:2, Associativity:"RightAssociative"}
formulation.ShuntingYardItem[string]{Item:"-", ItemType:"PrefixOperatorType", Precedence:1, Associativity:"RightAssociative"}
`

	assert.Equal(t, expected, actual)
}

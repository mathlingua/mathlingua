/*
 * Copyright 2023 Dominic Kramer
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

package backend

import (
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestXxx(t *testing.T) {
	text := "text1 A? text2 x+? text3 X?{abc...} text4 X+?{...xyz...} text5"
	actual, err := ParseCalledWritten(text)
	assert.Nil(t, err)
	expected := []TextItemKind{
		&StringItem{
			Text: "text1 ",
		},
		&SubstitutionItem{
			Name:       "A",
			NameSuffix: "",
			IsVarArg:   false,
			Prefix:     "",
			Suffix:     "",
			Infix:      "",
		},
		&StringItem{
			Text: " text2 ",
		},
		&SubstitutionItem{
			Name:       "x",
			NameSuffix: "+",
			IsVarArg:   false,
			Prefix:     "",
			Suffix:     "",
			Infix:      "",
		},
		&StringItem{
			Text: " text3 ",
		},
		&SubstitutionItem{
			Name:       "X",
			NameSuffix: "",
			IsVarArg:   true,
			Prefix:     "abc",
			Suffix:     "",
			Infix:      "",
		},
		&StringItem{
			Text: " text4 ",
		},
		&SubstitutionItem{
			Name:       "X",
			NameSuffix: "+",
			IsVarArg:   true,
			Prefix:     "",
			Suffix:     "",
			Infix:      "xyz",
		},
		&StringItem{
			Text: " text5",
		},
	}
	assert.Equal(t, expected, actual)
}

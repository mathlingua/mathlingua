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

package config

import (
	"mathlingua/internal/mlglib"
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestParseMlgConfigText(t *testing.T) {
	input := `
[mlg.view]
title = some title
description = some description
keywords = keyword 1, keyword 2, keyword3
`
	conf, err := ParseMlgConfig(input)
	assert.Nil(t, err)

	assert.Equal(t, mlglib.PrettyPrint(MlgConfig{
		View: MlgViewConfig{
			Title:       "some title",
			Description: "some description",
			Keywords: []string{
				"keyword 1",
				"keyword 2",
				"keyword3",
			},
		},
	}), mlglib.PrettyPrint(conf))
}

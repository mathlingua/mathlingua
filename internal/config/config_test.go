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

func TestParseConfigText(t *testing.T) {
	input := `
[section 1]
key = value
some key = some value
some.key=some.value
some.other key       =         some other    value



[section.2]
key=value
anotherKey = some.value
`
	conf, err := ParseConfig(input)
	assert.Nil(t, err)

	section1Values := make(map[string]string)
	section1Values["key"] = "value"
	section1Values["some key"] = "some value"
	section1Values["some.key"] = "some.value"
	section1Values["some.other key"] = "some other    value"

	section2Values := make(map[string]string)
	section2Values["key"] = "value"
	section2Values["anotherKey"] = "some.value"

	expectedSections := make(map[string]IConfigSection)
	expectedSections["section 1"] = &configSection{
		name:   "section 1",
		keys:   []string{"key", "some key", "some.key", "some.other key"},
		values: section1Values,
	}

	expectedSections["section.2"] = &configSection{
		name:   "section.2",
		keys:   []string{"key", "anotherKey"},
		values: section2Values,
	}

	assert.Equal(t, mlglib.PrettyPrint(&config{
		names:    []string{"section 1", "section.2"},
		sections: expectedSections,
	}), mlglib.PrettyPrint(conf))
}

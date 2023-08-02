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

func TestParseBasicConfigText(t *testing.T) {
	input := `
[section 1]
key = "value"
`
	conf, err := ParseConfig(input)
	assert.Nil(t, err)

	section1Values := make(map[string]string)
	section1Values["key"] = "value"

	expectedSections := make(map[string]IConfigSection)
	expectedSections["section 1"] = &configSection{
		name:   "section 1",
		keys:   []string{"key", "some key", "some.key", "some.other key"},
		values: section1Values,
	}

	assert.Equal(t, mlglib.PrettyPrint(&config{
		names:    []string{"section 1"},
		sections: expectedSections,
	}), mlglib.PrettyPrint(conf))
}

func TestParseConfigText(t *testing.T) {
	input := `
[section 1]
key = "value"
some key = "some value"
some.key="some.value"
some.other key       =         "some other    value"
some.longer.value = "this is some
 text and some more
   and even more
 and some more
     and some more"


[section.2]
key="value"
anotherKey = "some.value"
`
	conf, err := ParseConfig(input)
	assert.Nil(t, err)

	section1Values := make(map[string]string)
	section1Values["key"] = "value"
	section1Values["some key"] = "some value"
	section1Values["some.key"] = "some.value"
	section1Values["some.other key"] = "some other    value"
	section1Values["some.longer.value"] = "this is some	text" +
		" and some more" +
		"   and even more	" +
		"and some more" +
		"     and some more"

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

func TestParseConfigDuplicateKey(t *testing.T) {
	text := `
[section.1]
key1 = "value1"
key1 = "value2"
`
	_, err := ParseConfig(text)
	assert.NotNil(t, err)
	assert.Equal(t, "Duplicate key specified: key1", err.Error())
}

func TestParseConfigNoEquals(t *testing.T) {
	text := `
[section.1]
key1 value1
`
	_, err := ParseTocConfig(text)
	assert.NotNil(t, err)
	assert.Equal(t,
		"Expected = but found the end of text on line 2", err.Error())
}

func TestTocConfigUnexpectedSection(t *testing.T) {
	text := `
[section.1]
[key1 = "value1"
`
	_, err := ParseTocConfig(text)
	assert.NotNil(t, err)
	assert.Equal(t,
		"Unexpected key value that looks like the start of a section: [key1", err.Error())
}

func TestConfigDuplicateSection(t *testing.T) {
	text := `
[section.1]
key1 = "value1"

[section.1]
key2 = "value2"
`
	_, err := ParseTocConfig(text)
	assert.NotNil(t, err)
	assert.Equal(t,
		"Duplicate defined section: section.1", err.Error())
}

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
	"fmt"
	"strings"
)

type IConfig interface {
	SectionNames() []string
	Section(name string) (IConfigSection, bool)
}

type IConfigSection interface {
	Name() string
	Keys() []string
	Get(key string) (string, bool)
}

func ParseConfig(text string) (IConfig, error) {
	row := 0
	index := 0

	expect := func(c byte) error {
		if index >= len(text) {
			return fmt.Errorf("Expected %c but found the end of text on line %d", c, row+1)
		}
		if text[index] != c {
			return fmt.Errorf("Expected %c but found %c on line %d", c, text[index], row+1)
		}
		return nil
	}

	skipNewlines := func() {
		for index < len(text) && (text[index] == '\n' || text[index] == '\r') {
			index++
			row++
		}
	}

	skipWhitespace := func() {
		for index < len(text) && text[index] == ' ' {
			index++
			row++
		}
	}

	sectionTitle := func() (string, error) {
		err := expect('[')
		if err != nil {
			return "", err
		}
		title := ""
		for index < len(text) && text[index] != '\n' {
			title += string(text[index])
			index++
		}
		if !strings.HasPrefix(title, "[") {
			return "", fmt.Errorf("Expected a title of the form [title] on line %d", row+1)
		}
		if !strings.HasSuffix(title, "]") {
			return "", fmt.Errorf("Expected a title of the form [title] on line %d", row+1)
		}
		return strings.TrimPrefix(strings.TrimSuffix(title, "]"), "["), nil
	}

	key := func() (string, error) {
		result := ""
		for index < len(text) && text[index] != '=' {
			result += string(text[index])
			index++
		}
		if len(result) == 0 {
			return "", fmt.Errorf("A key cannot be empty on line %d", row+1)
		}
		if result[0] == ' ' {
			return "", fmt.Errorf("A key cannot start with whitespace on line %d", row+1)
		}
		return strings.Trim(result, " "), nil
	}

	value := func() (string, error) {
		err := expect('"')
		if err != nil {
			return "", err
		}
		// skip the "
		index++
		result := ""
		for index < len(text) && (text[index] != '"' || (index > 0 && text[index] == '\\')) {
			result += string(text[index])
			index++
		}
		err = expect('"')
		// skip the "
		index++
		if err != nil {
			return "", err
		}
		return result, nil
	}

	hasEmptyLine := func() bool {
		return index+1 < len(text) && text[index] == '\n' && text[index+1] == '\n'
	}

	sectionNames := make([]string, 0)
	sections := make(map[string]IConfigSection)

	for index < len(text) {
		keys := make([]string, 0)
		values := make(map[string]string)

		skipNewlines()

		title, err := sectionTitle()
		if err != nil {
			return nil, err
		}

		err = expect('\n')
		if err != nil {
			return nil, err
		}
		// move past the newline
		index++

		for index < len(text) && !hasEmptyLine() {
			k, err := key()
			if err != nil {
				return nil, err
			}
			if _, ok := values[k]; ok {
				return nil, fmt.Errorf("Duplicate key specified: %s", k)
			}
			if strings.HasPrefix(k, "[") {
				return nil, fmt.Errorf("Unexpected key value that looks like the start of a section: %s", k)
			}
			if err = expect('='); err != nil {
				return nil, err
			}
			// move past the =
			index++
			skipWhitespace()
			v, err := value()
			if err != nil {
				return nil, err
			}
			keys = append(keys, k)
			values[k] = v
			if !hasEmptyLine() && index < len(text) && text[index] == '\n' {
				// move past the newline
				index++
			}
		}

		if _, ok := sections[title]; ok {
			return nil, fmt.Errorf("Duplicate defined section: %s", title)
		}

		sectionNames = append(sectionNames, title)
		sections[title] = &configSection{
			name:   title,
			keys:   keys,
			values: values,
		}
	}

	return &config{
		names:    sectionNames,
		sections: sections,
	}, nil
}

////////////////////////////////////////////////////////////////////////////////////////////////////

type configSection struct {
	name   string
	keys   []string
	values map[string]string
}

func (cs *configSection) Name() string {
	return cs.name
}

func (cs *configSection) Keys() []string {
	return cs.keys
}

func (cs *configSection) Get(key string) (string, bool) {
	if sec, ok := cs.values[key]; ok {
		return sec, ok
	}
	return "", false
}

////////////////////////////////////////////////////////////////////////////////////////////////////

type config struct {
	names    []string
	sections map[string]IConfigSection
}

func (c *config) SectionNames() []string {
	return c.names
}

func (c *config) Section(name string) (IConfigSection, bool) {
	if sec, ok := c.sections[name]; ok {
		return sec, true
	}
	return nil, false
}

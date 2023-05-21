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
	lines := strings.Split(text, "\n")
	if len(lines) == 0 {
		return &config{}, nil
	}

	sectionNames := make([]string, 0)
	sections := make(map[string]IConfigSection)

	i := 0
	for i < len(lines) {
		// move past any blank lines
		for i < len(lines) && len(strings.Trim(lines[i], " ")) == 0 {
			i += 1
		}

		if i >= len(lines) {
			break
		}

		cur := strings.Trim(lines[i], " ")
		if !strings.HasPrefix(cur, "[") || !strings.HasSuffix(cur, "]") {
			return nil, fmt.Errorf("Expected a section header but found %s", cur)
		}

		sectionName := strings.TrimSuffix(strings.TrimPrefix(cur, "["), "]")
		i += 1 // move past the section name

		keys := make([]string, 0)
		values := make(map[string]string)
		for i < len(lines) {
			line := strings.Trim(lines[i], " ")
			i += 1
			if len(line) == 0 {
				break
			}

			if strings.HasPrefix(line, "[") {
				return nil, fmt.Errorf(
					"Unexpected key value that looks like the start of a section: %s", line)
			}

			index := strings.Index(line, "=")
			if index < 0 {
				return nil, fmt.Errorf(
					"Expected `key = value` cut couldn't find `=` token: %s", line)
			}

			key := strings.Trim(line[0:index], " ")
			value := strings.Trim(line[index+1:], " ")
			for i < len(lines) && strings.HasPrefix(lines[i], " ") {
				value += lines[i]
				i += 1
			}

			if _, ok := values[key]; ok {
				return nil, fmt.Errorf("Duplicate key specified: %s", key)
			}

			keys = append(keys, key)
			values[key] = value
		}

		sec := configSection{
			name:   sectionName,
			keys:   keys,
			values: values,
		}

		if _, ok := sections[sectionName]; ok {
			return nil, fmt.Errorf("Duplicate defined section: %s", sectionName)
		}

		sectionNames = append(sectionNames, sectionName)
		sections[sectionName] = &sec
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

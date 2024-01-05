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
	"mathlingua/internal/mlglib"
)

type MlgConfig struct {
	View MlgViewConfig
}

type MlgViewConfig struct {
	Title       string
	Keywords    string
	Description string
}

func ParseMlgConfig(text string) (*MlgConfig, error) {
	conf, err := ParseConfig(text)
	if err != nil {
		return nil, err
	}

	sectionNames := conf.SectionNames()
	if len(sectionNames) == 0 {
		return &MlgConfig{
			View: MlgViewConfig{
				Title:       "",
				Keywords:    "",
				Description: "",
			},
		}, nil
	}

	for _, name := range sectionNames {
		if !expected_mlg_section_names.Has(name) {
			return nil, fmt.Errorf("Unexpected section: %s", name)
		}
	}

	viewConf, ok := conf.Section(mlg_view_section_name)
	if !ok {
		return &MlgConfig{}, nil
	}

	for _, key := range viewConf.Keys() {
		if !expected_mlg_view_keys.Has(key) {
			return nil, fmt.Errorf("Unexpected key: %s", key)
		}
	}

	title, _ := viewConf.Get(title_mlg_view_key)
	keywords, _ := viewConf.Get(keywords_mlg_view_key)
	description, _ := viewConf.Get(description_mlg_view_key)

	return &MlgConfig{
		View: MlgViewConfig{
			Title:       title,
			Keywords:    keywords,
			Description: description,
		},
	}, nil
}

////////////////////////////////////////////////////////////////////////////////////////////////////

const title_mlg_view_key = "title"
const keywords_mlg_view_key = "keywords"
const description_mlg_view_key = "description"
const home_mlg_view_key = "home"

var mlg_view_section_name = "mlg.view"

var expected_mlg_view_keys = buildExpectedMlgViewKeys()
var expected_mlg_section_names = buildExpectedMlgSectionNames()

func buildExpectedMlgViewKeys() *mlglib.Set[string] {
	result := mlglib.NewSet[string]()
	result.Add(title_mlg_view_key)
	result.Add(keywords_mlg_view_key)
	result.Add(description_mlg_view_key)
	result.Add(home_mlg_view_key)
	return result
}

func buildExpectedMlgSectionNames() *mlglib.Set[string] {
	result := mlglib.NewSet[string]()
	result.Add(mlg_view_section_name)
	return result
}

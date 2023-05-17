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
	Home        string
}

func ParseMlgConfig(text string) (MlgConfig, error) {
	conf, err := ParseConfig(text)
	if err != nil {
		return MlgConfig{}, err
	}

	sectionNames := conf.SectionNames()
	if len(sectionNames) == 0 {
		return MlgConfig{
			View: MlgViewConfig{
				Title:       "",
				Keywords:    "",
				Description: "",
			},
		}, nil
	}

	expectedNames := mlglib.NewSet[string]()
	expectedNames.Add("mlg.view")

	for _, name := range sectionNames {
		if !expectedNames.Has(name) {
			return MlgConfig{}, fmt.Errorf("Unexpected section: %s", name)
		}
	}

	viewConf, ok := conf.Section("mlg.view")
	if !ok {
		return MlgConfig{}, nil
	}

	title, _ := viewConf.Get("title")
	keywords, _ := viewConf.Get("keywords")
	description, _ := viewConf.Get("description")
	home, _ := viewConf.Get("home")

	return MlgConfig{
		View: MlgViewConfig{
			Title:       title,
			Keywords:    keywords,
			Description: description,
			Home:        home,
		},
	}, nil
}

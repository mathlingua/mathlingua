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

type ITocConfig interface {
	// Returns: (label, true) if the filename should have 'label' used
	//     and: ("", false) if the filename should be hidden
	LabelForFilename(filename string) (string, bool)
	ExplicitFilenames() []string
}

func ParseTocConfig(text string) (ITocConfig, error) {
	conf, err := ParseConfig(text)
	if err != nil {
		return &tocConfig{}, err
	}

	names := conf.SectionNames()
	for _, name := range names {
		if name != "toc" {
			return &tocConfig{}, fmt.Errorf("Unexpected section: %s", name)
		}
	}

	sec, found := conf.Section("toc")
	if !found {
		return &tocConfig{}, fmt.Errorf("The [toc] section was not specified")
	}

	keys := sec.Keys()
	starIndex := -1
	for i, key := range keys {
		if key == "*" {
			if starIndex < 0 {
				starIndex = i
			} else {
				return &tocConfig{}, fmt.Errorf("If * is specified it must be specified only once")
			}
		}
	}

	result := tocConfig{
		explicitFilenames: make([]string, 0),
		nameToLabel:       make(map[string]string),
		namesToHide:       mlglib.NewSet[string](),
	}

	if starIndex >= 0 {
		if starIndex != len(keys)-1 {
			return &tocConfig{}, fmt.Errorf("If * is specified it must be the last element specified")
		} else {
			if starValue, ok := sec.Get("*"); ok {
				if starValue == string(keep) {
					result.setStarAction(keep)
				} else if starValue == string(hide) {
					result.setStarAction(hide)
				} else {
					return &tocConfig{}, fmt.Errorf("* must have a value of either 'keep' or 'hide'")
				}
			} else {
				return &tocConfig{}, fmt.Errorf("Could not determine the value for *")
			}
			keys = keys[0 : len(keys)-1]
		}
	}

	for _, filename := range keys {
		if label, ok := sec.Get(filename); ok {
			if label == string(keep) {
				result.markToKeep(filename)
			} else if label == string(hide) {
				result.markToHide(filename)
			} else {
				result.markNewLabel(filename, label)
			}
		} else {
			return &tocConfig{}, fmt.Errorf("Could not determine the lable for %s", filename)
		}
	}

	return &result, nil
}

////////////////////////////////////////////////////////////////////////////////////////////////////

type TocType string

const (
	keep TocType = "keep"
	hide TocType = "hide"
)

type tocConfig struct {
	explicitFilenames []string
	nameToLabel       map[string]string
	namesToHide       mlglib.ISet[string]
	starAction        TocType
}

func (tc *tocConfig) LabelForFilename(filename string) (string, bool) {
	label, ok := tc.nameToLabel[filename]
	if ok {
		return label, true
	}

	if tc.namesToHide.Has(filename) {
		return "", false
	}

	if tc.starAction == keep {
		return filename, true
	}

	return "", false
}

func (tc *tocConfig) ExplicitFilenames() []string {
	return tc.explicitFilenames
}

////////////////////////////////////////////////////////////////////////////////////////////////////

func (tc *tocConfig) markToKeep(filename string) {
	tc.markNewLabel(filename, filename)
}

func (tc *tocConfig) markToHide(filename string) {
	tc.explicitFilenames = append(tc.explicitFilenames, filename)
	tc.namesToHide.Add(filename)
}

func (tc *tocConfig) markNewLabel(filename string, label string) {
	tc.explicitFilenames = append(tc.explicitFilenames, filename)
	tc.nameToLabel[filename] = label
}

func (tc *tocConfig) setStarAction(action TocType) {
	tc.starAction = action
}

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

type TocType string

const (
	Keep TocType = "keep"
	Hide TocType = "hide"
)

func NewDefaultTocConfig() *TocConfig {
	config := &TocConfig{
		explicitFilenames: make([]string, 0),
		nameToLabel:       make(map[string]string),
		namesToHide:       *mlglib.NewSet[string](),
	}
	config.setStarAction(Keep)
	return config
}

func ParseTocConfig(text string) (*TocConfig, error) {
	conf, err := ParseConfig(text)
	if err != nil {
		return nil, err
	}

	names := conf.SectionNames()
	for _, name := range names {
		if name != "toc" {
			return nil, fmt.Errorf("Unexpected section: %s", name)
		}
	}

	sec, found := conf.Section("toc")
	if !found {
		return nil, fmt.Errorf("The [toc] section was not specified")
	}

	keys := sec.Keys()
	starIndex := -1
	for i, key := range keys {
		if key == "*" {
			if starIndex < 0 {
				starIndex = i
				// it is ok to break because the ParseConfig function will report
				// an error if the * key is specified more than once
				break
			}
		}
	}

	if starIndex < 0 {
		return &TocConfig{}, fmt.Errorf(
			"A toc.config must contain either '* = keep' or '* = hide' as its last entry")
	}

	result := &TocConfig{
		explicitFilenames: make([]string, 0),
		nameToLabel:       make(map[string]string),
		namesToHide:       *mlglib.NewSet[string](),
	}

	if starIndex != len(keys)-1 {
		return &TocConfig{}, fmt.Errorf("If * is specified it must be the last element specified")
	}

	if starValue, ok := sec.Get("*"); ok {
		if starValue == string(Keep) {
			result.setStarAction(Keep)
		} else if starValue == string(Hide) {
			result.setStarAction(Hide)
		} else {
			return &TocConfig{}, fmt.Errorf("* must have a value of either 'keep' or 'hide'")
		}
	} else {
		return &TocConfig{}, fmt.Errorf("Could not determine the value for *")
	}
	keys = keys[0 : len(keys)-1]

	for _, filename := range keys {
		if label, ok := sec.Get(filename); ok {
			if label == string(Keep) {
				result.markToKeep(filename)
			} else if label == string(Hide) {
				result.markToHide(filename)
			} else {
				result.markNewLabel(filename, label)
			}
		} else {
			return &TocConfig{}, fmt.Errorf("Could not determine the lable for %s", filename)
		}
	}

	return result, nil
}

////////////////////////////////////////////////////////////////////////////////////////////////////

type TocConfig struct {
	explicitFilenames []string
	nameToLabel       map[string]string
	namesToHide       mlglib.Set[string]
	starAction        TocType
}

// Returns: (label, true) if the filename should have 'label' used
// and: ("", false) if the filename should be hidden
func (tc *TocConfig) LabelForFilename(filename string) (string, bool) {
	label, ok := tc.nameToLabel[filename]
	if ok {
		return label, true
	}

	if tc.namesToHide.Has(filename) {
		return "", false
	}

	if tc.starAction == Keep {
		return filename, true
	}

	return "", false
}

func (tc *TocConfig) ExplicitFilenames() []string {
	return tc.explicitFilenames
}

func (tc *TocConfig) StarAction() TocType {
	return tc.starAction
}

////////////////////////////////////////////////////////////////////////////////////////////////////

func (tc *TocConfig) markToKeep(filename string) {
	tc.markNewLabel(filename, filename)
}

func (tc *TocConfig) markToHide(filename string) {
	tc.explicitFilenames = append(tc.explicitFilenames, filename)
	tc.namesToHide.Add(filename)
}

func (tc *TocConfig) markNewLabel(filename string, label string) {
	tc.explicitFilenames = append(tc.explicitFilenames, filename)
	tc.nameToLabel[filename] = label
}

func (tc *TocConfig) setStarAction(action TocType) {
	tc.starAction = action
}

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
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestParseTocConfigText(t *testing.T) {
	text := `
[toc]
file 1 = File 1 Label
file 2 = File 2 Label
file3 = keep
file4 = hide
* = keep
`
	tocConfig, err := ParseTocConfig(text)
	assert.Nil(t, err)

	assert.Equal(t, []string{
		"file 1",
		"file 2",
		"file3",
		"file4",
	}, tocConfig.ExplicitFilenames())

	file1Label, showFile1 := tocConfig.LabelForFilename("file 1")
	assert.True(t, showFile1)
	assert.Equal(t, "File 1 Label", file1Label)

	file2Label, showFile2 := tocConfig.LabelForFilename("file 2")
	assert.True(t, showFile2)
	assert.Equal(t, "File 2 Label", file2Label)

	file3Label, showFile3 := tocConfig.LabelForFilename("file3")
	assert.True(t, showFile3)
	assert.Equal(t, file3Label, file3Label)

	_, showFile4 := tocConfig.LabelForFilename("file4")
	assert.False(t, showFile4)

	other1, showOther1 := tocConfig.LabelForFilename("other1")
	assert.True(t, showOther1)
	assert.Equal(t, other1, other1)

	other2, showOther2 := tocConfig.LabelForFilename("other 2")
	assert.True(t, showOther2)
	assert.Equal(t, other2, other2)
}

func TestTocConfigWithoutStar(t *testing.T) {
	text := `
[toc]
file 1 = File 1 Label
`
	_, err := ParseTocConfig(text)
	assert.NotNil(t, err)
	assert.Equal(t,
		"A toc.config must contain either '* = keep' or '* = hide' as its last entry", err.Error())
}

func TestTocConfigStarNotLast(t *testing.T) {
	text := `
[toc]
* = keep
file 1 = File 1 Label
`
	_, err := ParseTocConfig(text)
	assert.NotNil(t, err)
	assert.Equal(t, "If * is specified it must be the last element specified", err.Error())
}

func TestTocConfigStarInvalidValue(t *testing.T) {
	text := `
[toc]
* = invalid
`
	_, err := ParseTocConfig(text)
	assert.NotNil(t, err)
	assert.Equal(t, "* must have a value of either 'keep' or 'hide'", err.Error())
}

func TestTocConfigStarNoValue(t *testing.T) {
	text := `
[toc]
* =
`
	_, err := ParseTocConfig(text)
	assert.NotNil(t, err)
	assert.Equal(t, "* must have a value of either 'keep' or 'hide'", err.Error())
}

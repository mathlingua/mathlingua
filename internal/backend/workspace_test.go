/*
 * Copyright 2024 Dominic Kramer
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

package backend

import (
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestSimpleNameToRenderedForm(t *testing.T) {
	expected := "someName"
	actual := nameToRenderedName("someName", false)
	assert.Equal(t, expected, actual)
}

func TestSimpleVarArgNameToRenderedForm(t *testing.T) {
	expected := "{someName}..."
	actual := nameToRenderedName("someName", true)
	assert.Equal(t, expected, actual)
}

func TestSimpleGreekNameToRenderedForm(t *testing.T) {
	expected := "\\beta"
	actual := nameToRenderedName("beta", false)
	assert.Equal(t, expected, actual)
}

func TestSimpleGreekVarArgNameToRenderedForm(t *testing.T) {
	expected := "{\\beta}..."
	actual := nameToRenderedName("beta", true)
	assert.Equal(t, expected, actual)
}

func TestPrimeGreekNameToRenderedForm(t *testing.T) {
	expected := "\\beta'"
	actual := nameToRenderedName("beta`", false)
	assert.Equal(t, expected, actual)
}

func TestPrimeGreekVarArgNameToRenderedForm(t *testing.T) {
	expected := "{\\beta'}..."
	actual := nameToRenderedName("beta`", true)
	assert.Equal(t, expected, actual)
}

func TestGreekNumberNameToRenderedForm(t *testing.T) {
	expected := "\\beta_{0}"
	actual := nameToRenderedName("beta0", false)
	assert.Equal(t, expected, actual)
}

func TestGreekNumberVarArgNameToRenderedForm(t *testing.T) {
	expected := "{\\beta_{0}}..."
	actual := nameToRenderedName("beta0", true)
	assert.Equal(t, expected, actual)
}

func TestGreekUnderscoreNumberNameToRenderedForm(t *testing.T) {
	expected := "\\beta_{0}"
	actual := nameToRenderedName("beta_0", false)
	assert.Equal(t, expected, actual)
}

func TestGreekUnderscoreNumberVarArgNameToRenderedForm(t *testing.T) {
	expected := "{\\beta_{0}}..."
	actual := nameToRenderedName("beta_0", true)
	assert.Equal(t, expected, actual)
}

func TestGreekPrimeNumberNameToRenderedForm(t *testing.T) {
	expected := "\\beta'_{0}"
	actual := nameToRenderedName("beta`0", false)
	assert.Equal(t, expected, actual)
}

func TestGreekPrimeNumberVarArgNameToRenderedForm(t *testing.T) {
	expected := "{\\beta'_{0}}..."
	actual := nameToRenderedName("beta`0", true)
	assert.Equal(t, expected, actual)
}

func TestNestedNameToRenderedForm(t *testing.T) {
	expected := "abc_{xyz_{abc_{123}}}"
	actual := nameToRenderedName("abc_xyz_abc_123", false)
	assert.Equal(t, expected, actual)
}

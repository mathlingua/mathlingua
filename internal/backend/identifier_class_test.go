/*
 * Copyright 2022 Dominic Kramer
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
	"sort"
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestEmptyClassHasNoNames(t *testing.T) {
	cls := NewIdentifierClass()
	assert.Equal(t, false, cls.HasIdentifier(""))
	assert.Equal(t, false, cls.HasIdentifier("a"))
	assert.Equal(t, false, cls.HasIdentifier("someName"))
	assert.Equal(t, false, cls.HasIdentifier("f()"))
}

func TestEmptyClassHasNoRepresentative(t *testing.T) {
	cls := NewIdentifierClass()
	_, ok := cls.GetRepresentative()
	assert.Equal(t, false, ok)
}

func TestRepresentativeIsFirstIdentifierAdded(t *testing.T) {
	cls := NewIdentifierClass()
	cls.AddIdentifier("abc")
	cls.AddIdentifier("xyz")
	cls.AddIdentifier("f()")
	repr, ok := cls.GetRepresentative()
	assert.Equal(t, true, ok)
	assert.Equal(t, "abc", repr)
}

func TestClassHasIdentifiers(t *testing.T) {
	cls := NewIdentifierClass()
	cls.AddIdentifier("abc")
	cls.AddIdentifier("xyz")
	cls.AddIdentifier("f()")
	cls.AddIdentifier("")
	assert.Equal(t, true, cls.HasIdentifier("abc"))
	assert.Equal(t, true, cls.HasIdentifier("xyz"))
	assert.Equal(t, true, cls.HasIdentifier("f()"))
	assert.Equal(t, true, cls.HasIdentifier(""))
	assert.Equal(t, false, cls.HasIdentifier("abc1"))
	assert.Equal(t, false, cls.HasIdentifier("someInvalidName"))
}

func TestClassGetAllIdentifiers(t *testing.T) {
	cls := NewIdentifierClass()
	cls.AddIdentifier("abc")
	cls.AddIdentifier("xyz")
	cls.AddIdentifier("f()")
	cls.AddIdentifier("")
	assert.Equal(t, sort.StringSlice([]string{
		"abc",
		"xyz",
		"f()",
		"",
	}), sort.StringSlice(cls.GetAllIdentifiers()))
}

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

type IdentifierClass interface {
	GetRepresentative() (string, bool)
	AddIdentifier(name string)
	HasIdentifier(name string) bool
	GetAllIdentifiers() []string
}

func NewIdentifierClass() IdentifierClass {
	return &identifierClass{
		hasRepr: false,
		names:   make(map[string]bool),
		repr:    "",
	}
}

/////////////////////////////////////////////////////////////////

type identifierClass struct {
	hasRepr bool
	names   map[string]bool
	repr    string
}

func (ic *identifierClass) GetRepresentative() (string, bool) {
	return ic.repr, ic.hasRepr
}

func (ic *identifierClass) AddIdentifier(name string) {
	ic.names[name] = true
	if !ic.hasRepr {
		ic.hasRepr = true
		ic.repr = name
	}
}

func (ic *identifierClass) HasIdentifier(name string) bool {
	_, ok := ic.names[name]
	return ok
}

func (ic *identifierClass) GetAllIdentifiers() []string {
	keys := make([]string, 0, len(ic.names))
	for k := range ic.names {
		keys = append(keys, k)
	}
	return keys
}

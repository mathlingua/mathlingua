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

package ast

func NewScope(parent *Scope) *Scope {
	return &Scope{
		parent:  parent,
		symbols: make(map[string]*Symbol, 0),
	}
}

type Scope struct {
	parent  *Scope
	symbols map[string]*Symbol
}

func (s *Scope) SetAliased(existingName string, newName string) {
	if _, ok := s.symbols[existingName]; ok {
		s.symbols[newName] = s.symbols[existingName]
		return
	}
	if s.parent != nil {
		s.parent.SetAliased(existingName, newName)
	}
}

func (s *Scope) GetSymbol(name string) (*Symbol, bool) {
	if sym, ok := s.symbols[name]; ok {
		return sym, ok
	}
	if s.parent != nil {
		return s.parent.GetSymbol(name)
	}
	return nil, false
}

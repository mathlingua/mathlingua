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

package backend

import (
	"mathlingua/internal/ast"
)

func NewScope() ast.Scope {
	return &scope{}
}

////////////////////////////////////////////////////////////////////////////////////////////////////

type scope struct {
	parent   *ast.Scope
	children []*ast.Scope
	names    map[string]*ast.NameInfo
}

func (s *scope) GetParent() (*ast.Scope, bool) {
	return s.parent, s.parent != nil
}

func (s *scope) SetParent(parent *ast.Scope) {
	s.parent = parent
}

func (s *scope) GetChildScopeCount() int {
	return len(s.children)
}

func (s *scope) GetChildScopeAt(index int) (*ast.Scope, bool) {
	if index < 0 || index >= s.GetChildScopeCount() {
		return nil, false
	}
	return s.children[index], true
}

func (s *scope) SetChildScopeAt(index int, scope *ast.Scope) {
	s.children[index] = scope
}

func (s *scope) GetNameInfo(name string) (ast.NameInfo, bool) {
	nameInfo, ok := s.names[name]
	return *nameInfo, ok
}

func (s *scope) SetNameInfo(name string, info ast.NameInfo) {
	s.names[name] = &info
}

////////////////////////////////////////////////////////////////////////////////////////////////////

func PopulateScopes(node ast.MlgNodeType) {
	if node == nil {
		return
	}
	node.GetCommonMetaData().Scope = NewScope()
	node.ForEach(PopulateScopes)
}

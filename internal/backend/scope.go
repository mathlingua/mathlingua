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
	parent *ast.Scope
	names  map[string]*ast.NameInfo
}

func (s *scope) GetParent() (*ast.Scope, bool) {
	return s.parent, s.parent != nil
}

func (s *scope) SetParent(parent *ast.Scope) {
	s.parent = parent
}

func (s *scope) GetNameInfo(name string) (ast.NameInfo, bool) {
	nameInfo, ok := s.names[name]
	return *nameInfo, ok
}

func (s *scope) SetNameInfo(name string, info ast.NameInfo) {
	s.names[name] = &info
}

/*
func (s *scope) AddSpecify(specifies *ast.SpecifyGroup) {
	for _, spec := range specifies.Specify.Specify {
		switch grp := spec.(type) {
		case *ast.ZeroGroup:
			clause := grp.Means.Means
			if formulation, ok := clause.(*ast.Formulation[ast.FormulationNodeType]); ok {
				if is, ok := formulation.Root.(*ast.IsExpression); ok {
					for _, r := range is.Rhs {
						if cmd, ok := r.(*ast.CommandExpression); ok {
							sig := GetSignatureString(*cmd)
							signatures := mlglib.NewSet[string]()
							signatures.Add(sig)
							resType := ast.ResolvedType{
								Signatures: signatures,
							}
							nameInfo := ast.NameInfo{
								IsInfereble:     false,
								IsPlaceholder:   false,
								IsNumberLiteral: true,
								Type:            resType,
							}
							s.names["0"] = &nameInfo
						}
					}
				}
			}
		case *ast.PositiveIntGroup:
			// TODO
		case *ast.NegativeIntGroup:
			// TODO
		case *ast.PositiveFloatGroup:
			// TODO
		case *ast.NegativeFloatGroup:
			// TODO
		}
	}
}
*/

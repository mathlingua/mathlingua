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

type IScope interface {
	SetIdentifierInfo(identifier string, info IdentifierInfo)
	GetMutableIdentifierInfo(identifier string) (*IdentifierInfo, bool)
	Clone() *Scope
}

func NewScope() *Scope {
	return &Scope{
		idenInfos: make(map[string]*IdentifierInfo, 0),
	}
}

////////////////////////////////////////////////////////////////////////////////////////////////////

type Scope struct {
	idenInfos map[string]*IdentifierInfo
}

func (s *Scope) SetIdentifierInfo(identifier string, info IdentifierInfo) {
	s.idenInfos[identifier] = &info
}

func (s *Scope) GetMutableIdentifierInfo(identifier string) (*IdentifierInfo, bool) {
	info, ok := s.idenInfos[identifier]
	return info, ok
}

func (s *Scope) Clone() *Scope {
	idensCopy := make(map[string]*IdentifierInfo, 0)
	for iden, info := range s.idenInfos {
		idensCopy[iden] = info.Clone()
	}
	return &Scope{
		idenInfos: idensCopy,
	}
}

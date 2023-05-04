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

import "mathlingua/internal/ast"

type Context struct {
	Mapping       *IdentifierMapping
	CallSiteScope ast.IScope
}

type IdentifierMapping struct {
	callIdenToDefIden map[string]string
	defIdenToCallIden map[string]string
}

func NewIdentifierMapping() *IdentifierMapping {
	return &IdentifierMapping{
		callIdenToDefIden: make(map[string]string, 0),
		defIdenToCallIden: make(map[string]string, 0),
	}
}

func (im *IdentifierMapping) GetCallSiteIdentifier(defSiteIden string) (string, bool) {
	iden, ok := im.defIdenToCallIden[defSiteIden]
	return iden, ok
}

func (im *IdentifierMapping) GetDefSiteIdentifier(callSiteIden string) (string, bool) {
	iden, ok := im.callIdenToDefIden[callSiteIden]
	return iden, ok
}

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

import (
	"mathlingua/internal/mlglib"
)

// An IdentifierInfo describes information about an identifier such as its type
// information, whether it is a number literal, or a placeholder etc.
type IdentifierInfo struct {
	IsInfereble     bool
	IsPlaceholder   bool
	IsNumberLiteral bool
	Signatures      mlglib.ISet[string]
	Specs           []SpecInfo
}

func (ii *IdentifierInfo) Clone() *IdentifierInfo {
	specsCopy := make([]SpecInfo, 0)
	for _, info := range ii.Specs {
		specsCopy = append(specsCopy, *info.Clone())
	}
	return &IdentifierInfo{
		IsInfereble:     ii.IsInfereble,
		IsPlaceholder:   ii.IsPlaceholder,
		IsNumberLiteral: ii.IsNumberLiteral,
		Signatures:      ii.Signatures.Clone(),
		Specs:           specsCopy,
	}
}

type SpecInfo struct {
	Name   string
	Target ExpressionKind
}

func (si *SpecInfo) Clone() *SpecInfo {
	return &SpecInfo{
		Name:   si.Name,
		Target: CloneNode(si.Target),
	}
}

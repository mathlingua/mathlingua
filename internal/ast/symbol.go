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

package ast

type Symbol struct {
	Names           []string
	IsInfereble     bool
	IsPlaceholder   bool
	IsNumberLiteral bool
	Pattern         PatternKind
	ResolvedSpec    *ResolvedSymbolSpec
	RawSpec         *RawSymbolSpec
	DeclaringScope  *Scope
}

////////////////////////////////////////////////////////////////////////////////////////////////////

type SymbolSpecKind interface {
	SymbolSpecKind()
}

func (*ResolvedSymbolSpec) SymbolSpecKind() {}
func (*RawSymbolSpec) SymbolSpecKind()      {}

type ResolvedSymbolSpec struct {
	Is   ResolvedTypeKind
	Also []ResolvedTypeKind
}

type RawSymbolSpec struct {
	Is   TypeKind
	Also []TypeKind
}

////////////////////////////////////////////////////////////////////////////////////////////////////

type TypeKind interface {
	TypeKind()
}

func (*RawNonInfixCommandType) TypeKind() {}
func (*RawInfixCommandType) TypeKind()    {}
func (*RawDynamicInfixType) TypeKind()    {}

func (*ResolvedNonInfixCommandType) TypeKind() {}
func (*ResolvedInfixCommandType) TypeKind()    {}

////////////////////////////////////////////////////////////////////////////////////////////////////

type RawTypeKind interface {
	TypeKind
	ResolvedType()
}

func (*RawNonInfixCommandType) RawTypeKind() {}
func (*RawInfixCommandType) RawTypeKind()    {}
func (*RawDynamicInfixType) RawTypeKind()    {}

type RawNonInfixCommandType struct {
	Names                []string
	NamedGroupTypeParams []RawNamedGroupTypeParam
}

type RawNamedGroupTypeParam struct {
	Name   string
	Params []Symbol
}

type RawInfixCommandType struct {
	Lhs    Symbol
	Target ResolvedNonInfixCommandType
	Rhs    Symbol
}

type RawDynamicInfixType struct {
	Operator []string
	Target   Symbol
}

////////////////////////////////////////////////////////////////////////////////////////////////////

type ResolvedTypeKind interface {
	TypeKind
	ResolvedTypeKind()
}

func (*ResolvedNonInfixCommandType) ResolvedTypeKind() {}
func (*ResolvedInfixCommandType) ResolvedTypeKind()    {}

type ResolvedNonInfixCommandType struct {
	Names                []string
	NamedGroupTypeParams []ResolvedNamedGroupTypeParam
}

type ResolvedNamedGroupTypeParam struct {
	Name   string
	Params []TypeKind
}

type ResolvedInfixCommandType struct {
	Lhs    TypeKind
	Target ResolvedNonInfixCommandType
	Rhs    TypeKind
}

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
	ResolvedSpec    *ResolvedSymbolSpec
	RawSpec         *RawSymbolSpec
	DeclaringScope  *Scope
}

////////////////////////////////////////////////////////////////////////////////////////////////////

type SymbolSpec interface {
	SymbolSpec()
}

func (*ResolvedSymbolSpec) SymbolSpec() {}
func (*RawSymbolSpec) SymbolSpec()      {}

type ResolvedSymbolSpec struct {
	Is        ResolvedType
	Satisfies []ResolvedType
}

type RawSymbolSpec struct {
	Is        Type
	Satisfies []Type
}

////////////////////////////////////////////////////////////////////////////////////////////////////

type Type interface {
	Type()
}

func (*RawNonInfixCommandType) Type() {}
func (*RawInfixCommandType) Type()    {}
func (*RawDynamicInfixType) Type()    {}

func (*ResolvedNonInfixCommandType) Type() {}
func (*ResolvedInfixCommandType) Type()    {}

////////////////////////////////////////////////////////////////////////////////////////////////////

type RawType interface {
	Type
	ResolvedType()
}

func (*RawNonInfixCommandType) RawType() {}
func (*RawInfixCommandType) RawType()    {}
func (*RawDynamicInfixType) RawType()    {}

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

type ResolvedType interface {
	Type
	ResolvedType()
}

func (*ResolvedNonInfixCommandType) ResolvedType() {}
func (*ResolvedInfixCommandType) ResolvedType()    {}

type ResolvedNonInfixCommandType struct {
	Names                []string
	NamedGroupTypeParams []ResolvedNamedGroupTypeParam
}

type ResolvedNamedGroupTypeParam struct {
	Name   string
	Params []Type
}

type ResolvedInfixCommandType struct {
	Lhs    Type
	Target ResolvedNonInfixCommandType
	Rhs    Type
}

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

import "mathlingua/internal/mlglib"

type SpecAliasInfo2 struct {
	SpecAlias string
	Exp       *ExpressionType
	Scope     ScopeType
	Context   ContextType
}

type UnResolvedType struct {
	Form               UnResolvedTypeFormType
	Signatures         mlglib.Set[string]
	DefSiteSpecAliases []SpecAliasInfo
}

type UnResolvedTypeFormType interface {
	UnResolvedTypeFormType()
}

func (UnResolvedFunctionTypeForm) UnResolvedTypeFormType() {}
func (UnResolvedTupleTypeForm) UnResolvedTypeFormType()    {}
func (UnResolvedSetTypeForm) UnResolvedTypeFormType()      {}

type UnResolvedFunctionTypeForm struct {
	Inputs []UnResolvedType
	Output UnResolvedType
}

type UnResolvedTupleTypeForm struct {
	Items []UnResolvedType
}

type UnResolvedSetTypeForm struct {
	Target UnResolvedType
}

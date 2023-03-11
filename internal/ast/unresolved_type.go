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

type SpecAliasInfo struct {
	SpecAlias string
	Exp       *ExpressionType
	Scope     *Scope
	Context   *Context
}

type UnResolvedType struct {
	Form               UnResolvedTypeForm
	Signatures         mlglib.Set[string]
	DefSiteSpecAliases []SpecAliasInfo
}

type UnResolvedTypeForm interface {
	UnResolvedTypeForm()
}

func (UnResolvedFunctionTypeForm) UnResolvedTypeForm() {}
func (UnResolvedTupleTypeForm) UnResolvedTypeForm()    {}
func (UnResolvedSetTypeForm) UnResolvedTypeForm()      {}

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

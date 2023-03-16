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

/*
 * A resolved type contains the type information for an item
 * after all spec aliases have been resolved.
 *
 * Note: Resolved types do not contain names because they
 *       describe variable types and variables themselves.
 */
type ResolvedType struct {
	Form       IResolvedTypeForm
	Signatures mlglib.ISet[string]
}

type IResolvedTypeForm interface {
	IResolvedTypeForm()
}

func (ResolvedFunctionTypeForm) IResolvedTypeForm() {}
func (ResolvedTupleTypeForm) IResolvedTypeForm()    {}
func (ResolvedISetForm) IResolvedTypeForm()         {}

type ResolvedFunctionTypeForm struct {
	Inputs []ResolvedType
	Output ResolvedType
}

type ResolvedTupleTypeForm struct {
	Items []ResolvedType
}

type ResolvedISetForm struct {
	Target ResolvedType
}

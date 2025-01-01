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

type AtomicCoreType struct {
	Signature string
}

////////////////////////////////////////////////////////////////////////////////////////////////////

type FunctionCoreType struct {
	Input  []CoreTypeKind
	Output CoreTypeKind
}

////////////////////////////////////////////////////////////////////////////////////////////////////

type TupleCoreType struct {
	fields       map[string]CoreTypeKind
	indexAliases map[int]string
}

func (t *TupleCoreType) getNumFields() int {
	return len(t.fields)
}

func (t *TupleCoreType) getFieldKeyForIndex(index int) (string, bool) {
	key, ok := t.indexAliases[index]
	return key, ok
}

func (t *TupleCoreType) getFieldTypeByKey(key string) (CoreTypeKind, bool) {
	coreType, ok := t.fields[key]
	return coreType, ok
}

func (t *TupleCoreType) getFieldTypeByIndex(index int) (CoreTypeKind, bool) {
	key, ok := t.getFieldKeyForIndex(index)
	if !ok {
		return nil, false
	}
	return t.getFieldTypeByKey(key)
}

////////////////////////////////////////////////////////////////////////////////////////////////////

type SetCoreType struct {
	Input  []CoreTypeKind
	Target CoreTypeKind
}

////////////////////////////////////////////////////////////////////////////////////////////////////

type RawAtomicCoreType struct {
	Signature string
}

type RawFunctionCoreType struct {
	Input  []ExpressionKind
	Output ExpressionKind
}

type RawTupleCoreType struct {
	Items []ExpressionKind
}

type RawSetCoreType struct {
	Inputs     []*StructuralFormKind
	Target     ExpressionKind
	Conditions []ExpressionKind
}

type RawSpecAliasCoreType struct {
	Lhs      *StructuralFormKind
	Operator ExpressionKind
	Rhs      ExpressionKind
}

////////////////////////////////////////////////////////////////////////////////////////////////////

type TypeSpec struct {
	Signature string
	CoreType  CoreTypeKind
}

type RawTypeSpec struct {
	Signature   string
	RawCoreType RawCoreTypeKind
}

////////////////////////////////////////////////////////////////////////////////////////////////////

type TypeDescription struct {
	Is        TypeSpec
	Satisfies []TypeSpec
}

type RawTypeDescription struct {
	Is        RawTypeSpec
	Satisfies []RawTypeSpec
}

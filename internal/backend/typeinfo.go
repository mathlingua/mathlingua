/*
 * Copyright 2022 Dominic Kramer
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

// Describes the set of one or more signatures that describe a name.
// For example, this interface describes what it means for a name,
// `x`, to be `x is \a` and `x is \b`.
type TypeInfo interface {
	TypeInfo()

	// Determines if there was an error identifying all of the signatures in
	// this type info.  This is useful to determine whether or not to show
	// an error to a user if this type info does not allow, for example, an
	// operator to be resolved.
	HasError() bool

	// Detrmines if the name described by this type info is of the type
	// described by `signature`, is of a type that extends the type
	// described by `signature`, or can be viewed as the type described by
	// `signature`.
	CheckCompatible(signature string) bool

	// Detrmines if the name described by this type info is a strict sub-type
	// of the type described by `signature`.
	CheckExtends(signature string) bool

	GetLeastCommonAncestors() []TypeInfo

	GetMember(name string) (TypeInfo, bool)

	GetOperation(name string) (Operation, bool)

	////////////////////////////////////////////////////////////////
	AddConstraint(command ast.CommandExpression)
}

type NameTypeInfo struct {
	Name string
}

type FunctionTypeInfo struct {
	Name string
	Args []string
}

type TupleTypeInfo struct {
	Items []TypeInfo
}

type FixedSetTypeInfo struct {
	Items []TypeInfo
}

type ConditionalSetTypeInfo struct {
	Target TypeInfo
}

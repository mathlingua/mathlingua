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

type ResolveTypeArg struct {
	Type               UnResolvedType
	CallToDefSiteNames mlglib.INameMapping
	CallSiteScope      IScope
	CallSiteContext    IContext
	DefSiteScope       IScope
}

type ResolveTypeFormArg struct {
	TypeForm           IUnResolvedFormType
	CallToDefSiteNames mlglib.INameMapping
	CallSiteScope      IScope
	CallSiteContext    IContext
	DefSiteScope       IScope
}

type ResolveExpressionArg struct {
	Exp                ExpressionType
	CallToDefSiteNames mlglib.INameMapping
	CallSiteScope      IScope
	CallSiteContext    IContext
	DefSiteScope       IScope
}

type IContext interface {
	GetParent() IContext
	SetParent(parent IContext)
	IsSubTypeOf(from string, to string) bool
	IsViewableAs(from string, to string) bool
	ResolveType(arg ResolveTypeArg) ResolvedType
	ResolveTypeForm(arg ResolveTypeFormArg) IResolvedTypeForm
	ResolveExpression(arg ResolveExpressionArg) ExpressionType
	GetWrittenAs(exp ExpressionType, scope IScope)
}

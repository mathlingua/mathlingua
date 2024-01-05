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

type AliasSummaryKind interface {
	AliasSummaryKind()
}

func (*SpecAliasSummary) AliasSummaryKind() {}

func (*InfixExpAliasSummary) AliasSummaryKind()          {}
func (*PrefixExpAliasSummary) AliasSummaryKind()         {}
func (*PostfixExpAliasSummary) AliasSummaryKind()        {}
func (*FunctionExpAliasSummary) AliasSummaryKind()       {}
func (*CommandExpAliasSummary) AliasSummaryKind()        {}
func (*MemberNameExpAliasSummary) AliasSummaryKind()     {}
func (*MemberFunctionExpAliasSummary) AliasSummaryKind() {}
func (*MemberInfixExpAliasSummary) AliasSummaryKind()    {}
func (*MemberPrefixExpAliasSummary) AliasSummaryKind()   {}
func (*MemberPostfixExpAliasSummary) AliasSummaryKind()  {}

////////////////////////////////////////////////////////////////////////////////////////////////////

type SpecAliasSummaryRhsKind interface {
	SpecAliasSummaryRhsKind()
}

func (*IsConstraint) SpecAliasSummaryRhsKind()   {}
func (*SpecConstraint) SpecAliasSummaryRhsKind() {}

type SpecAliasSummary struct {
	Lhs SpecAliasPattern
	Rhs SpecAliasSummaryRhsKind
}

////////////////////////////////////////////////////////////////////////////////////////////////////

type ExpAliasSummaryKind interface {
	AliasSummaryKind
	ExpAliasSummaryKind()
}

func (*InfixExpAliasSummary) ExpAliasSummaryKind()          {}
func (*PrefixExpAliasSummary) ExpAliasSummaryKind()         {}
func (*PostfixExpAliasSummary) ExpAliasSummaryKind()        {}
func (*FunctionExpAliasSummary) ExpAliasSummaryKind()       {}
func (*CommandExpAliasSummary) ExpAliasSummaryKind()        {}
func (*MemberNameExpAliasSummary) ExpAliasSummaryKind()     {}
func (*MemberFunctionExpAliasSummary) ExpAliasSummaryKind() {}
func (*MemberInfixExpAliasSummary) ExpAliasSummaryKind()    {}
func (*MemberPrefixExpAliasSummary) ExpAliasSummaryKind()   {}
func (*MemberPostfixExpAliasSummary) ExpAliasSummaryKind()  {}

////////////////////////////////////////////////////////////////////////////////////////////////////

type InfixExpAliasSummary struct {
	Lhs   InfixOperatorFormPattern
	Rhs   ast.ExpressionKind
	Scope *ast.Scope
}

type PrefixExpAliasSummary struct {
	Lhs   PrefixOperatorFormPattern
	Rhs   ast.ExpressionKind
	Scope *ast.Scope
}

type PostfixExpAliasSummary struct {
	Lhs   PostfixOperatorFormPattern
	Rhs   ast.ExpressionKind
	Scope *ast.Scope
}

type FunctionExpAliasSummary struct {
	Lhs   FunctionFormPattern
	Rhs   ast.ExpressionKind
	Scope *ast.Scope
}

type CommandExpAliasSummary struct {
	Lhs   CommandPattern
	Rhs   ast.ExpressionKind
	Scope *ast.Scope
}

type MemberNameExpAliasSummary struct {
	Lhs   ChainExpressionPattern
	Rhs   ast.ExpressionKind
	Scope *ast.Scope
}

type MemberFunctionExpAliasSummary struct {
	Lhs   ChainExpressionPattern
	Rhs   ast.ExpressionKind
	Scope *ast.Scope
}

type MemberInfixExpAliasSummary struct {
	Lhs   InfixOperatorFormPattern
	Rhs   ast.ExpressionKind
	Scope *ast.Scope
}

type MemberPrefixExpAliasSummary struct {
	Lhs   PrefixOperatorFormPattern
	Rhs   ast.ExpressionKind
	Scope *ast.Scope
}

type MemberPostfixExpAliasSummary struct {
	Lhs   PostfixOperatorFormPattern
	Rhs   ast.ExpressionKind
	Scope *ast.Scope
}

////////////////////////////////////////////////////////////////////////////////////////////////////

func ToInfixExpAliasSummary(node ast.ExpressionColonArrowItem) InfixExpAliasSummary {
	return InfixExpAliasSummary{}
}

////////////////////////////////////////////////////////////////////////////////////////////////////

func ExpandAliasInline(node ast.MlgNodeKind, aliasSummary AliasSummaryKind) bool {
	return false
}

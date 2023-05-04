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
	AliasSummaryType()
}

func (*SpecAliasSummary) AliasSummaryType() {}

func (*InfixExpAliasSummary) AliasSummaryType()          {}
func (*PrefixExpAliasSummary) AliasSummaryType()         {}
func (*PostfixExpAliasSummary) AliasSummaryType()        {}
func (*FunctionExpAliasSummary) AliasSummaryType()       {}
func (*CommandExpAliasSummary) AliasSummaryType()        {}
func (*MemberNameExpAliasSummary) AliasSummaryType()     {}
func (*MemberFunctionExpAliasSummary) AliasSummaryType() {}
func (*MemberInfixExpAliasSummary) AliasSummaryType()    {}
func (*MemberPrefixExpAliasSummary) AliasSummaryType()   {}
func (*MemberPostfixExpAliasSummary) AliasSummaryType()  {}

////////////////////////////////////////////////////////////////////////////////////////////////////

type SpecAliasSummaryRhsKind interface {
	SpecAliasSummaryRhsType()
}

func (*IsConstraint) SpecAliasSummaryRhsType()   {}
func (*SpecConstraint) SpecAliasSummaryRhsType() {}

type SpecAliasSummary struct {
	Lhs SpecAliasPattern
	Rhs SpecAliasSummaryRhsKind
}

////////////////////////////////////////////////////////////////////////////////////////////////////

type ExpAliasSummaryKind interface {
	AliasSummaryKind
	ExpAliasSummaryType()
}

func (*InfixExpAliasSummary) ExpAliasSummaryType()          {}
func (*PrefixExpAliasSummary) ExpAliasSummaryType()         {}
func (*PostfixExpAliasSummary) ExpAliasSummaryType()        {}
func (*FunctionExpAliasSummary) ExpAliasSummaryType()       {}
func (*CommandExpAliasSummary) ExpAliasSummaryType()        {}
func (*MemberNameExpAliasSummary) ExpAliasSummaryType()     {}
func (*MemberFunctionExpAliasSummary) ExpAliasSummaryType() {}
func (*MemberInfixExpAliasSummary) ExpAliasSummaryType()    {}
func (*MemberPrefixExpAliasSummary) ExpAliasSummaryType()   {}
func (*MemberPostfixExpAliasSummary) ExpAliasSummaryType()  {}

////////////////////////////////////////////////////////////////////////////////////////////////////

type InfixExpAliasSummary struct {
	Lhs   InfixOperatorFormPattern
	Rhs   ast.ExpressionKind
	Scope ast.IScope
}

type PrefixExpAliasSummary struct {
	Lhs   PrefixOperatorFormPattern
	Rhs   ast.ExpressionKind
	Scope ast.IScope
}

type PostfixExpAliasSummary struct {
	Lhs   PostfixOperatorFormPattern
	Rhs   ast.ExpressionKind
	Scope ast.IScope
}

type FunctionExpAliasSummary struct {
	Lhs   FunctionFormPattern
	Rhs   ast.ExpressionKind
	Scope ast.IScope
}

type CommandExpAliasSummary struct {
	Lhs   CommandPattern
	Rhs   ast.ExpressionKind
	Scope ast.IScope
}

type MemberNameExpAliasSummary struct {
	Lhs   ChainExpressionPattern
	Rhs   ast.ExpressionKind
	Scope ast.IScope
}

type MemberFunctionExpAliasSummary struct {
	Lhs   ChainExpressionPattern
	Rhs   ast.ExpressionKind
	Scope ast.IScope
}

type MemberInfixExpAliasSummary struct {
	Lhs   InfixOperatorFormPattern
	Rhs   ast.ExpressionKind
	Scope ast.IScope
}

type MemberPrefixExpAliasSummary struct {
	Lhs   PrefixOperatorFormPattern
	Rhs   ast.ExpressionKind
	Scope ast.IScope
}

type MemberPostfixExpAliasSummary struct {
	Lhs   PostfixOperatorFormPattern
	Rhs   ast.ExpressionKind
	Scope ast.IScope
}

////////////////////////////////////////////////////////////////////////////////////////////////////

func ToInfixExpAliasSummary(node ast.ExpressionColonArrowItem) InfixExpAliasSummary {
	return InfixExpAliasSummary{}
}

////////////////////////////////////////////////////////////////////////////////////////////////////

func ExpandAliasInline(node ast.MlgNodeKind, aliasSummary AliasSummaryKind) bool {
	return false
}

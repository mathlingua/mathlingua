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

// func (*IsConstraint) SpecAliasSummaryRhsKind()   {}
// func (*SpecConstraint) SpecAliasSummaryRhsKind() {}

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
	Rhs   ExpressionKind
	Scope *Scope
}

type PrefixExpAliasSummary struct {
	Lhs   PrefixOperatorFormPattern
	Rhs   ExpressionKind
	Scope *Scope
}

type PostfixExpAliasSummary struct {
	Lhs   PostfixOperatorFormPattern
	Rhs   ExpressionKind
	Scope *Scope
}

type FunctionExpAliasSummary struct {
	Lhs   FunctionFormPattern
	Rhs   ExpressionKind
	Scope *Scope
}

type CommandExpAliasSummary struct {
	Lhs   CommandPattern
	Rhs   ExpressionKind
	Scope *Scope
}

type MemberNameExpAliasSummary struct {
	Lhs   ChainExpressionPattern
	Rhs   ExpressionKind
	Scope *Scope
}

type MemberFunctionExpAliasSummary struct {
	Lhs   ChainExpressionPattern
	Rhs   ExpressionKind
	Scope *Scope
}

type MemberInfixExpAliasSummary struct {
	Lhs   InfixOperatorFormPattern
	Rhs   ExpressionKind
	Scope *Scope
}

type MemberPrefixExpAliasSummary struct {
	Lhs   PrefixOperatorFormPattern
	Rhs   ExpressionKind
	Scope *Scope
}

type MemberPostfixExpAliasSummary struct {
	Lhs   PostfixOperatorFormPattern
	Rhs   ExpressionKind
	Scope *Scope
}

////////////////////////////////////////////////////////////////////////////////////////////////////

func ToInfixExpAliasSummary(node ExpressionColonArrowItem) InfixExpAliasSummary {
	return InfixExpAliasSummary{}
}

////////////////////////////////////////////////////////////////////////////////////////////////////

func ExpandAliasInline(node MlgNodeKind, aliasSummary AliasSummaryKind) bool {
	return false
}

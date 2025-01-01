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

type SpecAliasSummary struct {
	Lhs SpecAliasPattern
	Rhs SpecAliasSummaryRhsKind
}

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

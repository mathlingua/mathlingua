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

type AliasSummaryType interface {
	AliasSummaryType()
}

func (SpecAliasSummary) AliasSummaryType() {}

func (InfixExpAliasSummary) AliasSummaryType()          {}
func (PrefixExpAliasSummary) AliasSummaryType()         {}
func (PostfixExpAliasSummary) AliasSummaryType()        {}
func (FunctionExpAliasSummary) AliasSummaryType()       {}
func (CommandExpAliasSummary) AliasSummaryType()        {}
func (MemberNameExpAliasSummary) AliasSummaryType()     {}
func (MemberFunctionExpAliasSummary) AliasSummaryType() {}
func (MemberInfixExpAliasSummary) AliasSummaryType()    {}
func (MemberPrefixExpAliasSummary) AliasSummaryType()   {}
func (MemberPostfixExpAliasSummary) AliasSummaryType()  {}

////////////////////////////////////////////////////////////////////////////////////////////////////

type SpecAliasSummaryRhsType interface {
	SpecAliasSummaryRhsType()
}

func (IsConstraint) SpecAliasSummaryRhsType()   {}
func (SpecConstraint) SpecAliasSummaryRhsType() {}

type SpecAliasSummary struct {
	Lhs SpecAliasPattern
	Rhs SpecAliasSummaryRhsType
}

////////////////////////////////////////////////////////////////////////////////////////////////////

type ExpAliasSummaryType interface {
	AliasSummaryType
	ExpAliasSummaryType()
}

func (InfixExpAliasSummary) ExpAliasSummaryType()          {}
func (PrefixExpAliasSummary) ExpAliasSummaryType()         {}
func (PostfixExpAliasSummary) ExpAliasSummaryType()        {}
func (FunctionExpAliasSummary) ExpAliasSummaryType()       {}
func (CommandExpAliasSummary) ExpAliasSummaryType()        {}
func (MemberNameExpAliasSummary) ExpAliasSummaryType()     {}
func (MemberFunctionExpAliasSummary) ExpAliasSummaryType() {}
func (MemberInfixExpAliasSummary) ExpAliasSummaryType()    {}
func (MemberPrefixExpAliasSummary) ExpAliasSummaryType()   {}
func (MemberPostfixExpAliasSummary) ExpAliasSummaryType()  {}

////////////////////////////////////////////////////////////////////////////////////////////////////

type InfixExpAliasSummary struct {
	Lhs   InfixOperatorFormPattern
	Rhs   ast.ExpressionType
	Scope *ast.Scope
}

type PrefixExpAliasSummary struct {
	Lhs   PrefixOperatorFormPattern
	Rhs   ast.ExpressionType
	Scope *ast.Scope
}

type PostfixExpAliasSummary struct {
	Lhs   PostfixOperatorFormPattern
	Rhs   ast.ExpressionType
	Scope *ast.Scope
}

type FunctionExpAliasSummary struct {
	Lhs   FunctionFormPattern
	Rhs   ast.ExpressionType
	Scope *ast.Scope
}

type CommandExpAliasSummary struct {
	Lsh   CommandPattern
	Rhs   ast.ExpressionType
	Scope *ast.Scope
}

type MemberNameExpAliasSummary struct {
	Lsh   MemberNamePattern
	Rhs   ast.ExpressionType
	Scope *ast.Scope
}

type MemberFunctionExpAliasSummary struct {
	Lsh   MemberFunctionPattern
	Rhs   ast.ExpressionType
	Scope *ast.Scope
}

type MemberInfixExpAliasSummary struct {
	Lsh   MemberInfixPattern
	Rhs   ast.ExpressionType
	Scope *ast.Scope
}

type MemberPrefixExpAliasSummary struct {
	Lhs   MemberPrefixPattern
	Rhs   ast.ExpressionType
	Scope *ast.Scope
}

type MemberPostfixExpAliasSummary struct {
	Lsh   MemberPostfixPattern
	Rhs   ast.ExpressionType
	Scope *ast.Scope
}

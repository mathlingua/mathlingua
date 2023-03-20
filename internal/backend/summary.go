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

type DescribesSummary struct {
	DefScope    *ast.Scope
	Input       IStaticPattern
	Output      IStaticPattern
	Usings      []IStaticPattern
	When        []IConstraint
	Extends     []IConstraint
	ExpAliases  []IExpAliasSummary
	SpecAliases []SpecAliasSummary
	Written     []WrittenSummary
	Called      []CalledSummary
}

type DefinesSummary struct {
	DefScope    *ast.Scope
	Input       IStaticPattern
	Output      IStaticPattern
	Usings      []IStaticPattern
	When        []IConstraint
	Means       []IConstraint
	ExpAliases  []IExpAliasSummary
	SpecAliases []SpecAliasSummary
	Written     []WrittenSummary
	Called      []CalledSummary
}

type StatesSummary struct {
	DefScope    *ast.Scope
	Input       IStaticPattern
	Output      IStaticPattern
	Usings      []IStaticPattern
	ExpAliases  []IExpAliasSummary
	SpecAliases []SpecAliasSummary
	Written     []WrittenSummary
	Called      []CalledSummary
}

/////////////////////////////////////////////////////////////////////////////////////////////////////

type ISpecAliasSummaryRhs interface {
	ISpecAliasSummaryRhs()
}

func (IsConstraint) ISpecAliasSummaryRhs()   {}
func (SpecConstraint) ISpecAliasSummaryRhs() {}

type SpecAliasSummary struct {
	Lhs SpecStaticPattern
	Rhs ISpecAliasSummaryRhs
}

////////////////////////////////////////////////////////////////////////////////////////////////////

type IExpAliasSummary interface {
	IExpAliasSummary()
}

func (InfixExpAliasSummary) IExpAliasSummary()          {}
func (PrefixExpAliasSummary) IExpAliasSummary()         {}
func (PostfixExpAliasSummary) IExpAliasSummary()        {}
func (FunctionExpAliasSummary) IExpAliasSummary()       {}
func (CommandExpAliasSummary) IExpAliasSummary()        {}
func (MemberNameExpAliasSummary) IExpAliasSummary()     {}
func (MemberFunctionExpAliasSummary) IExpAliasSummary() {}
func (MemberInfixExpAliasSummary) IExpAliasSummary()    {}
func (MemberPrefixExpAliasSummary) IExpAliasSummary()   {}
func (MemberPostfixExpAliasSummary) IExpAliasSummary()  {}

////////////////////////////////////////////////////////////////////////////////////////////////////

type InfixExpAliasSummary struct {
	Lhs InfixStaticPattern
	Rhs ast.ExpressionType
}

type PrefixExpAliasSummary struct {
	Lhs PrefixStaticPattern
	Rhs ast.ExpressionType
}

type PostfixExpAliasSummary struct {
	Lhs PostfixStaticPattern
	Rhs ast.ExpressionType
}

type FunctionExpAliasSummary struct {
	Lhs FunctionStaticPattern
	Rhs ast.ExpressionType
}

type CommandExpAliasSummary struct {
	Lsh CommandStaticPattern
	Rhs ast.ExpressionType
}

type MemberNameExpAliasSummary struct {
	Lsh MemberNameStaticPattern
	Rhs ast.ExpressionType
}

type MemberFunctionExpAliasSummary struct {
	Lsh MemberFunctionStaticPattern
	Rhs ast.ExpressionType
}

type MemberInfixExpAliasSummary struct {
	Lsh MemberInfixStaticPattern
	Rhs ast.ExpressionType
}

type MemberPrefixExpAliasSummary struct {
	Lhs MemberPrefixStaticPattern
	Rhs ast.ExpressionType
}

type MemberPostfixExpAliasSummary struct {
	Lsh MemberPostfixStaticPattern
	Rhs ast.ExpressionType
}

////////////////////////////////////////////////////////////////////////////////////////////////////

type CalledSummary struct {
	From   IStaticPattern
	Called string
}

////////////////////////////////////////////////////////////////////////////////////////////////////

type WrittenSummary struct {
	From    IStaticPattern
	Written string
}

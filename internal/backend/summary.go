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

type ISummaryType interface {
	ISummaryType()
}

func (DescribesSummary) ISummaryType() {}
func (DefinesSummary) ISummaryType()   {}
func (StatesSummary) ISummaryType()    {}

////////////////////////////////////////////////////////////////////////////////////////////////////

type DescribesSummary struct {
	DefScope    *ast.Scope
	Input       StaticPatternType
	Output      StaticPatternType
	Usings      []StaticPatternType
	When        []ConstraintType
	Extends     []ConstraintType
	ExpAliases  []ExpAliasSummaryType
	SpecAliases []SpecAliasSummary
	Written     []WrittenSummary
	Called      []CalledSummary
}

type DefinesSummary struct {
	DefScope    *ast.Scope
	Input       StaticPatternType
	Output      StaticPatternType
	Usings      []StaticPatternType
	When        []ConstraintType
	Means       []ConstraintType
	ExpAliases  []ExpAliasSummaryType
	SpecAliases []SpecAliasSummary
	Written     []WrittenSummary
	Called      []CalledSummary
}

type StatesSummary struct {
	DefScope    *ast.Scope
	Input       StaticPatternType
	Output      StaticPatternType
	Usings      []StaticPatternType
	ExpAliases  []ExpAliasSummaryType
	SpecAliases []SpecAliasSummary
	Written     []WrittenSummary
	Called      []CalledSummary
}

/////////////////////////////////////////////////////////////////////////////////////////////////////

type SpecAliasSummaryRhsType interface {
	SpecAliasSummaryRhsType()
}

func (IsConstraint) SpecAliasSummaryRhsType()   {}
func (SpecConstraint) SpecAliasSummaryRhsType() {}

type SpecAliasSummary struct {
	Lhs SpecStaticPattern
	Rhs SpecAliasSummaryRhsType
}

////////////////////////////////////////////////////////////////////////////////////////////////////

type ExpAliasSummaryType interface {
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
	From   StaticPatternType
	Called string
}

////////////////////////////////////////////////////////////////////////////////////////////////////

type WrittenSummary struct {
	From    StaticPatternType
	Written string
}

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

type SummaryType interface {
	SummaryType()
	GetExpAliasSummaries() []ExpAliasSummaryType
}

func (DescribesSummary) SummaryType() {}
func (DefinesSummary) SummaryType()   {}
func (StatesSummary) SummaryType()    {}

func (s *DescribesSummary) GetExpAliasSummaries() []ExpAliasSummaryType {
	return s.ExpAliases
}

func (s *DefinesSummary) GetExpAliasSummaries() []ExpAliasSummaryType {
	return s.ExpAliases
}

func (s *StatesSummary) GetExpAliasSummaries() []ExpAliasSummaryType {
	return s.ExpAliases
}

////////////////////////////////////////////////////////////////////////////////////////////////////

type DescribesSummary struct {
	DefScope    *ast.Scope
	Input       PatternType
	Output      PatternType
	Usings      []PatternType
	When        []ConstraintType
	Extends     []ConstraintType
	ExpAliases  []ExpAliasSummaryType
	SpecAliases []SpecAliasSummary
	Written     []WrittenSummary
	Called      []CalledSummary
}

type DefinesSummary struct {
	DefScope    *ast.Scope
	Input       PatternType
	Output      PatternType
	Usings      []PatternType
	When        []ConstraintType
	Means       []ConstraintType
	ExpAliases  []ExpAliasSummaryType
	SpecAliases []SpecAliasSummary
	Written     []WrittenSummary
	Called      []CalledSummary
}

type StatesSummary struct {
	DefScope    *ast.Scope
	Input       PatternType
	Output      PatternType
	Usings      []PatternType
	ExpAliases  []ExpAliasSummaryType
	SpecAliases []SpecAliasSummary
	Written     []WrittenSummary
	Called      []CalledSummary
}

////////////////////////////////////////////////////////////////////////////////////////////////////

type CalledSummary struct {
	From   PatternType
	Called string
}

////////////////////////////////////////////////////////////////////////////////////////////////////

type WrittenSummary struct {
	From    PatternType
	Written string
}

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

import (
	"fmt"
	"mathlingua/internal/ast"
)

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
	Writing     []WritingSummary
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
	Writing     []WritingSummary
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
	Writing     []WritingSummary
	Called      []CalledSummary
}

////////////////////////////////////////////////////////////////////////////////////////////////////

func GetResolvedInput(summary SummaryType) (PatternType, bool) {
	switch s := summary.(type) {
	case *DescribesSummary:
		return s.Input, true
	case *DefinesSummary:
		return s.Input, true
	case *StatesSummary:
		return s.Input, true
	default:
		return nil, false
	}
}

func GetResolvedWritten(summary SummaryType) (string, bool) {
	var called *string
	var written *string
	switch s := summary.(type) {
	case *DescribesSummary:
		called = getSingleCalled(s.Called)
		written = getSingleWritten(s.Written)
	case *DefinesSummary:
		called = getSingleCalled(s.Called)
		written = getSingleWritten(s.Written)
	case *StatesSummary:
		called = getSingleCalled(s.Called)
		written = getSingleWritten(s.Written)
	}

	if written != nil {
		return *written, true
	}

	if called != nil {
		return fmt.Sprintf("\\textrm{%s}", *called), true
	}

	return "", false
}

func getSingleCalled(called []CalledSummary) *string {
	if len(called) == 0 || len(called[0].Called) == 0 {
		return nil
	}
	text := called[0].Called
	return &text
}

func getSingleWritten(written []WrittenSummary) *string {
	if len(written) == 0 || len(written[0].Written) == 0 {
		return nil
	}
	text := written[0].Written
	return &text
}

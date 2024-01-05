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
	"mathlingua/internal/ast"
)

type SummaryKind interface {
	SummaryKind()
	GetExpAliasSummaries() []ExpAliasSummaryKind
}

func (*DescribesSummary) SummaryKind()  {}
func (*DefinesSummary) SummaryKind()    {}
func (*StatesSummary) SummaryKind()     {}
func (*CapturesSummary) SummaryKind()   {}
func (*AxiomSummary) SummaryKind()      {}
func (*ConjectureSummary) SummaryKind() {}
func (*TheoremSummary) SummaryKind()    {}
func (*CorollarySummary) SummaryKind()  {}
func (*LemmaSummary) SummaryKind()      {}

func (s *DescribesSummary) GetExpAliasSummaries() []ExpAliasSummaryKind {
	return s.ExpAliases
}

func (s *DefinesSummary) GetExpAliasSummaries() []ExpAliasSummaryKind {
	return s.ExpAliases
}

func (s *StatesSummary) GetExpAliasSummaries() []ExpAliasSummaryKind {
	return s.ExpAliases
}

func (s *CapturesSummary) GetExpAliasSummaries() []ExpAliasSummaryKind {
	return []ExpAliasSummaryKind{}
}

func (s *AxiomSummary) GetExpAliasSummaries() []ExpAliasSummaryKind {
	return []ExpAliasSummaryKind{}
}

func (s *ConjectureSummary) GetExpAliasSummaries() []ExpAliasSummaryKind {
	return []ExpAliasSummaryKind{}
}

func (s *TheoremSummary) GetExpAliasSummaries() []ExpAliasSummaryKind {
	return []ExpAliasSummaryKind{}
}

func (s *CorollarySummary) GetExpAliasSummaries() []ExpAliasSummaryKind {
	return []ExpAliasSummaryKind{}
}

func (s *LemmaSummary) GetExpAliasSummaries() []ExpAliasSummaryKind {
	return []ExpAliasSummaryKind{}
}

////////////////////////////////////////////////////////////////////////////////////////////////////

type DescribesSummary struct {
	DefScope    *ast.Scope
	Input       PatternKind
	Output      PatternKind
	Usings      []PatternKind
	When        []ConstraintKind
	Extends     []ConstraintKind
	ExpAliases  []ExpAliasSummaryKind
	SpecAliases []SpecAliasSummary
	Written     []WrittenSummary
	Writing     []WritingSummary
	Called      []CalledSummary
}

type DefinesSummary struct {
	DefScope    *ast.Scope
	Input       PatternKind
	Output      PatternKind
	Usings      []PatternKind
	When        []ConstraintKind
	Means       []ConstraintKind
	ExpAliases  []ExpAliasSummaryKind
	SpecAliases []SpecAliasSummary
	Written     []WrittenSummary
	Writing     []WritingSummary
	Called      []CalledSummary
}

type StatesSummary struct {
	DefScope    *ast.Scope
	Input       PatternKind
	Output      PatternKind
	Usings      []PatternKind
	ExpAliases  []ExpAliasSummaryKind
	SpecAliases []SpecAliasSummary
	Written     []WrittenSummary
	Writing     []WritingSummary
	Called      []CalledSummary
}

type CapturesSummary struct {
	Input   PatternKind
	Written []WrittenSummary
	Called  []CalledSummary
}

type AxiomSummary struct {
	Input  *PatternKind
	Called []CalledSummary
}

type ConjectureSummary struct {
	Input  *PatternKind
	Called []CalledSummary
}

type TheoremSummary struct {
	Input  *PatternKind
	Called []CalledSummary
}

type CorollarySummary struct {
	Input  *PatternKind
	Called []CalledSummary
}

type LemmaSummary struct {
	Input  *PatternKind
	Called []CalledSummary
}

////////////////////////////////////////////////////////////////////////////////////////////////////

func GetResolvedInput(summary SummaryKind) (PatternKind, bool) {
	switch s := summary.(type) {
	case *DescribesSummary:
		return s.Input, true
	case *DefinesSummary:
		return s.Input, true
	case *StatesSummary:
		return s.Input, true
	case *CapturesSummary:
		return s.Input, true
	case *AxiomSummary:
		if s.Input == nil {
			return nil, false
		}
		return *s.Input, true
	case *ConjectureSummary:
		if s.Input == nil {
			return nil, false
		}
		return *s.Input, true
	case *TheoremSummary:
		if s.Input == nil {
			return nil, false
		}
		return *s.Input, true
	default:
		return nil, false
	}
}

func GetResolvedWritten(summary SummaryKind) ([]TextItemKind, bool) {
	var called *[]TextItemKind
	var written *[]TextItemKind
	switch s := summary.(type) {
	case *DescribesSummary:
		if s != nil {
			called = getSingleCalled(s.Called)
			written = getSingleWritten(s.Written)
		}
	case *DefinesSummary:
		if s != nil {
			called = getSingleCalled(s.Called)
			written = getSingleWritten(s.Written)
		}
	case *StatesSummary:
		if s != nil {
			called = getSingleCalled(s.Called)
			written = getSingleWritten(s.Written)
		}
	case *CapturesSummary:
		if s != nil {
			called = getSingleCalled(s.Called)
			written = getSingleWritten(s.Written)
		}
	case *AxiomSummary:
		if s != nil {
			called = getSingleCalled(s.Called)
		}
	case *ConjectureSummary:
		if s != nil {
			called = getSingleCalled(s.Called)
		}
	case *TheoremSummary:
		if s != nil {
			called = getSingleCalled(s.Called)
		}
	}

	if written != nil {
		return *written, true
	}

	if called != nil {
		result := make([]TextItemKind, 0)
		result = append(result, &StringItem{
			Text: "\\textrm{",
		})
		result = append(result, *called...)
		result = append(result, &StringItem{
			Text: "}",
		})
		return result, true
	}

	return nil, false
}

////////////////////////////////////////////////////////////////////////////////////////////////////

func getSingleCalled(called []CalledSummary) *[]TextItemKind {
	if len(called) == 0 {
		return nil
	}
	text := called[0].ParsedCalled
	return &text
}

func getSingleWritten(written []WrittenSummary) *[]TextItemKind {
	if len(written) == 0 {
		return nil
	}
	text := written[0].ParsedWritten
	return &text
}

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

// A pattern describes the shape of inputs to a Defines, Describes, States
// provides, expression alias, or spec alias.
type PatternType interface {
	PatternType()
}

func (NameColonEqualsPatternPattern) PatternType()  {}
func (FunctionColonEqualsNamePattern) PatternType() {}
func (NameFormPattern) PatternType()                {}
func (FunctionFormPattern) PatternType()            {}
func (TupleFormPattern) PatternType()               {}
func (ConditionalSetFormPattern) PatternType()      {}
func (InfixOperatorFormPattern) PatternType()       {}
func (PrefixOperatorFormPattern) PatternType()      {}
func (PostfixOperatorFormPattern) PatternType()     {}
func (CommandPattern) PatternType()                 {}
func (MemberNamePattern) PatternType()              {}
func (MemberFunctionPattern) PatternType()          {}
func (MemberInfixPattern) PatternType()             {}
func (MemberPrefixPattern) PatternType()            {}
func (MemberPostfixPattern) PatternType()           {}
func (SpecAliasPattern) PatternType()               {}

type NameColonEqualsPatternPattern struct {
	Lhs string
	Rhs PatternType
}

type FunctionColonEqualsNamePattern struct {
	Lhs FunctionFormPattern
	Rhs string
}

type FormPatternType interface {
	PatternType
	FormPatternType()
}

func (NameFormPattern) FormPatternType()            {}
func (FunctionFormPattern) FormPatternType()        {}
func (TupleFormPattern) FormPatternType()           {}
func (ConditionalSetFormPattern) FormPatternType()  {}
func (InfixOperatorFormPattern) FormPatternType()   {}
func (PrefixOperatorFormPattern) FormPatternType()  {}
func (PostfixOperatorFormPattern) FormPatternType() {}

type NameFormPattern struct {
	Text            string
	IsStropped      bool
	HasQuestionMark bool
	VarArg          VarArgPatternData
}

type FunctionFormPattern struct {
	Target NameFormPattern
	Params []NameFormPattern
	VarArg VarArgPatternData
}

type InfixOperatorFormPattern struct {
	Operator NameFormPattern
	Lhs      FormPatternType
	Rhs      FormPatternType
}

type PrefixOperatorFormPattern struct {
	Operator NameFormPattern
	Param    FormPatternType
}

type PostfixOperatorFormPattern struct {
	Operator NameFormPattern
	Param    FormPatternType
}

type TupleFormPattern struct {
	Params []FormPatternType
	VarArg VarArgPatternData
}

type ConditionalSetFormPattern struct {
	Target FormPatternType
	VarArg VarArgPatternData
}

type VarArgPatternData struct {
	IsVarArg     bool
	VarArgNames  []string
	VarArgBounds []string
}

type CommandPattern struct {
	Signatures  string
	CurlyArgs   []PatternType
	ParenArgs   []PatternType
	NamedGroups []NamedGroupPattern
}

type NamedGroupPattern struct {
	Name string
	Args []PatternType
}

type MemberNamePattern struct {
	Target string
	Member NameFormPattern
}

type MemberFunctionPattern struct {
	Target string
	Member FunctionFormPattern
}

type MemberInfixPattern struct {
	Target string
	Member InfixOperatorFormPattern
}

type MemberPrefixPattern struct {
	Target string
	Member PrefixOperatorFormPattern
}

type MemberPostfixPattern struct {
	Target string
	Member PostfixOperatorFormPattern
}

type SpecAliasPattern struct {
	Lhs  PatternType
	Name string
	Rhs  PatternType
}

////////////////////////////////////////////////////////////////////////////////////////////////////

func ToPattern(exp ast.ExpressionType) PatternType {
	switch n := exp.(type) {
	case ast.StructuralFormType:
		return ToFormPattern(n)
	default:
		return nil
	}
}

func ToFormPattern(item ast.StructuralFormType) FormPatternType {
	switch n := item.(type) {
	case *ast.NameForm:
		return ToNameFormPattern(*n)
	case *ast.FunctionForm:
		return ToFunctionFormPattern(*n)
	case *ast.TupleForm:
		return ToTupleFormPattern(*n)
	case *ast.ConditionalSetForm:
		return ToConditionalSetFormPattern(*n)
	case *ast.InfixOperatorForm:
		return ToInfixOperatorFormPattern(*n)
	case *ast.PostfixOperatorForm:
		return ToPostfixOperatorFormPattern(*n)
	case *ast.PrefixOperatorForm:
		return ToPrefixOperatorFormPattern(*n)
	default:
		return nil
	}
}

func toFormPatterns(items []ast.StructuralFormType) []FormPatternType {
	patterns := make([]FormPatternType, 0)
	for _, item := range items {
		patterns = append(patterns, ToFormPattern(item))
	}
	return patterns
}

func ToVarArgPatternData(data ast.VarArgData) VarArgPatternData {
	varArgNames := make([]string, 0)
	varArgNames = append(varArgNames, data.VarArgNames...)

	varArgBounds := make([]string, 0)
	varArgBounds = append(varArgBounds, data.VarArgBounds...)

	return VarArgPatternData{
		IsVarArg:     data.IsVarArg,
		VarArgNames:  varArgNames,
		VarArgBounds: varArgBounds,
	}
}

func ToNameFormPattern(form ast.NameForm) NameFormPattern {
	return NameFormPattern{
		Text:            form.Text,
		IsStropped:      form.IsStropped,
		HasQuestionMark: form.HasQuestionMark,
		VarArg:          ToVarArgPatternData(form.VarArg),
	}
}

func toNameFormPatterns(params []ast.NameForm) []NameFormPattern {
	result := make([]NameFormPattern, 0)
	for _, p := range params {
		result = append(result, ToNameFormPattern(p))
	}
	return result
}

func ToFunctionFormPattern(form ast.FunctionForm) FunctionFormPattern {
	return FunctionFormPattern{
		Target: ToNameFormPattern(form.Target),
		Params: toNameFormPatterns(form.Params),
		VarArg: ToVarArgPatternData(form.VarArg),
	}
}

func ToTupleFormPattern(form ast.TupleForm) TupleFormPattern {
	return TupleFormPattern{
		Params: toFormPatterns(form.Params),
		VarArg: ToVarArgPatternData(form.VarArg),
	}
}

func ToConditionalSetFormPattern(form ast.ConditionalSetForm) ConditionalSetFormPattern {
	return ConditionalSetFormPattern{
		Target: ToFormPattern(form.Target),
		VarArg: ToVarArgPatternData(form.VarArg),
	}
}

func ToConditionalSetFormPatternFromId(form ast.ConditionalSetIdForm) ConditionalSetFormPattern {
	return ConditionalSetFormPattern{
		Target: ToFormPattern(form.Target),
		VarArg: ToVarArgPatternData(form.Condition.VarArg),
	}
}

func ToInfixOperatorFormPattern(form ast.InfixOperatorForm) InfixOperatorFormPattern {
	return InfixOperatorFormPattern{
		Operator: ToNameFormPattern(form.Operator),
		Lhs:      ToNameFormPattern(form.Lhs),
		Rhs:      ToNameFormPattern(form.Rhs),
	}
}

func toNameFormPatternFromText(text string) NameFormPattern {
	return NameFormPattern{
		Text:            text,
		IsStropped:      false,
		HasQuestionMark: false,
		VarArg: VarArgPatternData{
			IsVarArg:     false,
			VarArgNames:  make([]string, 0),
			VarArgBounds: make([]string, 0),
		},
	}
}

func ToInfixOperatorFormPatternFromId(form ast.InfixOperatorId) InfixOperatorFormPattern {
	return InfixOperatorFormPattern{
		Operator: toNameFormPatternFromText(form.Operator.Text),
		Lhs:      ToFormPattern(form.Lhs),
	}
}

func ToPrefixOperatorFormPattern(form ast.PrefixOperatorForm) PrefixOperatorFormPattern {
	return PrefixOperatorFormPattern{
		Operator: ToNameFormPattern(form.Operator),
		Param:    ToNameFormPattern(form.Param),
	}
}

func ToPrefixOperatorFormPatternFromId(form ast.PrefixOperatorId) PrefixOperatorFormPattern {
	return PrefixOperatorFormPattern{
		Operator: toNameFormPatternFromText(form.Operator.Text),
		Param:    ToFormPattern(form.Param),
	}
}

func ToPostfixOperatorFormPattern(form ast.PostfixOperatorForm) PostfixOperatorFormPattern {
	return PostfixOperatorFormPattern{
		Operator: ToNameFormPattern(form.Operator),
		Param:    ToNameFormPattern(form.Param),
	}
}

func ToPostfixOperatorFormPatternFromId(form ast.PostfixOperatorId) PostfixOperatorFormPattern {
	return PostfixOperatorFormPattern{
		Operator: toNameFormPatternFromText(form.Operator.Text),
		Param:    ToFormPattern(form.Param),
	}
}

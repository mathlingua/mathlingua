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
	"mathlingua/internal/mlglib"
)

// A pattern describes the shape of inputs to a Defines, Describes, States
// provides, expression alias, or spec alias.
type PatternKind interface {
	PatternKind()
	GetVarArgData() VarArgPatternData
}

func (*NameFormPattern) PatternKind()                 {}
func (*FunctionFormPattern) PatternKind()             {}
func (*TupleFormPattern) PatternKind()                {}
func (*ConditionalSetExpressionPattern) PatternKind() {}
func (*ConditionalSetFormPattern) PatternKind()       {}
func (*ConditionaSetIdFormPattern) PatternKind()      {}
func (*FunctionLiteralFormPattern) PatternKind()      {}
func (*InfixOperatorFormPattern) PatternKind()        {}
func (*PrefixOperatorFormPattern) PatternKind()       {}
func (*PostfixOperatorFormPattern) PatternKind()      {}
func (*OrdinalPattern) PatternKind()                  {}

func (*StructuralColonEqualsPattern) PatternKind() {}
func (*InfixCommandOperatorPattern) PatternKind()  {}
func (*InfixCommandPattern) PatternKind()          {}
func (*CommandPattern) PatternKind()               {}
func (*NamedGroupPattern) PatternKind()            {}
func (*ChainExpressionPattern) PatternKind()       {}

func (*SpecAliasPattern) PatternKind() {}
func (*AliasPattern) PatternKind()     {}

type StructuralColonEqualsPattern struct {
	Lhs PatternKind
	Rhs PatternKind
}

type FormPatternKind interface {
	PatternKind
	FormPatternKind()
}

func (*NameFormPattern) FormPatternKind()              {}
func (*FunctionFormPattern) FormPatternKind()          {}
func (*TupleFormPattern) FormPatternKind()             {}
func (*ConditionalSetFormPattern) FormPatternKind()    {}
func (*ConditionaSetIdFormPattern) FormPatternKind()   {}
func (*FunctionLiteralFormPattern) FormPatternKind()   {}
func (*InfixOperatorFormPattern) FormPatternKind()     {}
func (*PrefixOperatorFormPattern) FormPatternKind()    {}
func (*PostfixOperatorFormPattern) FormPatternKind()   {}
func (*StructuralColonEqualsPattern) FormPatternKind() {}

type NameFormPattern struct {
	Text            string
	IsStropped      bool
	HasQuestionMark bool
	VarArg          VarArgPatternData
}

type FunctionFormPattern struct {
	Target NameFormPattern
	Params []FormPatternKind
	VarArg VarArgPatternData
}

type LiteralFormPatternKind interface {
	LiteralFormPatternKind()
}

func (*NameFormPattern) LiteralFormPatternKind()            {}
func (*FunctionFormPattern) LiteralFormPatternKind()        {}
func (*TupleFormPattern) LiteralFormPatternKind()           {}
func (*ConditionalSetFormPattern) LiteralFormPatternKind()  {}
func (*ConditionaSetIdFormPattern) LiteralFormPatternKind() {}
func (*FunctionLiteralFormPattern) LiteralFormPatternKind() {}

type OrdinalPattern struct {
	Target LiteralFormPatternKind
	Params []NameFormPattern
}

type DirectionParamParamPatternKind interface {
	DirectionParamParamPatternKind()
}

func (*NameFormPattern) DirectionParamParamPatternKind()     {}
func (*FunctionFormPattern) DirectionParamParamPatternKind() {}
func (*OrdinalPattern) DirectionParamParamPatternKind()      {}

type InfixOperatorFormPattern struct {
	Operator NameFormPattern
	Lhs      FormPatternKind
	Rhs      FormPatternKind
}

type PrefixOperatorFormPattern struct {
	Operator NameFormPattern
	Param    FormPatternKind
}

type PostfixOperatorFormPattern struct {
	Operator NameFormPattern
	Param    FormPatternKind
}

type TupleFormPattern struct {
	Params []FormPatternKind
	VarArg VarArgPatternData
}

type ConditionalSetFormPattern struct {
	Target    FormPatternKind
	Condition FunctionFormPattern
	VarArg    VarArgPatternData
}

type ConditionaSetIdFormPattern struct {
	Symbols   []FormPatternKind
	Target    FormPatternKind
	Condition FunctionFormPattern
}

type FunctionLiteralFormPattern struct {
	Lhs TupleFormPattern
	Rhs FormPatternKind
}

type ConditionalSetExpressionPattern struct {
	Target     FormPatternKind
	Conditions []FormPatternKind
	VarArg     VarArgPatternData
}

type VarArgPatternData struct {
	IsVarArg     bool
	VarArgNames  []NameFormPattern
	VarArgBounds []NameFormPattern
}

type InfixCommandOperatorPattern struct {
	Lhs      FormPatternKind
	Operator InfixCommandPattern
	Rhs      FormPatternKind
}

type InfixCommandPattern struct {
	Signature   string
	Names       []NameFormPattern
	CurlyArg    *CurlyPattern
	NamedGroups *[]NamedGroupPattern
	ParenArgs   *[]NameFormPattern
}

type CommandPattern struct {
	Signature   string
	Names       []NameFormPattern
	CurlyArg    *CurlyPattern
	NamedGroups *[]NamedGroupPattern
	ParenArgs   *[]NameFormPattern
}

type CurlyPattern struct {
	SquareArgs *[]FormPatternKind
	CurlyArgs  *[]FormPatternKind
	Direction  *DirectionPattern
}

type DirectionPattern struct {
	Name       *NameFormPattern
	SquareArgs []DirectionParamParamPatternKind
}

type ChainExpressionPattern struct {
	Parts []FormPatternKind
}

type NamedGroupPattern struct {
	Name  NameFormPattern
	Curly CurlyPattern
}

type SpecAliasPattern struct {
	Lhs  PatternKind
	Name NameFormPattern
	Rhs  PatternKind
}

type AliasPattern struct {
	Lhs PatternKind
	Rhs PatternKind
}

func ToPatternFromTarget(item ast.Target) PatternKind {
	switch n := item.Root.(type) {
	case ast.StructuralFormKind:
		return ToFormPattern(n)
	default:
		return nil
	}
}

func ToPattern(exp ast.ExpressionKind) PatternKind {
	switch n := exp.(type) {
	case ast.StructuralFormKind:
		return ToFormPattern(n)
	default:
		return nil
	}
}

func ToFormPattern(item ast.StructuralFormKind) FormPatternKind {
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
	case *ast.ConditionalSetIdForm:
		return ToConditionalSetIdFormPattern(*n)
	case *ast.FunctionLiteralForm:
		return ToFunctionLiteralFormPattern(*n)
	case *ast.StructuralColonEqualsForm:
		return ToStructuralColonEqualsPattern(*n)
	default:
		panic("Could not process a pattern for " +
			item.ToCode(func(node ast.MlgNodeKind) (string, bool) { return "", false }))
	}
}

func ToDirectionParamParamPatternKind(item ast.StructuralFormKind) DirectionParamParamPatternKind {
	switch n := item.(type) {
	case *ast.NameForm:
		return ToNameFormPattern(*n)
	case *ast.FunctionForm:
		return ToFunctionFormPattern(*n)
	default:
		return nil
	}
}

func ToVarArgPatternData(data ast.VarArgData) VarArgPatternData {
	varArgNames := make([]NameFormPattern, 0)
	varArgNames = append(varArgNames,
		mlglib.Map(data.VarArgNames, func(name ast.NameForm) NameFormPattern {
			return NameFormPattern{
				Text:            name.Text,
				IsStropped:      false,
				HasQuestionMark: false,
				VarArg: VarArgPatternData{
					IsVarArg:     false,
					VarArgNames:  nil,
					VarArgBounds: nil,
				},
			}
		})...)

	varArgBounds := make([]NameFormPattern, 0)
	varArgBounds = append(varArgBounds,
		mlglib.Map(data.VarArgBounds, func(name ast.NameForm) NameFormPattern {
			return NameFormPattern{
				Text:            name.Text,
				IsStropped:      false,
				HasQuestionMark: false,
				VarArg: VarArgPatternData{
					IsVarArg:     false,
					VarArgNames:  nil,
					VarArgBounds: nil,
				},
			}
		})...)

	return VarArgPatternData{
		IsVarArg:     data.IsVarArg,
		VarArgNames:  varArgNames,
		VarArgBounds: varArgBounds,
	}
}

func ToNameFormPattern(form ast.NameForm) *NameFormPattern {
	return &NameFormPattern{
		Text:            form.Text,
		IsStropped:      form.IsStropped,
		HasQuestionMark: form.HasQuestionMark,
		VarArg:          ToVarArgPatternData(form.VarArg),
	}
}

func ToFunctionFormPattern(form ast.FunctionForm) *FunctionFormPattern {
	return &FunctionFormPattern{
		Target: *ToNameFormPattern(form.Target),
		Params: toFormPatterns(form.Params),
		VarArg: ToVarArgPatternData(form.VarArg),
	}
}

func ToTupleFormPattern(form ast.TupleForm) *TupleFormPattern {
	return &TupleFormPattern{
		Params: toFormPatterns(form.Params),
		VarArg: ToVarArgPatternData(form.VarArg),
	}
}

func ToConditionalSetFormPattern(form ast.ConditionalSetForm) *ConditionalSetFormPattern {
	return &ConditionalSetFormPattern{
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

func ToInfixOperatorFormPattern(form ast.InfixOperatorForm) *InfixOperatorFormPattern {
	return &InfixOperatorFormPattern{
		Operator: *ToNameFormPattern(form.Operator),
		Lhs:      ToFormPattern(form.Lhs),
		Rhs:      ToFormPattern(form.Rhs),
	}
}

func ToInfixOperatorFormPatternFromId(form ast.InfixOperatorId) InfixOperatorFormPattern {
	return InfixOperatorFormPattern{
		Operator: toNameFormPatternFromText(form.Operator.Text),
		Lhs:      ToFormPattern(form.Lhs),
	}
}

func ToConditionalSetIdFormPattern(form ast.ConditionalSetIdForm) *ConditionaSetIdFormPattern {
	return &ConditionaSetIdFormPattern{
		Symbols:   toFormPatterns(form.Symbols),
		Target:    ToFormPattern(form.Target),
		Condition: *ToFunctionFormPattern(form.Condition),
	}
}

func ToFunctionLiteralFormPattern(form ast.FunctionLiteralForm) *FunctionLiteralFormPattern {
	return &FunctionLiteralFormPattern{
		Lhs: *ToTupleFormPattern(form.Lhs),
		Rhs: ToFormPattern(form.Rhs),
	}
}

func ToPrefixOperatorFormPattern(form ast.PrefixOperatorForm) *PrefixOperatorFormPattern {
	return &PrefixOperatorFormPattern{
		Operator: *ToNameFormPattern(form.Operator),
		Param:    ToFormPattern(form.Param),
	}
}

func ToPrefixOperatorFormPatternFromId(form ast.PrefixOperatorId) PrefixOperatorFormPattern {
	return PrefixOperatorFormPattern{
		Operator: toNameFormPatternFromText(form.Operator.Text),
		Param:    ToFormPattern(form.Param),
	}
}

func ToPostfixOperatorFormPattern(form ast.PostfixOperatorForm) *PostfixOperatorFormPattern {
	return &PostfixOperatorFormPattern{
		Operator: *ToNameFormPattern(form.Operator),
		Param:    ToFormPattern(form.Param),
	}
}

func ToPostfixOperatorFormPatternFromId(form ast.PostfixOperatorId) PostfixOperatorFormPattern {
	return PostfixOperatorFormPattern{
		Operator: toNameFormPatternFromText(form.Operator.Text),
		Param:    ToFormPattern(form.Param),
	}
}

func ToCommandPattern(id ast.CommandId) CommandPattern {
	names := make([]NameFormPattern, 0)
	for _, n := range id.Names {
		names = append(names, NameFormPattern{
			Text:            n.Text,
			IsStropped:      false,
			HasQuestionMark: false,
			VarArg: VarArgPatternData{
				IsVarArg:     false,
				VarArgNames:  nil,
				VarArgBounds: nil,
			},
		})
	}
	return CommandPattern{
		Signature:   GetSignatureStringFromCommandId(id),
		Names:       names,
		NamedGroups: toNamedGroupPatterns(id.NamedParams),
		CurlyArg:    toCurlyArg(id.CurlyParam),
		ParenArgs:   toParenArgs(id.ParenParams),
	}
}

func ToInfixCommandPattern(id ast.InfixCommandOperatorId) InfixCommandPattern {
	names := make([]NameFormPattern, 0)
	for _, n := range id.Operator.Names {
		names = append(names, NameFormPattern{
			Text:            n.Text,
			IsStropped:      false,
			HasQuestionMark: false,
			VarArg: VarArgPatternData{
				IsVarArg:     false,
				VarArgNames:  nil,
				VarArgBounds: nil,
			},
		})
	}
	return InfixCommandPattern{
		Signature:   GetSignatureStringFromInfixCommandId(id.Operator),
		Names:       names,
		NamedGroups: toNamedGroupPatterns(id.Operator.NamedParams),
		CurlyArg:    toCurlyArg(id.Operator.CurlyParam),
		ParenArgs:   toParenArgs(id.Operator.ParenParams),
	}
}

func ToStructuralColonEqualsPattern(
	colonEquals ast.StructuralColonEqualsForm,
) *StructuralColonEqualsPattern {
	return &StructuralColonEqualsPattern{
		Lhs: ToFormPattern(colonEquals.Lhs),
		Rhs: ToFormPattern(colonEquals.Rhs),
	}
}

////////////////////////////////////////////////////////////////////////////////////////////////////

func toFormPatterns(items []ast.StructuralFormKind) []FormPatternKind {
	patterns := make([]FormPatternKind, 0)
	for _, item := range items {
		patterns = append(patterns, ToFormPattern(item))
	}
	return patterns
}

func toNameFormPatterns(params []ast.NameForm) []NameFormPattern {
	result := make([]NameFormPattern, 0)
	for _, p := range params {
		result = append(result, *ToNameFormPattern(p))
	}
	return result
}

func toNameFormPatternFromText(text string) NameFormPattern {
	return NameFormPattern{
		Text:            text,
		IsStropped:      false,
		HasQuestionMark: false,
		VarArg: VarArgPatternData{
			IsVarArg:     false,
			VarArgNames:  nil,
			VarArgBounds: nil,
		},
	}
}

func toCurlyArg(curlyParam *ast.CurlyParam) *CurlyPattern {
	if curlyParam == nil {
		return nil
	}

	var squareArgs *[]FormPatternKind
	var direction *DirectionPattern
	if curlyParam.Direction != nil {
		squareArgs := make([]DirectionParamParamPatternKind, 0)
		for _, param := range curlyParam.Direction.SquareParams {
			if form, ok := param.(ast.StructuralFormKind); ok {
				squareArgs = append(squareArgs, ToDirectionParamParamPatternKind(form))
			} else {
				panic(fmt.Sprintf("Cannot convert direction square param to a pattern: %s",
					mlglib.PrettyPrint(param)))
			}
		}
		var name *NameFormPattern
		if curlyParam.Direction.Name != nil {
			tmpPattern := ToNameFormPattern(*curlyParam.Direction.Name)
			name = tmpPattern
		}
		direction = &DirectionPattern{
			Name:       name,
			SquareArgs: squareArgs,
		}
	}
	var curlyArgs *[]FormPatternKind
	if curlyParam.CurlyParams != nil {
		tmpArgs := toFormPatterns(*curlyParam.CurlyParams)
		curlyArgs = &tmpArgs
	}
	return &CurlyPattern{
		SquareArgs: squareArgs,
		CurlyArgs:  curlyArgs,
		Direction:  direction,
	}
}

func toNamedGroupPatterns(nameParams *[]ast.NamedParam) *[]NamedGroupPattern {
	if nameParams == nil {
		return nil
	}
	result := make([]NamedGroupPattern, 0)
	for _, param := range *nameParams {
		var curlyParam CurlyPattern
		if param.CurlyParam != nil {
			curlyParam = *toCurlyArg(param.CurlyParam)
		}
		result = append(result, NamedGroupPattern{
			Name:  *ToNameFormPattern(param.Name),
			Curly: curlyParam,
		})
	}
	return &result
}

func toParenArgs(names *[]ast.NameForm) *[]NameFormPattern {
	if names == nil {
		return nil
	}
	result := make([]NameFormPattern, 0)
	for _, name := range *names {
		result = append(result, *ToNameFormPattern(name))
	}
	return &result
}

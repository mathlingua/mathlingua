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
	PatternType()
	GetVarArgData() VarArgPatternData
}

func (*NameFormPattern) PatternType()                 {}
func (*FunctionFormPattern) PatternType()             {}
func (*TupleFormPattern) PatternType()                {}
func (*ConditionalSetExpressionPattern) PatternType() {}
func (*ConditionalSetFormPattern) PatternType()       {}
func (*ConditionaSetIdFormPattern) PatternType()      {}
func (*FunctionLiteralFormPattern) PatternType()      {}
func (*InfixOperatorFormPattern) PatternType()        {}
func (*PrefixOperatorFormPattern) PatternType()       {}
func (*PostfixOperatorFormPattern) PatternType()      {}
func (*OrdinalPattern) PatternType()                  {}

func (*NameColonEqualsPatternPattern) PatternType()  {}
func (*FunctionColonEqualsNamePattern) PatternType() {}
func (*InfixCommandOperatorPattern) PatternType()    {}
func (*InfixCommandTargetPattern) PatternType()      {}
func (*CommandPattern) PatternType()                 {}
func (*NamedGroupPattern) PatternType()              {}
func (*ChainExpressionPattern) PatternType()         {}

func (*SpecAliasPattern) PatternType() {}
func (*AliasPattern) PatternType()     {}

type NameColonEqualsPatternPattern struct {
	Lhs NameFormPattern
	Rhs PatternKind
}

type FunctionColonEqualsNamePattern struct {
	Lhs FunctionFormPattern
	Rhs NameFormPattern
}

type FormPatternKind interface {
	PatternKind
	FormPatternType()
}

func (*NameFormPattern) FormPatternType()            {}
func (*FunctionFormPattern) FormPatternType()        {}
func (*TupleFormPattern) FormPatternType()           {}
func (*ConditionalSetFormPattern) FormPatternType()  {}
func (*ConditionaSetIdFormPattern) FormPatternType() {}
func (*FunctionLiteralFormPattern) FormPatternType() {}
func (*InfixOperatorFormPattern) FormPatternType()   {}
func (*PrefixOperatorFormPattern) FormPatternType()  {}
func (*PostfixOperatorFormPattern) FormPatternType() {}

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
	LiteralFormPatternType()
}

func (*NameFormPattern) LiteralFormPatternType()            {}
func (*FunctionFormPattern) LiteralFormPatternType()        {}
func (*TupleFormPattern) LiteralFormPatternType()           {}
func (*ConditionalSetFormPattern) LiteralFormPatternType()  {}
func (*ConditionaSetIdFormPattern) LiteralFormPatternType() {}
func (*FunctionLiteralFormPattern) LiteralFormPatternType() {}

type OrdinalPattern struct {
	Target LiteralFormPatternKind
	Params []NameFormPattern
}

type DirectionParamParamPatternKind interface {
	DirectionParamParamPatternType()
}

func (*NameFormPattern) DirectionParamParamPatternType()     {}
func (*FunctionFormPattern) DirectionParamParamPatternType() {}
func (*OrdinalPattern) DirectionParamParamPatternType()      {}

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
	Operator CommandPattern
	Rhs      FormPatternKind
}

type InfixCommandTargetPattern struct {
	Command CommandPattern
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
	default:
		panic("Could not process a pattern for " +
			item.ToCode(func(node ast.MlgNodeKind) (string, bool) { return "", false }))
	}
}

func ToDirectionParamParamPatternType(item ast.StructuralFormKind) DirectionParamParamPatternKind {
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
				squareArgs = append(squareArgs, ToDirectionParamParamPatternType(form))
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

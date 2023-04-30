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
type PatternType interface {
	PatternType()
	GetVarArgData() VarArgPatternData
}

func (*NameFormPattern) PatternType()                 {}
func (*FunctionFormPattern) PatternType()             {}
func (*TupleFormPattern) PatternType()                {}
func (*ConditionalSetExpressionPattern) PatternType() {}
func (*ConditionalSetFormPattern) PatternType()       {}
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

func (p *NameFormPattern) GetVarArgData() VarArgPatternData                 { return p.VarArg }
func (p *FunctionFormPattern) GetVarArgData() VarArgPatternData             { return p.VarArg }
func (p *TupleFormPattern) GetVarArgData() VarArgPatternData                { return p.VarArg }
func (p *ConditionalSetExpressionPattern) GetVarArgData() VarArgPatternData { return p.VarArg }
func (p *ConditionalSetFormPattern) GetVarArgData() VarArgPatternData       { return p.VarArg }

func (p *InfixOperatorFormPattern) GetVarArgData() VarArgPatternData {
	return VarArgPatternData{}
}
func (p *PrefixOperatorFormPattern) GetVarArgData() VarArgPatternData {
	return VarArgPatternData{}
}
func (p *PostfixOperatorFormPattern) GetVarArgData() VarArgPatternData {
	return VarArgPatternData{}
}
func (p *OrdinalPattern) GetVarArgData() VarArgPatternData { return VarArgPatternData{} }

func (p *NameColonEqualsPatternPattern) GetVarArgData() VarArgPatternData {
	return VarArgPatternData{}
}
func (p *FunctionColonEqualsNamePattern) GetVarArgData() VarArgPatternData {
	return VarArgPatternData{}
}
func (p *InfixCommandOperatorPattern) GetVarArgData() VarArgPatternData {
	return VarArgPatternData{}
}
func (p *InfixCommandTargetPattern) GetVarArgData() VarArgPatternData {
	return VarArgPatternData{}
}
func (p *CommandPattern) GetVarArgData() VarArgPatternData {
	return VarArgPatternData{}
}
func (p *NamedGroupPattern) GetVarArgData() VarArgPatternData {
	return VarArgPatternData{}
}
func (p *ChainExpressionPattern) GetVarArgData() VarArgPatternData {
	return VarArgPatternData{}
}

func (p *SpecAliasPattern) GetVarArgData() VarArgPatternData { return VarArgPatternData{} }
func (p *AliasPattern) GetVarArgData() VarArgPatternData     { return VarArgPatternData{} }

type NameColonEqualsPatternPattern struct {
	Lhs NameFormPattern
	Rhs PatternType
}

type FunctionColonEqualsNamePattern struct {
	Lhs FunctionFormPattern
	Rhs NameFormPattern
}

type FormPatternType interface {
	PatternType
	FormPatternType()
}

func (*NameFormPattern) FormPatternType()            {}
func (*FunctionFormPattern) FormPatternType()        {}
func (*TupleFormPattern) FormPatternType()           {}
func (*ConditionalSetFormPattern) FormPatternType()  {}
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
	Params []NameFormPattern
	VarArg VarArgPatternData
}

type LiteralFormPatternType interface {
	LiteralFormPatternType()
}

func (*NameFormPattern) LiteralFormPatternType()           {}
func (*FunctionFormPattern) LiteralFormPatternType()       {}
func (*TupleFormPattern) LiteralFormPatternType()          {}
func (*ConditionalSetFormPattern) LiteralFormPatternType() {}

type OrdinalPattern struct {
	Target LiteralFormPatternType
	Params []NameFormPattern
}

type DirectionParamParamPatternType interface {
	DirectionParamParamPatternType()
}

func (*NameFormPattern) DirectionParamParamPatternType()     {}
func (*FunctionFormPattern) DirectionParamParamPatternType() {}
func (*OrdinalPattern) DirectionParamParamPatternType()      {}

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
	Target    FormPatternType
	Condition FunctionFormPattern
	VarArg    VarArgPatternData
}

type ConditionalSetExpressionPattern struct {
	Target     FormPatternType
	Conditions []FormPatternType
	VarArg     VarArgPatternData
}

type VarArgPatternData struct {
	IsVarArg     bool
	VarArgNames  []NameFormPattern
	VarArgBounds []NameFormPattern
}

type InfixCommandOperatorPattern struct {
	Lhs      FormPatternType
	Operator CommandPattern
	Rhs      FormPatternType
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
	SquareArgs *[]FormPatternType
	CurlyArgs  *[]FormPatternType
	Direction  *DirectionPattern
}

type DirectionPattern struct {
	Name       *NameFormPattern
	SquareArgs []DirectionParamParamPatternType
}

type ChainExpressionPattern struct {
	Parts []FormPatternType
}

type NamedGroupPattern struct {
	Name  NameFormPattern
	Curly CurlyPattern
}

type SpecAliasPattern struct {
	Lhs  PatternType
	Name NameFormPattern
	Rhs  PatternType
}

type AliasPattern struct {
	Lhs PatternType
	Rhs PatternType
}

////////////////////////////////////////////////////////////////////////////////////////////////////

func ToPatternFromTarget(item ast.Target) PatternType {
	switch n := item.Root.(type) {
	case ast.StructuralFormType:
		return ToFormPattern(n)
	default:
		return nil
	}
}

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

func ToDirectionParamParamPatternType(item ast.StructuralFormType) DirectionParamParamPatternType {
	switch n := item.(type) {
	case *ast.NameForm:
		return ToNameFormPattern(*n)
	case *ast.FunctionForm:
		return ToFunctionFormPattern(*n)
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

func toNameFormPatterns(params []ast.NameForm) []NameFormPattern {
	result := make([]NameFormPattern, 0)
	for _, p := range params {
		result = append(result, *ToNameFormPattern(p))
	}
	return result
}

func ToFunctionFormPattern(form ast.FunctionForm) *FunctionFormPattern {
	return &FunctionFormPattern{
		Target: *ToNameFormPattern(form.Target),
		Params: toNameFormPatterns(form.Params),
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
			VarArgNames:  nil,
			VarArgBounds: nil,
		},
	}
}

func ToInfixOperatorFormPatternFromId(form ast.InfixOperatorId) InfixOperatorFormPattern {
	return InfixOperatorFormPattern{
		Operator: toNameFormPatternFromText(form.Operator.Text),
		Lhs:      ToFormPattern(form.Lhs),
	}
}

func ToPrefixOperatorFormPattern(form ast.PrefixOperatorForm) *PrefixOperatorFormPattern {
	return &PrefixOperatorFormPattern{
		Operator: *ToNameFormPattern(form.Operator),
		Param:    ToNameFormPattern(form.Param),
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
		Param:    ToNameFormPattern(form.Param),
	}
}

func ToPostfixOperatorFormPatternFromId(form ast.PostfixOperatorId) PostfixOperatorFormPattern {
	return PostfixOperatorFormPattern{
		Operator: toNameFormPatternFromText(form.Operator.Text),
		Param:    ToFormPattern(form.Param),
	}
}

func toCurlyArg(curlyParam *ast.CurlyParam) *CurlyPattern {
	if curlyParam == nil {
		return nil
	}

	var squareArgs *[]FormPatternType
	if curlyParam.SquareParams != nil {
		patterns := toFormPatterns(*curlyParam.SquareParams)
		squareArgs = &patterns
	}
	var direction *DirectionPattern
	if curlyParam.Direction != nil {
		squareArgs := make([]DirectionParamParamPatternType, 0)
		for _, param := range curlyParam.Direction.SquareParams {
			if form, ok := param.(ast.StructuralFormType); ok {
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
	curlyArgs := toFormPatterns(*&curlyParam.CurlyParams)
	return &CurlyPattern{
		SquareArgs: squareArgs,
		CurlyArgs:  &curlyArgs,
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

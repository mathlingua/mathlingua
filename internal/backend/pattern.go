/*
 * Copyright 2024 Dominic Kramer
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
	"mathlingua/internal/mlglib"
)

func ToPatternFromTarget(item ast.Target) ast.PatternKind {
	switch n := item.Root.(type) {
	case ast.StructuralFormKind:
		return ToFormPattern(n)
	default:
		return nil
	}
}

func ToPattern(exp ast.ExpressionKind) ast.PatternKind {
	switch n := exp.(type) {
	case ast.StructuralFormKind:
		return ToFormPattern(n)
	default:
		return nil
	}
}

func ToFormPattern(item ast.StructuralFormKind) ast.FormPatternKind {
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
	// TODO: add StructuralColonEqualsColonForm case
	default:
		panic("Could not process a pattern for " +
			item.ToCode(func(node ast.MlgNodeKind) (string, bool) { return "", false }))
	}
}

func ToDirectionParamParamPatternKind(item ast.StructuralFormKind) ast.DirectionParamParamPatternKind {
	switch n := item.(type) {
	case *ast.NameForm:
		return ToNameFormPattern(*n)
	case *ast.FunctionForm:
		return ToFunctionFormPattern(*n)
	default:
		return nil
	}
}

func ToVarArgPatternData(data ast.VarArgData) ast.VarArgPatternData {
	varArgNames := make([]ast.NameFormPattern, 0)
	varArgNames = append(varArgNames,
		mlglib.Map(data.VarArgNames, func(name ast.NameForm) ast.NameFormPattern {
			return ast.NameFormPattern{
				Text:            name.Text,
				IsStropped:      false,
				HasQuestionMark: false,
				VarArg: ast.VarArgPatternData{
					IsVarArg:     false,
					VarArgNames:  nil,
					VarArgBounds: nil,
				},
			}
		})...)

	varArgBounds := make([]ast.NameFormPattern, 0)
	varArgBounds = append(varArgBounds,
		mlglib.Map(data.VarArgBounds, func(name ast.NameForm) ast.NameFormPattern {
			return ast.NameFormPattern{
				Text:            name.Text,
				IsStropped:      false,
				HasQuestionMark: false,
				VarArg: ast.VarArgPatternData{
					IsVarArg:     false,
					VarArgNames:  nil,
					VarArgBounds: nil,
				},
			}
		})...)

	return ast.VarArgPatternData{
		IsVarArg:     data.IsVarArg,
		VarArgNames:  varArgNames,
		VarArgBounds: varArgBounds,
	}
}

func ToNameFormPattern(form ast.NameForm) *ast.NameFormPattern {
	return &ast.NameFormPattern{
		Text:            form.Text,
		IsStropped:      form.IsStropped,
		HasQuestionMark: form.HasQuestionMark,
		VarArg:          ToVarArgPatternData(form.VarArg),
	}
}

func ToFunctionFormPattern(form ast.FunctionForm) *ast.FunctionFormPattern {
	return &ast.FunctionFormPattern{
		Target: *ToNameFormPattern(form.Target),
		Params: toFormPatterns(form.Params),
		VarArg: ToVarArgPatternData(form.VarArg),
	}
}

func ToTupleFormPattern(form ast.TupleForm) *ast.TupleFormPattern {
	return &ast.TupleFormPattern{
		Params: toFormPatterns(form.Params),
		VarArg: ToVarArgPatternData(form.VarArg),
	}
}

func ToConditionalSetFormPattern(form ast.ConditionalSetForm) *ast.ConditionalSetFormPattern {
	var conditionPattern *ast.FunctionFormPattern
	if form.Condition != nil {
		conditionPattern = ToFunctionFormPattern(*form.Condition)
	}
	var specPattern *ast.FunctionFormPattern
	if form.Specification != nil {
		specPattern = ToFunctionFormPattern(*form.Specification)
	}
	return &ast.ConditionalSetFormPattern{
		Target:        ToFormPattern(form.Target),
		Specification: specPattern,
		Condition:     conditionPattern,
		VarArg:        ToVarArgPatternData(form.VarArg),
	}
}

func ToConditionalSetFormPatternFromId(form ast.ConditionalSetIdForm) ast.ConditionalSetFormPattern {
	var conditionPattern *ast.FunctionFormPattern
	if form.Condition != nil {
		conditionPattern = ToFunctionFormPattern(*form.Condition)
	}
	var specPattern *ast.FunctionFormPattern
	if form.Specification != nil {
		specPattern = ToFunctionFormPattern(*form.Specification)
	}
	return ast.ConditionalSetFormPattern{
		Target:        ToFormPattern(form.Target),
		Specification: specPattern,
		Condition:     conditionPattern,
		VarArg:        ToVarArgPatternData(form.Condition.VarArg),
	}
}

func ToInfixOperatorFormPattern(form ast.InfixOperatorForm) *ast.InfixOperatorFormPattern {
	return &ast.InfixOperatorFormPattern{
		Operator: *ToNameFormPattern(form.Operator),
		Lhs:      ToFormPattern(form.Lhs),
		Rhs:      ToFormPattern(form.Rhs),
	}
}

func ToInfixOperatorFormPatternFromId(form ast.InfixOperatorId) ast.InfixOperatorFormPattern {
	return ast.InfixOperatorFormPattern{
		Operator: toNameFormPatternFromText(form.Operator.Text),
		Lhs:      ToFormPattern(form.Lhs),
	}
}

func ToConditionalSetIdFormPattern(form ast.ConditionalSetIdForm) *ast.ConditionalSetIdFormPattern {
	var conditionPattern *ast.FunctionFormPattern
	if form.Condition != nil {
		conditionPattern = ToFunctionFormPattern(*form.Condition)
	}
	var specPattern *ast.FunctionFormPattern
	if form.Specification != nil {
		specPattern = ToFunctionFormPattern(*form.Specification)
	}
	return &ast.ConditionalSetIdFormPattern{
		Symbols:       toFormPatterns(form.Symbols),
		Target:        ToFormPattern(form.Target),
		Specification: specPattern,
		Condition:     conditionPattern,
	}
}

func ToFunctionLiteralFormPattern(form ast.FunctionLiteralForm) *ast.FunctionLiteralFormPattern {
	return &ast.FunctionLiteralFormPattern{
		Lhs: *ToTupleFormPattern(form.Lhs),
		Rhs: ToFormPattern(form.Rhs),
	}
}

func ToPrefixOperatorFormPattern(form ast.PrefixOperatorForm) *ast.PrefixOperatorFormPattern {
	return &ast.PrefixOperatorFormPattern{
		Operator: *ToNameFormPattern(form.Operator),
		Param:    ToFormPattern(form.Param),
	}
}

func ToPrefixOperatorFormPatternFromId(form ast.PrefixOperatorId) ast.PrefixOperatorFormPattern {
	return ast.PrefixOperatorFormPattern{
		Operator: toNameFormPatternFromText(form.Operator.Text),
		Param:    ToFormPattern(form.Param),
	}
}

func ToPostfixOperatorFormPattern(form ast.PostfixOperatorForm) *ast.PostfixOperatorFormPattern {
	return &ast.PostfixOperatorFormPattern{
		Operator: *ToNameFormPattern(form.Operator),
		Param:    ToFormPattern(form.Param),
	}
}

func ToPostfixOperatorFormPatternFromId(form ast.PostfixOperatorId) ast.PostfixOperatorFormPattern {
	return ast.PostfixOperatorFormPattern{
		Operator: toNameFormPatternFromText(form.Operator.Text),
		Param:    ToFormPattern(form.Param),
	}
}

func ToCommandPattern(id ast.CommandId) ast.CommandPattern {
	names := make([]ast.NameFormPattern, 0)
	for _, n := range id.Names {
		names = append(names, ast.NameFormPattern{
			Text:            n.Text,
			IsStropped:      false,
			HasQuestionMark: false,
			VarArg: ast.VarArgPatternData{
				IsVarArg:     false,
				VarArgNames:  nil,
				VarArgBounds: nil,
			},
		})
	}
	return ast.CommandPattern{
		Signature:   GetSignatureStringFromCommandId(id),
		Names:       names,
		NamedGroups: toNamedGroupPatterns(id.NamedParams),
		CurlyArg:    toCurlyArg(id.CurlyParam),
		ParenArgs:   toParenArgs(id.ParenParams),
	}
}

func ToInfixCommandPattern(id ast.InfixCommandOperatorId) ast.InfixCommandPattern {
	names := make([]ast.NameFormPattern, 0)
	for _, n := range id.Operator.Names {
		names = append(names, ast.NameFormPattern{
			Text:            n.Text,
			IsStropped:      false,
			HasQuestionMark: false,
			VarArg: ast.VarArgPatternData{
				IsVarArg:     false,
				VarArgNames:  nil,
				VarArgBounds: nil,
			},
		})
	}
	return ast.InfixCommandPattern{
		Signature:   GetSignatureStringFromInfixCommandId(id.Operator),
		Names:       names,
		NamedGroups: toNamedGroupPatterns(id.Operator.NamedParams),
		CurlyArg:    toCurlyArg(id.Operator.CurlyParam),
		ParenArgs:   toParenArgs(id.Operator.ParenParams),
	}
}

// TODO: add ToStructuralColonEqualsColonPattern

func ToStructuralColonEqualsPattern(
	colonEquals ast.StructuralColonEqualsForm,
) *ast.StructuralColonEqualsPattern {
	return &ast.StructuralColonEqualsPattern{
		Lhs: ToFormPattern(colonEquals.Lhs),
		Rhs: ToFormPattern(colonEquals.Rhs),
	}
}

////////////////////////////////////////////////////////////////////////////////////////////////////

func toFormPatterns(items []ast.StructuralFormKind) []ast.FormPatternKind {
	patterns := make([]ast.FormPatternKind, 0)
	for _, item := range items {
		patterns = append(patterns, ToFormPattern(item))
	}
	return patterns
}

// nolint:unused
func toNameFormPatterns(params []ast.NameForm) []ast.NameFormPattern {
	result := make([]ast.NameFormPattern, 0)
	for _, p := range params {
		result = append(result, *ToNameFormPattern(p))
	}
	return result
}

func toNameFormPatternFromText(text string) ast.NameFormPattern {
	return ast.NameFormPattern{
		Text:            text,
		IsStropped:      false,
		HasQuestionMark: false,
		VarArg: ast.VarArgPatternData{
			IsVarArg:     false,
			VarArgNames:  nil,
			VarArgBounds: nil,
		},
	}
}

func toCurlyArg(curlyParam *ast.CurlyParam) *ast.CurlyPattern {
	if curlyParam == nil {
		return nil
	}

	var squareArgs *[]ast.FormPatternKind
	var curlyArgs *[]ast.FormPatternKind
	if curlyParam.CurlyParams != nil {
		tmpArgs := toFormPatterns(*curlyParam.CurlyParams)
		curlyArgs = &tmpArgs
	}
	return &ast.CurlyPattern{
		SquareArgs: squareArgs,
		CurlyArgs:  curlyArgs,
	}
}

func toNamedGroupPatterns(nameParams *[]ast.NamedParam) *[]ast.NamedGroupPattern {
	if nameParams == nil {
		return nil
	}
	result := make([]ast.NamedGroupPattern, 0)
	for _, param := range *nameParams {
		var curlyParam ast.CurlyPattern
		if param.CurlyParam != nil {
			curlyParam = *toCurlyArg(param.CurlyParam)
		}
		result = append(result, ast.NamedGroupPattern{
			Name:  *ToNameFormPattern(param.Name),
			Curly: curlyParam,
		})
	}
	return &result
}

func toParenArgs(names *[]ast.NameForm) *[]ast.NameFormPattern {
	if names == nil {
		return nil
	}
	result := make([]ast.NameFormPattern, 0)
	for _, name := range *names {
		result = append(result, *ToNameFormPattern(name))
	}
	return &result
}

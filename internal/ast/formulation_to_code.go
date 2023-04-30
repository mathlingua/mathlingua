/*
 * Copyright 2022 Dominic Kramer
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

package ast

func FormulationNodeToCode(node FormulationNodeType,
	fn func(node MlgNodeType) (string, bool)) string {
	if node == nil {
		return ""
	}
	return node.ToCode(fn)
}

func (n *NameForm) ToCode(fn func(node MlgNodeType) (string, bool)) string {
	if res, ok := fn(n); ok {
		return res
	}
	return n.Text + n.VarArg.ToCode(fn)
}

func (n *FunctionForm) ToCode(fn func(node MlgNodeType) (string, bool)) string {
	if res, ok := fn(n); ok {
		return res
	}
	return n.Target.ToCode(fn) + "(" + commaSeparatedStringOfNameForms(n.Params, fn) +
		")" + n.VarArg.ToCode(fn)
}

func (n *TupleForm) ToCode(fn func(node MlgNodeType) (string, bool)) string {
	if res, ok := fn(n); ok {
		return res
	}
	return "(" + commaSeparatedString(n.Params, fn) + ")" + n.VarArg.ToCode(fn)
}

func (n *ConditionalSetForm) ToCode(fn func(node MlgNodeType) (string, bool)) string {
	if res, ok := fn(n); ok {
		return res
	}
	return "{" + n.Target.ToCode(fn) + " | ...}" + n.VarArg.ToCode(fn)
}

func (n *ConditionalSetIdForm) ToCode(fn func(node MlgNodeType) (string, bool)) string {
	if res, ok := fn(n); ok {
		return res
	}
	return "[" + commaSeparatedString(n.Symbols, fn) +
		"]{" + n.Target.ToCode(fn) + " | " + n.Condition.ToCode(fn) +
		"}" + n.Condition.VarArg.ToCode(fn)
}

func (n *FunctionCallExpression) ToCode(fn func(node MlgNodeType) (string, bool)) string {
	if res, ok := fn(n); ok {
		return res
	}
	return n.Target.ToCode(fn) + "(" + commaSeparatedString(n.Args, fn) + ")"
}

func (n *TupleExpression) ToCode(fn func(node MlgNodeType) (string, bool)) string {
	if res, ok := fn(n); ok {
		return res
	}
	return "(" + commaSeparatedString(n.Args, fn) + ")"
}

func (n *ConditionalSetExpression) ToCode(fn func(node MlgNodeType) (string, bool)) string {
	if res, ok := fn(n); ok {
		return res
	}
	return "[" + commaSeparatedString(n.Symbols, fn) + "]{" +
		n.Target.ToCode(fn) + " | " + semicolonSeparatedString(n.Conditions, fn) + "}"
}

func (n *CommandExpression) ToCode(fn func(node MlgNodeType) (string, bool)) string {
	if res, ok := fn(n); ok {
		return res
	}
	result := "\\"
	for i, n := range n.Names {
		if i > 0 {
			result += "."
		}
		result += n.ToCode(fn)
	}
	if n.CurlyArg != nil {
		result += n.CurlyArg.ToCode(fn)
	}
	if n.NamedArgs != nil {
		for _, item := range *n.NamedArgs {
			result += ":" + item.Name.ToCode(fn)
			if item.CurlyArg != nil {
				result += item.CurlyArg.ToCode(fn)
			}
		}
	}
	if n.ParenArgs != nil {
		result += "("
		result += commaSeparatedString(*n.ParenArgs, fn)
		result += ")"
	}
	return result
}

func (n *PrefixOperatorCallExpression) ToCode(
	fn func(node MlgNodeType) (string, bool)) string {
	if res, ok := fn(n); ok {
		return res
	}
	return n.Target.ToCode(fn) + n.Arg.ToCode(fn)
}

func (n *PostfixOperatorCallExpression) ToCode(
	fn func(node MlgNodeType) (string, bool)) string {
	if res, ok := fn(n); ok {
		return res
	}
	return n.Arg.ToCode(fn) + n.Target.ToCode(fn)
}

func (n *InfixOperatorCallExpression) ToCode(
	fn func(node MlgNodeType) (string, bool)) string {
	if res, ok := fn(n); ok {
		return res
	}
	return n.Lhs.ToCode(fn) + " " + n.Target.ToCode(fn) + " " + n.Rhs.ToCode(fn)
}

func (n *IsExpression) ToCode(fn func(node MlgNodeType) (string, bool)) string {
	if res, ok := fn(n); ok {
		return res
	}
	result := ""
	result += commaSeparatedString(n.Lhs, fn)
	result += " is "
	result += commaSeparatedString(n.Rhs, fn)
	return result
}

func (n *ExtendsExpression) ToCode(fn func(node MlgNodeType) (string, bool)) string {
	if res, ok := fn(n); ok {
		return res
	}
	result := ""
	result += commaSeparatedString(n.Lhs, fn)
	result += " extends "
	result += commaSeparatedString(n.Rhs, fn)
	return result
}

func (n *AsExpression) ToCode(fn func(node MlgNodeType) (string, bool)) string {
	if res, ok := fn(n); ok {
		return res
	}
	return n.Lhs.ToCode(fn) + " as " + n.Rhs.ToCode(fn)
}

func (n *OrdinalCallExpression) ToCode(fn func(node MlgNodeType) (string, bool)) string {
	if res, ok := fn(n); ok {
		return res
	}
	return n.Target.ToCode(fn) + "{" + commaSeparatedString(n.Args, fn) + "}"
}

func (n *ChainExpression) ToCode(fn func(node MlgNodeType) (string, bool)) string {
	if res, ok := fn(n); ok {
		return res
	}
	result := ""
	for i, item := range n.Parts {
		if i > 0 {
			result += "."
		}
		result += item.ToCode(fn)
	}
	return result
}

func (n *Signature) ToCode(fn func(node MlgNodeType) (string, bool)) string {
	if res, ok := fn(n); ok {
		return res
	}
	result := "\\("
	for i, item := range n.MainNames {
		if i > 0 {
			result += "."
		}
		result += item
	}
	for _, item := range n.NamedGroupNames {
		result += ":"
		result += item
	}
	result += ")"
	if n.IsInfix {
		result += "/"
	}
	if n.InnerLabel != nil {
		result += "::[" + *n.InnerLabel + "]"
	}
	return result
}

func (n *MetaKinds) ToCode(fn func(node MlgNodeType) (string, bool)) string {
	if res, ok := fn(n); ok {
		return res
	}
	result := "[:"
	for i, name := range n.Kinds {
		if i > 0 {
			result += ", "
		}
		result += name
	}
	result += ":]"
	return result
}

func (n *StructuralColonEqualsForm) ToCode(fn func(node MlgNodeType) (string, bool)) string {
	if res, ok := fn(n); ok {
		return res
	}
	return n.Lhs.ToCode(fn) + " := " + n.Rhs.ToCode(fn)
}

func (n *ExpressionColonEqualsItem) ToCode(fn func(node MlgNodeType) (string, bool)) string {
	if res, ok := fn(n); ok {
		return res
	}
	return n.Lhs.ToCode(fn) + " := " + n.Rhs.ToCode(fn)
}

func (n *ExpressionColonArrowItem) ToCode(fn func(node MlgNodeType) (string, bool)) string {
	if res, ok := fn(n); ok {
		return res
	}
	return n.Lhs.ToCode(fn) + " :=> " + n.Rhs.ToCode(fn)
}

func (n *ExpressionColonDashArrowItem) ToCode(
	fn func(node MlgNodeType) (string, bool)) string {
	if res, ok := fn(n); ok {
		return res
	}
	return n.Lhs.ToCode(fn) + " :-> " + n.Rhs.ToCode(fn)
}

func (n *EnclosedNonCommandOperatorTarget) ToCode(
	fn func(node MlgNodeType) (string, bool)) string {
	if res, ok := fn(n); ok {
		return res
	}
	result := ""
	if n.HasLeftColon {
		result += ":"
	}
	result += "["
	result += n.Target.ToCode(fn)
	result += "]"
	if n.HasRightColon {
		result += ":"
	}
	return result
}

func (n *NonEnclosedNonCommandOperatorTarget) ToCode(
	fn func(node MlgNodeType) (string, bool)) string {
	if res, ok := fn(n); ok {
		return res
	}
	result := ""
	if n.HasLeftColon {
		result += ":"
	}
	result += n.Text
	if n.HasRightColon {
		result += ":"
	}
	return result
}

func (n *CommandOperatorTarget) ToCode(fn func(node MlgNodeType) (string, bool)) string {
	if res, ok := fn(n); ok {
		return res
	}
	return n.Command.ToCode(fn) + "/"
}

func (n *CommandId) ToCode(fn func(node MlgNodeType) (string, bool)) string {
	if res, ok := fn(n); ok {
		return res
	}
	result := "\\"
	for i, n := range n.Names {
		if i > 0 {
			result += "."
		}
		result += n.ToCode(fn)
	}
	if n.CurlyParam != nil {
		result += n.CurlyParam.ToCode(fn)
	}
	if n.NamedParams != nil {
		for _, item := range *n.NamedParams {
			result += ":" + item.Name.ToCode(fn)
			if item.CurlyParam != nil {
				result += item.CurlyParam.ToCode(fn)
			}
		}
	}
	if n.ParenParams != nil {
		result += "("
		result += commaSeparatedStringOfNameForms(*n.ParenParams, fn)
		result += ")"
	}
	return result
}

func (n *PrefixOperatorId) ToCode(fn func(node MlgNodeType) (string, bool)) string {
	if res, ok := fn(n); ok {
		return res
	}
	return n.Operator.ToCode(fn) + n.Param.ToCode(fn)
}

func (n *PostfixOperatorId) ToCode(fn func(node MlgNodeType) (string, bool)) string {
	if res, ok := fn(n); ok {
		return res
	}
	return n.Param.ToCode(fn) + n.Operator.ToCode(fn)
}

func (n *InfixOperatorId) ToCode(fn func(node MlgNodeType) (string, bool)) string {
	if res, ok := fn(n); ok {
		return res
	}
	return n.Lhs.ToCode(fn) + " " + n.Operator.ToCode(fn) + " " + n.Rhs.ToCode(fn)
}

func (n *InfixCommandId) ToCode(fn func(node MlgNodeType) (string, bool)) string {
	if res, ok := fn(n); ok {
		return res
	}
	result := "\\"
	for i, n := range n.Names {
		if i > 0 {
			result += "."
		}
		result += n.ToCode(fn)
	}
	if n.CurlyParam != nil {
		result += n.CurlyParam.ToCode(fn)
	}
	if n.NamedParams != nil {
		for _, item := range *n.NamedParams {
			result += ":" + item.Name.ToCode(fn)
			if item.CurlyParam != nil {
				result += item.CurlyParam.ToCode(fn)
			}
		}
	}
	if n.ParenParams != nil {
		result += "("
		result += commaSeparatedStringOfNameForms(*n.ParenParams, fn)
		result += ")"
	}
	result += "/"
	return result
}

func (n *InfixCommandOperatorId) ToCode(fn func(node MlgNodeType) (string, bool)) string {
	if res, ok := fn(n); ok {
		return res
	}
	return n.Lhs.ToCode(fn) + " " + n.Operator.ToCode(fn) + " " + n.Rhs.ToCode(fn)
}

func (n *PseudoTokenNode) ToCode(fn func(node MlgNodeType) (string, bool)) string {
	if res, ok := fn(n); ok {
		return res
	}
	return n.Text
}

func (n *PseudoExpression) ToCode(fn func(node MlgNodeType) (string, bool)) string {
	if res, ok := fn(n); ok {
		return res
	}
	result := ""
	for i, item := range n.Children {
		if i > 0 {
			result += " "
		}
		result += item.ToCode(fn)
	}
	return result
}

func (n *MultiplexedInfixOperatorCallExpression) ToCode(
	fn func(node MlgNodeType) (string, bool)) string {
	if res, ok := fn(n); ok {
		return res
	}
	result := ""
	result += commaSeparatedString(n.Lhs, fn)
	result += " "
	result += n.Target.ToCode(fn)
	result += " "
	result += commaSeparatedString(n.Rhs, fn)
	return result
}

func (n *InfixOperatorForm) ToCode(fn func(node MlgNodeType) (string, bool)) string {
	if res, ok := fn(n); ok {
		return res
	}
	return n.Lhs.ToCode(fn) + " " + n.Operator.ToCode(fn) + " " + n.Rhs.ToCode(fn)
}

func (n *PrefixOperatorForm) ToCode(fn func(node MlgNodeType) (string, bool)) string {
	if res, ok := fn(n); ok {
		return res
	}
	return n.Operator.ToCode(fn) + n.Param.ToCode(fn)
}

func (n *PostfixOperatorForm) ToCode(fn func(node MlgNodeType) (string, bool)) string {
	if res, ok := fn(n); ok {
		return res
	}
	return n.Param.ToCode(fn) + n.Operator.ToCode(fn)
}

func (n *VarArgData) ToCode(fn func(node MlgNodeType) (string, bool)) string {
	if n.IsVarArg {
		if len(n.VarArgNames) == 0 && len(n.VarArgBounds) == 0 {
			return "..."
		} else if len(n.VarArgNames) == 1 && len(n.VarArgBounds) == 1 {
			return "{" + n.VarArgNames[0].Text + "..." + n.VarArgBounds[0].Text + "}"
		} else {
			result := "{("
			for i, name := range n.VarArgNames {
				if i > 0 {
					result += ","
				}
				result += name.Text
			}
			result += ")...("
			for i, bound := range n.VarArgBounds {
				if i > 0 {
					result += ","
				}
				result += bound.Text
			}
			result += ")}"
			return result
		}
	}
	return ""
}

func (n *FunctionLiteralExpression) ToCode(fn func(node MlgNodeType) (string, bool)) string {
	if res, ok := fn(n); ok {
		return res
	}
	return n.Lhs.ToCode(fn) + " => " + n.Rhs.ToCode(fn)
}

func (n *CurlyParam) ToCode(fn func(node MlgNodeType) (string, bool)) string {
	if res, ok := fn(n); ok {
		return res
	}
	result := ""
	if n.SquareParams != nil {
		result += "["
		result += commaSeparatedString(*n.SquareParams, fn)
		result += "]"
	}
	result += "{"
	result += commaSeparatedString(n.CurlyParams, fn)
	result += "}"
	if n.Direction != nil {
		result += n.Direction.ToCode(fn)
	}
	return result
}

func (n *CurlyArg) ToCode(fn func(node MlgNodeType) (string, bool)) string {
	if res, ok := fn(n); ok {
		return res
	}
	result := "{"
	if n.CurlyArgs != nil {
		result += commaSeparatedString(*n.CurlyArgs, fn)
	}
	result += "}"
	if n.Direction != nil {
		result += n.Direction.ToCode(fn)
	}
	return result
}

func (n *DirectionalParam) ToCode(fn func(node MlgNodeType) (string, bool)) string {
	if res, ok := fn(n); ok {
		return res
	}
	result := "@"
	if n.Name != nil {
		result += n.Name.ToCode(fn)
	}
	result += "["
	result += commaSeparatedString(n.SquareParams, fn)
	result += "]"
	return result
}

func commaSeparatedString[T FormulationNodeType](forms []T,
	fn func(node MlgNodeType) (string, bool)) string {
	return separatedString(forms, ", ", fn)
}

func commaSeparatedStringOfNameForms(forms []NameForm,
	fn func(node MlgNodeType) (string, bool)) string {
	result := ""
	for i, _ := range forms {
		if i > 0 {
			result += ", "
		}
		result += forms[i].ToCode(fn)
	}
	return result
}

func semicolonSeparatedString[T FormulationNodeType](forms []T,
	fn func(node MlgNodeType) (string, bool)) string {
	return separatedString(forms, "; ", fn)
}

func separatedString[T FormulationNodeType](forms []T, separator string,
	fn func(node MlgNodeType) (string, bool)) string {
	result := ""
	for i, _ := range forms {
		if i > 0 {
			result += separator
		}
		result += forms[i].ToCode(fn)
	}
	return result
}

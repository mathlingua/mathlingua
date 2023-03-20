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

func FormulationNodeToCode(node FormulationNodeType) string {
	if node == nil {
		return ""
	}
	return node.ToCode()
}

type IFormulationToCode interface {
	ToCode() string
}

func (n NameForm) ToCode() string {
	return n.Text + n.VarArg.ToCode()
}

func (n FunctionForm) ToCode() string {
	return n.Target.ToCode() + "(" + commaSeparatedStringOfNameForms(n.Params) + ")" + n.VarArg.ToCode()
}

func (n TupleForm) ToCode() string {
	return "(" + commaSeparatedString(n.Params) + ")" + n.VarArg.ToCode()
}

func (n ConditionalSetForm) ToCode() string {
	return "{" + n.Target.ToCode() + " | ...}" + n.VarArg.ToCode()
}

func (n ConditionalSetIdForm) ToCode() string {
	return "[" + commaSeparatedString(n.Symbols) +
		"]{" + n.Target.ToCode() + " | " + n.Condition.ToCode() +
		"}" + n.Condition.VarArg.ToCode()
}

func (n FunctionCallExpression) ToCode() string {
	return n.Target.ToCode() + "(" + commaSeparatedString(n.Args) + ")"
}

func (n TupleExpression) ToCode() string {
	return "(" + commaSeparatedString(n.Args) + ")"
}

func (n ConditionalSetExpression) ToCode() string {
	return "[" + commaSeparatedString(n.Symbols) + "]{" +
		n.Target.ToCode() + " | " + semicolonSeparatedString(n.Conditions) + "}"
}

func (n CommandExpression) ToCode() string {
	result := "\\"
	for i, n := range n.Names {
		if i > 0 {
			result += "."
		}
		result += n.ToCode()
	}
	if n.CurlyArg != nil {
		result += n.CurlyArg.ToCode()
	}
	if n.NamedArgs != nil {
		for _, item := range *n.NamedArgs {
			result += ":" + item.Name.ToCode()
			if item.CurlyArg != nil {
				result += item.CurlyArg.ToCode()
			}
		}
	}
	if n.ParenArgs != nil {
		result += "("
		result += commaSeparatedString(*n.ParenArgs)
		result += ")"
	}
	return result
}

func (n PrefixOperatorCallExpression) ToCode() string {
	return n.Target.ToCode() + n.Arg.ToCode()
}

func (n PostfixOperatorCallExpression) ToCode() string {
	return n.Arg.ToCode() + n.Target.ToCode()
}

func (n InfixOperatorCallExpression) ToCode() string {
	return n.Lhs.ToCode() + " " + n.Target.ToCode() + " " + n.Rhs.ToCode()
}

func (n IsExpression) ToCode() string {
	result := ""
	result += commaSeparatedString(n.Lhs)
	result += " is "
	result += commaSeparatedString(n.Rhs)
	return result
}

func (n ExtendsExpression) ToCode() string {
	result := ""
	result += commaSeparatedString(n.Lhs)
	result += " extends "
	result += commaSeparatedString(n.Rhs)
	return result
}

func (n AsExpression) ToCode() string {
	return n.Lhs.ToCode() + " as " + n.Rhs.ToCode()
}

func (n OrdinalCallExpression) ToCode() string {
	return n.Target.ToCode() + "{" + commaSeparatedString(n.Args) + "}"
}

func (n ChainExpression) ToCode() string {
	result := ""
	for i, item := range n.Parts {
		if i > 0 {
			result += "."
		}
		result += item.ToCode()
	}
	return result
}

func (n Signature) ToCode() string {
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

func (n MetaKinds) ToCode() string {
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

func (n StructuralColonEqualsForm) ToCode() string {
	return n.Lhs.ToCode() + " is " + n.Rhs.ToCode()
}

func (n ExpressionColonEqualsItem) ToCode() string {
	return n.Lhs.ToCode() + " := " + n.Rhs.ToCode()
}

func (n ExpressionColonArrowItem) ToCode() string {
	return n.Lhs.ToCode() + " :=> " + n.Rhs.ToCode()
}

func (n ExpressionColonDashArrowItem) ToCode() string {
	return n.Lhs.ToCode() + " :-> " + n.Rhs.ToCode()
}

func (n EnclosedNonCommandOperatorTarget) ToCode() string {
	result := ""
	if n.HasLeftColon {
		result += ":"
	}
	result += "["
	result += n.Target.ToCode()
	result += "]"
	if n.HasRightColon {
		result += ":"
	}
	return result
}

func (n NonEnclosedNonCommandOperatorTarget) ToCode() string {
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

func (n CommandOperatorTarget) ToCode() string {
	return n.Command.ToCode() + "/"
}

func (n CommandId) ToCode() string {
	result := "\\"
	for i, n := range n.Names {
		if i > 0 {
			result += "."
		}
		result += n.ToCode()
	}
	if n.CurlyParam != nil {
		result += n.CurlyParam.ToCode()
	}
	if n.NamedParams != nil {
		for _, item := range *n.NamedParams {
			result += ":" + item.Name.ToCode()
			if item.CurlyParam != nil {
				result += item.CurlyParam.ToCode()
			}
		}
	}
	if n.ParenParams != nil {
		result += "("
		result += commaSeparatedStringOfNameForms(*n.ParenParams)
		result += ")"
	}
	return result
}

func (n PrefixOperatorId) ToCode() string {
	return n.Operator.ToCode() + n.Param.ToCode()
}

func (n PostfixOperatorId) ToCode() string {
	return n.Param.ToCode() + n.Operator.ToCode()
}

func (n InfixOperatorId) ToCode() string {
	return n.Lhs.ToCode() + " " + n.Operator.ToCode() + " " + n.Rhs.ToCode()
}

func (n InfixCommandId) ToCode() string {
	result := "\\"
	for i, n := range n.Names {
		if i > 0 {
			result += "."
		}
		result += n.ToCode()
	}
	if n.CurlyParam != nil {
		result += n.CurlyParam.ToCode()
	}
	if n.NamedParams != nil {
		for _, item := range *n.NamedParams {
			result += ":" + item.Name.ToCode()
			if item.CurlyParam != nil {
				result += item.CurlyParam.ToCode()
			}
		}
	}
	if n.ParenParams != nil {
		result += "("
		result += commaSeparatedStringOfNameForms(*n.ParenParams)
		result += ")"
	}
	result += "/"
	return result
}

func (n InfixCommandOperatorId) ToCode() string {
	return n.Lhs.ToCode() + " " + n.Operator.ToCode() + " " + n.Rhs.ToCode()
}

func (n PseudoTokenNode) ToCode() string {
	return n.Text
}

func (n PseudoExpression) ToCode() string {
	result := ""
	for i, item := range n.Children {
		if i > 0 {
			result += " "
		}
		result += item.ToCode()
	}
	return result
}

func (n MultiplexedInfixOperatorCallExpression) ToCode() string {
	result := ""
	result += commaSeparatedString(n.Lhs)
	result += " "
	result += n.Target.ToCode()
	result += " "
	result += commaSeparatedString(n.Rhs)
	return result
}

func (n InfixOperatorForm) ToCode() string {
	return n.Lhs.ToCode() + " " + n.Operator.ToCode() + " " + n.Rhs.ToCode()
}

func (n PrefixOperatorForm) ToCode() string {
	return n.Operator.ToCode() + n.Param.ToCode()
}

func (n PostfixOperatorForm) ToCode() string {
	return n.Param.ToCode() + n.Operator.ToCode()
}

func (n VarArgData) ToCode() string {
	if n.IsVarArg {
		if len(n.VarArgNames) == 0 && len(n.VarArgBounds) == 0 {
			return "..."
		} else if len(n.VarArgNames) == 1 && len(n.VarArgBounds) == 1 {
			return "{" + n.VarArgNames[0] + "..." + n.VarArgBounds[0] + "}"
		} else {
			result := "{("
			for i, name := range n.VarArgNames {
				if i > 0 {
					result += ","
				}
				result += name
			}
			result += ")...("
			for i, bound := range n.VarArgBounds {
				if i > 0 {
					result += ","
				}
				result += bound
			}
			result += ")}"
			return result
		}
	}
	return ""
}

func (n FunctionLiteralExpression) ToCode() string {
	return n.Lhs.ToCode() + " => " + n.Rhs.ToCode()
}

func (n CurlyParam) ToCode() string {
	result := ""
	if n.SquareParams != nil {
		result += "["
		result += commaSeparatedString(*n.SquareParams)
		result += "]"
	}
	result += "{"
	result += commaSeparatedString(n.CurlyParams)
	result += "}"
	if n.Direction != nil {
		result += n.Direction.ToCode()
	}
	return result
}

func (n CurlyArg) ToCode() string {
	result := "{"
	if n.CurlyArgs != nil {
		result += commaSeparatedString(*n.CurlyArgs)
	}
	result += "}"
	if n.Direction != nil {
		result += n.Direction.ToCode()
	}
	return result
}

func (n DirectionalParam) ToCode() string {
	result := "@"
	if n.Name != nil {
		result += n.Name.ToCode()
	}
	result += "["
	result += commaSeparatedString(n.SquareParams)
	result += "]"
	return result
}

func commaSeparatedString[T FormulationNodeType](forms []T) string {
	return separatedString(forms, ", ")
}

func commaSeparatedStringOfNameForms(forms []NameForm) string {
	result := ""
	for i, _ := range forms {
		if i > 0 {
			result += ", "
		}
		result += forms[i].ToCode()
	}
	return result
}

func semicolonSeparatedString[T FormulationNodeType](forms []T) string {
	return separatedString(forms, "; ")
}

func separatedString[T FormulationNodeType](forms []T, separator string) string {
	result := ""
	for i, _ := range forms {
		if i > 0 {
			result += separator
		}
		result += forms[i].ToCode()
	}
	return result
}

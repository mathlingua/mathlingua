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

func DebugFormulationNode(node FormulationNodeType) string {
	if node == nil {
		return ""
	}
	return node.Debug()
}

type FormulationDebuggable interface {
	Debug() string
}

func (n NameForm) Debug() string {
	return n.Text + n.VarArg.Debug()
}

func (n FunctionForm) Debug() string {
	return n.Target.Debug() + "(" + commaSeparatedStringOfNameForms(n.Params) + ")" + n.VarArg.Debug()
}

func (n TupleForm) Debug() string {
	return "(" + commaSeparatedString(n.Params) + ")" + n.VarArg.Debug()
}

func (n FixedSetForm) Debug() string {
	return "{" + commaSeparatedString(n.Params) + "}" + n.VarArg.Debug()
}

func (n ConditionalSetForm) Debug() string {
	return "{" + n.Target.Debug() + " | ...}" + n.VarArg.Debug()
}

func (n ConditionalSetIdForm) Debug() string {
	return "[" + commaSeparatedString(n.Symbols) +
		"]{" + n.Target.Debug() + " | " + n.Condition.Debug() +
		"}" + n.Condition.VarArg.Debug()
}

func (n FunctionCallExpression) Debug() string {
	return n.Target.Debug() + "(" + commaSeparatedString(n.Args) + ")"
}

func (n TupleExpression) Debug() string {
	return "(" + commaSeparatedString(n.Args) + ")"
}

func (n FixedSetExpression) Debug() string {
	return "{" + commaSeparatedString(n.Args) + "}"
}

func (n ConditionalSetExpression) Debug() string {
	return "[" + commaSeparatedString(n.Symbols) + "]{" +
		n.Target.Debug() + " | " + semicolonSeparatedString(n.Conditions) + "}"
}

func (n CommandExpression) Debug() string {
	result := "\\"
	for i, n := range n.Names {
		if i > 0 {
			result += "."
		}
		result += n.Debug()
	}
	if n.CurlyArg != nil {
		result += n.CurlyArg.Debug()
	}
	if n.NamedArgs != nil {
		for _, item := range *n.NamedArgs {
			result += ":" + item.Name.Debug()
			if item.CurlyArg != nil {
				result += item.CurlyArg.Debug()
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

func (n PrefixOperatorCallExpression) Debug() string {
	return n.Target.Debug() + n.Arg.Debug()
}

func (n PostfixOperatorCallExpression) Debug() string {
	return n.Arg.Debug() + n.Target.Debug()
}

func (n InfixOperatorCallExpression) Debug() string {
	return n.Lhs.Debug() + " " + n.Target.Debug() + " " + n.Rhs.Debug()
}

func (n IsExpression) Debug() string {
	result := ""
	result += commaSeparatedString(n.Lhs)
	result += " is "
	result += commaSeparatedString(n.Rhs)
	return result
}

func (n ExtendsExpression) Debug() string {
	result := ""
	result += commaSeparatedString(n.Lhs)
	result += " extends "
	result += commaSeparatedString(n.Rhs)
	return result
}

func (n AsExpression) Debug() string {
	return n.Lhs.Debug() + " as " + n.Rhs.Debug()
}

func (n OrdinalCallExpression) Debug() string {
	return n.Target.Debug() + "{" + commaSeparatedString(n.Args) + "}"
}

func (n ChainExpression) Debug() string {
	result := ""
	for i, item := range n.Parts {
		if i > 0 {
			result += "."
		}
		result += item.Debug()
	}
	return result
}

func (n Signature) Debug() string {
	result := "\\["
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
	result += "]"
	if n.InnerLabel != nil {
		result += "::[" + *n.InnerLabel + "]"
	}
	return result
}

func (n MetaKinds) Debug() string {
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

func (n StructuralColonEqualsForm) Debug() string {
	return n.Lhs.Debug() + " is " + n.Rhs.Debug()
}

func (n ExpressionColonEqualsItem) Debug() string {
	return n.Lhs.Debug() + " := " + n.Rhs.Debug()
}

func (n ExpressionColonArrowItem) Debug() string {
	return n.Lhs.Debug() + " :=> " + n.Rhs.Debug()
}

func (n EnclosedNonCommandOperatorTarget) Debug() string {
	result := ""
	if n.HasLeftColon {
		result += ":"
	}
	result += "["
	result += n.Target.Debug()
	result += "]"
	if n.HasRightColon {
		result += ":"
	}
	return result
}

func (n NonEnclosedNonCommandOperatorTarget) Debug() string {
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

func (n CommandOperatorTarget) Debug() string {
	return n.Command.Debug() + "/"
}

func (n CommandId) Debug() string {
	result := "\\"
	for i, n := range n.Names {
		if i > 0 {
			result += "."
		}
		result += n.Debug()
	}
	if n.CurlyParam != nil {
		result += n.CurlyParam.Debug()
	}
	if n.NamedParams != nil {
		for _, item := range *n.NamedParams {
			result += ":" + item.Name.Debug()
			if item.CurlyParam != nil {
				result += item.CurlyParam.Debug()
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

func (n PrefixOperatorId) Debug() string {
	return n.Operator.Debug() + n.Param.Debug()
}

func (n PostfixOperatorId) Debug() string {
	return n.Param.Debug() + n.Operator.Debug()
}

func (n InfixOperatorId) Debug() string {
	return n.Lhs.Debug() + " " + n.Operator.Debug() + " " + n.Rhs.Debug()
}

func (n InfixCommandId) Debug() string {
	result := "\\"
	for i, n := range n.Names {
		if i > 0 {
			result += "."
		}
		result += n.Debug()
	}
	if n.CurlyParam != nil {
		result += n.CurlyParam.Debug()
	}
	if n.NamedParams != nil {
		for _, item := range *n.NamedParams {
			result += ":" + item.Name.Debug()
			if item.CurlyParam != nil {
				result += item.CurlyParam.Debug()
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

func (n InfixCommandOperatorId) Debug() string {
	return n.Lhs.Debug() + " " + n.Operator.Debug() + " " + n.Rhs.Debug()
}

func (n PseudoTokenNode) Debug() string {
	return n.Text
}

func (n PseudoExpression) Debug() string {
	result := ""
	for i, item := range n.Children {
		if i > 0 {
			result += " "
		}
		result += item.Debug()
	}
	return result
}

func (n MultiplexedInfixOperatorCallExpression) Debug() string {
	result := ""
	result += commaSeparatedString(n.Lhs)
	result += " "
	result += n.Target.Debug()
	result += " "
	result += commaSeparatedString(n.Rhs)
	return result
}

func (n InfixOperatorForm) Debug() string {
	return n.Lhs.Debug() + " " + n.Operator.Debug() + " " + n.Rhs.Debug()
}

func (n PrefixOperatorForm) Debug() string {
	return n.Operator.Debug() + n.Param.Debug()
}

func (n PostfixOperatorForm) Debug() string {
	return n.Param.Debug() + n.Operator.Debug()
}

func (n VarArgData) Debug() string {
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

func (n FunctionLiteralExpression) Debug() string {
	return n.Lhs.Debug() + " => " + n.Rhs.Debug()
}

func (n CurlyParam) Debug() string {
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
		result += n.Direction.Debug()
	}
	return result
}

func (n CurlyArg) Debug() string {
	result := ""
	if n.SquareArgs != nil {
		result += "["
		result += commaSeparatedString(*n.SquareArgs)
		result += "]"
	}
	result += "{"
	result += commaSeparatedString(*n.CurlyArgs)
	result += "}"
	if n.Direction != nil {
		result += n.Direction.Debug()
	}
	return result
}

func (n DirectionalParam) Debug() string {
	result := "@"
	if n.Name != nil {
		result += n.Name.Debug()
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
		result += forms[i].Debug()
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
		result += forms[i].Debug()
	}
	return result
}

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

func DebugFormulationNode(node NodeType) string {
	if node == nil {
		return ""
	}
	return node.Debug()
}

type FormulationDebuggable interface {
	Debug() string
}

func (n NameForm) Debug() string {
	return n.Text
}

func (n FunctionForm) Debug() string {
	return n.Target.Debug() + "(" + commaSeparatedString(n.Params) + ")" + n.VarArg.Debug()
}

func (n FunctionExpressionForm) Debug() string {
	return n.Target.Debug() + "(" + commaSeparatedString(n.Params) + ")" + n.VarArg.Debug()
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
	if n.SquareArgs != nil {
		result += "["
		result += commaSeparatedString(*n.SquareArgs)
		result += "]"
	}
	if n.CurlyArgs != nil {
		result += "{"
		result += commaSeparatedString(*n.CurlyArgs)
		result += "}"
	}
	if n.NamedArgs != nil {
		for _, item := range *n.NamedArgs {
			result += ":" + item.Name.Debug()
			if item.Args != nil {
				result += "{" + commaSeparatedString(*item.Args) + "}"
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

func (n CommandAtExpression) Debug() string {
	result := "\\"
	for i, n := range n.Names {
		if i > 0 {
			result += "."
		}
		result += n.Debug()
	}
	result += "@"
	result += n.Expression.Debug()
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

func (n NameOrdinalCallExpression) Debug() string {
	return n.Target.Debug() + "{" + n.Arg.Debug() + "}"
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
	for i, item := range n.NamedGroupNames {
		if i > 0 {
			result += ":"
		}
		result += item
	}
	if n.HasAtSymbol {
		result += "@"
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
	if n.SquareParams != nil {
		result += "["
		result += commaSeparatedString(*n.SquareParams)
		result += "]"
	}
	if n.CurlyParams != nil {
		result += "{"
		result += commaSeparatedString(*n.CurlyParams)
		result += "}"
	}
	if n.NamedParams != nil {
		for _, item := range *n.NamedParams {
			result += ":" + item.Name.Debug()
			if item.Params != nil {
				result += "{" + commaSeparatedString(*item.Params) + "}"
			}
		}
	}
	if n.ParenParams != nil {
		result += "("
		result += commaSeparatedString(*n.ParenParams)
		result += ")"
	}
	return result
}

func (n CommandAtId) Debug() string {
	result := "\\"
	for i, n := range n.Names {
		if i > 0 {
			result += "."
		}
		result += n.Debug()
	}
	result += "@"
	result += n.Param.Debug()
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
	if n.SquareParams != nil {
		result += "["
		result += commaSeparatedString(*n.SquareParams)
		result += "]"
	}
	if n.CurlyParams != nil {
		result += "{"
		result += commaSeparatedString(*n.CurlyParams)
		result += "}"
	}
	if n.NamedParams != nil {
		for _, item := range *n.NamedParams {
			result += ":" + item.Name.Debug()
			if item.Params != nil {
				result += "{" + commaSeparatedString(*item.Params) + "}"
			}
		}
	}
	if n.ParenParams != nil {
		result += "("
		result += commaSeparatedString(*n.ParenParams)
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
	result := ""
	if n.IsVarArg {
		result += "..."
		if n.VarArgCount != nil {
			result += "#" + *n.VarArgCount
		}
	}
	return result
}

func commaSeparatedString[T NodeType](forms []T) string {
	return separatedString(forms, ", ")
}

func semicolonSeparatedString[T NodeType](forms []T) string {
	return separatedString(forms, "; ")
}

func separatedString[T NodeType](forms []T, separator string) string {
	result := ""
	for i, f := range forms {
		if i > 0 {
			result += separator
		}
		result += f.Debug()
	}
	return result
}

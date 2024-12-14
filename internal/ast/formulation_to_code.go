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

func FormulationNodeToCode(node FormulationNodeKind,
	fn func(node MlgNodeKind) (string, bool)) string {
	if node == nil {
		return ""
	}
	return node.ToCode(fn)
}

func (n *NameForm) ToCode(fn func(node MlgNodeKind) (string, bool)) string {
	if res, ok := fn(n); ok {
		return res
	}
	return n.Text + n.VarArg.ToCode(fn)
}

func (n *FunctionForm) ToCode(fn func(node MlgNodeKind) (string, bool)) string {
	if res, ok := fn(n); ok {
		return res
	}
	return n.Target.ToCode(fn) + "(" + commaSeparatedString(n.Params, fn) +
		")" + n.VarArg.ToCode(fn)
}

func (n *TupleForm) ToCode(fn func(node MlgNodeKind) (string, bool)) string {
	if res, ok := fn(n); ok {
		return res
	}
	return "(" + commaSeparatedString(n.Params, fn) + ")" + n.VarArg.ToCode(fn)
}

func (n *ConditionalSetForm) ToCode(fn func(node MlgNodeKind) (string, bool)) string {
	if res, ok := fn(n); ok {
		return res
	}
	result := "[" + commaSeparatedString(n.Symbols, fn) + "]{"
	result += n.Target.ToCode(fn)
	if n.Specification != nil {
		result += " : "
		result += n.Specification.ToCode(fn)
	}
	if n.Condition != nil {
		result += " | "
		result += n.Condition.ToCode(fn)
	}
	result += "}"
	return result
}

func (n *ConditionalSetIdForm) ToCode(fn func(node MlgNodeKind) (string, bool)) string {
	if res, ok := fn(n); ok {
		return res
	}
	result := "[" + commaSeparatedString(n.Symbols, fn) +
		"]{" + n.Target.ToCode(fn)
	if n.Specification != nil {
		result += " : " + n.Specification.ToCode(fn)
	}
	if n.Condition != nil {
		result += " | " + n.Condition.ToCode(fn)
	}
	result += "}"
	return result
}

func (n *FunctionCallExpression) ToCode(fn func(node MlgNodeKind) (string, bool)) string {
	if res, ok := fn(n); ok {
		return res
	}
	return n.Target.ToCode(fn) + "(" + commaSeparatedString(n.Args, fn) + ")" + n.VarArg.ToCode(fn)
}

func (n *TupleExpression) ToCode(fn func(node MlgNodeKind) (string, bool)) string {
	if res, ok := fn(n); ok {
		return res
	}

	result := ""
	if n.IsInvisible {
		result += "(."
	} else {
		result += "("
	}

	result += commaSeparatedString(n.Args, fn)

	if n.IsInvisible {
		result += ".)"
	} else {
		result += ")"
	}

	return result
}

func (n *LabeledGrouping) ToCode(fn func(node MlgNodeKind) (string, bool)) string {
	if res, ok := fn(n); ok {
		return res
	}

	result := "{."
	result += n.Arg.ToCode(fn)
	result += ".}("
	result += n.Label
	result += ")"

	return result
}

func (n *ConditionalSetExpression) ToCode(fn func(node MlgNodeKind) (string, bool)) string {
	if res, ok := fn(n); ok {
		return res
	}
	result := "[" + commaSeparatedString(n.Symbols, fn) + "]{" +
		n.Target.ToCode(fn)
	if len(n.Specifications) > 0 {
		result += " : " + semicolonSeparatedString(n.Specifications, fn)
	}
	if condition, ok := n.Condition.Get(); ok {
		result += " | " + condition.ToCode(fn)
	}
	result += "}"
	return result
}

func (n *CommandExpression) ToCode(fn func(node MlgNodeKind) (string, bool)) string {
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
	fn func(node MlgNodeKind) (string, bool)) string {
	if res, ok := fn(n); ok {
		return res
	}
	return n.Target.ToCode(fn) + n.Arg.ToCode(fn)
}

func (n *PostfixOperatorCallExpression) ToCode(
	fn func(node MlgNodeKind) (string, bool)) string {
	if res, ok := fn(n); ok {
		return res
	}
	return n.Arg.ToCode(fn) + n.Target.ToCode(fn)
}

func (n *InfixOperatorCallExpression) ToCode(
	fn func(node MlgNodeKind) (string, bool)) string {
	if res, ok := fn(n); ok {
		return res
	}
	return n.Lhs.ToCode(fn) + " " + n.Target.ToCode(fn) + " " + n.Rhs.ToCode(fn)
}

func (n *IsExpression) ToCode(fn func(node MlgNodeKind) (string, bool)) string {
	if res, ok := fn(n); ok {
		return res
	}
	result := ""
	result += commaSeparatedString(n.Lhs, fn)
	result += " is "
	result += commaSeparatedString(n.Rhs, fn)
	return result
}

func (n *AsExpression) ToCode(fn func(node MlgNodeKind) (string, bool)) string {
	if res, ok := fn(n); ok {
		return res
	}
	return n.Lhs.ToCode(fn) + " as " + n.Rhs.ToCode(fn)
}

func (n *OrdinalCallExpression) ToCode(fn func(node MlgNodeKind) (string, bool)) string {
	if res, ok := fn(n); ok {
		return res
	}
	return n.Target.ToCode(fn) + "{" + commaSeparatedString(n.Args, fn) + "}"
}

func (n *ChainExpression) ToCode(fn func(node MlgNodeKind) (string, bool)) string {
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

func (n *Signature) ToCode(fn func(node MlgNodeKind) (string, bool)) string {
	if res, ok := fn(n); ok {
		return res
	}

	suffix := ""
	if n.IsInfix {
		suffix = ":/"
	} else {
		suffix = ""
	}

	result := "\\:"
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
	result += suffix
	if n.InnerLabel != nil {
		result += "::(" + *n.InnerLabel + ")"
	}
	return result
}

func (n *MapToElseBuiltinExpression) ToCode(fn func(node MlgNodeKind) (string, bool)) string {
	if res, ok := fn(n); ok {
		return res
	}
	result := "\\\\map{"
	result += n.Target.ToCode(fn)
	result += "}:to{"
	result += n.To.ToCode(fn)
	result += "}"
	if n.Else != nil {
		result += ":else{"
		result += n.Else.ToCode(fn)
		result += "}"
	}
	return result
}

func (n *DefinitionBuiltinExpression) ToCode(fn func(node MlgNodeKind) (string, bool)) string {
	if res, ok := fn(n); ok {
		return res
	}
	result := "\\\\definition"
	if n.Of != nil {
		result += ":of{"
		result += n.Of.ToCode(fn)
		result += "}"
	}
	if n.Satisfies != nil {
		result += ":satisfies{"
		result += n.Satisfies.ToCode(fn)
		result += "}"
	}
	return result
}

func (n *AbstractBuiltinExpression) ToCode(fn func(node MlgNodeKind) (string, bool)) string {
	if res, ok := fn(n); ok {
		return res
	}
	return "\\\\abstract"
}

func (n *SpecificationBuiltinExpression) ToCode(fn func(node MlgNodeKind) (string, bool)) string {
	if res, ok := fn(n); ok {
		return res
	}
	return "\\\\specification"
}

func (n *StatementBuiltinExpression) ToCode(fn func(node MlgNodeKind) (string, bool)) string {
	if res, ok := fn(n); ok {
		return res
	}
	return "\\\\statement"
}

func (n *ExpressionBuiltinExpression) ToCode(fn func(node MlgNodeKind) (string, bool)) string {
	if res, ok := fn(n); ok {
		return res
	}
	return "\\\\expression"
}

func (n *TypeBuiltinExpression) ToCode(fn func(node MlgNodeKind) (string, bool)) string {
	if res, ok := fn(n); ok {
		return res
	}
	return "\\\\type"
}

func (n *StructuralColonEqualsForm) ToCode(fn func(node MlgNodeKind) (string, bool)) string {
	if res, ok := fn(n); ok {
		return res
	}
	return n.Lhs.ToCode(fn) + " := " + n.Rhs.ToCode(fn)
}

func (n *StructuralColonEqualsColonForm) ToCode(fn func(node MlgNodeKind) (string, bool)) string {
	if res, ok := fn(n); ok {
		return res
	}
	return n.Lhs.ToCode(fn) + " :=: " + n.Rhs.ToCode(fn)
}

func (n *ExpressionColonEqualsItem) ToCode(fn func(node MlgNodeKind) (string, bool)) string {
	if res, ok := fn(n); ok {
		return res
	}
	return n.Lhs.ToCode(fn) + " := " + n.Rhs.ToCode(fn)
}

func (n *ExpressionColonArrowItem) ToCode(fn func(node MlgNodeKind) (string, bool)) string {
	if res, ok := fn(n); ok {
		return res
	}
	return n.Lhs.ToCode(fn) + " :=> " + n.Rhs.ToCode(fn)
}

func (n *ExpressionColonDashArrowItem) ToCode(
	fn func(node MlgNodeKind) (string, bool)) string {
	if res, ok := fn(n); ok {
		return res
	}
	res := n.Lhs.ToCode(fn) + " :-> "
	for i, rhsItem := range n.Rhs {
		if i > 0 {
			res += "; "
		}
		res += rhsItem.ToCode(fn)
	}
	return res
}

func (n *EnclosedNonCommandOperatorTarget) ToCode(
	fn func(node MlgNodeKind) (string, bool)) string {
	if res, ok := fn(n); ok {
		return res
	}

	result := ""
	if n.HasLeftColon {
		result += ":"
	}

	result += "[."
	result += n.Target.ToCode(fn)
	result += ".]"

	if n.HasRightColon {
		result += ":"
	}
	return result
}

func (n *NonEnclosedNonCommandOperatorTarget) ToCode(
	fn func(node MlgNodeKind) (string, bool)) string {
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

func (n *InfixCommandExpression) ToCode(fn func(node MlgNodeKind) (string, bool)) string {
	if res, ok := fn(n); ok {
		return res
	}

	result := "\\."
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
	result += "./"
	return result
}

func (n *CommandId) ToCode(fn func(node MlgNodeKind) (string, bool)) string {
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

func (n *PrefixOperatorId) ToCode(fn func(node MlgNodeKind) (string, bool)) string {
	if res, ok := fn(n); ok {
		return res
	}
	return n.Operator.ToCode(fn) + n.Param.ToCode(fn)
}

func (n *PostfixOperatorId) ToCode(fn func(node MlgNodeKind) (string, bool)) string {
	if res, ok := fn(n); ok {
		return res
	}
	return n.Param.ToCode(fn) + n.Operator.ToCode(fn)
}

func (n *InfixOperatorId) ToCode(fn func(node MlgNodeKind) (string, bool)) string {
	if res, ok := fn(n); ok {
		return res
	}
	return n.Lhs.ToCode(fn) + " " + n.Operator.ToCode(fn) + " " + n.Rhs.ToCode(fn)
}

func (n *InfixCommandId) ToCode(fn func(node MlgNodeKind) (string, bool)) string {
	if res, ok := fn(n); ok {
		return res
	}

	result := "\\."
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
	result += "./"
	return result
}

func (n *InfixCommandOperatorId) ToCode(fn func(node MlgNodeKind) (string, bool)) string {
	if res, ok := fn(n); ok {
		return res
	}
	return n.Lhs.ToCode(fn) + " " + n.Operator.ToCode(fn) + " " + n.Rhs.ToCode(fn)
}

func (n *PseudoTokenNode) ToCode(fn func(node MlgNodeKind) (string, bool)) string {
	if res, ok := fn(n); ok {
		return res
	}
	return n.Text
}

func (n *PseudoExpression) ToCode(fn func(node MlgNodeKind) (string, bool)) string {
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
	fn func(node MlgNodeKind) (string, bool)) string {
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

func (n *InfixOperatorForm) ToCode(fn func(node MlgNodeKind) (string, bool)) string {
	if res, ok := fn(n); ok {
		return res
	}
	return n.Lhs.ToCode(fn) + " " + n.Operator.ToCode(fn) + " " + n.Rhs.ToCode(fn)
}

func (n *PrefixOperatorForm) ToCode(fn func(node MlgNodeKind) (string, bool)) string {
	if res, ok := fn(n); ok {
		return res
	}
	return n.Operator.ToCode(fn) + n.Param.ToCode(fn)
}

func (n *PostfixOperatorForm) ToCode(fn func(node MlgNodeKind) (string, bool)) string {
	if res, ok := fn(n); ok {
		return res
	}
	return n.Param.ToCode(fn) + n.Operator.ToCode(fn)
}

func (n *VarArgData) ToCode(fn func(node MlgNodeKind) (string, bool)) string {
	if !n.IsVarArg {
		return ""
	}

	result := ""
	if len(n.VarArgNames) > 0 {
		result += "["
	}
	if len(n.VarArgNames) > 1 {
		result += "("
	}
	for i, name := range n.VarArgNames {
		if i > 0 {
			result += ","
		}
		result += name.Text
	}
	if len(n.VarArgNames) > 1 {
		result += ")"
	}
	result += "..."
	if len(n.VarArgBounds) > 0 {
		if len(n.VarArgBounds) > 1 {
			result += "("
		}
		for i, bound := range n.VarArgBounds {
			if i > 0 {
				result += ","
			}
			result += bound.Text
		}
		if len(n.VarArgBounds) > 1 {
			result += ")"
		}
	}
	if len(n.VarArgNames) > 0 {
		result += "]"
	}
	return result
}

func (n *FunctionLiteralExpression) ToCode(fn func(node MlgNodeKind) (string, bool)) string {
	if res, ok := fn(n); ok {
		return res
	}
	result := ""
	if len(n.Lhs.Params) == 1 {
		// render x |-> ... instead of (x) |-> ...
		result += n.Lhs.Params[0].ToCode(fn)
	} else {
		result += n.Lhs.ToCode(fn)
	}
	result += " |-> "
	result += n.Rhs.ToCode(fn)
	return result
}

func (n *CurlyParam) ToCode(fn func(node MlgNodeKind) (string, bool)) string {
	if res, ok := fn(n); ok {
		return res
	}
	result := ""
	if n.CurlyParams != nil {
		result += "{"
		result += commaSeparatedString(*n.CurlyParams, fn)
		result += "}"
	}
	return result
}

func (n *CurlyTypeParam) ToCode(fn func(node MlgNodeKind) (string, bool)) string {
	if res, ok := fn(n); ok {
		return res
	}
	result := ""
	if n.CurlyTypeParams != nil {
		result += "{"
		result += commaSeparatedString(*n.CurlyTypeParams, fn)
		result += "}"
	}
	return result
}

func (n *CurlyArg) ToCode(fn func(node MlgNodeKind) (string, bool)) string {
	if res, ok := fn(n); ok {
		return res
	}
	result := "{"
	if n.CurlyArgs != nil {
		result += commaSeparatedString(*n.CurlyArgs, fn)
	}
	result += "}"
	return result
}

func (n *FunctionLiteralForm) ToCode(fn func(node MlgNodeKind) (string, bool)) string {
	if res, ok := fn(n); ok {
		return res
	}
	result := ""
	if len(n.Lhs.Params) == 1 {
		// render x |-> ... instead of (x) |-> ...
		result += n.Lhs.Params[0].ToCode(fn)
	} else {
		result += n.Lhs.ToCode(fn)
	}
	result += " |-> "
	result += n.Rhs.ToCode(fn)
	return result
}

func (n *CommandTypeForm) ToCode(fn func(node MlgNodeKind) (string, bool)) string {
	if res, ok := fn(n); ok {
		return res
	}
	result := "\\:"
	for i, n := range n.Names {
		if i > 0 {
			result += "."
		}
		result += n.ToCode(fn)
	}
	if n.CurlyTypeParam != nil {
		result += n.CurlyTypeParam.ToCode(fn)
	}
	if n.NamedTypeParams != nil {
		for _, item := range *n.NamedTypeParams {
			result += ":" + item.Name.ToCode(fn)
			if item.CurlyTypeParam != nil {
				result += item.CurlyTypeParam.ToCode(fn)
			}
		}
	}
	if n.ParenTypeParams != nil {
		result += "("
		result += commaSeparatedString(*n.ParenTypeParams, fn)
		result += ")"
	}
	return result
}

func (n *InfixCommandTypeForm) ToCode(fn func(node MlgNodeKind) (string, bool)) string {
	if res, ok := fn(n); ok {
		return res
	}

	result := "\\:"
	for i, n := range n.Names {
		if i > 0 {
			result += "."
		}
		result += n.ToCode(fn)
	}
	if n.CurlyTypeParam != nil {
		result += n.CurlyTypeParam.ToCode(fn)
	}
	if n.NamedTypeParams != nil {
		for _, item := range *n.NamedTypeParams {
			result += ":" + item.Name.ToCode(fn)
			if item.CurlyTypeParam != nil {
				result += item.CurlyTypeParam.ToCode(fn)
			}
		}
	}
	if n.ParenTypeParams != nil {
		result += "("
		result += commaSeparatedString(*n.ParenTypeParams, fn)
		result += ")"
	}
	result += ":/"
	return result
}

////////////////////////////////////////////////////////////////////////////////////////////////////

func commaSeparatedString[T FormulationNodeKind](forms []T,
	fn func(node MlgNodeKind) (string, bool)) string {
	return separatedString(forms, ", ", fn)
}

func commaSeparatedStringOfNameForms(forms []NameForm,
	fn func(node MlgNodeKind) (string, bool)) string {
	result := ""
	for i := range forms {
		if i > 0 {
			result += ", "
		}
		result += forms[i].ToCode(fn)
	}
	return result
}

func semicolonSeparatedString[T FormulationNodeKind](forms []T,
	fn func(node MlgNodeKind) (string, bool)) string {
	return separatedString(forms, "; ", fn)
}

func separatedString[T FormulationNodeKind](forms []T, separator string,
	fn func(node MlgNodeKind) (string, bool)) string {
	result := ""
	for i := range forms {
		if i > 0 {
			result += separator
		}
		result += forms[i].ToCode(fn)
	}
	return result
}

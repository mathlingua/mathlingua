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
)

// There are a couple scenarios where matching needs to occur:
// 1. Given a command expression, one needs to match the command's signature to a Describes
//    or Defines, determine if the expression matches the Describes/Defines input pattern,
//    and map the expression's argument's expressions to string names in the input pattern.
//    (This will be used for performing resolution of summaries to match their exact usage.)
// 2. Given an expression alias or a spec alias and an expression (assuming it has already been
//    determined, using type information, that the alias is what  should be used for the expression)
//    determine if the alias's  input pattern matches the expression, and map parts
//    of the expression to the names in the input pattern.
// 3. Given the left-hand-side, input  pattern, of an expression or spec alias, map the
//    names in the pattern to expression in the right-hand-side of the alias.
// Note: 2 and 3 will be used together to expand aliases inline in expressions.

type MatchResult struct {
	Mapping         map[string]ast.MlgNodeKind
	VarArgMapping   map[string][]ast.MlgNodeKind
	Messages        []string
	MatchMakesSense bool
}

func Match(node ast.MlgNodeKind, pattern ast.PatternKind) MatchResult {
	switch p := pattern.(type) {
	case *ast.FunctionFormPattern:
		return matchFunction(node, *p)
	case *ast.NameFormPattern:
		return matchName(node, *p)
	case *ast.TupleFormPattern:
		return matchTuple(node, *p)
	case *ast.ConditionalSetExpressionPattern:
		return matchConditionalSetExpression(node, *p)
	case *ast.ConditionalSetFormPattern:
		return matchConditionalSetForm(node, *p)
	case *ast.ConditionalSetIdFormPattern:
		return matchConditionalSetIdForm(node, *p)
	case *ast.InfixOperatorFormPattern:
		return matchInfixOperator(node, *p)
	case *ast.PrefixOperatorFormPattern:
		return matchPrefixOperator(node, *p)
	case *ast.PostfixOperatorFormPattern:
		return matchPostfixOperator(node, *p)
	case *ast.StructuralColonEqualsPattern:
		return matchStructuralColonEquals(node, *p)
	case *ast.InfixCommandOperatorPattern:
		return matchInfixOperatorCommand(node, *p)
	case *ast.ChainExpressionPattern:
		return matchChainExpression(node, *p)
	case *ast.InfixCommandPattern:
		return matchInfixCommandExpression(node, *p)
	case *ast.CommandPattern:
		return matchCommandExpression(node, *p)
	case *ast.SpecAliasPattern:
		return matchSpecAlias(node, *p)
	case *ast.AliasPattern:
		return matchAlias(node, *p)
	case *ast.OrdinalPattern:
		return matchOrdinal(node, *p)
	case *ast.FunctionLiteralFormPattern:
		return matchFunctionLiteralExpression(node, *p)
	default:
		return MatchResult{
			Messages: []string{
				"Could not match the given node",
			},
			MatchMakesSense: true,
		}
	}
}

////////////////////////////////////////////////////////////////////////////////////////////////////

func unionMatches(result1 MatchResult, result2 MatchResult) MatchResult {
	mapping := make(map[string]ast.MlgNodeKind)
	for name, node := range result1.Mapping {
		mapping[name] = node
	}
	for name, node := range result2.Mapping {
		mapping[name] = node
	}

	varArgMapping := make(map[string][]ast.MlgNodeKind)
	for name, nodes := range result1.VarArgMapping {
		varArgMapping[name] = nodes
	}
	for name, nodes := range result2.VarArgMapping {
		varArgMapping[name] = nodes
	}

	messages := make([]string, 0)
	messages = append(messages, result1.Messages...)
	messages = append(messages, result2.Messages...)

	matchMakesSense := result1.MatchMakesSense && result2.MatchMakesSense

	return MatchResult{
		Mapping:         mapping,
		VarArgMapping:   varArgMapping,
		Messages:        messages,
		MatchMakesSense: matchMakesSense,
	}
}

// TODO: add matchStructuralColonEqualsColon

func matchStructuralColonEquals(node ast.MlgNodeKind,
	pattern ast.StructuralColonEqualsPattern) MatchResult {
	switch n := node.(type) {
	case *ast.NameForm:
		return Match(n, pattern.Lhs)
	case *ast.FunctionForm:
		return Match(n, pattern.Lhs)
	case *ast.StructuralColonEqualsForm:
		name, ok := n.Lhs.(*ast.NameForm)
		if !ok {
			return MatchResult{
				MatchMakesSense: false,
			}
		}
		lhsMatch := Match(name, pattern.Lhs)
		rhsMatch := Match(n.Rhs, pattern.Rhs)
		return unionMatches(lhsMatch, rhsMatch)
	case *ast.ExpressionColonEqualsItem:
		name, ok := n.Lhs.(*ast.NameForm)
		if !ok {
			return MatchResult{
				MatchMakesSense: false,
			}
		}
		lhsMatch := Match(name, pattern.Lhs)
		rhsMatch := Match(n.Rhs, pattern.Rhs)
		return unionMatches(lhsMatch, rhsMatch)
	default:
		return MatchResult{
			Messages: []string{
				"Expected `... := ...`",
			},
			MatchMakesSense: true,
		}
	}
}

func matchName(node ast.MlgNodeKind, pattern ast.NameFormPattern) MatchResult {
	switch n := node.(type) {
	case *ast.ExpressionColonEqualsItem:
		mapping := make(map[string]ast.MlgNodeKind)
		// if the node is `X := ...` then the mapping maps to `X`
		mapping[pattern.Text] = n.Lhs
		return MatchResult{
			Mapping:         mapping,
			Messages:        []string{},
			MatchMakesSense: true,
		}
	default:
		mapping := make(map[string]ast.MlgNodeKind)
		mapping[pattern.Text] = n
		return MatchResult{
			Mapping:         mapping,
			Messages:        []string{},
			MatchMakesSense: true,
		}
	}
}

func matchVarArg(node ast.VarArgData, pattern ast.VarArgPatternData) MatchResult {
	if node.IsVarArg && !pattern.IsVarArg {
		return MatchResult{
			Mapping: make(map[string]ast.MlgNodeKind),
			Messages: []string{
				"Received a variadic node but did not expect one",
			},
			MatchMakesSense: true,
		}
	}

	if !node.IsVarArg && pattern.IsVarArg {
		return MatchResult{
			Mapping: make(map[string]ast.MlgNodeKind),
			Messages: []string{
				"Did not receive a variadic node but expected one",
			},
			MatchMakesSense: true,
		}
	}

	namesMatch := matchAllNames(node.VarArgNames, pattern.VarArgNames)
	boundsMatch := matchAllNames(node.VarArgBounds, pattern.VarArgBounds)
	return unionMatches(namesMatch, boundsMatch)
}

func matchFunction(node ast.MlgNodeKind, pattern ast.FunctionFormPattern) MatchResult {
	switch n := node.(type) {
	case *ast.FunctionForm:
		targetMatch := matchName(&n.Target, pattern.Target)
		paramsMatch := matchAllStructuralForms(n.Params, pattern.Params)
		varArgMatch := matchVarArg(n.VarArg, pattern.VarArg)
		return unionMatches(targetMatch, unionMatches(paramsMatch, varArgMatch))
	case *ast.FunctionCallExpression:
		targetMatch := Match(n.Target, &pattern.Target)
		argsMatch := matchAllExpressions(n.Args, pattern.Params)
		return unionMatches(targetMatch, argsMatch)
	case ast.ExpressionKind:
		mapping := make(map[string]ast.MlgNodeKind)
		mapping[pattern.Target.Text] = node
		return MatchResult{
			Mapping:         mapping,
			MatchMakesSense: true,
		}
	default:
		return MatchResult{
			Messages: []string{
				fmt.Sprintf("Expected a function call but found %s",
					ast.Debug(node, func(node ast.MlgNodeKind) (string, bool) {
						return "", false
					})),
			},
			MatchMakesSense: true,
		}
	}
}

func matchTuple(node ast.MlgNodeKind, pattern ast.TupleFormPattern) MatchResult {
	switch n := node.(type) {
	case *ast.TupleForm:
		paramsMatch := matchAllStructuralForms(n.Params, pattern.Params)
		varArgMatch := matchVarArg(n.VarArg, pattern.VarArg)
		return unionMatches(paramsMatch, varArgMatch)
	case *ast.TupleExpression:
		return matchAllExpressions(n.Args, pattern.Params)
	default:
		return MatchResult{
			Messages: []string{
				"Expected a tuple",
			},
			MatchMakesSense: true,
		}
	}
}

func matchConditionalSetForm(
	node ast.MlgNodeKind,
	pattern ast.ConditionalSetFormPattern,
) MatchResult {
	switch n := node.(type) {
	case *ast.ConditionalSetForm:
		targetMatch := Match(n.Target, pattern.Target)
		specMatch := Match(&n.Specification, &pattern.Specification)
		varArgMatch := matchVarArg(n.VarArg, pattern.VarArg)
		result := unionMatches(specMatch, unionMatches(targetMatch, varArgMatch))
		if n.Condition != nil {
			conditionMatch := Match(n.Condition, pattern.Condition)
			result = unionMatches(result, conditionMatch)
		}
		return result
	default:
		return MatchResult{
			Messages: []string{
				"Expected a set",
			},
			MatchMakesSense: true,
		}
	}
}

func matchFunctionLiteralExpression(
	node ast.MlgNodeKind,
	pattern ast.FunctionLiteralFormPattern,
) MatchResult {
	switch n := node.(type) {
	case *ast.FunctionLiteralExpression:
		lhsMatch := Match(&n.Lhs, &pattern.Lhs)
		rhsMatch := Match(n.Rhs, pattern.Rhs)
		return unionMatches(lhsMatch, rhsMatch)
	default:
		return MatchResult{
			Messages: []string{
				"Expected functional literal expression",
			},
			MatchMakesSense: true,
		}
	}
}

func matchConditionalSetIdForm(
	node ast.MlgNodeKind,
	pattern ast.ConditionalSetIdFormPattern,
) MatchResult {
	switch n := node.(type) {
	case *ast.ConditionalSetIdForm:
		targetMatch := Match(n.Target, pattern.Target)
		specMatch := matchFunction(&n.Specification, pattern.Specification)
		result := unionMatches(targetMatch, specMatch)
		if n.Condition != nil && pattern.Condition != nil {
			result = unionMatches(result, matchFunction(n.Condition, *pattern.Condition))
		}
		return result
	case *ast.ConditionalSetExpression:
		targetMatch := Match(n.Target, pattern.Target)
		specsMatch := matchAllExpressions(n.Specifications, []ast.FormPatternKind{
			&pattern.Specification,
		})
		result := unionMatches(targetMatch, specsMatch)
		if n.Condition != nil {
			conditionMatch := Match(n.Condition, pattern.Condition)
			result = unionMatches(result, conditionMatch)
		}
		return result
	default:
		return MatchResult{
			Messages: []string{
				"Expected a set",
			},
			MatchMakesSense: true,
		}
	}
}

func matchConditionalSetExpression(
	node ast.MlgNodeKind,
	pattern ast.ConditionalSetExpressionPattern,
) MatchResult {
	switch n := node.(type) {
	case *ast.ConditionalSetExpression:
		targetMatch := Match(n.Target, pattern.Target)
		specsMatch := matchAllExpressions(n.Specifications, pattern.Specifications)
		result := unionMatches(targetMatch, specsMatch)
		if n.Condition != nil && pattern.Condition != nil {
			conditionMatch := Match(n.Condition, *pattern.Condition)
			result = unionMatches(result, conditionMatch)
		}
		return result
	default:
		return MatchResult{
			Messages: []string{
				"Expected a set",
			},
			MatchMakesSense: true,
		}
	}
}

func matchInfixOperator(node ast.MlgNodeKind, pattern ast.InfixOperatorFormPattern) MatchResult {
	switch n := node.(type) {
	case *ast.InfixOperatorCallExpression:
		lhsMatch := Match(n.Lhs, pattern.Lhs)
		opMatch := Match(n.Target, &pattern.Operator)
		rhsMatch := Match(n.Rhs, pattern.Rhs)
		return unionMatches(lhsMatch, unionMatches(opMatch, rhsMatch))
	case *ast.InfixOperatorForm:
		lhsMatch := Match(n.Lhs, pattern.Lhs)
		opMatch := Match(&n.Operator, &pattern.Operator)
		rhsMatch := Match(n.Rhs, pattern.Rhs)
		return unionMatches(lhsMatch, unionMatches(opMatch, rhsMatch))
	case *ast.InfixOperatorId:
		lhsMatch := Match(n.Lhs, pattern.Lhs)
		opMatch := Match(&n.Operator, &pattern.Operator)
		rhsMatch := Match(n.Rhs, pattern.Rhs)
		return unionMatches(lhsMatch, unionMatches(opMatch, rhsMatch))
	default:
		return MatchResult{
			Messages: []string{
				"Expected an infix operator",
			},
			MatchMakesSense: true,
		}
	}
}

func matchPrefixOperator(node ast.MlgNodeKind, pattern ast.PrefixOperatorFormPattern) MatchResult {
	switch n := node.(type) {
	case *ast.PrefixOperatorCallExpression:
		opMatch := Match(n.Target, &pattern.Operator)
		argMatch := Match(n.Arg, pattern.Param)
		return unionMatches(opMatch, argMatch)
	case *ast.PrefixOperatorForm:
		opMatch := Match(&n.Operator, &pattern.Operator)
		argMatch := Match(n.Param, pattern.Param)
		return unionMatches(opMatch, argMatch)
	case *ast.PrefixOperatorId:
		opMatch := Match(&n.Operator, &pattern.Operator)
		argMatch := Match(n.Param, pattern.Param)
		return unionMatches(opMatch, argMatch)
	default:
		return MatchResult{
			Messages: []string{
				"Expected a prefix operator",
			},
			MatchMakesSense: true,
		}
	}
}

func matchPostfixOperator(
	node ast.MlgNodeKind,
	pattern ast.PostfixOperatorFormPattern,
) MatchResult {
	switch n := node.(type) {
	case *ast.PostfixOperatorCallExpression:
		opMatch := Match(n.Target, &pattern.Operator)
		argMatch := Match(n.Arg, pattern.Param)
		return unionMatches(opMatch, argMatch)
	case *ast.PostfixOperatorForm:
		opMatch := Match(&n.Operator, &pattern.Operator)
		argMatch := Match(n.Param, pattern.Param)
		return unionMatches(opMatch, argMatch)
	case *ast.PostfixOperatorId:
		opMatch := Match(&n.Operator, &pattern.Operator)
		argMatch := Match(n.Param, pattern.Param)
		return unionMatches(opMatch, argMatch)
	default:
		return MatchResult{
			Messages: []string{
				"Expected a postfix operator",
			},
			MatchMakesSense: true,
		}
	}
}

func matchInfixOperatorCommand(
	node ast.MlgNodeKind,
	pattern ast.InfixCommandOperatorPattern,
) MatchResult {
	switch n := node.(type) {
	case *ast.InfixCommandOperatorId:
		lhsMatch := Match(n.Lhs, pattern.Lhs)
		opMatch := matchInfixCommandExpression(&n.Operator, pattern.Operator)
		rhsMatch := Match(n.Rhs, pattern.Rhs)
		return unionMatches(lhsMatch, unionMatches(opMatch, rhsMatch))
	default:
		return MatchResult{
			Messages: []string{
				"Expected an infix command operator",
			},
			MatchMakesSense: true,
		}
	}
}

func matchInfixCommandExpression(
	node ast.MlgNodeKind,
	pattern ast.InfixCommandPattern,
) MatchResult {
	switch n := node.(type) {
	case *ast.InfixCommandExpression:
		nodeSig := GetSignatureStringFromInfixCommand(*n)
		if nodeSig != pattern.Signature {
			return MatchResult{
				MatchMakesSense: false,
			}
		}
		namesMatch := matchAllNames(n.Names, pattern.Names)
		curlyMatch := matchCurlyArg(n.CurlyArg, pattern.CurlyArg)
		namedArgsMatch := matchAllNamedArgs(n.NamedArgs, pattern.NamedGroups)
		parensMatch := matchAllOptionalExpressionsToNames(n.ParenArgs, pattern.ParenArgs)
		return unionMatches(namesMatch,
			unionMatches(curlyMatch,
				unionMatches(namedArgsMatch, parensMatch)))
	case *ast.InfixCommandId:
		namesMatch := matchAllNames(n.Names, pattern.Names)
		curlyMatch := matchCurlyParam(n.CurlyParam, pattern.CurlyArg)
		namedArgsMatch := matchAllNamedParams(n.NamedParams, pattern.NamedGroups)
		parensMatch := matchAllOptionalNames(n.ParenParams, pattern.ParenArgs)
		return unionMatches(namesMatch,
			unionMatches(curlyMatch,
				unionMatches(namedArgsMatch, parensMatch)))
	default:
		return MatchResult{
			Messages: []string{
				"Expected an infix command",
			},
			MatchMakesSense: true,
		}
	}
}

func matchCommandExpression(node ast.MlgNodeKind, pattern ast.CommandPattern) MatchResult {
	switch n := node.(type) {
	case *ast.CommandExpression:
		nodeSig := GetSignatureStringFromCommand(*n)
		if nodeSig != pattern.Signature {
			return MatchResult{
				MatchMakesSense: false,
			}
		}
		namesMatch := matchAllNames(n.Names, pattern.Names)
		curlyMatch := matchCurlyArg(n.CurlyArg, pattern.CurlyArg)
		namedArgsMatch := matchAllNamedArgs(n.NamedArgs, pattern.NamedGroups)
		parensMatch := matchAllOptionalExpressionsToNames(n.ParenArgs, pattern.ParenArgs)
		return unionMatches(namesMatch,
			unionMatches(curlyMatch,
				unionMatches(namedArgsMatch, parensMatch)))
	case *ast.CommandId:
		namesMatch := matchAllNames(n.Names, pattern.Names)
		curlyMatch := matchCurlyParam(n.CurlyParam, pattern.CurlyArg)
		namedArgsMatch := matchAllNamedParams(n.NamedParams, pattern.NamedGroups)
		parensMatch := matchAllOptionalNames(n.ParenParams, pattern.ParenArgs)
		return unionMatches(namesMatch,
			unionMatches(curlyMatch,
				unionMatches(namedArgsMatch, parensMatch)))
	default:
		return MatchResult{
			Messages: []string{
				"Expected a command",
			},
			MatchMakesSense: true,
		}
	}
}

func matchCurlyArg(node *ast.CurlyArg, pattern *ast.CurlyPattern) MatchResult {
	if node == nil || pattern == nil {
		if node == nil && pattern == nil {
			return MatchResult{
				Mapping:         make(map[string]ast.MlgNodeKind),
				Messages:        make([]string, 0),
				MatchMakesSense: true,
			}
		} else if node == nil && pattern != nil {
			return MatchResult{
				Mapping: make(map[string]ast.MlgNodeKind),
				Messages: []string{
					"Expected a {} argument but found none",
				},
				MatchMakesSense: true,
			}
		} else if node != nil && pattern == nil {
			return MatchResult{
				Mapping: make(map[string]ast.MlgNodeKind),
				Messages: []string{
					"Did not expect to find a {} but found one",
				},
				MatchMakesSense: true,
			}
		}
	}

	argsMatch := matchAllOptionalExpressionsToForms(node.CurlyArgs, pattern.CurlyArgs)
	directionMatch := matchDirection(node.Direction, pattern.Direction)

	return unionMatches(argsMatch, directionMatch)
}

func matchOrdinal(node ast.MlgNodeKind, pattern ast.OrdinalPattern) MatchResult {
	switch n := node.(type) {
	case *ast.OrdinalCallExpression:
		targetMatch := Match(n.Target, &pattern)
		argsMatch := matchAllExpressionsAsNames(n.Args, pattern.Params)
		return unionMatches(targetMatch, argsMatch)
	default:
		return MatchResult{
			Messages: []string{
				"Expected an ordinal",
			},
			MatchMakesSense: true,
		}
	}
}

func matchAllDirectionParamParamKind(
	nodes []ast.DirectionParamParamKind,
	patterns []ast.DirectionParamParamPatternKind,
) MatchResult {
	if len(nodes) != len(patterns) {
		return MatchResult{
			Mapping: make(map[string]ast.MlgNodeKind),
			Messages: []string{
				fmt.Sprintf("Expected %d values but found %d: Received: %s",
					len(patterns), len(nodes), sliceToString(nodes)),
			},
			MatchMakesSense: true,
		}
	}

	if len(nodes) == 0 {
		return MatchResult{
			MatchMakesSense: true,
		}
	}

	result := matchDirectionParamParamKind(nodes[0], patterns[0])
	for i := 1; i < len(nodes); i++ {
		result = unionMatches(result, matchDirectionParamParamKind(nodes[i], patterns[i]))
	}
	return result
}

func matchDirectionParamParamKind(
	node ast.DirectionParamParamKind,
	pattern ast.DirectionParamParamPatternKind,
) MatchResult {
	switch p := pattern.(type) {
	case *ast.NameFormPattern:
		return matchName(node, *p)
	case *ast.FunctionFormPattern:
		return matchFunction(node, *p)
	case *ast.OrdinalPattern:
		return matchOrdinal(node, *p)
	default:
		return MatchResult{
			Messages: []string{
				"Expected a direction",
			},
			MatchMakesSense: true,
		}
	}
}

func matchDirection(node *ast.DirectionalParam, pattern *ast.DirectionPattern) MatchResult {
	if node == nil || pattern == nil {
		if node == nil && pattern != nil {
			return MatchResult{
				Mapping: make(map[string]ast.MlgNodeKind),
				Messages: []string{
					"Expected a direction but didn't receive one",
				},
				MatchMakesSense: true,
			}
		}

		if node != nil && pattern == nil {
			return MatchResult{
				Mapping: make(map[string]ast.MlgNodeKind),
				Messages: []string{
					"Did not expect a direction but received one",
				},
				MatchMakesSense: true,
			}
		}

		if node == nil && pattern == nil {
			return MatchResult{
				Mapping:         make(map[string]ast.MlgNodeKind),
				Messages:        []string{},
				MatchMakesSense: true,
			}
		}
	}

	nodeName := node.Name
	patternName := pattern.Name

	if nodeName == nil || patternName == nil {
		if nodeName == nil && patternName != nil {
			return MatchResult{
				Mapping: make(map[string]ast.MlgNodeKind),
				Messages: []string{
					"Expected a direction name but didn't receive one",
				},
				MatchMakesSense: true,
			}
		}

		if nodeName != nil && patternName == nil {
			return MatchResult{
				Mapping: make(map[string]ast.MlgNodeKind),
				Messages: []string{
					"Did not expect a direction name but received one",
				},
				MatchMakesSense: true,
			}
		}

		if node == nil && pattern == nil {
			return MatchResult{
				Mapping:         make(map[string]ast.MlgNodeKind),
				Messages:        []string{},
				MatchMakesSense: true,
			}
		}
	}

	nameMatch := matchName(nodeName, *patternName)
	squareMatch := matchAllDirectionParamParamKind(node.CurlyParams, pattern.CurlyArgs)

	return unionMatches(nameMatch, squareMatch)
}

func matchCurlyParam(node *ast.CurlyParam, pattern *ast.CurlyPattern) MatchResult {
	if node == nil || pattern == nil {
		if node == nil && pattern == nil {
			return MatchResult{
				Mapping:         make(map[string]ast.MlgNodeKind),
				Messages:        make([]string, 0),
				MatchMakesSense: true,
			}
		} else if node == nil && pattern != nil {
			return MatchResult{
				Mapping: make(map[string]ast.MlgNodeKind),
				Messages: []string{
					"Expected a {} argument but found none",
				},
				MatchMakesSense: true,
			}
		} else if node != nil && pattern == nil {
			return MatchResult{
				Mapping: make(map[string]ast.MlgNodeKind),
				Messages: []string{
					"Did not expect to find a {} but found one",
				},
				MatchMakesSense: true,
			}
		}
	}

	curlyMatch := matchAllOptionalStructuralForms(node.CurlyParams, pattern.CurlyArgs)
	directionMatch := matchDirection(node.Direction, pattern.Direction)

	return unionMatches(curlyMatch, directionMatch)
}

func matchNamedGroup(node ast.MlgNodeKind, pattern ast.NamedGroupPattern) MatchResult {
	switch n := node.(type) {
	case *ast.NamedArg:
		nameMatch := matchName(&n.Name, pattern.Name)
		curlyMatch := matchCurlyArg(n.CurlyArg, &pattern.Curly)
		return unionMatches(nameMatch, curlyMatch)
	case *ast.NamedParam:
		nameMatch := matchName(&n.Name, pattern.Name)
		curlyMatch := matchCurlyParam(n.CurlyParam, &pattern.Curly)
		return unionMatches(nameMatch, curlyMatch)
	default:
		return MatchResult{
			Messages: []string{
				"Expected a named group",
			},
			MatchMakesSense: true,
		}
	}
}

func matchChainExpression(node ast.MlgNodeKind, pattern ast.ChainExpressionPattern) MatchResult {
	switch n := node.(type) {
	case *ast.ChainExpression:
		return matchAllExpressions(n.Parts, pattern.Parts)
	default:
		return MatchResult{
			Messages: []string{
				"Expected a chain expression",
			},
			MatchMakesSense: true,
		}
	}
}

func matchSpecAlias(node ast.MlgNodeKind, pattern ast.SpecAliasPattern) MatchResult {
	switch node.(type) {
	case *ast.ExpressionColonDashArrowItem:
		return MatchResult{}
	default:
		return MatchResult{
			MatchMakesSense: false,
		}
	}
}

func matchAlias(node ast.MlgNodeKind, pattern ast.AliasPattern) MatchResult {
	switch node.(type) {
	case *ast.ExpressionColonArrowItem:
		return MatchResult{}
	default:
		return MatchResult{
			MatchMakesSense: false,
		}
	}
}

////////////////////////////////////////////////////////////////////////////////////////////////////

// if the `error != nil` then the variadic pattern description is invalid
// otherwise if `error == nil` then the bool return value is true if and
// only if the patterns slice has one pattern and it is variadic
func checkPatternsForVarArg(patterns []ast.PatternKind) (error, bool) {
	numWithVarArg := 0
	for _, pattern := range patterns {
		if pattern.GetVarArgData().IsVarArg {
			numWithVarArg += 1
		}
	}

	if numWithVarArg > 1 {
		return fmt.Errorf("At most one variadic parameter can be specified"), false
	}

	if numWithVarArg > 0 && len(patterns) != 1 {
		return fmt.Errorf(
			"If a variadic parameter is specified, it must be the only parameter specified"), false
	}

	return nil, numWithVarArg == 1
}

func checkNameFormPatternsForVarArg(
	nodes []ast.ExpressionKind,
	patterns []ast.NameFormPattern,
) *MatchResult {
	generalPatterns := make([]ast.PatternKind, 0)
	for _, pattern := range patterns {
		generalPatterns = append(generalPatterns, &pattern)
	}
	err, ok := checkPatternsForVarArg(generalPatterns)
	if err != nil {
		return &MatchResult{
			Messages:        []string{err.Error()},
			MatchMakesSense: true,
		}
	}

	if ok {
		varArgMapping := make(map[string][]ast.MlgNodeKind)
		values := make([]ast.MlgNodeKind, 0)
		for _, name := range nodes {
			values = append(values, name)
		}
		varArgMapping[patterns[0].Text] = values
		return &MatchResult{
			Mapping:         make(map[string]ast.MlgNodeKind),
			VarArgMapping:   varArgMapping,
			Messages:        []string{},
			MatchMakesSense: true,
		}
	}

	return nil
}

func checkExpressionFormPatternsForVarArg(
	nodes []ast.ExpressionKind,
	patterns []ast.FormPatternKind,
) *MatchResult {
	generalPatterns := make([]ast.PatternKind, 0)
	for _, pattern := range patterns {
		generalPatterns = append(generalPatterns, pattern)
	}

	err, ok := checkPatternsForVarArg(generalPatterns)
	if err != nil {
		return &MatchResult{
			Messages:        []string{err.Error()},
			MatchMakesSense: true,
		}
	}

	if ok {
		first := patterns[0]
		varArgMapping := make(map[string][]ast.MlgNodeKind)
		values := make([]ast.MlgNodeKind, 0)
		for _, n := range nodes {
			values = append(values, n)
		}
		if f, ok := first.(*ast.NameFormPattern); ok {
			varArgMapping[f.Text] = values
			return &MatchResult{
				Mapping:         make(map[string]ast.MlgNodeKind),
				VarArgMapping:   varArgMapping,
				Messages:        []string{},
				MatchMakesSense: true,
			}
		} else if f, ok := first.(*ast.FunctionFormPattern); ok {
			varArgMapping[f.Target.Text] = values
			return &MatchResult{
				Mapping:         make(map[string]ast.MlgNodeKind),
				VarArgMapping:   varArgMapping,
				Messages:        []string{},
				MatchMakesSense: true,
			}
		} else {
			return &MatchResult{
				Messages:        []string{"Only a name or function can be variadic"},
				MatchMakesSense: true,
			}
		}
	}

	return nil
}

////////////////////////////////////////////////////////////////////////////////////////////////////

func matchAllOptionalNames(nodes *[]ast.NameForm, patterns *[]ast.NameFormPattern) MatchResult {
	if nodes == nil {
		zeroNodes := make([]ast.NameForm, 0)
		nodes = &zeroNodes
	}

	if patterns == nil {
		zeroPatterns := make([]ast.NameFormPattern, 0)
		patterns = &zeroPatterns
	}

	return matchAllNames(*nodes, *patterns)
}

func matchAllNames(nodes []ast.NameForm, patterns []ast.NameFormPattern) MatchResult {
	numWithVarArg := 0
	for _, pattern := range patterns {
		if pattern.VarArg.IsVarArg {
			numWithVarArg += 1
		}
	}

	if numWithVarArg > 1 {
		return MatchResult{
			Mapping: make(map[string]ast.MlgNodeKind),
			Messages: []string{
				"At most one variadic parameter can be specified",
			},
			MatchMakesSense: true,
		}
	}

	if numWithVarArg > 0 && len(patterns) != 1 {
		return MatchResult{
			Mapping: make(map[string]ast.MlgNodeKind),
			Messages: []string{
				"If a variadic parameter is specified, it must be the only parameter specified",
			},
			MatchMakesSense: true,
		}
	}

	if numWithVarArg == 1 {
		varArgMapping := make(map[string][]ast.MlgNodeKind)
		values := make([]ast.MlgNodeKind, 0)
		for _, name := range nodes {
			values = append(values, &name)
		}
		varArgMapping[patterns[0].Text] = values
		return MatchResult{
			Mapping:         make(map[string]ast.MlgNodeKind),
			VarArgMapping:   varArgMapping,
			Messages:        []string{},
			MatchMakesSense: true,
		}
	}

	if len(nodes) != len(patterns) {
		return MatchResult{
			Mapping: make(map[string]ast.MlgNodeKind),
			Messages: []string{
				fmt.Sprintf("Expected %d values but found %d: Received: %s",
					len(patterns), len(nodes), nameFormSliceToString(nodes)),
			},
			MatchMakesSense: true,
		}
	}

	if len(nodes) == 0 {
		return MatchResult{
			MatchMakesSense: true,
		}
	}

	result := Match(&nodes[0], &patterns[0])
	for i := 1; i < len(nodes); i++ {
		result = unionMatches(result, Match(&nodes[i], &patterns[i]))
	}
	return result
}

func matchAllExpressionsAsNames(
	nodes []ast.ExpressionKind,
	patterns []ast.NameFormPattern,
) MatchResult {
	res := checkNameFormPatternsForVarArg(nodes, patterns)
	if res != nil {
		return *res
	}

	if len(nodes) != len(patterns) {
		return MatchResult{
			Mapping: make(map[string]ast.MlgNodeKind),
			Messages: []string{
				fmt.Sprintf("Expected %d values but found %d: Received: %s",
					len(patterns), len(nodes), sliceToString(nodes)),
			},
			MatchMakesSense: true,
		}
	}

	if len(nodes) == 0 {
		return MatchResult{
			MatchMakesSense: true,
		}
	}

	result := Match(nodes[0], &patterns[0])
	for i := 1; i < len(nodes); i++ {
		result = unionMatches(result, Match(nodes[i], &patterns[i]))
	}
	return result
}

func matchAllOptionalExpressionsToNames(
	nodes *[]ast.ExpressionKind,
	patterns *[]ast.NameFormPattern,
) MatchResult {
	if nodes == nil {
		zeroNodes := make([]ast.ExpressionKind, 0)
		nodes = &zeroNodes
	}

	if patterns == nil {
		zeroPatterns := make([]ast.NameFormPattern, 0)
		patterns = &zeroPatterns
	}

	if len(*nodes) != len(*patterns) {
		return MatchResult{
			Mapping: make(map[string]ast.MlgNodeKind),
			Messages: []string{
				fmt.Sprintf("Expected %d values but found %d: Received: %s",
					len(*patterns), len(*nodes), sliceToString(*nodes)),
			},
			MatchMakesSense: true,
		}
	}

	if len(*nodes) == 0 {
		return MatchResult{
			MatchMakesSense: true,
		}
	}

	nodesValue := *nodes
	patternsValue := *patterns
	result := Match(nodesValue[0], &patternsValue[0])
	for i := 1; i < len(*nodes); i++ {
		result = unionMatches(result, Match(nodesValue[i], &patternsValue[i]))
	}
	return result
}

func matchAllOptionalExpressionsToForms(
	nodes *[]ast.ExpressionKind,
	patterns *[]ast.FormPatternKind,
) MatchResult {
	if nodes == nil {
		zeroNodes := make([]ast.ExpressionKind, 0)
		nodes = &zeroNodes
	}

	if patterns == nil {
		zeroPatterns := make([]ast.FormPatternKind, 0)
		patterns = &zeroPatterns
	}

	res := checkExpressionFormPatternsForVarArg(*nodes, *patterns)
	if res != nil {
		return *res
	}

	if len(*nodes) != len(*patterns) {
		return MatchResult{
			Mapping: make(map[string]ast.MlgNodeKind),
			Messages: []string{
				fmt.Sprintf("Expected %d values but found %d: Received: %s",
					len(*patterns), len(*nodes), sliceToString(*nodes)),
			},
			MatchMakesSense: true,
		}
	}

	if len(*nodes) == 0 {
		return MatchResult{
			MatchMakesSense: true,
		}
	}

	return matchAllExpressions(*nodes, *patterns)
}

func matchAllExpressions(nodes []ast.ExpressionKind, patterns []ast.FormPatternKind) MatchResult {
	res := checkExpressionFormPatternsForVarArg(nodes, patterns)
	if res != nil {
		return *res
	}

	if len(nodes) != len(patterns) {
		return MatchResult{
			Mapping: make(map[string]ast.MlgNodeKind),
			Messages: []string{
				fmt.Sprintf("Expected %d values but found %d: Received: %s",
					len(patterns), len(nodes), sliceToString(nodes)),
			},
			MatchMakesSense: true,
		}
	}

	if len(nodes) == 0 {
		return MatchResult{
			MatchMakesSense: true,
		}
	}

	result := Match(nodes[0], patterns[0])
	for i := 1; i < len(nodes); i++ {
		result = unionMatches(result, Match(nodes[i], patterns[i]))
	}
	return result
}

func matchAllOptionalStructuralForms(
	nodes *[]ast.StructuralFormKind,
	patterns *[]ast.FormPatternKind,
) MatchResult {
	if nodes == nil {
		zeroNodes := make([]ast.StructuralFormKind, 0)
		nodes = &zeroNodes
	}

	if patterns == nil {
		zeroPatterns := make([]ast.FormPatternKind, 0)
		patterns = &zeroPatterns
	}

	return matchAllStructuralForms(*nodes, *patterns)
}

func matchAllStructuralForms(
	nodes []ast.StructuralFormKind,
	patterns []ast.FormPatternKind,
) MatchResult {
	if len(nodes) != len(patterns) {
		return MatchResult{
			Mapping: make(map[string]ast.MlgNodeKind),
			Messages: []string{
				fmt.Sprintf("Expected %d values but found %d: Received: %s",
					len(patterns), len(nodes), sliceToString(nodes)),
			},
			MatchMakesSense: true,
		}
	}

	if len(nodes) == 0 {
		return MatchResult{
			MatchMakesSense: true,
		}
	}

	result := Match(nodes[0], patterns[0])
	for i := 1; i < len(nodes); i++ {
		result = unionMatches(result, Match(nodes[i], patterns[i]))
	}
	return result
}

func matchAllNamedArgs(nodes *[]ast.NamedArg, patterns *[]ast.NamedGroupPattern) MatchResult {
	if nodes == nil {
		zeroNodes := make([]ast.NamedArg, 0)
		nodes = &zeroNodes
	}

	if patterns == nil {
		zeroPatterns := make([]ast.NamedGroupPattern, 0)
		patterns = &zeroPatterns
	}

	if len(*nodes) != len(*patterns) {
		return MatchResult{
			Mapping: make(map[string]ast.MlgNodeKind),
			Messages: []string{
				fmt.Sprintf("Expected %d values but found %d: Received: %s",
					len(*patterns), len(*nodes), namedArgSliceToString(*nodes)),
			},
			MatchMakesSense: true,
		}
	}

	if len(*nodes) == 0 {
		return MatchResult{
			MatchMakesSense: true,
		}
	}

	nodesValue := *nodes
	patternsValue := *patterns
	result := matchNamedGroup(&nodesValue[0], patternsValue[0])
	for i := 1; i < len(*nodes); i++ {
		result = unionMatches(result, matchNamedGroup(&nodesValue[i], patternsValue[i]))
	}
	return result
}

func matchAllNamedParams(nodes *[]ast.NamedParam, patterns *[]ast.NamedGroupPattern) MatchResult {
	if nodes == nil {
		zeroNodes := make([]ast.NamedParam, 0)
		nodes = &zeroNodes
	}

	if patterns == nil {
		zeroPatterns := make([]ast.NamedGroupPattern, 0)
		patterns = &zeroPatterns
	}

	if len(*nodes) != len(*patterns) {
		return MatchResult{
			Mapping: make(map[string]ast.MlgNodeKind),
			Messages: []string{
				fmt.Sprintf("Expected %d values but found %d: Received: %s",
					len(*patterns), len(*nodes), namedParamSliceToString(*nodes)),
			},
			MatchMakesSense: true,
		}
	}

	if len(*nodes) == 0 {
		return MatchResult{
			MatchMakesSense: true,
		}
	}

	nodesValue := *nodes
	patternsValue := *patterns
	result := Match(&nodesValue[0], &patternsValue[0])
	for i := 1; i < len(*nodes); i++ {
		result = unionMatches(result, Match(&nodesValue[i], &patternsValue[i]))
	}
	return result
}

// nolint:unused
func matchDirectionParams(
	nodes *[]ast.DirectionParamParamKind,
	patterns *[]ast.FormPatternKind,
) MatchResult {
	if nodes == nil {
		zeroNodes := make([]ast.DirectionParamParamKind, 0)
		nodes = &zeroNodes
	}

	if patterns == nil {
		zeroPatterns := make([]ast.FormPatternKind, 0)
		patterns = &zeroPatterns
	}

	if len(*nodes) != len(*patterns) {
		return MatchResult{
			Mapping: make(map[string]ast.MlgNodeKind),
			Messages: []string{
				fmt.Sprintf("Expected %d values but found %d: Received: %s",
					len(*patterns), len(*nodes), sliceToString(*nodes)),
			},
			MatchMakesSense: true,
		}
	}

	if len(*nodes) == 0 {
		return MatchResult{
			MatchMakesSense: true,
		}
	}

	nodesValue := *nodes
	patternsValue := *patterns
	result := Match(nodesValue[0], patternsValue[0])
	for i := 1; i < len(*nodes); i++ {
		result = unionMatches(result, Match(nodesValue[i], patternsValue[i]))
	}
	return result
}

func sliceToString[T ast.FormulationNodeKind](nodes []T) string {
	result := ""
	for i, n := range nodes {
		if i > 0 {
			result += ", "
		}
		result += n.ToCode(ast.NoOp)
	}
	return result
}

func namedParamSliceToString(nodes []ast.NamedParam) string {
	result := ""
	for i, n := range nodes {
		if i > 0 {
			result += ", "
		}
		result += n.Name.Text
	}
	return result
}

func namedArgSliceToString(nodes []ast.NamedArg) string {
	result := ""
	for i, n := range nodes {
		if i > 0 {
			result += ", "
		}
		result += n.Name.Text
	}
	return result
}

func nameFormSliceToString(nodes []ast.NameForm) string {
	result := ""
	for i, n := range nodes {
		if i > 0 {
			result += ", "
		}
		result += n.Text
	}
	return result
}

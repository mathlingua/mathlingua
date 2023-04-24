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
	Mapping         map[string]ast.MlgNodeType
	Messages        []string
	MatchMakesSense bool
}

func Match(node ast.MlgNodeType, pattern PatternType) MatchResult {
	switch p := pattern.(type) {
	case FunctionFormPattern:
		return matchFunction(node, p)
	case NameFormPattern:
		return matchName(node, p)
	case TupleFormPattern:
		return matchTuple(node, p)
	case ConditionalSetExpressionPattern:
		return matchConditionalSetExpression(node, p)
	case ConditionalSetFormPattern:
		return matchConditionalSetForm(node, p)
	case InfixOperatorFormPattern:
		return matchInfixOperator(node, p)
	case PrefixOperatorFormPattern:
		return matchPrefixOperator(node, p)
	case PostfixOperatorFormPattern:
		return matchPostfixOperator(node, p)
	case NameColonEqualsPatternPattern:
		return matchNameColonEquals(node, p)
	case FunctionColonEqualsNamePattern:
		return matchFunctionColonEquals(node, p)
	case InfixCommandOperatorPattern:
		return matchInfixOperatorCommand(node, p)
	case ChainExpressionPattern:
		return matchChainExpression(node, p)
	case InfixCommandTargetPattern:
		return matchInfixCommandTarget(node, p)
	case CommandPattern:
		return matchCommand(node, p)
	case SpecAliasPattern:
		return matchSpecAlias(node, p)
	case AliasPattern:
		return matchAlias(node, p)
	case OrdinalPattern:
		return matchOrdinal(node, p)
	default:
		return MatchResult{
			Messages: []string{
				"Could not match the given node",
			},
			MatchMakesSense: true,
		}
	}
}

func unionMatches(result1 MatchResult, result2 MatchResult) MatchResult {
	mapping := make(map[string]ast.MlgNodeType)
	for name, node := range result1.Mapping {
		mapping[name] = node
	}
	for name, node := range result2.Mapping {
		mapping[name] = node
	}

	messages := make([]string, 0)
	messages = append(messages, result1.Messages...)
	messages = append(messages, result2.Messages...)

	matchMakesSense := result1.MatchMakesSense && result2.MatchMakesSense

	return MatchResult{
		Mapping:         mapping,
		Messages:        messages,
		MatchMakesSense: matchMakesSense,
	}
}

func matchNameColonEquals(node ast.MlgNodeType,
	pattern NameColonEqualsPatternPattern) MatchResult {
	switch n := node.(type) {
	case *ast.StructuralColonEqualsForm:
		name, ok := n.Lhs.(*ast.NameForm)
		if !ok {
			return MatchResult{
				MatchMakesSense: false,
			}
		}
		lhsMatch := matchName(name, pattern.Lhs)
		rhsMatch := Match(n.Rhs, pattern.Rhs)
		return unionMatches(lhsMatch, rhsMatch)
	case *ast.ExpressionColonEqualsItem:
		name, ok := n.Lhs.(*ast.NameForm)
		if !ok {
			return MatchResult{
				MatchMakesSense: false,
			}
		}
		lhsMatch := matchName(name, pattern.Lhs)
		rhsMatch := Match(n.Rhs, pattern.Rhs)
		return unionMatches(lhsMatch, rhsMatch)
	default:
		return MatchResult{
			Messages: []string{
				"Expected `X :=`",
			},
			MatchMakesSense: true,
		}
	}
}

func matchFunctionColonEquals(node ast.MlgNodeType,
	pattern FunctionColonEqualsNamePattern) MatchResult {
	switch n := node.(type) {
	case *ast.StructuralColonEqualsForm:
		fn, ok := n.Lhs.(*ast.FunctionForm)
		if !ok {
			return MatchResult{
				MatchMakesSense: false,
			}
		}
		lhsMatch := matchFunction(fn, pattern.Lhs)
		rhsMatch := Match(n.Rhs, pattern.Rhs)
		return unionMatches(lhsMatch, rhsMatch)
	case *ast.ExpressionColonEqualsItem:
		fn, ok := n.Lhs.(*ast.FunctionCallExpression)
		if !ok {
			return MatchResult{
				MatchMakesSense: false,
			}
		}
		lhsMatch := matchFunction(fn, pattern.Lhs)
		rhsMatch := Match(n.Rhs, pattern.Rhs)
		return unionMatches(lhsMatch, rhsMatch)
	default:
		return MatchResult{
			Messages: []string{
				"Expected a `f(x) :=`",
			},
			MatchMakesSense: true,
		}
	}
}

func matchName(node ast.MlgNodeType, pattern NameFormPattern) MatchResult {
	switch n := node.(type) {
	case *ast.NameForm:
		mapping := make(map[string]ast.MlgNodeType)
		mapping[pattern.Text] = n
		return MatchResult{
			Mapping:         mapping,
			Messages:        []string{},
			MatchMakesSense: true,
		}
	default:
		return MatchResult{
			Messages: []string{
				"Expected a name",
			},
			MatchMakesSense: true,
		}
	}
}

func matchVarArg(node ast.VarArgData, pattern VarArgPatternData) MatchResult {
	if node.IsVarArg && !pattern.IsVarArg {
		return MatchResult{
			Mapping: make(map[string]ast.MlgNodeType),
			Messages: []string{
				"Received a variadic node but did not expect one",
			},
			MatchMakesSense: true,
		}
	}

	if !node.IsVarArg && pattern.IsVarArg {
		return MatchResult{
			Mapping: make(map[string]ast.MlgNodeType),
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

func matchFunction(node ast.MlgNodeType, pattern FunctionFormPattern) MatchResult {
	switch n := node.(type) {
	case *ast.FunctionForm:
		targetMatch := matchName(&n.Target, pattern.Target)
		paramsMatch := matchAllNames(n.Params, pattern.Params)
		varArgMatch := matchVarArg(n.VarArg, pattern.VarArg)
		return unionMatches(targetMatch, unionMatches(paramsMatch, varArgMatch))
	case *ast.FunctionCallExpression:
		targetMatch := Match(n.Target, pattern.Target)
		argsMatch := matchAllExpressionsAsNames(n.Args, pattern.Params)
		return unionMatches(targetMatch, argsMatch)
	default:
		return MatchResult{
			Messages: []string{
				"Expected a function call",
			},
			MatchMakesSense: true,
		}
	}
}

func matchTuple(node ast.MlgNodeType, pattern TupleFormPattern) MatchResult {
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

func matchConditionalSetForm(node ast.MlgNodeType, pattern ConditionalSetFormPattern) MatchResult {
	switch n := node.(type) {
	case *ast.ConditionalSetForm:
		targetMatch := Match(n.Target, pattern.Target)
		varArgMatch := matchVarArg(n.VarArg, pattern.VarArg)
		return unionMatches(targetMatch, varArgMatch)
	case *ast.ConditionalSetIdForm:
		targetMatch := Match(n.Target, pattern.Target)
		conditionMatch := matchFunction(&n.Condition, pattern.Condition)
		return unionMatches(targetMatch, conditionMatch)
	default:
		return MatchResult{
			Messages: []string{
				"Expected a set",
			},
			MatchMakesSense: true,
		}
	}
}

func matchConditionalSetExpression(node ast.MlgNodeType,
	pattern ConditionalSetExpressionPattern) MatchResult {
	switch n := node.(type) {
	case *ast.ConditionalSetExpression:
		targetMatch := Match(n.Target, pattern.Target)
		conditionsMatch := matchAllExpressions(n.Conditions, pattern.Conditions)
		return unionMatches(targetMatch, conditionsMatch)
	default:
		return MatchResult{
			Messages: []string{
				"Expected a set",
			},
			MatchMakesSense: true,
		}
	}
}

func matchInfixOperator(node ast.MlgNodeType, pattern InfixOperatorFormPattern) MatchResult {
	switch n := node.(type) {
	case *ast.InfixOperatorCallExpression:
		lhsMatch := Match(n.Lhs, pattern.Lhs)
		opMatch := Match(n.Target, pattern.Operator)
		rhsMatch := Match(n.Rhs, pattern.Rhs)
		return unionMatches(lhsMatch, unionMatches(opMatch, rhsMatch))
	case *ast.InfixOperatorForm:
		lhsMatch := Match(&n.Lhs, pattern.Lhs)
		opMatch := Match(&n.Operator, pattern.Operator)
		rhsMatch := Match(&n.Rhs, pattern.Rhs)
		return unionMatches(lhsMatch, unionMatches(opMatch, rhsMatch))
	case *ast.InfixOperatorId:
		lhsMatch := Match(n.Lhs, pattern.Lhs)
		opMatch := Match(&n.Operator, pattern.Operator)
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

func matchPrefixOperator(node ast.MlgNodeType, pattern PrefixOperatorFormPattern) MatchResult {
	switch n := node.(type) {
	case *ast.PrefixOperatorCallExpression:
		opMatch := Match(n.Target, pattern.Operator)
		argMatch := Match(n.Arg, pattern.Param)
		return unionMatches(opMatch, argMatch)
	case *ast.PrefixOperatorForm:
		opMatch := Match(&n.Operator, pattern.Operator)
		argMatch := Match(&n.Param, pattern.Param)
		return unionMatches(opMatch, argMatch)
	case *ast.PrefixOperatorId:
		opMatch := Match(&n.Operator, pattern.Operator)
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

func matchPostfixOperator(node ast.MlgNodeType, pattern PostfixOperatorFormPattern) MatchResult {
	switch n := node.(type) {
	case *ast.PostfixOperatorCallExpression:
		opMatch := Match(n.Target, pattern.Operator)
		argMatch := Match(n.Arg, pattern.Param)
		return unionMatches(opMatch, argMatch)
	case *ast.PostfixOperatorForm:
		opMatch := Match(&n.Operator, pattern.Operator)
		argMatch := Match(&n.Param, pattern.Param)
		return unionMatches(opMatch, argMatch)
	case *ast.PostfixOperatorId:
		opMatch := Match(&n.Operator, pattern.Operator)
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

func matchInfixOperatorCommand(node ast.MlgNodeType,
	pattern InfixCommandOperatorPattern) MatchResult {
	switch n := node.(type) {
	case *ast.InfixCommandOperatorId:
		lhsMatch := Match(n.Lhs, pattern.Lhs)
		opMatch := matchInfixCommandTarget(&n.Operator, InfixCommandTargetPattern{
			Command: pattern.Operator,
		})
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

func matchInfixCommandTarget(node ast.MlgNodeType, pattern InfixCommandTargetPattern) MatchResult {
	switch n := node.(type) {
	case *ast.CommandOperatorTarget:
		return matchCommand(&n.Command, pattern.Command)
	case *ast.InfixCommandId:
		namesMatch := matchAllNames(n.Names, pattern.Command.Names)
		curlyMatch := matchCurlyParam(n.CurlyParam, pattern.Command.CurlyArg)
		namedArgsMatch := matchAllNamedParams(n.NamedParams, pattern.Command.NamedGroups)
		parensMatch := matchAllOptionalNames(n.ParenParams, pattern.Command.ParenArgs)
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

func matchCommand(node ast.MlgNodeType, pattern CommandPattern) MatchResult {
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

func matchCurlyArg(node *ast.CurlyArg, pattern *CurlyPattern) MatchResult {
	if node == nil || pattern == nil {
		if node == nil && pattern == nil {
			return MatchResult{
				Mapping:         make(map[string]ast.MlgNodeType),
				Messages:        make([]string, 0),
				MatchMakesSense: true,
			}
		} else if node == nil && pattern != nil {
			return MatchResult{
				Mapping: make(map[string]ast.MlgNodeType),
				Messages: []string{
					"Expected a {} argument but found none",
				},
				MatchMakesSense: true,
			}
		} else if node != nil && pattern == nil {
			return MatchResult{
				Mapping: make(map[string]ast.MlgNodeType),
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

func matchOrdinal(node ast.MlgNodeType, pattern OrdinalPattern) MatchResult {
	switch n := node.(type) {
	case *ast.OrdinalCallExpression:
		targetMatch := Match(n.Target, pattern)
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

func matchAllDirectionParamParamType(nodes []ast.DirectionParamParamType,
	patterns []DirectionParamParamPatternType) MatchResult {
	if len(nodes) != len(patterns) {
		return MatchResult{
			Mapping: make(map[string]ast.MlgNodeType),
			Messages: []string{
				fmt.Sprintf("Expected %d values but found %d", len(patterns), len(nodes)),
			},
			MatchMakesSense: true,
		}
	}

	if len(nodes) == 0 {
		return MatchResult{
			MatchMakesSense: true,
		}
	}

	result := matchDirectionParamParamType(nodes[0], patterns[0])
	for i := 1; i < len(nodes); i++ {
		result = unionMatches(result, matchDirectionParamParamType(nodes[i], patterns[i]))
	}
	return result
}

func matchDirectionParamParamType(node ast.DirectionParamParamType,
	pattern DirectionParamParamPatternType) MatchResult {
	switch p := pattern.(type) {
	case NameFormPattern:
		return matchName(node, p)
	case FunctionFormPattern:
		return matchFunction(node, p)
	case OrdinalPattern:
		return matchOrdinal(node, p)
	default:
		return MatchResult{
			Messages: []string{
				"Expected a direction",
			},
			MatchMakesSense: true,
		}
	}
}

func matchDirection(node *ast.DirectionalParam, pattern *DirectionPattern) MatchResult {
	if node == nil || pattern == nil {
		if node == nil && pattern != nil {
			return MatchResult{
				Mapping: make(map[string]ast.MlgNodeType),
				Messages: []string{
					"Expected a direction but didn't receive one",
				},
				MatchMakesSense: true,
			}
		}

		if node != nil && pattern == nil {
			return MatchResult{
				Mapping: make(map[string]ast.MlgNodeType),
				Messages: []string{
					"Did not expect a direction but received one",
				},
				MatchMakesSense: true,
			}
		}

		if node == nil && pattern == nil {
			return MatchResult{
				Mapping:         make(map[string]ast.MlgNodeType),
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
				Mapping: make(map[string]ast.MlgNodeType),
				Messages: []string{
					"Expected a direction name but didn't receive one",
				},
				MatchMakesSense: true,
			}
		}

		if nodeName != nil && patternName == nil {
			return MatchResult{
				Mapping: make(map[string]ast.MlgNodeType),
				Messages: []string{
					"Did not expect a direction name but received one",
				},
				MatchMakesSense: true,
			}
		}

		if node == nil && pattern == nil {
			return MatchResult{
				Mapping:         make(map[string]ast.MlgNodeType),
				Messages:        []string{},
				MatchMakesSense: true,
			}
		}
	}

	nameMatch := matchName(nodeName, *patternName)
	squareMatch := matchAllDirectionParamParamType(node.SquareParams, pattern.SquareArgs)

	return unionMatches(nameMatch, squareMatch)
}

func matchCurlyParam(node *ast.CurlyParam, pattern *CurlyPattern) MatchResult {
	if node == nil || pattern == nil {
		if node == nil && pattern == nil {
			return MatchResult{
				Mapping:         make(map[string]ast.MlgNodeType),
				Messages:        make([]string, 0),
				MatchMakesSense: true,
			}
		} else if node == nil && pattern != nil {
			return MatchResult{
				Mapping: make(map[string]ast.MlgNodeType),
				Messages: []string{
					"Expected a {} argument but found none",
				},
				MatchMakesSense: true,
			}
		} else if node != nil && pattern == nil {
			return MatchResult{
				Mapping: make(map[string]ast.MlgNodeType),
				Messages: []string{
					"Did not expect to find a {} but found one",
				},
				MatchMakesSense: true,
			}
		}
	}

	curlyMatch := matchAllOptionalStructuralForms(&node.CurlyParams, pattern.CurlyArgs)
	squareMatch := matchAllOptionalStructuralForms(node.SquareParams, pattern.SquareArgs)
	directionMatch := matchDirection(node.Direction, pattern.Direction)

	return unionMatches(curlyMatch, unionMatches(squareMatch, directionMatch))
}

func matchNamedGroup(node ast.MlgNodeType, pattern NamedGroupPattern) MatchResult {
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

func matchChainExpression(node ast.MlgNodeType, pattern ChainExpressionPattern) MatchResult {
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

func matchSpecAlias(node ast.MlgNodeType, pattern SpecAliasPattern) MatchResult {
	switch n := node.(type) {
	case *ast.ExpressionColonDashArrowItem:
		n = n
		return MatchResult{}
	default:
		return MatchResult{
			MatchMakesSense: false,
		}
	}
}

func matchAlias(node ast.MlgNodeType, pattern AliasPattern) MatchResult {
	switch n := node.(type) {
	case *ast.ExpressionColonArrowItem:
		n = n
		return MatchResult{}
	default:
		return MatchResult{
			MatchMakesSense: false,
		}
	}
}

////////////////////////////////////////////////////////////////////////////////////////////////////

func matchAllOptionalNames(nodes *[]ast.NameForm, patterns *[]NameFormPattern) MatchResult {
	if nodes == nil {
		zeroNodes := make([]ast.NameForm, 0)
		nodes = &zeroNodes
	}

	if patterns == nil {
		zeroPatterns := make([]NameFormPattern, 0)
		patterns = &zeroPatterns
	}

	return matchAllNames(*nodes, *patterns)
}

func matchAllNames(nodes []ast.NameForm, patterns []NameFormPattern) MatchResult {
	if len(nodes) != len(patterns) {
		return MatchResult{
			Mapping: make(map[string]ast.MlgNodeType),
			Messages: []string{
				fmt.Sprintf("Expected %d values but found %d", len(patterns), len(nodes)),
			},
			MatchMakesSense: true,
		}
	}

	if len(nodes) == 0 {
		return MatchResult{
			MatchMakesSense: true,
		}
	}

	result := Match(&nodes[0], patterns[0])
	for i := 1; i < len(nodes); i++ {
		result = unionMatches(result, Match(&nodes[i], patterns[i]))
	}
	return result
}

func matchAllExpressionsAsNames(nodes []ast.ExpressionType,
	patterns []NameFormPattern) MatchResult {
	if len(nodes) != len(patterns) {
		return MatchResult{
			Mapping: make(map[string]ast.MlgNodeType),
			Messages: []string{
				fmt.Sprintf("Expected %d values but found %d", len(patterns), len(nodes)),
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

func matchAllOptionalExpressionsToNames(nodes *[]ast.ExpressionType,
	patterns *[]NameFormPattern) MatchResult {
	if nodes == nil {
		zeroNodes := make([]ast.ExpressionType, 0)
		nodes = &zeroNodes
	}

	if patterns == nil {
		zeroPatterns := make([]NameFormPattern, 0)
		patterns = &zeroPatterns
	}

	if len(*nodes) != len(*patterns) {
		return MatchResult{
			Mapping: make(map[string]ast.MlgNodeType),
			Messages: []string{
				fmt.Sprintf("Expected %d values but found %d", len(*patterns), len(*nodes)),
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

func matchAllOptionalExpressionsToForms(nodes *[]ast.ExpressionType,
	patterns *[]FormPatternType) MatchResult {
	if nodes == nil {
		zeroNodes := make([]ast.ExpressionType, 0)
		nodes = &zeroNodes
	}

	if patterns == nil {
		zeroPatterns := make([]FormPatternType, 0)
		patterns = &zeroPatterns
	}

	if len(*nodes) != len(*patterns) {
		return MatchResult{
			Mapping: make(map[string]ast.MlgNodeType),
			Messages: []string{
				fmt.Sprintf("Expected %d values but found %d", len(*patterns), len(*nodes)),
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

func matchAllExpressions(nodes []ast.ExpressionType, patterns []FormPatternType) MatchResult {
	if len(nodes) != len(patterns) {
		return MatchResult{
			Mapping: make(map[string]ast.MlgNodeType),
			Messages: []string{
				fmt.Sprintf("Expected %d values but found %d", len(patterns), len(nodes)),
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

func matchAllOptionalStructuralForms(nodes *[]ast.StructuralFormType,
	patterns *[]FormPatternType) MatchResult {
	if nodes == nil {
		zeroNodes := make([]ast.StructuralFormType, 0)
		nodes = &zeroNodes
	}

	if patterns == nil {
		zeroPatterns := make([]FormPatternType, 0)
		patterns = &zeroPatterns
	}

	return matchAllStructuralForms(*nodes, *patterns)
}

func matchAllStructuralForms(nodes []ast.StructuralFormType,
	patterns []FormPatternType) MatchResult {
	if len(nodes) != len(patterns) {
		return MatchResult{
			Mapping: make(map[string]ast.MlgNodeType),
			Messages: []string{
				fmt.Sprintf("Expected %d values but found %d", len(patterns), len(nodes)),
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

func matchAllNamedArgs(nodes *[]ast.NamedArg, patterns *[]NamedGroupPattern) MatchResult {
	if nodes == nil {
		zeroNodes := make([]ast.NamedArg, 0)
		nodes = &zeroNodes
	}

	if patterns == nil {
		zeroPatterns := make([]NamedGroupPattern, 0)
		patterns = &zeroPatterns
	}

	if len(*nodes) != len(*patterns) {
		return MatchResult{
			Mapping: make(map[string]ast.MlgNodeType),
			Messages: []string{
				fmt.Sprintf("Expected %d values but found %d", len(*patterns), len(*nodes)),
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

func matchAllNamedParams(nodes *[]ast.NamedParam, patterns *[]NamedGroupPattern) MatchResult {
	if nodes == nil {
		zeroNodes := make([]ast.NamedParam, 0)
		nodes = &zeroNodes
	}

	if patterns == nil {
		zeroPatterns := make([]NamedGroupPattern, 0)
		patterns = &zeroPatterns
	}

	if len(*nodes) != len(*patterns) {
		return MatchResult{
			Mapping: make(map[string]ast.MlgNodeType),
			Messages: []string{
				fmt.Sprintf("Expected %d values but found %d", len(*patterns), len(*nodes)),
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
	result := Match(&nodesValue[0], patternsValue[0])
	for i := 1; i < len(*nodes); i++ {
		result = unionMatches(result, Match(&nodesValue[i], patternsValue[i]))
	}
	return result
}

func matchDirectionParams(nodes *[]ast.DirectionParamParamType,
	patterns *[]FormPatternType) MatchResult {
	if nodes == nil {
		zeroNodes := make([]ast.DirectionParamParamType, 0)
		nodes = &zeroNodes
	}

	if patterns == nil {
		zeroPatterns := make([]FormPatternType, 0)
		patterns = &zeroPatterns
	}

	if len(*nodes) != len(*patterns) {
		return MatchResult{
			Mapping: make(map[string]ast.MlgNodeType),
			Messages: []string{
				fmt.Sprintf("Expected %d values but found %d", len(*patterns), len(*nodes)),
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
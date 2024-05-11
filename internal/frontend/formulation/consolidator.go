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

package formulation

import (
	"fmt"
	"mathlingua/internal/ast"
	"mathlingua/internal/frontend"
	"mathlingua/internal/mlglib"
	"strings"
)

func Consolidate(
	path ast.Path,
	nodes []ast.FormulationNodeKind,
	tracker *frontend.DiagnosticTracker,
) (ast.FormulationNodeKind, bool) {
	if colonArrowDash, ok := maybeProcessExpressionColonDashArrowItem(path, nodes, tracker); ok {
		return &colonArrowDash, true
	}

	items := mlglib.NewStack[ShuntingYardItem[ast.FormulationNodeKind]]()
	for _, item := range ShuntingYard(toShuntingYardItems(nodes)) {
		items.Push(item)
	}

	stack := mlglib.NewStack[ast.FormulationNodeKind]()
	for !items.IsEmpty() {
		stack.Push(toNode(path, items, tracker))
	}

	if stack.IsEmpty() {
		return nil, false
	}

	top := stack.Pop()
	return top, stack.IsEmpty()
}

func GetPrecedenceAndIfInfix(node ast.ExpressionKind) (int, bool) {
	if _, infixOk := node.(*ast.InfixOperatorCallExpression); !infixOk {
		return -1, false
	}
	precedence, _ := getPrecedenceAssociativity(node, InfixOperatorType)
	return precedence, true
}

////////////////////////////////////////////////////////////////////////////////////////////////////

func maybeProcessExpressionColonDashArrowItem(
	path ast.Path,
	nodes []ast.FormulationNodeKind,
	tracker *frontend.DiagnosticTracker,
) (ast.ExpressionColonDashArrowItem, bool) {
	index := -1
	for i, n := range nodes {
		if tok, ok := n.(*ast.PseudoTokenNode); ok {
			if tok.Type == ast.ColonDashArrow {
				index = i
				break
			}
		}
	}

	if index < 0 {
		return ast.ExpressionColonDashArrowItem{}, false
	}

	lhsNodes := make([]ast.FormulationNodeKind, 0)
	for i := 0; i < index; i++ {
		lhsNodes = append(lhsNodes, nodes[i])
	}

	lhs, ok := Consolidate(path, lhsNodes, tracker)
	if !ok {
		return ast.ExpressionColonDashArrowItem{}, false
	}
	lhsExp, ok := lhs.(ast.ExpressionKind)
	if !ok {
		return ast.ExpressionColonDashArrowItem{}, false
	}

	rhsNodes := make([]ast.ExpressionKind, 0)
	i := index + 1
	for i < len(nodes) {
		partNodes := make([]ast.FormulationNodeKind, 0)
		for i < len(nodes) {
			cur := nodes[i]
			i += 1
			tok, ok := cur.(*ast.PseudoTokenNode)
			if ok && tok.Text == ";" {
				break
			}
			partNodes = append(partNodes, cur)
		}
		part, ok := Consolidate(path, partNodes, tracker)
		if !ok {
			return ast.ExpressionColonDashArrowItem{}, false
		}
		exp, ok := part.(ast.ExpressionKind)
		if !ok {
			return ast.ExpressionColonDashArrowItem{}, false
		}
		rhsNodes = append(rhsNodes, exp)
	}

	return ast.ExpressionColonDashArrowItem{
		Lhs:                 lhsExp,
		Rhs:                 rhsNodes,
		CommonMetaData:      *lhs.GetCommonMetaData(),
		FormulationMetaData: *lhs.GetFormulationMetaData(),
	}, true
}

var default_expression ast.ExpressionKind = &ast.NameForm{}
var default_kind_type ast.KindKind = &ast.NameForm{}

func toNode(
	path ast.Path,
	items *mlglib.Stack[ShuntingYardItem[ast.FormulationNodeKind]],
	tracker *frontend.DiagnosticTracker,
) ast.FormulationNodeKind {
	if items.IsEmpty() {
		return nil
	}

	rawTop := items.Pop()
	top := rawTop.Item
	switch top := top.(type) {
	case *ast.PrefixOperatorCallExpression:
		// prefix operators
		top.Arg = checkType(path, toNode(path, items, tracker), default_expression, "Expression",
			tracker, top.Start())
		return top
	case *ast.PostfixOperatorCallExpression:
		// postfix operators
		top.Arg = checkType(path, toNode(path, items, tracker), default_expression, "Expression",
			tracker, top.Start())
		return top
	case *ast.InfixOperatorCallExpression:
		// infix operators
		top.Lhs = checkType(path, toNode(path, items, tracker), default_expression, "Expression",
			tracker, top.Start())
		top.Rhs = checkType(path, toNode(path, items, tracker), default_expression, "Expression",
			tracker, top.Start())
		return top
	case *ast.EnclosedNonCommandOperatorTarget:
		// for example [x]
		target := top
		if rawTop.ItemType == PrefixOperatorType {
			arg := checkType(path, toNode(path, items, tracker), default_expression, "Expression",
				tracker, top.Start())
			return &ast.PrefixOperatorCallExpression{
				Target: target,
				Arg:    arg,
			}
		} else if rawTop.ItemType == PostfixOperatorType {
			arg := checkType(path, toNode(path, items, tracker), default_expression, "Expression",
				tracker, top.Start())
			return &ast.PostfixOperatorCallExpression{
				Target: target,
				Arg:    arg,
			}
		} else {
			// it is an infix
			lhs := checkType(path, toNode(path, items, tracker), default_expression, "Expression", tracker,
				top.Start())
			rhs := checkType(path, toNode(path, items, tracker), default_expression, "Expression", tracker,
				top.Start())
			return &ast.InfixOperatorCallExpression{
				Target: target,
				Lhs:    rhs,
				Rhs:    lhs,
			}
		}
	case *ast.NonEnclosedNonCommandOperatorTarget:
		// for example + or **
		target := top
		if rawTop.ItemType == PrefixOperatorType {
			arg := checkType(path, toNode(path, items, tracker), default_expression, "Expression",
				tracker, top.Start())
			return &ast.PrefixOperatorCallExpression{
				Target: target,
				Arg:    arg,
			}
		} else if rawTop.ItemType == PostfixOperatorType {
			arg := checkType(path, toNode(path, items, tracker), default_expression, "Expression",
				tracker, top.Start())
			return &ast.PostfixOperatorCallExpression{
				Target: target,
				Arg:    arg,
			}
		} else {
			// it is an infix
			lhs := checkType(path, toNode(path, items, tracker), default_expression, "Expression",
				tracker, top.Start())
			rhs := checkType(path, toNode(path, items, tracker), default_expression, "Expression",
				tracker, top.Start())
			return &ast.InfixOperatorCallExpression{
				Target: target,
				Lhs:    rhs,
				Rhs:    lhs,
			}
		}
	case *ast.InfixCommandExpression:
		// for example \f/
		target := top
		lhs := checkType(path, toNode(path, items, tracker), default_expression, "Expression",
			tracker, top.Start())
		rhs := checkType(path, toNode(path, items, tracker), default_expression, "Expression",
			tracker, top.Start())
		return &ast.InfixOperatorCallExpression{
			Target: target,
			Lhs:    rhs,
			Rhs:    lhs,
		}
	case *ast.InfixCommandTypeForm:
		// for example \:f:/
		target := top
		lhs := checkType(path, toNode(path, items, tracker), default_expression, "Expression",
			tracker, top.Start())
		rhs := checkType(path, toNode(path, items, tracker), default_expression, "Expression",
			tracker, top.Start())
		return &ast.InfixOperatorCallExpression{
			Target: target,
			Lhs:    rhs,
			Rhs:    lhs,
		}
	case *ast.PseudoTokenNode:
		// a token, for example :=, :=>, :->, is
		switch {
		case top.Type == ast.ColonArrow:
			rhs := checkType(path, toNode(path, items, tracker), default_expression, "Expression",
				tracker, top.Start())
			tmp := toNode(path, items, tracker)
			lhs := checkType(path, tmp, default_expression, "Expression",
				tracker, top.Start())
			return &ast.ExpressionColonArrowItem{
				Lhs: lhs,
				Rhs: rhs,
			}
			//		case top.Type == ast.ColonDashArrow:
			//			rhs := checkType(path, toNode(path, items, tracker), default_expression, "Expression",
			//				tracker, top.Start())
			//			tmp := toNode(path, items, tracker)
			//			lhs := checkType(path, tmp, default_expression, "Expression", tracker, top.Start())
			//			return &ast.ExpressionColonDashArrowItem{
			//				Lhs: lhs,
			//				Rhs: rhs,
			//			}
		case top.Type == ast.ColonEquals:
			rhs := checkType(path, toNode(path, items, tracker), default_expression, "Expression",
				tracker, top.Start())
			lhs := checkType(path, toNode(path, items, tracker), default_expression, "Expression",
				tracker, top.Start())
			return &ast.ExpressionColonEqualsItem{
				Lhs: lhs,
				Rhs: rhs,
			}
		case top.Type == ast.Is:
			rhs := checkType(path, toNode(path, items, tracker), default_kind_type, "Kind Type",
				tracker, top.Start())
			lhs := checkType(path, toNode(path, items, tracker), default_expression, "Expression",
				tracker, top.Start())
			return &ast.IsExpression{
				Lhs: []ast.ExpressionKind{lhs},
				Rhs: []ast.KindKind{rhs},
			}
		case top.Type == ast.Extends:
			rhs := checkType(path, toNode(path, items, tracker), default_kind_type, "Kind Type",
				tracker, top.Start())
			lhs := checkType(path, toNode(path, items, tracker), default_expression, "Expression",
				tracker, top.Start())
			return &ast.ExtendsExpression{
				Lhs: []ast.ExpressionKind{lhs},
				Rhs: []ast.KindKind{rhs},
			}
		case top.Type == ast.As:
			rhs := checkType(path, toNode(path, items, tracker), default_expression, "Type",
				tracker, top.Start())
			lhs := checkType(path, toNode(path, items, tracker), default_expression, "Expression",
				tracker, top.Start())
			return &ast.AsExpression{
				Lhs: lhs,
				Rhs: rhs,
			}
		default:
			return top
		}
	default:
		return top
	}
}

type firstPassType string

const (
	operandType             firstPassType = "operandType"
	generalOperatorType     firstPassType = "generalOperatorType"
	forcedInfixOperatorType firstPassType = "forcedInfixOperatorType"
	noneType                firstPassType = "noneType"
)

func toShuntingYardItems(
	nodes []ast.FormulationNodeKind,
) []ShuntingYardItem[ast.FormulationNodeKind] {
	result := make([]ShuntingYardItem[ast.FormulationNodeKind], 0)
	firstPassTypes := make([]firstPassType, len(nodes))
	for i, node := range nodes {
		firstPassTypes[i] = getFirstPassType(node)
	}
	itemTypes := make([]ItemType, len(nodes))
	for i := range firstPassTypes {
		// nolint:ineffassign
		prevType := noneType
		curType := firstPassTypes[i]
		// nolint:ineffassign
		nextType := noneType

		if i == 0 {
			prevType = noneType
		} else {
			prevType = firstPassTypes[i-1]
		}

		if i == len(firstPassTypes)-1 {
			nextType = noneType
		} else {
			nextType = firstPassTypes[i+1]
		}

		itemTypes[i] = getGeneralOperatorKind(prevType, curType, nextType)
	}

	for i, node := range nodes {
		prec, assoc := getPrecedenceAssociativity(node, itemTypes[i])
		result = append(result, ShuntingYardItem[ast.FormulationNodeKind]{
			Item:          node,
			ItemType:      itemTypes[i],
			Precedence:    prec,
			Associativity: assoc,
		})
	}

	return result
}

func getFirstPassType(node ast.FormulationNodeKind) firstPassType {
	switch node := node.(type) {
	case *ast.PrefixOperatorCallExpression:
		return generalOperatorType
	case *ast.PostfixOperatorCallExpression:
		return generalOperatorType
	case *ast.InfixOperatorCallExpression:
		return generalOperatorType
	case *ast.EnclosedNonCommandOperatorTarget:
		return generalOperatorType
	case *ast.NonEnclosedNonCommandOperatorTarget:
		return generalOperatorType
	case *ast.InfixCommandExpression:
		// for example \f/
		return forcedInfixOperatorType
	case *ast.InfixCommandTypeForm:
		// for example \:f:/
		return forcedInfixOperatorType
	case *ast.PseudoTokenNode:
		// a token, for example :=, is
		switch node.Type {
		case ast.ColonEquals:
			return forcedInfixOperatorType
		case ast.ColonArrow:
			return forcedInfixOperatorType
		case ast.ColonDashArrow:
			return forcedInfixOperatorType
		case ast.Operator:
			return generalOperatorType
		case ast.Is:
			return forcedInfixOperatorType
		case ast.Extends:
			return forcedInfixOperatorType
		case ast.As:
			return forcedInfixOperatorType
		default:
			return operandType
		}
	default:
		return operandType
	}
}

// The lint checker incorrectly reports that this function needs a return statement.
// nolint:typecheck
func getGeneralOperatorKind(
	prevType firstPassType,
	curType firstPassType,
	nextType firstPassType,
) ItemType {
	switch {
	case curType == noneType:
		panic("Cannot get the operator type of 'none'")
	case curType == operandType:
		return OperandType
	case curType == forcedInfixOperatorType:
		return InfixOperatorType
	// for the rest of the cases curType == generalOperatorType
	case prevType == generalOperatorType && nextType == generalOperatorType:
		return InfixOperatorType
	case prevType == operandType && nextType == operandType:
		return InfixOperatorType
	case prevType == generalOperatorType && nextType == operandType:
		return InfixOperatorType
	case prevType == operandType && nextType == generalOperatorType:
		return PostfixOperatorType
	case prevType == noneType:
		return PrefixOperatorType
	case nextType == noneType:
		return PostfixOperatorType
	case prevType == generalOperatorType && nextType == operandType:
		return PrefixOperatorType
	case prevType == operandType && nextType == generalOperatorType:
		return PostfixOperatorType
	case prevType == operandType && nextType == forcedInfixOperatorType:
		return PostfixOperatorType
	case prevType == forcedInfixOperatorType && nextType == operandType:
		return PrefixOperatorType
	case prevType == forcedInfixOperatorType && nextType == forcedInfixOperatorType:
		// users could write `:=> ! :=>` and we want to surface parse errors
		// and not crash in this case
		return InfixOperatorType
	default:
		panic(fmt.Sprintf("Cannot determine the operator type if the previous type "+
			"is %s the current type is %s and the next type is %s", prevType, curType, nextType))
	}
}

const is_precedence = 1
const as_precedence = 2
const colon_equal_precedence = 3
const colon_arrow_precedence = 3
const colon_dash_arrow_precedence = 3
const bar_right_dash_arrow_precedence = 3
const equal_like_precedence = 4
const enclosed_infix_precedence = 5
const command_infix_precedence = 6
const other_special_infix_precedence = 7
const other_special_prefix_precedence = 8
const other_special_postfix_precedence = 9
const plus_minus_precedence = 7
const times_divide_precedence = 10
const caret_precedence = 11

const is_associativity = LeftAssociative
const as_associativity = LeftAssociative
const colon_equal_associativity = RightAssociative
const colon_arrow_associativity = RightAssociative
const colon_dash_arrow_associativity = RightAssociative
const bar_right_dash_arrow_associativity = RightAssociative
const equal_like_associativity = RightAssociative
const enclosed_infix_associativity = LeftAssociative
const command_infix_associativity = LeftAssociative
const other_special_infix_associativity = LeftAssociative
const other_special_prefix_associativity = LeftAssociative
const other_special_postfix_associativity = RightAssociative
const plus_minus_associativity = LeftAssociative
const times_divides_associativity = LeftAssociative
const caret_associativity = RightAssociative

func getOperatorPrecedenceAssociativityByText(
	text string,
	itemType ItemType,
) (int, Associativity) {
	switch {
	case itemType == PrefixOperatorType:
		return other_special_prefix_precedence, other_special_prefix_associativity
	case itemType == PostfixOperatorType:
		return other_special_postfix_precedence, other_special_postfix_associativity
	case text == ":=":
		return colon_equal_precedence, colon_equal_associativity
	case text == ":=>":
		return colon_arrow_precedence, colon_arrow_associativity
	case text == ":->":
		return colon_dash_arrow_precedence, colon_dash_arrow_associativity
	case text == "|->":
		return bar_right_dash_arrow_precedence, bar_right_dash_arrow_associativity
	case strings.Contains(text, "!="):
		return equal_like_precedence, equal_like_associativity
	case strings.Contains(text, "="):
		return equal_like_precedence, equal_like_associativity
	case strings.Contains(text, "<"):
		return equal_like_precedence, equal_like_associativity
	case strings.Contains(text, ">"):
		return equal_like_precedence, equal_like_associativity
	case text == "+" && itemType == InfixOperatorType:
		return plus_minus_precedence, plus_minus_associativity
	case text == "-" && itemType == InfixOperatorType:
		return plus_minus_precedence, plus_minus_associativity
	case text == "*" && itemType == InfixOperatorType:
		return times_divide_precedence, times_divides_associativity
	case text == "/" && itemType == InfixOperatorType:
		return times_divide_precedence, times_divides_associativity
	case text == "^" && itemType == InfixOperatorType:
		return caret_precedence, caret_associativity
	case text == "is":
		return is_precedence, is_associativity
	case text == "as":
		return as_precedence, as_associativity
	case itemType == InfixOperatorType:
		return other_special_infix_precedence, other_special_infix_associativity
	default:
		return -1, NotAssociative
	}
}

func getPrecedenceAssociativity(
	node ast.FormulationNodeKind,
	itemType ItemType,
) (int, Associativity) {
	switch node := node.(type) {
	case *ast.PrefixOperatorCallExpression:
		// prefix operators
		return getPrecedenceAssociativity(node.Target, itemType)
	case *ast.PostfixOperatorCallExpression:
		// postfix operators
		return getPrecedenceAssociativity(node.Target, itemType)
	case *ast.InfixOperatorCallExpression:
		// infix operators
		return getPrecedenceAssociativity(node.Target, itemType)
	case *ast.EnclosedNonCommandOperatorTarget:
		// for example [x]
		return enclosed_infix_precedence, enclosed_infix_associativity
	case *ast.NonEnclosedNonCommandOperatorTarget:
		// for example + or **
		return getOperatorPrecedenceAssociativityByText(node.Text, itemType)
	case *ast.InfixCommandExpression:
		// for example \f/
		return command_infix_precedence, command_infix_associativity
	case *ast.InfixCommandTypeForm:
		// for example \:f:/
		return command_infix_precedence, command_infix_associativity
	case *ast.PseudoTokenNode:
		// a token, for example :=, :=>, is
		return getOperatorPrecedenceAssociativityByText(node.Text, itemType)
	default:
		return -1, NotAssociative
	}
}

func checkType[T any](
	path ast.Path,
	node ast.FormulationNodeKind,
	def T,
	typeName string,
	tracker *frontend.DiagnosticTracker,
	fallbackPosition ast.Position,
) T {
	cast, ok := node.(T)
	if ok {
		return cast
	} else {
		position := fallbackPosition
		if node != nil {
			position = node.Start()
		}
		tracker.Append(frontend.Diagnostic{
			Type:     frontend.Error,
			Path:     path,
			Origin:   frontend.FormulationConsolidatorOrigin,
			Message:  fmt.Sprintf("Expected a %s but found %s", typeName, mlglib.PrettyPrint(node)),
			Position: position,
		})
		return def
	}
}

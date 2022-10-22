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

func Consolidate(nodes []ast.NodeType, tracker frontend.DiagnosticTracker) (ast.NodeType, bool) {
	items := mlglib.NewStack[ShuntingYardItem[ast.NodeType]]()
	for _, item := range ShuntingYard(toShuntingYardItems(nodes)) {
		items.Push(item)
	}

	stack := mlglib.NewStack[ast.NodeType]()
	for !items.IsEmpty() {
		stack.Push(toNode(items, tracker))
	}

	if stack.IsEmpty() {
		return nil, false
	}

	top := stack.Pop()
	return top, stack.IsEmpty()
}

func GetPrecedenceAndIfInfix(node ast.ExpressionType) (int, bool) {
	if _, infixOk := node.(ast.InfixOperatorCallExpression); !infixOk {
		return -1, false
	}
	precedence, _ := getPrecedenceAssociativity(node, InfixOperatorType)
	return precedence, true
}

///////////////////////////////////////////////////////////////////////////////////////////////////

var default_expression ast.ExpressionType = ast.NameForm{}
var default_structural_form ast.StructuralFormType = ast.NameForm{}
var default_kind_type ast.KindType = ast.NameForm{}
var default_signature ast.Signature = ast.Signature{}

func toNode(items mlglib.Stack[ShuntingYardItem[ast.NodeType]], tracker frontend.DiagnosticTracker) ast.NodeType {
	if items.IsEmpty() {
		return nil
	}

	rawTop := items.Pop()
	top := rawTop.Item
	switch top := top.(type) {
	case ast.PrefixOperatorCallExpression:
		// prefix operators
		top.Arg = checkType(toNode(items, tracker), default_expression, "Expression", tracker, top.Start())
		return top
	case ast.PostfixOperatorCallExpression:
		// postfix operators
		top.Arg = checkType(toNode(items, tracker), default_expression, "Expression", tracker, top.Start())
		return top
	case ast.InfixOperatorCallExpression:
		// infix operators
		top.Lhs = checkType(toNode(items, tracker), default_expression, "Expression", tracker, top.Start())
		top.Rhs = checkType(toNode(items, tracker), default_expression, "Expression", tracker, top.Start())
		return top
	case ast.EnclosedNonCommandOperatorTarget:
		// for example [x]
		target := top
		lhs := checkType(toNode(items, tracker), default_expression, "Expression", tracker, top.Start())
		rhs := checkType(toNode(items, tracker), default_expression, "Expression", tracker, top.Start())
		return ast.InfixOperatorCallExpression{
			Target: target,
			Lhs:    rhs,
			Rhs:    lhs,
		}
	case ast.NonEnclosedNonCommandOperatorTarget:
		// for example + or **
		target := top
		if rawTop.ItemType == PrefixOperatorType {
			arg := checkType(toNode(items, tracker), default_expression, "Expression", tracker, top.Start())
			return ast.PrefixOperatorCallExpression{
				Target: target,
				Arg:    arg,
			}
		} else if rawTop.ItemType == PostfixOperatorType {
			arg := checkType(toNode(items, tracker), default_expression, "Expression", tracker, top.Start())
			return ast.PostfixOperatorCallExpression{
				Target: target,
				Arg:    arg,
			}
		} else {
			// it is an infix
			lhs := checkType(toNode(items, tracker), default_expression, "Expression", tracker, top.Start())
			rhs := checkType(toNode(items, tracker), default_expression, "Expression", tracker, top.Start())
			return ast.InfixOperatorCallExpression{
				Target: target,
				Lhs:    rhs,
				Rhs:    lhs,
			}
		}
	case ast.CommandOperatorTarget:
		// for example \f/
		target := top
		lhs := checkType(toNode(items, tracker), default_expression, "Expression", tracker, top.Start())
		rhs := checkType(toNode(items, tracker), default_expression, "Expression", tracker, top.Start())
		return ast.InfixOperatorCallExpression{
			Target: target,
			Lhs:    rhs,
			Rhs:    lhs,
		}
	case ast.PseudoTokenNode:
		// a token, for example :=, :=>, is, isnot
		switch {
		case top.Type == ast.ColonArrow:
			rhs := checkType(toNode(items, tracker), default_expression, "Expression", tracker, top.Start())
			lhs := checkType(toNode(items, tracker), default_expression, "Expression", tracker, top.Start())
			return ast.ExpressionColonArrowItem{
				Lhs: lhs,
				Rhs: rhs,
			}
		case top.Type == ast.ColonEquals:
			rhs := checkType(toNode(items, tracker), default_expression, "Expression", tracker, top.Start())
			lhs := checkType(toNode(items, tracker), default_structural_form, "Structural Form", tracker, top.Start())
			return ast.ExpressionColonEqualsItem{
				Lhs: lhs,
				Rhs: rhs,
			}
		case top.Type == ast.Is:
			rhs := checkType(toNode(items, tracker), default_kind_type, "Kind Type", tracker, top.Start())
			lhs := checkType(toNode(items, tracker), default_expression, "Expression", tracker, top.Start())
			return ast.IsExpression{
				Lhs: []ast.ExpressionType{lhs},
				Rhs: []ast.KindType{rhs},
			}
		case top.Type == ast.Extends:
			rhs := checkType(toNode(items, tracker), default_kind_type, "Kind Type", tracker, top.Start())
			lhs := checkType(toNode(items, tracker), default_expression, "Expression", tracker, top.Start())
			return ast.ExtendsExpression{
				Lhs: []ast.ExpressionType{lhs},
				Rhs: []ast.KindType{rhs},
			}
		case top.Type == ast.As:
			lhs := checkType(toNode(items, tracker), default_expression, "Expression", tracker, top.Start())
			rhs := checkType(toNode(items, tracker), default_signature, "Signature", tracker, top.Start())
			return ast.AsExpression{
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

func toShuntingYardItems(nodes []ast.NodeType) []ShuntingYardItem[ast.NodeType] {
	result := make([]ShuntingYardItem[ast.NodeType], 0)
	isOperators := make([]bool, len(nodes))
	for i, node := range nodes {
		isOperators[i] = isOperator(node)
	}
	itemTypes := make([]ItemType, len(nodes))
	for i, curIsOp := range isOperators {
		prevType := none
		curType := toIsOperatorToGeneralItemType(curIsOp)
		nextType := none

		if i == 0 {
			prevType = none
		} else {
			prevType = toIsOperatorToGeneralItemType(isOperators[i-1])
		}

		if i == len(isOperators)-1 {
			nextType = none
		} else {
			nextType = toIsOperatorToGeneralItemType(isOperators[i+1])
		}

		itemTypes[i] = getGeneralOperatorType(prevType, curType, nextType)
	}

	for i, node := range nodes {
		prec, assoc := getPrecedenceAssociativity(node, itemTypes[i])
		result = append(result, ShuntingYardItem[ast.NodeType]{
			Item:          node,
			ItemType:      itemTypes[i],
			Precedence:    prec,
			Associativity: assoc,
		})
	}

	return result
}

func toIsOperatorToGeneralItemType(isOperator bool) generalItemType {
	if isOperator {
		return operator
	} else {
		return operand
	}
}

func isOperator(node ast.NodeType) bool {
	switch node := node.(type) {
	case ast.PrefixOperatorCallExpression:
		// prefix operators
		return true
	case ast.PostfixOperatorCallExpression:
		// postfix operators
		return true
	case ast.InfixOperatorCallExpression:
		// infix operators
		return true
	case ast.EnclosedNonCommandOperatorTarget:
		// for example [x]
		return true
	case ast.NonEnclosedNonCommandOperatorTarget:
		// for example + or **
		return true
	case ast.CommandOperatorTarget:
		// for example \f/
		return true
	case ast.PseudoTokenNode:
		// a token, for example :=, is, isnot
		switch node.Type {
		case ast.ColonEquals:
			return true
		case ast.ColonArrow:
			return true
		case ast.Operator:
			return true
		case ast.Is:
			return true
		case ast.Extends:
			return true
		case ast.As:
			return true
		default:
			return false
		}
	default:
		return false
	}
}

type generalItemType string

const (
	operand  generalItemType = "operand"
	operator generalItemType = "operator"
	none     generalItemType = "none"
)

func getGeneralOperatorType(prevType generalItemType, curType generalItemType, nextType generalItemType) ItemType {
	switch {
	case curType == none:
		panic("Cannot get the operator type of 'none'")
	case curType == operand:
		return OperandType
	// for all of these cases, the curType is operator
	case prevType == operator && nextType == operator:
		return InfixOperatorType
	case prevType == operand && nextType == operand:
		return InfixOperatorType
	case prevType == operator && nextType == operand:
		return InfixOperatorType
	case prevType == operand && nextType == operator:
		return InfixOperatorType
	case prevType == none:
		return PrefixOperatorType
	case nextType == none:
		return PostfixOperatorType
	case prevType == operator && nextType == operand:
		return PrefixOperatorType
	case prevType == operand && nextType == operator:
		return PostfixOperatorType
	default:
		panic(fmt.Sprintf("Cannot determine the operator type if the previous type "+
			"is %s the current type is %s and the next type is %s", prevType, curType, nextType))
	}
}

const is_not_is_precedence = 1
const as_precedence = 2
const colon_equal_precedence = 3
const colon_arrow_precedence = 3
const equal_like_precedence = 4
const enclosed_infix_precedence = 5
const command_infix_precedence = 6
const other_special_infix_precedence = 7
const other_special_prefix_precedence = 8
const other_special_postfix_precedence = 9
const plus_minus_precedence = 7
const times_divide_precedence = 10
const caret_precedence = 11

const is_not_is_associativity = LeftAssociative
const as_associativity = LeftAssociative
const colon_equal_associativity = RightAssociative
const colon_arrow_associativity = RightAssociative
const equal_like_associativity = RightAssociative
const enclosed_infix_associativity = LeftAssociative
const command_infix_associativity = LeftAssociative
const other_special_infix_associativity = LeftAssociative
const other_special_prefix_associativity = LeftAssociative
const other_special_postfix_associativity = RightAssociative
const plus_minus_associativity = LeftAssociative
const times_divides_associativity = LeftAssociative
const caret_associativity = RightAssociative

func getOperatorPrecedenceAssociativityByText(text string, itemType ItemType) (int, Associativity) {
	switch {
	case itemType == PrefixOperatorType:
		return other_special_prefix_precedence, other_special_prefix_associativity
	case itemType == PostfixOperatorType:
		return other_special_postfix_precedence, other_special_postfix_associativity
	case text == ":=":
		return colon_equal_precedence, colon_equal_associativity
	case text == ":=>":
		return colon_arrow_precedence, colon_arrow_associativity
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
		return is_not_is_precedence, is_not_is_associativity
	case text == "isnot":
		return is_not_is_precedence, is_not_is_associativity
	case text == "as":
		return as_precedence, as_associativity
	case itemType == InfixOperatorType:
		return other_special_infix_precedence, other_special_infix_associativity
	default:
		return -1, NotAssociative
	}
}

func getPrecedenceAssociativity(node ast.NodeType, itemType ItemType) (int, Associativity) {
	switch node := node.(type) {
	case ast.PrefixOperatorCallExpression:
		// prefix operators
		return getPrecedenceAssociativity(node.Target, itemType)
	case ast.PostfixOperatorCallExpression:
		// postfix operators
		return getPrecedenceAssociativity(node.Target, itemType)
	case ast.InfixOperatorCallExpression:
		// infix operators
		return getPrecedenceAssociativity(node.Target, itemType)
	case ast.EnclosedNonCommandOperatorTarget:
		// for example [x]
		return enclosed_infix_precedence, enclosed_infix_associativity
	case ast.NonEnclosedNonCommandOperatorTarget:
		// for example + or **
		return getOperatorPrecedenceAssociativityByText(node.Text, itemType)
	case ast.CommandOperatorTarget:
		// for example \f/
		return command_infix_precedence, command_infix_associativity
	case ast.PseudoTokenNode:
		// a token, for example :=, :=>, is
		return getOperatorPrecedenceAssociativityByText(node.Text, itemType)
	default:
		return -1, NotAssociative
	}
}

func checkType[T any](node ast.NodeType, def T, typeName string, tracker frontend.DiagnosticTracker, fallbackPosition ast.Position) T {
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
			Origin:   frontend.FormulationConsolidatorOrigin,
			Message:  fmt.Sprintf("Expected a %s", typeName),
			Position: position,
		})
		return def
	}
}

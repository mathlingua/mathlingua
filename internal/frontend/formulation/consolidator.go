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
		top.Arg = checkType(toNode(items, tracker), default_expression, "Expression", tracker)
		return top
	case ast.PostfixOperatorCallExpression:
		// postfix operators
		top.Arg = checkType(toNode(items, tracker), default_expression, "Expression", tracker)
		return top
	case ast.InfixOperatorCallExpression:
		// infix operators
		top.Lhs = checkType(toNode(items, tracker), default_expression, "Expression", tracker)
		top.Rhs = checkType(toNode(items, tracker), default_expression, "Expression", tracker)
		return top
	case ast.EnclosedNonCommandOperatorTarget:
		// for example [x]
		target := top
		lhs := checkType(toNode(items, tracker), default_expression, "Expression", tracker)
		rhs := checkType(toNode(items, tracker), default_expression, "Expression", tracker)
		return ast.InfixOperatorCallExpression{
			Target: target,
			Lhs:    rhs,
			Rhs:    lhs,
		}
	case ast.NonEnclosedNonCommandOperatorTarget:
		// for example + or **
		target := top
		if rawTop.ItemType == PrefixOperatorType {
			arg := checkType(toNode(items, tracker), default_expression, "Expression", tracker)
			return ast.PrefixOperatorCallExpression{
				Target: target,
				Arg:    arg,
			}
		} else if rawTop.ItemType == PostfixOperatorType {
			arg := checkType(toNode(items, tracker), default_expression, "Expression", tracker)
			return ast.PostfixOperatorCallExpression{
				Target: target,
				Arg:    arg,
			}
		} else {
			// it is an infix
			lhs := checkType(toNode(items, tracker), default_expression, "Expression", tracker)
			rhs := checkType(toNode(items, tracker), default_expression, "Expression", tracker)
			return ast.InfixOperatorCallExpression{
				Target: target,
				Lhs:    rhs,
				Rhs:    lhs,
			}
		}
	case ast.CommandOperatorTarget:
		// for example \f/
		target := top
		lhs := checkType(toNode(items, tracker), default_expression, "Expression", tracker)
		rhs := checkType(toNode(items, tracker), default_expression, "Expression", tracker)
		return ast.InfixOperatorCallExpression{
			Target: target,
			Lhs:    rhs,
			Rhs:    lhs,
		}
	case ast.PseudoTokenNode:
		// a token, for example :=, is, isnot
		switch {
		case top.Type == ast.ColonEquals:
			lhs := checkType(toNode(items, tracker), default_structural_form, "Structural Form", tracker)
			rhs := checkType(toNode(items, tracker), default_expression, "Expression", tracker)
			return ast.ExpressionColonEqualsItem{
				Lhs: lhs,
				Rhs: rhs,
			}
		case top.Type == ast.Is:
			rhs := checkType(toNode(items, tracker), default_kind_type, "Kind Type", tracker)
			lhs := checkType(toNode(items, tracker), default_expression, "Expression", tracker)
			return ast.IsExpression{
				Lhs: []ast.ExpressionType{lhs},
				Rhs: []ast.KindType{rhs},
			}
		case top.Type == ast.IsNot:
			rhs := checkType(toNode(items, tracker), default_kind_type, "Kind Type", tracker)
			lhs := checkType(toNode(items, tracker), default_expression, "Expression", tracker)
			return ast.IsNotExpression{
				Lhs: []ast.ExpressionType{lhs},
				Rhs: []ast.KindType{rhs},
			}
		case top.Type == ast.As:
			lhs := checkType(toNode(items, tracker), default_expression, "Expression", tracker)
			rhs := checkType(toNode(items, tracker), default_signature, "Signature", tracker)
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
		case ast.Operator:
			return true
		case ast.Is:
			return true
		case ast.IsNot:
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

const IS_NOT_IS_PRECEDENCE = 1
const AS_PRECEDENCE = 2
const COLON_EQUAL_PRECEDENCE = 3
const EQUAL_LIKE_PRECEDENCE = 4
const ENCLOSED_INFIX_PRECEDENCE = 5
const COMMAND_INFIX_PRECEDENCE = 6
const OTHER_SPECIAL_INFIX_PRECEDENCE = 7
const OTHER_SPECIAL_PREFIX_PRECEDENCE = 8
const OTHER_SPECIAL_POSTFIX_PRECEDENCE = 9
const PLUS_MINUS_PRECEDENCE = 7
const TIMES_DIVIDE_PRECEDENCE = 10
const CARET_PRECEDENCE = 11

const IS_NOT_IS_ASSOCIATIVITY = LeftAssociative
const AS_ASSOCIATIVITY = LeftAssociative
const COLON_EQUAL_ASSOCIATIVITY = RightAssociative
const EQUAL_LIKE_ASSOCIATIVITY = RightAssociative
const ENCLOSED_INFIX_ASSOCIATIVITY = LeftAssociative
const COMMAND_INFIX_ASSOCIATIVITY = LeftAssociative
const OTHER_SPECIAL_INFIX_ASSOCIATIVITY = LeftAssociative
const OTHER_SPECIAL_PREFIX_ASSOCIATIVITY = LeftAssociative
const OTHER_SPECIAL_POSTFIX_ASSOCIATIVITY = RightAssociative
const PLUS_MINUS_ASSOCIATIVITY = LeftAssociative
const TIMES_DIVIDEASSOCIATIVITY = LeftAssociative
const CARET_ASSOCIATIVITY = RightAssociative

func getOperatorPrecedenceAssociativityByText(text string, itemType ItemType) (int, Associativity) {
	switch {
	case itemType == PrefixOperatorType:
		return OTHER_SPECIAL_PREFIX_PRECEDENCE, OTHER_SPECIAL_PREFIX_ASSOCIATIVITY
	case itemType == PostfixOperatorType:
		return OTHER_SPECIAL_POSTFIX_PRECEDENCE, OTHER_SPECIAL_POSTFIX_ASSOCIATIVITY
	case text == ":=":
		return COLON_EQUAL_PRECEDENCE, COLON_EQUAL_ASSOCIATIVITY
	case strings.Contains(text, "!="):
		return EQUAL_LIKE_PRECEDENCE, EQUAL_LIKE_ASSOCIATIVITY
	case strings.Contains(text, "="):
		return EQUAL_LIKE_PRECEDENCE, EQUAL_LIKE_ASSOCIATIVITY
	case strings.Contains(text, "<"):
		return EQUAL_LIKE_PRECEDENCE, EQUAL_LIKE_ASSOCIATIVITY
	case strings.Contains(text, ">"):
		return EQUAL_LIKE_PRECEDENCE, EQUAL_LIKE_ASSOCIATIVITY
	case text == "+" && itemType == InfixOperatorType:
		return PLUS_MINUS_PRECEDENCE, PLUS_MINUS_ASSOCIATIVITY
	case text == "-" && itemType == InfixOperatorType:
		return PLUS_MINUS_PRECEDENCE, PLUS_MINUS_ASSOCIATIVITY
	case text == "*" && itemType == InfixOperatorType:
		return TIMES_DIVIDE_PRECEDENCE, TIMES_DIVIDEASSOCIATIVITY
	case text == "/" && itemType == InfixOperatorType:
		return TIMES_DIVIDE_PRECEDENCE, TIMES_DIVIDEASSOCIATIVITY
	case text == "^" && itemType == InfixOperatorType:
		return CARET_PRECEDENCE, CARET_ASSOCIATIVITY
	case text == "is":
		return IS_NOT_IS_PRECEDENCE, IS_NOT_IS_ASSOCIATIVITY
	case text == "isnot":
		return IS_NOT_IS_PRECEDENCE, IS_NOT_IS_ASSOCIATIVITY
	case text == "as":
		return AS_PRECEDENCE, AS_ASSOCIATIVITY
	case itemType == InfixOperatorType:
		return OTHER_SPECIAL_INFIX_PRECEDENCE, OTHER_SPECIAL_INFIX_ASSOCIATIVITY
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
		return ENCLOSED_INFIX_PRECEDENCE, ENCLOSED_INFIX_ASSOCIATIVITY
	case ast.NonEnclosedNonCommandOperatorTarget:
		// for example + or **
		return getOperatorPrecedenceAssociativityByText(node.Text, itemType)
	case ast.CommandOperatorTarget:
		// for example \f/
		return COMMAND_INFIX_PRECEDENCE, COMMAND_INFIX_ASSOCIATIVITY
	case ast.PseudoTokenNode:
		// a token, for example :=, is, isnot
		return getOperatorPrecedenceAssociativityByText(node.Text, itemType)
	default:
		return -1, NotAssociative
	}
}

func checkType[T any](node ast.NodeType, def T, typeName string, tracker frontend.DiagnosticTracker) T {
	cast, ok := node.(T)
	if ok {
		return cast
	} else {
		tracker.Append(frontend.Diagnostic{
			Type:    frontend.Error,
			Origin:  frontend.FormulationConsolidatorOrigin,
			Message: fmt.Sprintf("Expected a %s", typeName),
			Position: ast.Position{
				Offset: -1,
				Row:    -1,
				Column: -1,
			},
		})
		return def
	}
}

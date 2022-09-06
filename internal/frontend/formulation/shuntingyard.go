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
	"mathlingua/internal/mlglib"
)

func ShuntingYard[T any](items []ShuntingYardItem[T]) []ShuntingYardItem[T] {
	stack := mlglib.NewStack[ShuntingYardItem[T]]()
	output := mlglib.NewQueue[ShuntingYardItem[T]]()

	for _, item := range items {
		switch {
		case item.ItemType == OperandType:
			output.Push(item)
		case item.ItemType == PostfixOperatorType:
			output.Push(item)
		case item.ItemType == PrefixOperatorType:
			stack.Push(item)
		default:
			if item.Associativity == LeftAssociative {
				for !stack.IsEmpty() && stack.Peek().ItemType != OperandType &&
					stack.Peek().Precedence >= item.Precedence {
					output.Push(stack.Pop())
				}
				stack.Push(item)
			} else if item.Associativity == RightAssociative {
				for !stack.IsEmpty() && stack.Peek().ItemType != OperandType &&
					stack.Peek().Precedence > item.Precedence {
					output.Push(stack.Pop())
				}
				stack.Push(item)
			}
		}
	}

	for !stack.IsEmpty() {
		output.Push(stack.Pop())
	}

	result := make([]ShuntingYardItem[T], 0)
	for !output.IsEmpty() {
		result = append(result, output.Pop())
	}

	return result
}

type Associativity string

const (
	LeftAssociative  Associativity = "LeftAssociative"
	RightAssociative Associativity = "RightAssociative"
)

type ItemType string

const (
	OperandType         ItemType = "OperandType"
	PrefixOperatorType  ItemType = "PrefixOperatorType"
	PostfixOperatorType ItemType = "PostfixOperatorType"
	InfixOperatorType   ItemType = "InfixOperatorType"
)

type ShuntingYardItem[T any] struct {
	Item          T
	ItemType      ItemType
	Precedence    int
	Associativity Associativity
}

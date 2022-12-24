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

package backend

type PatternMatch[T any] struct {
}

func (pm PatternMatch[T]) Matches() bool {
	return false
}

// func (pm PatternMatch[T]) GetMatch(name string) (T, bool) {
// 	return nil, false
// }

type Pattern interface {
	Pattern()

	Vars() []string
}

type CommandPattern struct {
}

// func (cp CommandPattern) Matches(command ast.CommandExpression) PatternMatch[ast.ExpressionType]

type PrefixOperatorPattern struct {
}

// func (pp PrefixOperatorPattern) Matches(command ast.PrefixOperatorCallExpression) PatternMatch[ast.ExpressionType]

type PostfixOperatorPattern struct {
}

// func (pp PostfixOperatorPattern) Matches(command ast.PostfixOperatorCallExpression) PatternMatch[ast.ExpressionType]

type InfixOperatorExpression struct {
}

// func (ip InfixOperatorExpression) Matches(command ast.InfixOperatorCallExpression) PatternMatch[ast.ExpressionType]

type NamePattern struct {
}

type FunctionPattern struct {
}

type TuplePattern struct {
}

type FixedSetPattern struct {
}

type ConditionalSetPattern struct {
}

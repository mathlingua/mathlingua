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

package mlglib

import "fmt"

type StackType[T any] interface {
	IsEmpty() bool
	Peek() T
	Pop() T
	Push(value T)
}

func NewStack[T any]() StackType[T] {
	return &stack[T]{}
}

////////////////////////////////////////////////////////////////////////////////////////////////////

type stack[T any] struct {
	data []T
}

func (s *stack[T]) IsEmpty() bool {
	return len(s.data) == 0
}

func (s *stack[T]) Peek() T {
	return s.data[len(s.data)-1]
}

func (s *stack[T]) Push(value T) {
	s.data = append(s.data, value)
}

func (s *stack[T]) Pop() T {
	index := len(s.data) - 1
	value := s.data[index]
	s.data = s.data[:index]
	return value
}

func (s *stack[T]) String() string {
	return fmt.Sprintf("%v", s.data)
}

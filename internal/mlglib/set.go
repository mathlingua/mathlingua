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

package mlglib

type SetType[T comparable] interface {
	Size() int
	Add(value T)
	Remove(value T)
	Has(value T) bool
	Clone() SetType[T]
}

func NewSet[T comparable]() SetType[T] {
	return &set[T]{
		values: make(map[T]interface{}),
	}
}

////////////////////////////////////////////////////////////////////////////////////////////////////

type set[T comparable] struct {
	values map[T]interface{}
}

func (s *set[T]) Size() int {
	return len(s.values)
}

func (s *set[T]) Add(value T) {
	s.values[value] = nil
}

func (s *set[T]) Remove(value T) {
	delete(s.values, value)
}

func (s *set[T]) Has(value T) bool {
	_, ok := s.values[value]
	return ok
}

func (s *set[T]) Clone() SetType[T] {
	valuesCopy := make(map[T]interface{}, 0)
	for key, value := range s.values {
		valuesCopy[key] = value
	}
	return &set[T]{
		values: valuesCopy,
	}
}

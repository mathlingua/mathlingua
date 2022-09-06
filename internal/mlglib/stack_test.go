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

import (
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestStack(t *testing.T) {
	stack := NewStack[int]()
	stack.Push(1)
	stack.Push(2)
	stack.Push(3)

	assert.Equal(t, 3, stack.Peek())
	assert.Equal(t, false, stack.IsEmpty())
	assert.Equal(t, 3, stack.Pop())

	assert.Equal(t, 2, stack.Peek())
	assert.Equal(t, false, stack.IsEmpty())
	assert.Equal(t, 2, stack.Pop())

	assert.Equal(t, 1, stack.Peek())
	assert.Equal(t, false, stack.IsEmpty())
	assert.Equal(t, 1, stack.Pop())

	assert.Equal(t, true, stack.IsEmpty())
}

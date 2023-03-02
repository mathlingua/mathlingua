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

import (
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestSet(t *testing.T) {
	set := NewSet[int]()

	assert.Equal(t, 0, set.Size())

	set.Add(1)
	assert.Equal(t, true, set.Has(1))
	assert.Equal(t, 1, set.Size())

	// ensure adding 1 again doesn't increase the size
	set.Add(1)
	assert.Equal(t, true, set.Has(1))
	assert.Equal(t, 1, set.Size())

	set.Add(2)
	assert.Equal(t, true, set.Has(1))
	assert.Equal(t, true, set.Has(2))
	assert.Equal(t, 2, set.Size())

	assert.Equal(t, false, set.Has(3))

	set.Remove(1)
	assert.Equal(t, false, set.Has(1))
	assert.Equal(t, true, set.Has(2))
	assert.Equal(t, false, set.Has(3))

	set.Remove(2)
	assert.Equal(t, false, set.Has(1))
	assert.Equal(t, false, set.Has(2))
	assert.Equal(t, false, set.Has(3))
	assert.Equal(t, 0, set.Size())

	// ensure removing 2 works as expected and doesn't cause a panic
	set.Remove(2)
	assert.Equal(t, false, set.Has(1))
	assert.Equal(t, false, set.Has(2))
	assert.Equal(t, false, set.Has(3))
	assert.Equal(t, 0, set.Size())
}

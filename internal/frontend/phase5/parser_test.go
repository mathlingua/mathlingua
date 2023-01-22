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

package phase5

import (
	"mathlingua/internal/frontend"
	"mathlingua/internal/frontend/phase1"
	"mathlingua/internal/frontend/phase2"
	"mathlingua/internal/frontend/phase3"
	"mathlingua/internal/frontend/phase4"
	"mathlingua/internal/mlglib"
	"os"
	"path"
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestParserSmoke(t *testing.T) {
	text, err := os.ReadFile(path.Join("..", "..", "..", "testdata", "main.math"))
	assert.Nil(t, err)

	tracker := frontend.NewDiagnosticTracker()

	lexer1 := phase1.NewLexer(string(text), tracker)
	lexer2 := phase2.NewLexer(lexer1, tracker)
	lexer3 := phase3.NewLexer(lexer2, tracker)

	root := phase4.Parse(lexer3, tracker)
	_, ok := Parse(root, tracker, mlglib.NewKeyGenerator())
	assert.True(t, ok)
	assert.Equal(t, 0, len(tracker.Diagnostics()))
}

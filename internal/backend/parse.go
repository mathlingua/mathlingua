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

package backend

import (
	"mathlingua/internal/ast"
	"mathlingua/internal/frontend"
	"mathlingua/internal/frontend/phase1"
	"mathlingua/internal/frontend/phase2"
	"mathlingua/internal/frontend/phase3"
	"mathlingua/internal/frontend/phase4"
	"mathlingua/internal/frontend/phase5"
	"mathlingua/internal/mlglib"
)

func ParseDocument(text string) (ast.Document, []frontend.Diagnostic) {
	tracker := frontend.NewDiagnosticTracker()

	lexer1 := phase1.NewLexer(text, tracker)
	lexer2 := phase2.NewLexer(lexer1, tracker)
	lexer3 := phase3.NewLexer(lexer2, tracker)

	root := phase4.Parse(lexer3, tracker)
	doc, _ := phase5.Parse(root, tracker, mlglib.NewKeyGenerator())

	return doc, tracker.Diagnostics()
}

func ParseRoot(texts map[ast.Path]string) (ast.Root, map[ast.Path][]frontend.Diagnostic) {
	docs := make(map[ast.Path]ast.Document, 0)
	pathDiags := make(map[ast.Path][]frontend.Diagnostic, 0)

	for path, content := range texts {
		doc, diags := ParseDocument(content)
		docs[path] = doc
		pathDiags[path] = diags
	}

	root := ast.Root{
		Documents: docs,
		CommonMetaData: ast.CommonMetaData{
			Start: ast.Position{
				Offset: -1,
				Row:    -1,
				Column: -1,
			},
			Key: -1,
		},
	}
	return root, pathDiags
}
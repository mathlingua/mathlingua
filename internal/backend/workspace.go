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
	"mathlingua/internal/server"
)

type ViewResult struct {
	Diagnostics map[ast.Path][]frontend.Diagnostic
	Pages       []WorkspacePageResponse
}

type WorkspacePageResponse struct {
	Path ast.Path
	Page server.PageResponse
}

type CheckResult struct {
	Diagnostics map[ast.Path][]frontend.Diagnostic
}

type Workspace interface {
	DocumentCount() int
	Check() CheckResult
	View() ViewResult
}

func NewWorkspace(contents map[ast.Path]string) Workspace {
	return &workspace{
		contents: contents,
	}
}

////////////////////////////////////////////////////////////////////////////////////////////////////

type workspace struct {
	contents map[ast.Path]string
}

func (w *workspace) DocumentCount() int {
	return len(w.contents)
}

func (w *workspace) Check() CheckResult {
	_, diagnostics := ParseRoot(w.contents)
	return CheckResult{
		Diagnostics: diagnostics,
	}
}

func (w *workspace) View() ViewResult {
	return ViewResult{}
}

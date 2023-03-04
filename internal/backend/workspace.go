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
	"path/filepath"
)

type Path string

func ToPath(str string) Path {
	return Path(filepath.ToSlash(str))
}

type PathDiagnostic struct {
	Path       Path
	Diagnostic frontend.Diagnostic
}

type WorkspaceResponse struct {
	Error string
	Pages []WorkspacePageResponse
}

type WorkspacePageResponse struct {
	Path Path
	Page server.PageResponse
}

type Workspace interface {
	AddDocument(path Path, doc *ast.Document)
	Check() []PathDiagnostic
	View() WorkspaceResponse
}

func NewWorkspace() Workspace {
	return &workspace{}
}

////////////////////////////////////////////////////////////////////////////////////////////////////

type workspace struct {
	documents map[string]*ast.Document
	context   *Context
	scope     *Scope
}

func (w *workspace) AddDocument(path Path, doc *ast.Document) {
}

func (w *workspace) Check() []PathDiagnostic {
	return []PathDiagnostic{}
}

func (w *workspace) View() WorkspaceResponse {
	return WorkspaceResponse{}
}

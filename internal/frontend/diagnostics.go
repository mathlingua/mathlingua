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

package frontend

import "mathlingua/internal/ast"

type DiagnosticType string

const (
	Error DiagnosticType = "Error"
)

type DiagnosticOrigin string

const (
	Phase1LexerOrigin             DiagnosticOrigin = "Phase1LexerOrigin"
	Phase2LexerOrigin             DiagnosticOrigin = "Phase2LexerOrigin"
	Phase3LexerOrigin             DiagnosticOrigin = "Phase3LexerOrigin"
	Phase4ParserOrigin            DiagnosticOrigin = "Phase4ParserOrigin"
	Phase5ParserOrigin            DiagnosticOrigin = "Phase5ParserOrigin"
	FormulationLexerOrigin        DiagnosticOrigin = "FormulationLexerOrigin"
	FormulationParserOrigin       DiagnosticOrigin = "FormulationParserOrigin"
	FormulationConsolidatorOrigin DiagnosticOrigin = "FormulationConsolidatorOrigin"
	BackendOrigin                 DiagnosticOrigin = "BackendOrigin"
)

type Diagnostic struct {
	Type     DiagnosticType
	Origin   DiagnosticOrigin
	Message  string
	Path     ast.Path
	Position ast.Position
}

type IDiagnosticTracker interface {
	Append(diagnostic Diagnostic)
	Length() int
	Diagnostics() []Diagnostic
}

func NewDiagnosticTracker() IDiagnosticTracker {
	return &diagnosticTracker{
		diagnostics: make([]Diagnostic, 0),
	}
}

////////////////////////////////////////////////////////////////////////////////////////////////////

type diagnosticTracker struct {
	diagnostics []Diagnostic
}

func (dt *diagnosticTracker) Append(diagnostic Diagnostic) {
	dt.diagnostics = append(dt.diagnostics, diagnostic)
}

func (dt *diagnosticTracker) Diagnostics() []Diagnostic {
	return dt.diagnostics
}

func (dt *diagnosticTracker) Length() int {
	return len(dt.diagnostics)
}

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

package mlg

import (
	"encoding/json"
	"fmt"
	"mathlingua/internal/ast"
	"mathlingua/internal/frontend"
	"mathlingua/internal/frontend/phase1"
	"mathlingua/internal/frontend/phase2"
	"mathlingua/internal/frontend/phase3"
	"mathlingua/internal/frontend/phase4"
	"mathlingua/internal/frontend/phase5"
	"mathlingua/internal/server"
	"os"
	"path/filepath"
	"strings"
)

type Mlg interface {
	/**
	 * The paths specified are files or directories.  Only `.math` files
	 * will be processed and for any directory, all `.math` files recursively
	 * in the directory will be processed.
	 */
	Check(paths []string, json bool, dedug bool)
	View()
	Version() string
}

func NewMlg(logger Logger) Mlg {
	return &mlg{
		logger: logger,
	}
}

/////////////////////////////////////////

type mlg struct {
	logger Logger
}

type diagnosticInfo struct {
	Type      string `json:"type"`
	Path      string `json:"path"`
	Message   string `json:"message"`
	DebugInfo string `json:"debugInfo"`
	Row       int    `json:"row"`
	Column    int    `json:"column"`
}

type checkResult struct {
	GeneralWarnings []string         `json:"generalWarnings"`
	GeneralErrors   []string         `json:"generalErrors"`
	Diagnostics     []diagnosticInfo `json:"diagnostics"`
}

func parse(text string) (phase4.Root, []frontend.Diagnostic) {
	tracker := frontend.NewDiagnosticTracker()

	lexer1 := phase1.NewLexer(text, tracker)
	lexer2 := phase2.NewLexer(lexer1, tracker)
	lexer3 := phase3.NewLexer(lexer2, tracker)

	root := phase4.Parse(lexer3, tracker)

	return root, tracker.Diagnostics()
}

func parseDocument(text string) (ast.Document, []frontend.Diagnostic) {
	tracker := frontend.NewDiagnosticTracker()

	lexer1 := phase1.NewLexer(text, tracker)
	lexer2 := phase2.NewLexer(lexer1, tracker)
	lexer3 := phase3.NewLexer(lexer2, tracker)

	root := phase4.Parse(lexer3, tracker)
	doc, _ := phase5.Parse(root, tracker)

	return doc, tracker.Diagnostics()
}

func (m *mlg) Check(paths []string, showJson bool, debug bool) {
	result := checkResult{
		GeneralWarnings: make([]string, 0),
		GeneralErrors:   make([]string, 0),
		Diagnostics:     make([]diagnosticInfo, 0),
	}

	if len(paths) == 0 {
		paths = append(paths, ".")
	} else {
		for _, p := range paths {
			stat, _ := os.Stat(p)
			isDir := stat != nil && stat.IsDir()
			if !isDir && !strings.HasSuffix(p, ".math") {
				result.GeneralWarnings = append(result.GeneralWarnings,
					fmt.Sprintf("File %s is not a MathLingua (.math) file and will be ignored", p))
			}
		}
	}
	numErrors := 0
	numFilesProcessed := 0
	for _, p := range paths {
		err := filepath.Walk(p, func(p string, info os.FileInfo, err error) error {
			if err != nil {
				numErrors++
				result.GeneralErrors = append(result.GeneralErrors, err.Error())
				return err
			}

			stat, err := os.Stat(p)
			if err != nil {
				numErrors++
				result.GeneralErrors = append(result.GeneralErrors, err.Error())
				return err
			}

			if stat.IsDir() || !strings.HasSuffix(p, ".math") {
				return nil
			}

			bytes, err := os.ReadFile(p)
			if err != nil {
				numErrors++
				result.GeneralErrors = append(result.GeneralErrors, err.Error())
				return err
			}

			numFilesProcessed++
			_, diagnostics := parseDocument(string(bytes))
			for _, diag := range diagnostics {
				debugInfo := ""
				if debug {
					debugInfo = fmt.Sprintf(" [%s]", diag.Origin)
				}

				result.Diagnostics = append(result.Diagnostics, diagnosticInfo{
					Type:      string(diag.Type),
					Path:      p,
					Message:   diag.Message,
					DebugInfo: debugInfo,
					Row:       diag.Position.Row,
					Column:    diag.Position.Column,
				})
			}
			numErrors += len(diagnostics)

			return nil
		})

		if err != nil {
			result.GeneralErrors = append(result.GeneralErrors, err.Error())
			continue
		}
	}

	if showJson {
		if data, err := json.MarshalIndent(result, "", "  "); err != nil {
			m.logger.Error(fmt.Sprintf("{\"generalErrors\": [\"%s\"]}", err))
		} else {
			m.logger.Log(string(data))
		}
		return
	}

	var errorText string
	if numErrors == 1 {
		errorText = "error"
	} else {
		errorText = "errors"
	}

	var filesText string
	if numFilesProcessed == 1 {
		filesText = "file"
	} else {
		filesText = "files"
	}

	for _, warning := range result.GeneralWarnings {
		m.logger.Warning(warning)
	}

	for _, err := range result.GeneralErrors {
		m.logger.Error(err)
	}

	for index, info := range result.Diagnostics {
		if index > 0 {
			// print a line between each error
			m.logger.Log("")
		}
		m.logger.Error(fmt.Sprintf("%s (%d, %d)%s\n%s",
			info.Path, info.Row+1, info.Column+1, info.DebugInfo, info.Message))
	}

	if numErrors > 0 {
		m.logger.Log("")
		m.logger.Failure(fmt.Sprintf("Processed %d %s and found %d %s", numFilesProcessed, filesText, numErrors, errorText))
	} else {
		m.logger.Success(fmt.Sprintf("Processed %d %s and found 0 errors", numFilesProcessed, filesText))
	}
}

func (m *mlg) View() {
	server.Start()
}

func (m *mlg) Version() string {
	return "v0.2"
}

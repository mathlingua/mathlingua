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
	"mathlingua/internal/backend"
	"mathlingua/internal/frontend"
	"mathlingua/internal/server"
	"os"
	"path/filepath"
	"strings"
)

type IMlg interface {
	/**
	 * The paths specified are files or directories.  Only `.math` files
	 * will be processed and for any directory, all `.math` files recursively
	 * in the directory will be processed.
	 */
	Check(paths []string, json bool, dedug bool)
	View()
	Version() string
}

func NewMlg(logger ILogger) *Mlg {
	return &Mlg{
		logger: logger,
	}
}

////////////////////////////////////////////////////////////////////////////////////////////////////

type Mlg struct {
	logger ILogger
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

func (m *Mlg) Check(paths []string, showJson bool, debug bool) {
	findFiles, errors, warnings := getMathlinguaFiles(paths)

	contents, contentErrors := getFileContents(findFiles)
	errors = append(errors, contentErrors...)

	workspace := backend.NewWorkspace(contents)

	checkResult := workspace.Check()
	numErrors := len(errors) + len(checkResult.Diagnostics)
	numFilesProcessed := workspace.DocumentCount()

	if showJson {
		m.printAsJson(checkResult)
		return
	}

	m.printCheckStats(numErrors, numFilesProcessed, debug, warnings, errors, checkResult.Diagnostics)
}

func (m *Mlg) View() {
	server.Start()
}

func (m *Mlg) Version() string {
	return "v0.2"
}

func getMathlinguaFiles(paths []string) (files []ast.Path, errors []string, warnings []string) {
	files = make([]ast.Path, 0)

	warnings = make([]string, 0)
	errors = make([]string, 0)

	if len(paths) == 0 {
		paths = append(paths, ".")
	} else {
		for _, p := range paths {
			stat, _ := os.Stat(p)
			isDir := stat != nil && stat.IsDir()
			if !isDir && !strings.HasSuffix(p, ".math") {
				warnings = append(warnings,
					fmt.Sprintf("File %s is not a MathLingua (.math) file and will be ignored", p))
			}
		}
	}

	for _, p := range paths {
		err := filepath.Walk(p, func(p string, info os.FileInfo, err error) error {
			if err != nil {
				errors = append(errors, err.Error())
				return err
			}

			stat, err := os.Stat(p)
			if err != nil {
				errors = append(errors, err.Error())
				return err
			}

			if stat.IsDir() || !strings.HasSuffix(p, ".math") {
				return nil
			}

			files = append(files, ast.ToPath(p))

			return nil
		})

		if err != nil {
			errors = append(errors, err.Error())
			continue
		}
	}

	return files, errors, warnings
}

func getFileContents(filePaths []ast.Path) (contents map[ast.Path]string, errors []string) {
	contents = make(map[ast.Path]string, 0)
	errors = make([]string, 0)

	for _, p := range filePaths {
		text, err := appendMetaIds(string(p))
		if err != nil {
			errors = append(errors, err.Error())
		} else {
			contents[p] = text
		}
	}

	return contents, errors
}

func appendMetaIds(path string) (string, error) {
	bytes, err := os.ReadFile(path)
	if err != nil {
		return "", err
	}

	endText, err := backend.AppendMetaIds(string(bytes))
	if err != nil {
		return "", err
	}

	if err := os.WriteFile(path, []byte(endText), 0644); err != nil {
		return "", err
	}

	return endText, nil
}

func (m *Mlg) printAsJson(checkResult backend.CheckResult) {
	if data, err := json.MarshalIndent(checkResult, "", "  "); err != nil {
		m.logger.Error(fmt.Sprintf("{\"generalErrors\": [\"%s\"]}", err))
	} else {
		m.logger.Log(string(data))
	}
}

func (m *Mlg) printCheckStats(numErrors int, numFilesProcessed int, debug bool,
	warnings []string, errors []string, diagnostics []frontend.Diagnostic) {
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

	for _, warning := range warnings {
		m.logger.Warning(warning)
	}

	for _, err := range errors {
		m.logger.Error(err)
	}

	for index, diag := range diagnostics {
		if index > 0 {
			// print a line between each error
			m.logger.Log("")
		}
		debugInfo := ""
		if debug {
			debugInfo = fmt.Sprintf(" [%s]", diag.Origin)
		}
		m.logger.Error(fmt.Sprintf("%s (%d, %d)%s\n%s",
			diag.Path, diag.Position.Row+1, diag.Position.Column+1,
			debugInfo, diag.Message))
	}

	if numErrors > 0 {
		m.logger.Log("")
		m.logger.Failure(fmt.Sprintf("Processed %d %s and found %d %s",
			numFilesProcessed, filesText, numErrors, errorText))
	} else {
		m.logger.Success(fmt.Sprintf("Processed %d %s and found 0 errors",
			numFilesProcessed, filesText))
	}
}

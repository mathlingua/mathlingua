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
	"mathlingua/internal/server"
	"os"
	"path/filepath"
	"strings"

	"github.com/google/uuid"
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

func (m *mlg) Check(paths []string, showJson bool, debug bool) {
	workspace := backend.NewWorkspace()

	pathWarnings := make([]string, 0)
	pathErrors := make([]string, 0)

	if len(paths) == 0 {
		paths = append(paths, ".")
	} else {
		for _, p := range paths {
			stat, _ := os.Stat(p)
			isDir := stat != nil && stat.IsDir()
			if !isDir && !strings.HasSuffix(p, ".math") {
				pathWarnings = append(pathWarnings,
					fmt.Sprintf("File %s is not a MathLingua (.math) file and will be ignored", p))
			}
		}
	}

	for _, p := range paths {
		err := filepath.Walk(p, func(p string, info os.FileInfo, err error) error {
			if err != nil {
				pathErrors = append(pathErrors, err.Error())
				return err
			}

			stat, err := os.Stat(p)
			if err != nil {
				pathErrors = append(pathErrors, err.Error())
				return err
			}

			if stat.IsDir() || !strings.HasSuffix(p, ".math") {
				return nil
			}

			text, err := appendMetaIds(p)
			if err != nil {
				pathErrors = append(pathErrors, err.Error())
				return err
			}

			workspace.AddDocument(ast.ToPath(p), text)
			return nil
		})

		if err != nil {
			pathErrors = append(pathErrors, err.Error())
			continue
		}
	}

	checkResult := workspace.Check()
	numErrors := len(pathErrors)
	for _, diags := range checkResult.Diagnostics {
		numErrors += len(diags)
	}
	numFilesProcessed := workspace.DocumentCount()

	if showJson {
		if data, err := json.MarshalIndent(checkResult, "", "  "); err != nil {
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

	for _, warning := range pathWarnings {
		m.logger.Warning(warning)
	}

	for _, err := range pathErrors {
		m.logger.Error(err)
	}

	for path, diags := range checkResult.Diagnostics {
		for index, diag := range diags {
			if index > 0 {
				// print a line between each error
				m.logger.Log("")
			}
			debugInfo := ""
			if debug {
				debugInfo = fmt.Sprintf(" [%s]", diag.Origin)
			}
			m.logger.Error(fmt.Sprintf("%s (%d, %d)%s\n%s",
				path, diag.Position.Row+1, diag.Position.Column+1,
				debugInfo, diag.Message))
		}
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

func appendMetaIds(path string) (string, error) {
	bytes, err := os.ReadFile(path)
	if err != nil {
		return "", err
	}

	startText := string(bytes)
	endText := ""
	lines := strings.Split(startText, "\n")
	i := 0
	isInTopLevel := false
	hasComment := false
	hasMetaId := false
	for i < len(lines) {
		cur := lines[i]
		i++

		trimmedCur := strings.TrimSpace(cur)
		if strings.HasPrefix(trimmedCur, "::") {
			endText += cur
			endText += "\n"

			isInTopLevel = false
			hasComment = false
			hasMetaId = false

			if trimmedCur == "::" || !strings.HasSuffix(trimmedCur, "::") {
				// it is a textblock
				for i < len(lines) {
					next := lines[i]
					i++
					endText += next
					endText += "\n"
					if strings.HasSuffix(strings.TrimSpace(next), "::") {
						break
					}
				}
			}
		} else if isInTopLevel && strings.HasPrefix(cur, "--") {
			hasComment = true
			endText += cur
			endText += "\n"
		} else if isInTopLevel && strings.HasPrefix(cur, "Id:") {
			hasMetaId = true
			endText += cur
			endText += "\n"
		} else if strings.HasPrefix(cur, "Defines:") ||
			strings.HasPrefix(cur, "Describes:") ||
			strings.HasPrefix(cur, "States:") ||
			strings.HasPrefix(cur, "Axiom:") ||
			strings.HasPrefix(cur, "Conjecture:") ||
			strings.HasPrefix(cur, "Theorem:") ||
			strings.HasPrefix(cur, "Topic:") ||
			strings.HasPrefix(cur, "Resource:") ||
			strings.HasPrefix(cur, "Person:") ||
			strings.HasPrefix(cur, "Specify:") ||
			strings.HasPrefix(cur, "Proof:") {
			isInTopLevel = true
			endText += cur
			endText += "\n"
		} else if cur == "" {
			if isInTopLevel {
				if !hasComment {
					endText += "------------------------------------------\n"
				}
				if !hasMetaId {
					newId, _ := uuid.NewRandom()
					endText += fmt.Sprintf("Id: \"%s\"\n", newId)
				}
			}
			isInTopLevel = false
			hasComment = false
			hasMetaId = false

			endText += cur
			endText += "\n"
		} else {
			endText += cur
			endText += "\n"
		}
	}

	if strings.HasSuffix(endText, "\n") {
		endText = string(endText[0 : len(endText)-1])
	}

	if err := os.WriteFile(path, []byte(endText), 0644); err != nil {
		return "", err
	}

	return endText, nil
}

func (m *mlg) View() {
	server.Start()
}

func (m *mlg) Version() string {
	return "v0.2"
}

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
	"fmt"
	"mathlingua/internal/ast"
	"mathlingua/internal/frontend"
	"os"
	"path/filepath"
	"strings"
	"time"
)

func NewWorkspaceFromPaths(paths []string,
	tracker frontend.IDiagnosticTracker) (*Workspace, []frontend.Diagnostic) {
	diagnostics := make([]frontend.Diagnostic, 0)

	findFiles, findDiagnostics := getMathlinguaFiles(paths)
	diagnostics = append(diagnostics, findDiagnostics...)

	contents, contentDiagnostics := getFileContents(findFiles)
	diagnostics = append(diagnostics, contentDiagnostics...)

	return NewWorkspace(contents, tracker), diagnostics
}

////////////////////////////////////////////////////////////////////////////////////////////////////

func getMathlinguaFiles(paths []string) (files []ast.Path, diagnostics []frontend.Diagnostic) {
	files = make([]ast.Path, 0)

	diagnostics = make([]frontend.Diagnostic, 0)

	if len(paths) == 0 {
		paths = append(paths, ".")
	} else {
		for _, p := range paths {
			stat, _ := os.Stat(p)
			isDir := stat != nil && stat.IsDir()
			if !isDir && !strings.HasSuffix(p, ".math") {
				diagnostics = append(diagnostics, frontend.Diagnostic{
					Type:    frontend.Warning,
					Origin:  frontend.MlgCheckOrigin,
					Path:    ast.Path(p),
					Message: fmt.Sprintf("File %s is not a Mathlingua (.math) file and will be ignored", p),
				})
			}
		}
	}

	for _, p := range paths {
		err := filepath.Walk(p, func(p string, info os.FileInfo, err error) error {
			if err != nil {
				diagnostics = append(diagnostics, frontend.Diagnostic{
					Type:    frontend.Error,
					Origin:  frontend.MlgCheckOrigin,
					Path:    ast.Path(p),
					Message: err.Error(),
				})
				return err
			}

			stat, err := os.Stat(p)
			if err != nil {
				diagnostics = append(diagnostics, frontend.Diagnostic{
					Type:    frontend.Error,
					Origin:  frontend.MlgCheckOrigin,
					Path:    ast.Path(p),
					Message: err.Error(),
				})
				return err
			}

			if stat.IsDir() || !strings.HasSuffix(p, ".math") {
				return nil
			}

			files = append(files, ast.ToPath(p))

			return nil
		})

		if err != nil {
			diagnostics = append(diagnostics, frontend.Diagnostic{
				Type:    frontend.Error,
				Origin:  frontend.MlgCheckOrigin,
				Path:    ast.Path(p),
				Message: err.Error(),
			})
			continue
		}
	}

	return files, diagnostics
}

func getFileContents(filePaths []ast.Path) (contents map[ast.Path]string,
	diagnostics []frontend.Diagnostic) {
	contents = make(map[ast.Path]string, 0)
	diagnostics = make([]frontend.Diagnostic, 0)

	for _, p := range filePaths {
		text, err := appendMetaIds(string(p))
		if err != nil {
			diagnostics = append(diagnostics, frontend.Diagnostic{
				Type:    frontend.Error,
				Origin:  frontend.MlgCheckOrigin,
				Path:    ast.Path(p),
				Message: err.Error(),
			})
		} else {
			contents[p] = text
		}
	}

	return contents, diagnostics
}

func appendMetaIds(path string) (string, error) {
	bytes, err := os.ReadFile(path)
	if err != nil {
		return "", err
	}

	startText := string(bytes)
	endText, err := AppendMetaIds(startText)
	if err != nil {
		return "", err
	}

	if startText == endText {
		return startText, nil
	}

	lockPath := fmt.Sprintf("%s.%d.lock", path, time.Now().UnixNano())
	if err := os.WriteFile(lockPath, []byte(endText), 0644); err != nil {
		return "", err
	}

	if err := os.Rename(lockPath, path); err != nil {
		return "", err
	}

	return endText, nil
}

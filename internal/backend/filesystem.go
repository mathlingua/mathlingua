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
	"mathlingua/internal/config"
	"mathlingua/internal/frontend"
	"mathlingua/internal/mlglib"
	"os"
	"path/filepath"
	"sort"
	"strings"
	"time"

	"golang.org/x/text/cases"
	"golang.org/x/text/language"
)

func NewWorkspaceFromPaths(
	paths []string,
	tracker frontend.IDiagnosticTracker,
) (IWorkspace, []frontend.Diagnostic) {
	diagnostics := make([]frontend.Diagnostic, 0)

	findFiles, findDiagnostics := getMathlinguaFiles(paths)
	diagnostics = append(diagnostics, findDiagnostics...)

	contents, contentDiagnostics := getFileContents(findFiles)
	diagnostics = append(diagnostics, contentDiagnostics...)

	return NewWorkspace(contents, tracker), diagnostics
}

////////////////////////////////////////////////////////////////////////////////////////////////////

const toc_conf_name = "toc.conf"

func getMathlinguaFiles(paths []string) ([]PathLabelPair, []frontend.Diagnostic) {
	result := make([]PathLabelPair, 0)
	diagnostics := make([]frontend.Diagnostic, 0)
	mathPaths := make([]string, 0)

	if len(paths) == 0 {
		mathPaths = append(mathPaths, ".")
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
			} else {
				mathPaths = append(mathPaths, p)
			}
		}
	}

	for _, p := range mathPaths {
		getMathlinguaFilesImpl(p, &result, &diagnostics)
	}

	return result, diagnostics
}

func getMathlinguaFilesImpl(
	path string,
	result *[]PathLabelPair,
	diagnostics *[]frontend.Diagnostic,
) {
	// ignore hidden files and directories like .git
	if strings.HasPrefix(path, ".") && path != "." {
		return
	}

	stat, err := os.Stat(path)
	if err != nil {
		*diagnostics = append(*diagnostics, frontend.Diagnostic{
			Type:    frontend.Error,
			Origin:  frontend.MlgCheckOrigin,
			Path:    ast.Path(path),
			Message: err.Error(),
		})
		return
	}

	if stat.IsDir() {
		// load the toc.conf file if available, and if not, fallback
		// to the default config
		tocConfig := config.NewDefaultTocConfig()

		tocConfigPath := filepath.Join(path, toc_conf_name)
		tocBytes, err := os.ReadFile(tocConfigPath)
		if err != nil {
			if !os.IsNotExist(err) {
				*diagnostics = append(*diagnostics, frontend.Diagnostic{
					Type:    frontend.Error,
					Origin:  frontend.MlgCheckOrigin,
					Path:    ast.ToPath(tocConfigPath),
					Message: err.Error(),
				})
			}
		} else {
			tmpConfig, err := config.ParseTocConfig(string(tocBytes))
			if err != nil {
				*diagnostics = append(*diagnostics, frontend.Diagnostic{
					Type:    frontend.Error,
					Origin:  frontend.MlgCheckOrigin,
					Path:    ast.ToPath(tocConfigPath),
					Message: err.Error(),
				})
			} else {
				tocConfig = tmpConfig
			}
		}

		// at this point the tocConfig is either the loaded config
		// or the default config
		entries, err := os.ReadDir(path)
		if err != nil {
			*diagnostics = append(*diagnostics, frontend.Diagnostic{
				Type:    frontend.Error,
				Origin:  frontend.MlgCheckOrigin,
				Path:    ast.ToPath(tocConfigPath),
				Message: err.Error(),
			})
		} else {
			allPaths := make([]string, 0)
			trueDirs := mlglib.NewSet[string]()
			trueFiles := mlglib.NewSet[string]()
			for _, entry := range entries {
				name := entry.Name()
				if entry.IsDir() {
					allPaths = append(allPaths, name)
					trueDirs.Add(name)
				} else if strings.HasSuffix(name, ".math") {
					allPaths = append(allPaths, name)
					trueFiles.Add(name)
				}
			}

			usedPaths := mlglib.NewSet[string]()
			specifiedPaths := tocConfig.ExplicitFilenames()

			for _, specPath := range specifiedPaths {
				if !trueFiles.Has(specPath) && !trueDirs.Has(specPath) {
					*diagnostics = append(*diagnostics, frontend.Diagnostic{
						Type:    frontend.Error,
						Origin:  frontend.MlgCheckOrigin,
						Path:    ast.ToPath(tocConfigPath),
						Message: fmt.Sprintf("The path %s does not exist", specPath),
					})
				} else if trueDirs.Has(specPath) {
					dir := filepath.Join(path, specPath)
					label, shouldShow := tocConfig.LabelForFilename(specPath)
					if shouldShow {
						*result = append(*result, PathLabelPair{
							Path:  ast.ToPath(dir),
							Label: label,
							IsDir: true,
						})
					}
					getMathlinguaFilesImpl(dir, result, diagnostics)
				} else {
					// specPath is a file
					usedPaths.Add(specPath)
					label, shouldShow := tocConfig.LabelForFilename(specPath)
					if shouldShow {
						*result = append(*result, PathLabelPair{
							Path:  ast.ToPath(filepath.Join(path, specPath)),
							Label: label,
							IsDir: false,
						})
					}
				}
			}

			if tocConfig.StarAction() == config.Keep {
				// sort the paths by label
				sort.SliceStable(allPaths, func(index1 int, index2 int) bool {
					label1 := allPaths[index1]
					if text, hasLabel := tocConfig.LabelForFilename(label1); hasLabel {
						label1 = text
					}
					label2 := allPaths[index2]
					if text, hasLabel := tocConfig.LabelForFilename(label2); hasLabel {
						label2 = text
					}
					return strings.Compare(label1, label2) < 0
				})

				for _, p := range allPaths {
					if usedPaths.Has(p) {
						continue
					}

					if trueDirs.Has(p) {
						getMathlinguaFilesImpl(filepath.Join(path, p), result, diagnostics)
					} else {
						*result = append(*result, PathLabelPair{
							Path:  ast.ToPath(filepath.Join(path, p)),
							Label: pathNameToLabel(p),
							IsDir: false,
						})
					}
				}
			}
		}
	}

	// ignore non .math files
	if !strings.HasSuffix(path, ".math") {
		return
	}

	*result = append(*result, PathLabelPair{
		Path:  ast.ToPath(path),
		Label: pathNameToLabel(path),
		IsDir: false,
	})
}

func pathNameToLabel(name string) string {
	result := ""
	withoutSuffix := strings.TrimSuffix(name, ".math")
	dashedProcessed := strings.ReplaceAll(withoutSuffix, "_", " ")
	parts := strings.Split(dashedProcessed, "\n")
	for i, part := range parts {
		if i > 0 {
			result += " "
		}
		result += cases.Title(language.English).String(part)
	}
	return result
}

func getFileContents(filePaths []PathLabelPair) (contents []PathLabelContent,
	diagnostics []frontend.Diagnostic) {
	contents = make([]PathLabelContent, 0)
	diagnostics = make([]frontend.Diagnostic, 0)

	for _, p := range filePaths {
		if p.IsDir {
			// record a directory as having nil content
			contents = append(contents, PathLabelContent{
				Path:    p.Path,
				Label:   p.Label,
				Content: nil,
			})
		}

		// if it is a file but not a .math file don't use it
		if !strings.HasSuffix(string(p.Path), ".math") {
			continue
		}

		text, err := appendMetaIds(string(p.Path))
		if err != nil {
			diagnostics = append(diagnostics, frontend.Diagnostic{
				Type:    frontend.Error,
				Origin:  frontend.MlgCheckOrigin,
				Path:    p.Path,
				Message: err.Error(),
			})
		} else {
			contents = append(contents, PathLabelContent{
				Path:    p.Path,
				Label:   p.Label,
				Content: &text,
			})
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

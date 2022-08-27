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
	"fmt"
	"mathlingua/internal/frontend"
	"mathlingua/internal/frontend/phase1"
	"mathlingua/internal/frontend/phase2"
	"mathlingua/internal/frontend/phase3"
	"mathlingua/internal/frontend/phase4"
	"os"
	"path"
	"path/filepath"
	"strings"
)

type Mlg interface {
	/**
	 * The paths specified are files or directories.  Only `.math` files
	 * will be processed and for any directory, all `.math` files recursively
	 * in the directory will be processed.
	 */
	Check(paths []string)
	Doc()
	Clean()
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

func parse(text string) (phase4.Root, []frontend.Diagnostic) {
	allDiagnostics := make([]frontend.Diagnostic, 0)

	lexer1 := phase1.NewLexer(text)
	lexer2 := phase2.NewLexer(lexer1)
	lexer3 := phase3.NewLexer(lexer2)

	root, phase4Diagnostics := phase4.Parse(lexer3)

	allDiagnostics = append(allDiagnostics, lexer1.Diagnostics()...)
	allDiagnostics = append(allDiagnostics, lexer2.Diagnostics()...)
	allDiagnostics = append(allDiagnostics, lexer3.Diagnostics()...)
	allDiagnostics = append(allDiagnostics, phase4Diagnostics...)

	return root, allDiagnostics
}

func (m *mlg) Check(paths []string) {
	if len(paths) == 0 {
		paths = append(paths, "contents")
	} else {
		for _, p := range paths {
			stat, _ := os.Stat(p)
			isDir := stat != nil && stat.IsDir()
			if !isDir && !strings.HasSuffix(p, ".math") {
				m.logger.Warning(fmt.Sprintf("File %s is not a MathLingua (.math) file and will be ignored", p))
			}
		}
	}
	numErrors := 0
	numFilesProcessed := 0
	for _, p := range paths {
		err := filepath.Walk(p, func(p string, info os.FileInfo, err error) error {
			if err != nil {
				numErrors++
				return err
			}

			stat, err := os.Stat(p)
			if err != nil {
				numErrors++
				return err
			}

			if stat.IsDir() || !strings.HasSuffix(p, ".math") {
				return nil
			}

			bytes, err := os.ReadFile(p)
			if err != nil {
				numErrors++
				return err
			}

			numFilesProcessed++
			_, diagnostics := parse(string(bytes))
			for index, diag := range diagnostics {
				// make sure there is a space between errors including errors
				// from other files that have already been reported
				if index > 0 || numErrors > 0 {
					// have a blank line between errors
					m.logger.Log("")
				}
				m.logger.Error(fmt.Sprintf("%s (%d, %d)\n%s", p, diag.Position.Row+1, diag.Position.Column+1, diag.Message))
			}
			numErrors += len(diagnostics)

			return nil
		})

		if err != nil {
			m.logger.Error(err.Error())
			continue
		}
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

	if numErrors > 0 {
		m.logger.Log("")
		m.logger.Failure(fmt.Sprintf("Processed %d %s and found %d %s", numFilesProcessed, filesText, numErrors, errorText))
	} else {
		m.logger.Success(fmt.Sprintf("Processed %d %s and found 0 errors", numFilesProcessed, filesText))
	}
}

func (m *mlg) Doc() {
	err := filepath.Walk("contents", func(p string, info os.FileInfo, err error) error {
		if err != nil {
			return err
		}

		if !info.IsDir() && strings.HasSuffix(p, ".math") {
			bytes, err := os.ReadFile(p)
			if err != nil {
				m.logger.Error(fmt.Sprintf("Failed to read '%s': %s", p, err))
			} else {
				htmlRelPath := strings.TrimSuffix(p, ".math") + ".html"
				htmlDocPath := path.Join("docs", htmlRelPath)

				output := fmt.Sprintf("<html><pre>%s</pre></html>", string(bytes))

				base := path.Dir(htmlDocPath)
				doWrite := true
				if err := os.MkdirAll(base, 0700); err != nil {
					if !os.IsExist(err) {
						doWrite = false
						m.logger.Error(fmt.Sprintf("Failed to create the directory '%s': %s", base, err))
					}
				}

				if doWrite {
					if err := os.WriteFile(htmlDocPath, []byte(output), 0644); err != nil {
						m.logger.Error(fmt.Sprintf("Failed to generate '%s': %s", htmlDocPath, err))
					}
				}
			}
		}

		return nil
	})

	if err != nil && !os.IsNotExist(err) {
		fmt.Printf("ERROR: Failed to generate documentation: %s", err)
	}
}

func (m *mlg) logCleanError(message string) {
	m.logger.Error(fmt.Sprintf("Failed to delete the 'docs' directory: %s", message))
}

func (m *mlg) Clean() {
	stat, err := os.Stat("docs")
	if err != nil {
		// if there is an error getting stats for the directory
		// it could be because it doesn't exist or for some other
		// reason.  If it is because it doesn't exist, then there
		// is nothing that needs to be done.  Otherwise, log an
		// error and stop.
		if !os.IsNotExist(err) {
			m.logCleanError(err.Error())
			return
		}
	}

	if !stat.IsDir() {
		m.logCleanError("It exists but is not a directory")
		return
	}

	if err := os.RemoveAll("docs"); err != nil {
		m.logCleanError(err.Error())
	} else {
		m.logger.Success("Deleted the 'docs' directory.")
	}
}

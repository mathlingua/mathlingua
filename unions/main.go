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

package main

import (
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
)

const UNIONS_SUFFIX = ".unions"

func getUnionFiles() ([]string, error) {
	cwd, err := os.Getwd()
	if err != nil {
		return nil, err
	}

	result := make([]string, 0)

	processUnionFiles := func(path string, info os.FileInfo, err error) error {
		if err != nil {
			return err
		}
		if !info.IsDir() && strings.HasSuffix(info.Name(), UNIONS_SUFFIX) {
			result = append(result, path)
		}
		return nil
	}

	err = filepath.Walk(cwd, processUnionFiles)
	if err != nil {
		return nil, err
	}

	return result, nil
}

func processUnionFile(unionFilePath string) error {
	bytes, err := os.ReadFile(unionFilePath)
	if err != nil {
		return err
	}
	content := string(bytes)
	lines := strings.Split(content, "\n")

	out := ""

	row := 0
	for row < len(lines) {
		ln := lines[row]
		row += 1

		if strings.HasPrefix(ln, "union") {
			parts := strings.Fields(ln)

			if parts[0] != "union" {
				return fmt.Errorf("%s (%d): Expected the first token to be 'union'",
					unionFilePath, row+1)
			}

			if parts[len(parts)-1] != "{" {
				return fmt.Errorf("%s (%d): Expected the line to end with }",
					unionFilePath, row+1)
			}

			if len(parts) != 3 && len(parts) != 5 {
				return fmt.Errorf("%s (%d): Expected 'union X {' or 'union X extends {'",
					unionFilePath, row+1)
			}

			if len(parts) == 5 && parts[2] != "extends" {
				return fmt.Errorf("%s (%d): Expected the third token to be 'extends'",
					unionFilePath, row+1)
			}

			unionName := parts[1]

			var extendsNames []string
			if len(parts) == 5 {
				extendsNames = strings.Split(parts[3], ",")
			}

			elements := make([]string, 0)
			functions := make([]string, 0)

			for row < len(lines) {
				elementLine := strings.Trim(lines[row], " ")
				row += 1
				if len(elementLine) == 0 {
					continue
				}

				if strings.Contains(elementLine, "(") {
					functions = append(functions, elementLine)
				} else if elementLine == "}" {
					break
				} else {
					elements = append(elements, elementLine)
				}
			}

			out += fmt.Sprintf("type %s interface {\n", unionName)
			for _, name := range extendsNames {
				out += fmt.Sprintf("\t%s\n", name)
			}
			out += fmt.Sprintf("\t%s()\n", unionName)
			for _, functionDecl := range functions {
				out += fmt.Sprintf("\t%s\n", functionDecl)
			}
			out += "}\n\n"

			for _, element := range elements {
				out += fmt.Sprintf("func (*%s) %s() {}\n", element, unionName)
			}

			out += "\n"
		} else {
			out += ln
			out += "\n"
		}
	}

	outputFilepath := strings.Replace(unionFilePath, UNIONS_SUFFIX, ".go", 1)
	err = os.WriteFile(outputFilepath, []byte(out), 0644)
	if err != nil {
		return err
	}

	return exec.Command("go", "fmt", "./...").Run()
}

func main() {
	unionFiles, err := getUnionFiles()
	if err != nil {
		fmt.Println(err)
		return
	}
	for _, filename := range unionFiles {
		processUnionFile(filename)
	}
}

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

package cmd

import (
	"fmt"
	"os"
	"path"
	"path/filepath"
	"strings"

	"github.com/spf13/cobra"
)

var docCommand = &cobra.Command{
	Use:   "doc",
	Short: "Generate HTML documentation for all MathLingua files",
	Long: "Generates HTML documents in the 'docs' directory for all MathLingua (.math) " +
		"files in the 'content' directory and all sub-directories.",
	Run: func(cmd *cobra.Command, args []string) {
		err := filepath.Walk("contents", func(p string, info os.FileInfo, err error) error {
			if err != nil {
				return err
			}

			if !info.IsDir() && strings.HasSuffix(p, ".math") {
				bytes, err := os.ReadFile(p)
				if err != nil {
					fmt.Printf("ERROR: Failed to read '%s': %s\n", p, err)
				} else {
					htmlRelPath := strings.TrimSuffix(p, ".math") + ".html"
					htmlDocPath := path.Join("docs", htmlRelPath)

					output := fmt.Sprintf("<html><pre>%s</pre></html>", string(bytes))

					base := path.Dir(htmlDocPath)
					doWrite := true
					if err := os.MkdirAll(base, 0700); err != nil {
						if !os.IsExist(err) {
							doWrite = false
							fmt.Printf("ERROR: Failed to create the directory '%s': %s\n", base, err)
						}
					}

					if doWrite {
						if err := os.WriteFile(htmlDocPath, []byte(output), 0644); err != nil {
							fmt.Printf("ERROR: Failed to generate '%s': %s\n", htmlDocPath, err)
						}
					}
				}
			}

			return nil
		})

		if err != nil && !os.IsNotExist(err) {
			fmt.Printf("ERROR: Failed to generate documentation: %s\n", err)
		}
	},
}

func init() {
	rootCmd.AddCommand(docCommand)
}

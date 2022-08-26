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
	"mathlingua/pkg/mlg"

	"github.com/spf13/cobra"
)

var docCommand = &cobra.Command{
	Use:   "doc",
	Short: "Generate HTML documentation for all MathLingua files",
	Long: "Generates HTML documents in the 'docs' directory for all MathLingua (.math) " +
		"files in the 'content' directory and all sub-directories.",
	Run: func(cmd *cobra.Command, args []string) {
		mlg.NewMlg(mlg.NewLogger()).Doc()
	},
}

func init() {
	rootCmd.AddCommand(docCommand)
}

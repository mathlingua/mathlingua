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

	"github.com/spf13/cobra"
)

var checkCommand = &cobra.Command{
	Use:   "check [FILE...]",
	Short: "Check MathLingua files for errors",
	Long: "Checks the specified MathLingua (.math) files for errors, defaulting to all MathLingua " +
		"files in the 'content' directory and all sub-directories if none are explicitly provided.",
	Args: cobra.MinimumNArgs(0),
	Run: func(cmd *cobra.Command, args []string) {
		fmt.Println("in check")
		fmt.Println(args)
	},
}

func init() {
	rootCmd.AddCommand(checkCommand)
}

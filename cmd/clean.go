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

	"github.com/spf13/cobra"
)

func printCleanError(message string) {
	fmt.Printf("ERROR: Failed to delete the 'docs' directory: %s\n", message)
}

var cleanCommand = &cobra.Command{
	Use:   "clean",
	Short: "Remove all generated HTML documents",
	Long:  "Removes all generated HTML documents by deleting the 'docs' directory.",
	Run: func(cmd *cobra.Command, args []string) {
		if stat, err := os.Stat("docs"); err != nil {
			if !os.IsNotExist(err) {
				printCleanError(err.Error())
			}
		} else {
			if stat.IsDir() {
				if err := os.RemoveAll("docs"); err != nil {
					printCleanError(err.Error())
				} else {
					fmt.Println("SUCCESS: Deleted the 'docs' directory.")
				}
			} else {
				printCleanError("It exists but is not a directory")
			}
		}
	},
}

func init() {
	rootCmd.AddCommand(cleanCommand)
}

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
	"mathlingua/internal/logger"
	"mathlingua/internal/mlg"
	"os"

	"github.com/spf13/cobra"
)

var port int

var viewCommand = &cobra.Command{
	Use:   "view",
	Short: "View rendered Mathlingua files",
	Long:  "Renders the Mathlingua (.math) files in the current directory.",
	Run: func(cmd *cobra.Command, args []string) {
		logger := logger.NewLogger(os.Stdout)
		mlg.NewMlg(logger).View(port)
	},
}

func init() {
	flags := viewCommand.Flags()
	flags.IntVarP(&port, "port", "p", 8080, "The port on which to view the documents")
	rootCmd.AddCommand(viewCommand)
}

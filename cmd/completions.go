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

package cmd

import (
	"encoding/json"
	"fmt"
	"mathlingua/internal/ast"
	"strings"

	"github.com/spf13/cobra"
)

var completionsCommand = &cobra.Command{
	Use:    "completions",
	Hidden: true,
	Run: func(cmd *cobra.Command, args []string) {
		printCompletions()
	},
}

func init() {
	rootCmd.AddCommand(completionsCommand)
}

type completionsResult struct {
	Completions []string
}

func printCompletions() {
	if data, err := json.MarshalIndent(COMPLETION_RESULT, "", "  "); err != nil {
		fmt.Println("{\"Completions\": []}")
	} else {
		fmt.Println(string(data))
	}
}

var COMPLETION_RESULT = getCompletionResult()

func getCompletionResult() completionsResult {
	completions := []string{
		"in",
		"is",
		"as",
		"extends",
		join(ast.GivenSections),
		join(ast.AllOfSections),
		join(ast.NotSections),
		join(ast.AnyOfSections),
		join(ast.OneOfSections),
		join(ast.ExistsSections),
		join(ast.ExistsUniqueSections),
		join(ast.ForAllSections),
		join(ast.IfSections),
		join(ast.IffSections),
		join(ast.PiecewiseSections),
		join(ast.WhenSections),
		join(ast.SymbolSections),
		join(ast.ConnectionSections),
		join(ast.WrittenSections),
		join(ast.CalledSections),
		join(ast.ExpressedSections),
		join(ast.OverviewSections),
		join(ast.RelatedSections),
		join(ast.LabelSections),
		join(ast.BySections),
		join(ast.DescribesSections),
		join(ast.DefinesSections),
		join(ast.StatesSections),
		join(ast.ProofSections),
		join(ast.AxiomSections),
		join(ast.ConjectureSections),
		join(ast.TheoremSections),
		join(ast.ZeroSections),
		join(ast.PositiveIntSections),
		join(ast.NegativeIntSections),
		join(ast.PositiveFloatSections),
		join(ast.NegativeFloatSections),
		join(ast.SpecifySections),
		join(ast.PersonSections),
		join(ast.NameSections),
		join(ast.BiographySections),
		join(ast.ResourceSections),
	}
	return completionsResult{
		Completions: completions,
	}
}

func join(parts []string) string {
	// convert ["a", "b"] to "a:\nb:"
	return strings.Join(parts, ":\n") + ":"
}

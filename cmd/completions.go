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
	"mathlingua/pkg/mlg"
	"os"
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
	logger := mlg.NewLogger(os.Stdout)
	usages := mlg.NewMlg(logger).GetUsages()
	usagesAndCompletions := make([]string, 0)
	usagesAndCompletions = append(usagesAndCompletions, FIXED_COMPLETIONS...)
	usagesAndCompletions = append(usagesAndCompletions, usages...)

	result := completionsResult{
		Completions: usagesAndCompletions,
	}

	if data, err := json.MarshalIndent(result, "", "  "); err != nil {
		fmt.Println("{\"Completions\": []}")
	} else {
		fmt.Println(string(data))
	}
}

var FIXED_COMPLETIONS = getFixedCompletions()

func getFixedCompletions() []string {
	return []string{
		"is",
		"as",
		"extends",
		join(ast.LetSections),
		join(ast.AllOfSections),
		join(ast.EquivalentlySections),
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
		join(ast.SymbolWrittenSections),
		join(ast.ViewSections),
		join(ast.EncodingSections),
		join(ast.WrittenSections),
		join(ast.CalledSections),
		join(ast.WritingSections),
		join(ast.OverviewSections),
		join(ast.RelatedSections),
		join(ast.LabelSections),
		join(ast.BySections),
		join(ast.DescribesSections),
		join(ast.DefinesSections),
		join(ast.LowerDefineSections),
		join(ast.CapturesSections),
		join(ast.StatesSections),
		join(ast.AxiomSections),
		join(ast.ConjectureSections),
		join(ast.TheoremSections),
		join(ast.CorollarySections),
		join(ast.LemmaSections),
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
		join(ast.TitleSections),
		join(ast.AuthorSections),
		join(ast.OffsetSections),
		join(ast.UrlSections),
		join(ast.HomepageSections),
		join(ast.TypeSections),
		join(ast.EditorSections),
		join(ast.EditionSections),
		join(ast.InstitutionSections),
		join(ast.JournalSections),
		join(ast.PublisherSections),
		join(ast.VolumeSections),
		join(ast.MonthSections),
		join(ast.YearSections),
		join(ast.DescriptionSections),
		join(ast.ProofByBecauseThenSections),
		join(ast.ProofBecauseThenSections),
		join(ast.ProofStepwiseSections),
		join(ast.ProofSupposeSections),
		join(ast.ProofBlockSections),
		join(ast.ProofWithoutLossOfGeneralitySections),
		join(ast.ProofContradictionSections),
		join(ast.ProofForContradictionSections),
		join(ast.ProofForInductionSections),
		join(ast.ProofClaimSections),
		join(ast.ProofCasewiseSections),
		join(ast.ProofEquivalentlySections),
		join(ast.ProofAllOfSections),
		join(ast.ProofNotSections),
		join(ast.ProofAnyOfSections),
		join(ast.ProofOneOfSections),
		join(ast.ProofExistsSections),
		join(ast.ProofExistsUniqueSections),
		join(ast.ProofForAllSections),
		join(ast.ProofLetSections),
		join(ast.ProofIfSections),
		join(ast.ProofIffSections),
		join(ast.ProofForContrapositiveSections),
		join(ast.ProofQedSections),
		join(ast.ProofAbsurdSections),
		join(ast.ProofDoneSections),
		join(ast.ProofPartwiseSections),
		join(ast.ProofSufficesToShowSections),
		join(ast.ProofToShowSections),
	}
}

func join(parts []string) string {
	// convert ["a", "b"] to "a:\nb:"
	return strings.Join(parts, ":\n") + ":"
}

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
	"errors"
	"fmt"
	"mathlingua/internal/ast"
	"mathlingua/internal/frontend"
	"mathlingua/internal/frontend/formulation"
	"mathlingua/internal/frontend/phase1"
	"mathlingua/internal/frontend/phase2"
	"mathlingua/internal/frontend/phase3"
	"mathlingua/internal/frontend/phase4"
	"mathlingua/internal/frontend/phase5"
	"mathlingua/internal/mlglib"
	"os"
	"path"

	tcell "github.com/gdamore/tcell/v2"
	"github.com/muesli/termenv"
	"github.com/rivo/tview"
	"github.com/spf13/cobra"
)

const test_case_success_message = "Successfully generated test case in testbed/testcases.txt"

var debugCommand = &cobra.Command{
	Use:    "debug",
	Hidden: true,
	Run: func(cmd *cobra.Command, args []string) {
		setupScreen()
	},
}

var useStructuralParser bool
var useFormulationParser bool
var useIdParser bool
var useFormParser bool
var useSignatureParser bool
var direct bool
var showAst bool

func init() {
	flags := debugCommand.Flags()
	flags.BoolVar(&useStructuralParser, "structural", false, "Use the structural parser")
	flags.BoolVar(&useFormulationParser, "formulation", false, "Use the formulation parser")
	flags.BoolVar(&useIdParser, "id", false, "Use the id parser")
	flags.BoolVar(&useFormParser, "form", false, "Use the form parser")
	flags.BoolVar(&useSignatureParser, "signature", false, "Use the signature parser")
	flags.BoolVar(&direct, "direct", false, "Directly check the input.txt text and don't open the ux")
	flags.BoolVar(&showAst, "ast", false, "Show the AST in the output pane")
	debugCommand.MarkFlagsMutuallyExclusive("structural", "formulation", "id", "form")
	rootCmd.AddCommand(debugCommand)
}

func setupScreen() {
	if !useStructuralParser && !useFormulationParser &&
		!useIdParser && !useFormParser && !useSignatureParser {
		useStructuralParser = true
	}

	isDarkMode := termenv.HasDarkBackground()

	home, err := os.UserHomeDir()
	if err != nil {
		fmt.Println(err.Error())
		os.Exit(1)
	}
	inputFile := path.Join(home, ".mlg_debug_backup.math")

	if direct {
		_, err := os.Stat(inputFile)
		if err != nil && !errors.Is(err, os.ErrNotExist) {
			fmt.Println(err.Error())
			os.Exit(1)
		}

		bytes, err := os.ReadFile(inputFile)
		if err != nil {
			fmt.Println(err.Error())
			os.Exit(1)
		}

		node, ast, tracker := parse(string(bytes))

		fmt.Println("Output:")
		fmt.Println(node)

		fmt.Println("AST:")
		fmt.Println(ast)

		fmt.Println("Diagnostics:")
		diagnostics := tracker.Diagnostics()
		if len(diagnostics) > 0 {
			for _, diag := range diagnostics {
				fmt.Printf("%s (%d, %d) {%s}: %s\n",
					diag.Type,
					(diag.Position.Row + 1),
					(diag.Position.Column + 1),
					diag.Origin,
					diag.Message)
			}
		}
		os.Exit(0)
	}

	testcaseFile := path.Join("testbed", "testcases.txt")

	app := tview.NewApplication()

	outputArea := tview.NewTextView()
	outputArea.SetTitle("Output").SetBorder(true)
	outputArea.SetDynamicColors(false)
	if !isDarkMode {
		outputArea.SetTitleColor(tcell.ColorBlack)
		outputArea.SetBorderColor(tcell.ColorBlack)
		outputArea.SetTextColor(tcell.ColorBlack)
		outputArea.SetBackgroundColor(tcell.ColorWhite)
	}

	helpInfo := tview.NewTextView().SetDynamicColors(true)
	if !isDarkMode {
		helpInfo.SetBackgroundColor(tcell.ColorWhite)
		helpInfo.SetTextColor(tcell.ColorBlack)
	}
	resetHelpInfo := func() {
		helpInfo.SetText(" Ctrl-R: check, Ctrl-T: write test case, Ctrl-C: exit, Ctrl-K: clear")
	}
	setHelpInfoError := func(message string) {
		helpInfo.SetText(fmt.Sprintf(" %s", message))
	}
	setHelpInfoSuccess := func(message string) {
		helpInfo.SetText(fmt.Sprintf(" %s", message))
	}
	resetHelpInfo()

	position := tview.NewTextView()
	position.SetDynamicColors(true)
	position.SetTextAlign(tview.AlignRight)
	if !isDarkMode {
		position.SetBackgroundColor(tcell.ColorWhite)
		position.SetTextColor(tcell.ColorBlack)
	}

	startText := ""
	if _, err := os.Stat(inputFile); !errors.Is(err, os.ErrNotExist) {
		bytes, err := os.ReadFile(inputFile)
		if err != nil {
			setHelpInfoError(err.Error())
			app.Stop()
		}
		startText = string(bytes)
	}

	inputArea := tview.NewTextArea()
	inputArea.SetBorder(true)
	inputArea.SetText(startText, false)
	if !isDarkMode {
		inputArea.SetTitleColor(tcell.ColorBlack)
		inputArea.SetBorderColor(tcell.ColorBlack)
		inputArea.SetBackgroundColor(tcell.ColorWhite)
		inputArea.SetTextStyle(tcell.StyleDefault)
	}

	if useStructuralParser {
		inputArea.SetTitle("Input (structural)")
	} else if useFormulationParser {
		inputArea.SetTitle("Input (formulation)")
	} else if useIdParser {
		inputArea.SetTitle("Input (id)")
	} else if useFormParser {
		inputArea.SetTitle("Input (form)")
	} else if useSignatureParser {
		inputArea.SetTitle("Input (signature)")
	} else {
		inputArea.SetTitle("Input")
	}

	updateInfos := func() {
		_, _, toRow, toColumn := inputArea.GetCursor()
		position.SetText(fmt.Sprintf("Row: %d, Column: %d ", toRow+1, toColumn+1))
	}

	inputArea.SetMovedFunc(updateInfos)
	updateInfos()

	mainView := tview.NewGrid().
		SetRows(0, 1).
		AddItem(inputArea, 0, 0, 1, 1, 0, 0, true).
		AddItem(outputArea, 0, 1, 1, 1, 0, 0, false).
		AddItem(helpInfo, 1, 0, 1, 1, 0, 0, false).
		AddItem(position, 1, 1, 1, 1, 0, 0, false)

	app.SetInputCapture(func(event *tcell.EventKey) *tcell.EventKey {
		if event.Key() == tcell.KeyCtrlSpace {
			if app.GetFocus() == outputArea {
				app.SetFocus(inputArea)
			} else {
				app.SetFocus(outputArea)
			}
			return event
		}

		resetHelpInfo()
		if err := os.WriteFile(inputFile, []byte(inputArea.GetText()), 0644); err != nil {
			setHelpInfoError(err.Error())
		}

		if event.Key() == tcell.KeyCtrlK {
			inputArea.SetText("", false)
			outputArea.SetText("")
			return nil
		}

		if event.Key() == tcell.KeyCtrlR || event.Key() == tcell.KeyCtrlT {
			outputArea.SetText("")

			srcText, astText, tracker := parse(inputArea.GetText())
			diagnostics := tracker.Diagnostics()
			if len(diagnostics) > 0 {
				output := ""
				for _, diag := range diagnostics {
					output += fmt.Sprintf("%s (%d, %d) {%s}: %s\n",
						diag.Type,
						(diag.Position.Row + 1),
						(diag.Position.Column + 1),
						diag.Origin,
						diag.Message)
				}
				outputArea.SetText(output)
			} else {
				if showAst {
					outputArea.SetText(astText)
				} else {
					outputArea.SetText(srcText)
				}
			}
		}

		if event.Key() == tcell.KeyCtrlT {
			testcase, message, ok := createTestCase(inputArea.GetText())
			if ok {
				setHelpInfoSuccess(message)
				if err := os.WriteFile(testcaseFile, []byte(testcase), 0644); err != nil {
					setHelpInfoError(err.Error())
				}
			} else {
				setHelpInfoError(message)
			}
		}

		return event
	})

	if err := app.SetRoot(mainView,
		true).EnableMouse(true).Run(); err != nil {
		panic(err)
	}
}

////////////////////////////////////////////////////////////////////////////////////////////////////

func parse(text string) (string, string, *frontend.DiagnosticTracker) {
	if useStructuralParser {
		return parseForStructural(text)
	} else if useFormulationParser {
		return parseForFormulation(text)
	} else if useFormParser {
		return parseForForm(text)
	} else if useIdParser {
		return parseForId(text)
	} else if useSignatureParser {
		return parseForSignature(text)
	} else {
		panic("Unsupported parser")
	}
}

// The output is: (testCase, errorMessage, successOrFailure)
// errorMessage is blank if it is a success and a string if it is a failure
func createTestCase(input string) (string, string, bool) {
	if useStructuralParser {
		return createTestCaseForStructural(input)
	} else if useFormulationParser {
		return createTestCaseForFormulation(input)
	} else if useFormParser {
		return createTestCaseForForm(input)
	} else if useIdParser {
		return createTestCaseForId(input)
	} else {
		return "", "Unsupported parser", false
	}
}

////////////////////////////////////// structural parser ///////////////////////////////////////////

func parseForStructural(text string) (string, string, *frontend.DiagnosticTracker) {
	tracker := frontend.NewDiagnosticTracker()

	lexer1 := phase1.NewLexer(text, "", tracker)
	lexer2 := phase2.NewLexer(lexer1, "", tracker)
	lexer3 := phase3.NewLexer(lexer2, "", tracker)

	root := phase4.Parse(lexer3, "", tracker)
	doc, ok := phase5.Parse(root, "", tracker, mlglib.NewKeyGenerator())

	astText := ""
	if ok {
		astText = ast.StructuralNodeToCode(&doc)
	}
	return astText, mlglib.PrettyPrint(doc), tracker
}

func createTestCaseForStructural(input string) (string, string, bool) {
	parseCode := `
func parse(text string) (ast.Document, frontend.IDiagnosticTracker) {
	tracker := frontend.NewDiagnosticTracker()

	lexer1 := phase1.NewLexer(text, "", tracker)
	lexer2 := phase2.NewLexer(lexer1, "", tracker)
	lexer3 := phase3.NewLexer(lexer2, "", tracker)

	root := phase4.Parse(lexer3, "", tracker)
	doc, _ := phase5.Parse(root, "", tracker)

	return doc, tracker
}
`
	return createGeneralTestCase(input, func(input string) (any, *frontend.DiagnosticTracker) {
		text, _, tracker := parseForStructural(input)
		return text, tracker
	}, parseCode)
}

////////////////////////////////////// formulation /////////////////////////////////////////////////

func parseForFormulation(text string) (string, string, *frontend.DiagnosticTracker) {
	tracker := frontend.NewDiagnosticTracker()
	node, ok := formulation.ParseExpression(
		"", text, ast.Position{}, tracker, mlglib.NewKeyGenerator())
	astText := ""
	if ok {
		astText = ast.FormulationNodeToCode(node, ast.NoOp)
	}
	return astText, mlglib.PrettyPrint(node), tracker
}

func createTestCaseForFormulation(input string) (string, string, bool) {
	parseCode := `
func parse(text string) (ast.MlgNodeKind, frontend.IDiagnosticTracker) {
	tracker := frontend.NewDiagnosticTracker()
	node, _ := formulation.ParseExpression(text, ast.Position{}, tracker)
	return node, tracker
}
`
	return createGeneralTestCase(input, func(input string) (any, *frontend.DiagnosticTracker) {
		text, _, tracker := parseForFormulation(input)
		return text, tracker
	}, parseCode)
}

///////////////////////////////////////// form /////////////////////////////////////////////////////

func parseForForm(text string) (string, string, *frontend.DiagnosticTracker) {
	tracker := frontend.NewDiagnosticTracker()
	node, ok := formulation.ParseForm("", text, ast.Position{}, tracker, mlglib.NewKeyGenerator())
	astText := ""
	if ok {
		astText = ast.FormulationNodeToCode(node, ast.NoOp)
	}
	return astText, mlglib.PrettyPrint(node), tracker
}

func createTestCaseForForm(input string) (string, string, bool) {
	parseCode := `
func parse(text string) (ast.MlgNodeKind, frontend.IDiagnosticTracker) {
	tracker := frontend.NewDiagnosticTracker()
	node, _ := formulation.ParseForm(text, ast.Position{}, tracker)
	return node, tracker
}
`
	return createGeneralTestCase(input, func(input string) (any, *frontend.DiagnosticTracker) {
		text, _, tracker := parseForForm(input)
		return text, tracker
	}, parseCode)
}

//////////////////////////////////// signature /////////////////////////////////////////////////////

func parseForSignature(text string) (string, string, *frontend.DiagnosticTracker) {
	tracker := frontend.NewDiagnosticTracker()
	node, ok := formulation.ParseSignature(
		"", text, ast.Position{}, tracker, mlglib.NewKeyGenerator())
	astText := ""
	if ok {
		astText = ast.FormulationNodeToCode(&node, ast.NoOp)
	}
	return astText, mlglib.PrettyPrint(node), tracker
}

/////////////////////////////////////////// id /////////////////////////////////////////////////////

func parseForId(text string) (string, string, *frontend.DiagnosticTracker) {
	tracker := frontend.NewDiagnosticTracker()
	node, ok := formulation.ParseId("", text, ast.Position{}, tracker, mlglib.NewKeyGenerator())
	astText := ""
	if ok {
		astText = ast.FormulationNodeToCode(node, ast.NoOp)
	}
	return astText, mlglib.PrettyPrint(node), tracker
}

func createTestCaseForId(input string) (string, string, bool) {
	parseCode := `
func parse(text string) (ast.MlgNodeKind, frontend.IDiagnosticTracker) {
	tracker := frontend.NewDiagnosticTracker()
	node, _ := formulation.ParseId(text, ast.Position{}, tracker)
	return node, tracker
}
`
	return createGeneralTestCase(input, func(input string) (any, *frontend.DiagnosticTracker) {
		text, _, tracker := parseForId(input)
		return text, tracker
	}, parseCode)
}

////////////////////////////////////////////////////////////////////////////////////////////////////

func createGeneralTestCase(input string, parseFn func(input string) (
	any, *frontend.DiagnosticTracker), parseCode string) (string, string, bool) {
	data, tracker := parseFn(input)
	diagnostics := tracker.Diagnostics()
	if len(diagnostics) > 0 {
		return "", "Diagnostics exist: Cannot create test case", false
	}
	expectedOutput := mlglib.PrettyPrint(data)

	testcase := fmt.Sprintf(parseCode + `

func runTest(t *testing.T, input string, expected string) {
	doc, tracker := parse(input)
	actual := mlglib.PrettyPrint(doc)

	assert.Equal(t, expected, actual)
	assert.Equal(t, 0, len(tracker.Diagnostics()))
}

func Test_____(t *testing.T) {
	input := ` + "`" + input + "`" + `
	expected := ` + "`" + expectedOutput + "`" + `
	runTest(t, input, expected)
}

`)
	return testcase, test_case_success_message, true
}

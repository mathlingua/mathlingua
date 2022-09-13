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
	"mathlingua/cmd"
	"mathlingua/internal/ast"
	"mathlingua/internal/frontend"
	"mathlingua/internal/frontend/formulation"
	"mathlingua/internal/frontend/phase1"
	"mathlingua/internal/frontend/phase2"
	"mathlingua/internal/frontend/phase3"
	"mathlingua/internal/frontend/phase4"
	"mathlingua/internal/frontend/phase5"
	"os"

	"github.com/kr/pretty"
)

func main() {
	if len(os.Getenv("TESTBED1")) > 0 {
		if len(os.Args) != 2 {
			fmt.Println("Expected a single argument that is the text to process")
			os.Exit(1)
		}
		text := os.Args[1]
		fmt.Println(text)
		tracker := frontend.NewDiagnosticTracker()
		node, ok := formulation.ParseExpression(text, tracker)
		fmt.Println("ok=", ok)
		fmt.Printf("%s\n", pretty.Sprintf("%# v", node))
		for _, d := range tracker.Diagnostics() {
			fmt.Printf("%#v\n", d)
		}

		switch node := node.(type) {
		case ast.PseudoExpression:
			root, ok := formulation.Consolidate(node.Children, tracker)
			fmt.Println("ok=", ok)
			fmt.Printf("%s\n", pretty.Sprintf("%# v", root))
		}
	} else if len(os.Getenv("TESTBED2")) > 0 {
		text := `
exists: f(x)
`
		tracker := frontend.NewDiagnosticTracker()

		lexer1 := phase1.NewLexer(text, tracker)
		lexer2 := phase2.NewLexer(lexer1, tracker)
		lexer3 := phase3.NewLexer(lexer2, tracker)

		root := phase4.Parse(lexer3, tracker)
		group := root.Nodes[0].(phase4.Group)

		parser := phase5.NewPhase5Parser(tracker)
		grp, ok := parser.ToExistsGroup(group)
		fmt.Println("ok=", ok)
		fmt.Printf("%s\n", pretty.Sprintf("%# v", grp))

		for _, diag := range tracker.Diagnostics() {
			fmt.Println(diag.Message)
		}
	} else {
		cmd.Execute()
	}
}

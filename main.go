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
	"os"

	"github.com/kr/pretty"
)

func main() {
	if len(os.Getenv("TESTBED")) > 0 {
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
	} else {
		cmd.Execute()
	}
}
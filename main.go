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
	"bufio"
	"fmt"
	"mathlingua/internal/frontend/phase1"
	"mathlingua/internal/frontend/phase2"
	"mathlingua/internal/frontend/phase3"
	"mathlingua/internal/frontend/phase4"
	"os"
	"strings"

	"github.com/kr/pretty"
)

func main() {
	reader := bufio.NewReader(os.Stdin)
	text := ""
	for !strings.HasSuffix(text, "\n\n\n") {
		line, _ := reader.ReadString('\n')
		text += line
	}

	lexer1 := phase1.NewLexer(text)
	lexer2 := phase2.NewLexer(lexer1)
	lexer3 := phase3.NewLexer(lexer2)

	root, diagnostics := phase4.Parse(lexer3)

	fmt.Println("AST:")
	fmt.Println("----")
	fmt.Printf("%# v\n\n", pretty.Formatter(root))

	fmt.Println("Generated Code:")
	fmt.Println("---------------")
	fmt.Println(root.ToCode())

	if len(diagnostics) > 0 {
		fmt.Println("\n\nDiagnostics:")
		for _, diag := range diagnostics {
			fmt.Printf("%#v\n", diag)
		}
	}
}

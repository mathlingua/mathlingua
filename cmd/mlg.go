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
	"mathlingua/internal/frontend"
	"os"
	"strings"
)

func main() {
	reader := bufio.NewReader(os.Stdin)
	text := ""
	for !strings.HasSuffix(text, "\n\n\n") {
		line, _ := reader.ReadString('\n')
		text += line
	}

	lexer1 := frontend.NewPhase1Lexer(text)
	lexer2 := frontend.NewPhase2Lexer(lexer1)
	lexer3 := frontend.NewPhase3Lexer(lexer2)

	lexer := lexer3
	for lexer.HasNext() {
		fmt.Printf("%s\n", lexer.Next().Text)
	}

	diagnostics := lexer.Diagnostics()
	if len(diagnostics) > 0 {
		fmt.Println("\nDiagnostics:")
		for _, diag := range lexer.Diagnostics() {
			fmt.Printf("%#v\n", diag)
		}
	}
}

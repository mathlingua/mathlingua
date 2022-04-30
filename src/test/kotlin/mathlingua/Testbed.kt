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

package mathlingua

import mathlingua.lib.frontend.Frontend

fun main() {
    var exit = false
    while (!exit) {
        println("Enter MathLingua content (type ':help' for help):")
        println("-------------------------------------------------")
        val input = StringBuilder()
        while (true) {
            val line = readLine() ?: break
            when (line) {
                ":help" -> {
                    println("Enter MathLingua text to process, which can contain many lines, followed by a command.")
                    println("Commands are entered on a new line and are of the form ':<command>'")
                    println("Supported commands:")
                    println()
                    println(":help    Show this help message")
                    println(":ignore  Ignore the entered MathLingua text and start over with new input")
                    println(":exit    Immediately exit without processing the entered MathLingua text")
                    println(":done    Process the entered MathLingua text and start over with new input")
                    println()
                    break
                }
                ":ignore" -> {
                    break
                }
                ":exit" -> {
                    exit = true
                    break
                }
                ":done" -> {
                    val parse = Frontend.parse(input.toString())
                    println()
                    println("Document:")
                    println("---------")
                    println(parse.doc.toCode())
                    println()
                    val message = "Diagnostics (${parse.diagnostics.size}):"
                    println(message)
                    println("-".repeat(message.length))
                    parse.diagnostics.forEach(::println)
                    println()
                    break
                }
                else -> {
                    input.append(line)
                    input.append('\n')
                }
            }
        }
    }
}

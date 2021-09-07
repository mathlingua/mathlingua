/*
 * Copyright 2019 The MathLingua Authors
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

package mathlingua.mathlingua.chalktalk.phase1

import assertk.assertThat
import assertk.assertions.isEqualTo
import assertk.assertions.isNotNull
import mathlingua.GoldenType
import mathlingua.OVERWRITE_GOLDEN_FILES
import mathlingua.frontend.chalktalk.phase1.newChalkTalkLexer
import mathlingua.frontend.chalktalk.phase1.newChalkTalkParser
import mathlingua.frontend.support.ParseError
import mathlingua.loadTestCases
import mathlingua.serialize
import org.junit.jupiter.api.DynamicTest
import org.junit.jupiter.api.TestFactory

internal class ChalkTalkParserTest {
    @TestFactory
    fun `Golden Phase 1 Parse Tests`(): Collection<DynamicTest> {
        val testCases = loadTestCases(GoldenType.Chalktalk)

        return testCases.map {
            DynamicTest.dynamicTest("ChalkTalk Parser: ${it.name}") {
                val lexer = newChalkTalkLexer(it.input.readText())
                assertThat(lexer.errors().size).isEqualTo(0)

                val parser = newChalkTalkParser()
                val result = parser.parse(lexer)
                if (result.errors.isNotEmpty()) {
                    for (err in result.errors) {
                        println("ERROR: $err")
                    }
                }
                assertThat(result.errors).isEqualTo(emptyList<ParseError>())
                assertThat(result.root).isNotNull()

                val actualOutput = result.root!!.toCode()
                val actualStructure = serialize(result.root!!)
                if (OVERWRITE_GOLDEN_FILES) {
                    println("Overwriting phase1 ChalkTalk test: ${it.name}")
                    it.phase1Output.writeText(actualOutput)
                    it.phase1Structure.writeText(actualStructure)
                } else {
                    assertThat(actualOutput).isEqualTo(it.phase1Output.readText())
                    assertThat(actualStructure).isEqualTo(it.phase1Structure.readText())
                }
            }
        }
    }
}

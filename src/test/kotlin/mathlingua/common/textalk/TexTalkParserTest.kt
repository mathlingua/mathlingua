/*
 * Copyright 2019 Google LLC
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

package mathlingua.common.textalk

import assertk.assertThat
import assertk.assertions.isEqualTo
import assertk.assertions.isNotNull
import mathlingua.GoldenType
import mathlingua.loadTestCases
import org.junit.jupiter.api.DynamicTest
import org.junit.jupiter.api.TestFactory

internal class TexTalkParserTest {
    @TestFactory
    fun `Golden TexTalk Tests`(): Collection<DynamicTest> {
        val testCases = loadTestCases(GoldenType.Textalk)

        return testCases.map {
            DynamicTest.dynamicTest("TexTalk Parser: ${it.name}") {
                val lexer = newTexTalkLexer(it.input)
                assertThat(lexer.errors.size).isEqualTo(0)

                val parser = newTexTalkParser()
                val result = parser.parse(lexer)
                for (err in result.errors) {
                    println("ERROR: $err")
                }
                assertThat(result.errors.size).isEqualTo(0)
                assertThat(result.root).isNotNull()

                assertThat(result.root.toCode().trim()).isEqualTo(it.phase1Output.trim())
            }
        }
    }
}

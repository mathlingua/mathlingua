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

package mathlingua.common.chalktalk.phase1

import assertk.assertThat
import assertk.assertions.isEqualTo
import assertk.assertions.isNotNull
import mathlingua.loadTestCases
import org.junit.jupiter.api.DynamicTest
import org.junit.jupiter.api.TestFactory
import java.nio.file.Paths

internal class ChalkTalkParserTest {
    @TestFactory
    fun `Golden Phase 1 Parse Tests`(): Collection<DynamicTest> {
        val goldenFile = Paths.get("src", "test", "resources", "golden-chalktalk.txt").toFile()
        val testCases = loadTestCases(goldenFile)

        return testCases.map {
            DynamicTest.dynamicTest("ChalkTalk Parser: ${it.name}") {
                val lexer = newChalkTalkLexer(it.input)
                assertThat(lexer.errors().size).isEqualTo(0)

                val parser = newChalkTalkParser()
                val result = parser.parse(lexer)
                if (result.errors.isNotEmpty()) {
                    for (err in result.errors) {
                        println("ERROR: $err")
                    }
                }
                assertThat(result.errors.size).isEqualTo(0)
                assertThat(result.root).isNotNull()

                assertThat(result.root!!.toCode()).isEqualTo(it.phase1Output)
            }
        }
    }
}

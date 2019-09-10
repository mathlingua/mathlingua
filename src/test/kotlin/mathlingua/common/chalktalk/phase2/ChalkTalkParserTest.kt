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

package mathlingua.common.chalktalk.phase2

import assertk.assertThat
import assertk.assertions.isEqualTo
import assertk.assertions.isNotNull
import mathlingua.common.ValidationSuccess
import mathlingua.common.chalktalk.phase1.newChalkTalkLexer
import mathlingua.common.chalktalk.phase1.newChalkTalkParser
import mathlingua.loadTestCases
import org.junit.jupiter.api.DynamicTest
import org.junit.jupiter.api.TestFactory
import java.nio.file.Paths

internal class ChalkTalkParserTest {
    @TestFactory
    fun `Golden Validation Tests`(): Collection<DynamicTest> {
        val goldenFile = Paths.get("src", "test", "resources", "golden-chalktalk.txt").toFile()
        val testCases = loadTestCases(goldenFile)

        return testCases.map {
            DynamicTest.dynamicTest("ChalkTalk Validation: ${it.name}") {
                val lexer = newChalkTalkLexer(it.input)
                assertThat(lexer.errors().size).isEqualTo(0)

                val parser = newChalkTalkParser()
                val result = parser.parse(lexer)
                assertThat(result.errors.size).isEqualTo(0)
                assertThat(result.root).isNotNull()

                val validation = validateDocument(result.root!!)
                assert(validation is ValidationSuccess)

                val doc = (validation as ValidationSuccess).value
                assertThat(doc.toCode(false, 0).trim()).isEqualTo(it.expectedOutput.trim())
            }
        }
    }
}

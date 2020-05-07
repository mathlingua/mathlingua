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
import assertk.assertions.isInstanceOf
import assertk.assertions.isNotNull
import assertk.assertions.isTrue
import mathlingua.GoldenType
import mathlingua.common.LocationTracker
import mathlingua.common.ValidationSuccess
import mathlingua.common.chalktalk.phase1.newChalkTalkLexer
import mathlingua.common.chalktalk.phase1.newChalkTalkParser
import mathlingua.common.chalktalk.phase2.ast.Phase2Node
import mathlingua.common.chalktalk.phase2.ast.validateDocument
import mathlingua.common.newLocationTracker
import mathlingua.loadTestCases
import mathlingua.serialize
import org.junit.jupiter.api.DynamicTest
import org.junit.jupiter.api.TestFactory

internal class ChalkTalkParserTest {
    @TestFactory
    fun `Golden Validation Tests`(): Collection<DynamicTest> {
        val testCases = loadTestCases(GoldenType.Chalktalk)

        return testCases.map {
            DynamicTest.dynamicTest("ChalkTalk Validation: ${it.name}") {
                val lexer = newChalkTalkLexer(it.input)
                assertThat(lexer.errors().size).isEqualTo(0)

                val parser = newChalkTalkParser()
                val result = parser.parse(lexer)
                assertThat(result.errors.size).isEqualTo(0)
                assertThat(result.root).isNotNull()

                val tracker = newLocationTracker()
                val validation = validateDocument(result.root!!, tracker)
                assertThat(validation).isInstanceOf(ValidationSuccess::class.java)

                val doc = (validation as ValidationSuccess).value
                assertThat(doc.toCode(false, 0).getCode().trim()).isEqualTo(it.phase2Output.trim())
                assertThat(serialize(doc)).isEqualTo(it.phase2Structure)

                assertTrackerContainsNode(tracker, doc)
            }
        }
    }

    private fun assertTrackerContainsNode(tracker: LocationTracker, node: Phase2Node) {
        assertThat(tracker.hasLocationOf(node)).isTrue()
        assertThat(tracker.getLocationOf(node)).isNotNull()

        node.forEach { assertTrackerContainsNode(tracker, it) }
    }
}

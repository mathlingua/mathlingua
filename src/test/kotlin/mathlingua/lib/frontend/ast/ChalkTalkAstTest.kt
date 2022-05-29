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

package mathlingua.lib.frontend.ast

import kotlin.test.Test
import kotlin.test.assertEquals
import mathlingua.lib.frontend.MetaData

internal class ChalkTalkAstTest {
    @Test
    fun `correctly renders args to code`() {
        val metadata = MetaData(row = -1, column = -1, isInline = false)
        assertEquals(
            expected =
                """
                if: 'a', 'b'
                . 'c', 'd'
                . 'e'
            """.trimIndent(),
            actual =
                IfSection(
                        clauses =
                            listOf(
                                Statement(
                                    text = "a",
                                    metadata = MetaData(row = 0, column = 0, isInline = true)),
                                Statement(
                                    text = "b",
                                    metadata = MetaData(row = 0, column = 0, isInline = true)),
                                Statement(
                                    text = "c",
                                    metadata = MetaData(row = 0, column = 0, isInline = false)),
                                Statement(
                                    text = "d",
                                    metadata = MetaData(row = 0, column = 0, isInline = true)),
                                Statement(
                                    text = "e",
                                    metadata = MetaData(row = 0, column = 0, isInline = false)),
                            ),
                        metadata = metadata)
                    .toCode())
    }

    @Test
    fun `correctly renders a group with multiple sections`() {
        val metadata = MetaData(row = -1, column = -1, isInline = false)
        assertEquals(
            expected =
                """
                if: 'a'
                then: 'b'
            """.trimIndent(),
            actual =
                IfGroup(
                        ifSection =
                            IfSection(
                                clauses =
                                    listOf(
                                        Statement(
                                            text = "a",
                                            metadata =
                                                MetaData(row = 0, column = 0, isInline = true))),
                                metadata = metadata),
                        thenSection =
                            ThenSection(
                                clauses =
                                    listOf(
                                        Statement(
                                            text = "b",
                                            metadata =
                                                MetaData(row = 0, column = 0, isInline = true)),
                                    ),
                                metadata = metadata),
                        metadata = metadata)
                    .toCode())
    }
}

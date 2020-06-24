/*
 * Copyright 2020 Google LLC
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

package mathlingua.common.transform

import assertk.assertThat
import assertk.assertions.isEqualTo
import mathlingua.common.textalk.Command
import mathlingua.common.textalk.ExpressionTexTalkNode
import mathlingua.common.textalk.newTexTalkLexer
import mathlingua.common.textalk.newTexTalkParser
import org.junit.jupiter.api.Test
import java.lang.RuntimeException

private fun buildNode(code: String): ExpressionTexTalkNode {
    val result = newTexTalkParser().parse(newTexTalkLexer(code))
    if (result.errors.isNotEmpty()) {
        throw RuntimeException("Failed to build statement: ${result.errors.joinToString("\n")}")
    }
    return result.root
}

private fun buildCommand(code: String) = buildNode(code).children[0] as Command

class MatcherKtTest {
    @Test
    fun testSimpleExpandAsWritten() {
        val node = buildNode("\\function:on{X}to{Y}")
        val patternToExpansion = mapOf(
                buildCommand("\\function:on{A}to{B}") to "\\cdot : A? \\rightarrow B?"
        )
        val expanded = expandAsWritten(node, patternToExpansion)
        assertThat(expanded).isEqualTo("\\cdot : X \\rightarrow Y")
    }

    @Test
    fun testVarArgInfixExpandAsWritten() {
        val node = buildNode("\\and{a > 0}{b < 0}{c = 0} + \\and{A = 0}{B < 0}")
        val patternToExpansion = mapOf(
                buildCommand("\\and{form}...") to "form{... \\textrm{and} ...}?"
        )
        val expanded = expandAsWritten(node, patternToExpansion)
        assertThat(expanded).isEqualTo(
                "a > 0 \\textrm{and} b < 0 \\textrm{and} c = 0 + A = 0 \\textrm{and} B < 0")
    }

    @Test
    fun testVarArgPrefixExpandAsWritten() {
        val node = buildNode("\\and{a > 0}{b < 0}{c = 0} + \\and{A = 0}{B < 0}")
        val patternToExpansion = mapOf(
                buildCommand("\\and{form}...") to "form{ \\textrm{and} ...}?"
        )
        val expanded = expandAsWritten(node, patternToExpansion)
        assertThat(expanded).isEqualTo(
                " \\textrm{and} a > 0 \\textrm{and} b < 0 \\textrm{and} c = 0 +  \\textrm{and} A = 0 \\textrm{and} B < 0")
    }

    @Test
    fun testVarArgSuffixExpandAsWritten() {
        val node = buildNode("\\and{a > 0}{b < 0}{c = 0} + \\and{A = 0}{B < 0}")
        val patternToExpansion = mapOf(
                buildCommand("\\and{form}...") to "form{... \\textrm{and} }?"
        )
        val expanded = expandAsWritten(node, patternToExpansion)
        assertThat(expanded).isEqualTo(
                "a > 0 \\textrm{and} b < 0 \\textrm{and} c = 0 \\textrm{and}  + A = 0 \\textrm{and} B < 0 \\textrm{and} ")
    }
}

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
import mathlingua.common.textalk.OperatorTexTalkNode
import mathlingua.common.textalk.TexTalkNodeType
import mathlingua.common.textalk.TexTalkTokenType
import mathlingua.common.textalk.TextTexTalkNode
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

private fun buildOperator(cmd: Command) = OperatorTexTalkNode(
    lhs = null,
    command = cmd,
    rhs = null
)

private fun buildText(text: String) = TextTexTalkNode(
    text = text,
    type = TexTalkNodeType.Identifier,
    tokenType = TexTalkTokenType.Identifier,
    isVarArg = false
)

class MatcherKtTest {
    @Test
    fun testSimpleOperatorExpandAsWritten() {
        val patternToExpansion = mapOf(
            OperatorTexTalkNode(
                lhs = buildText("A"),
                command = buildCommand("\\set.in"),
                rhs = buildText("B")
            ) to "A? \\in B?"
        )
        val node = buildNode("X \\set.in Y")
        val expanded = expandAsWritten(node, patternToExpansion)
        assertThat(expanded).isEqualTo("X \\in Y")
    }

    @Test
    fun testOperatorWithArgumentExpandAsWritten() {
        val patternToExpansion = mapOf(
            OperatorTexTalkNode(
                lhs = buildText("A"),
                command = buildCommand("\\set.in{T}"),
                rhs = buildText("B")
            ) to "A? \\in B? (with respect to T?)"
        )
        val node = buildNode("X \\set.in{Z} Y")
        val expanded = expandAsWritten(node, patternToExpansion)
        assertThat(expanded).isEqualTo("X \\in Y (with respect to Z)")
    }

    @Test
    fun testSimpleCommandExpandAsWritten() {
        val node = buildNode("\\function:on{X}to{Y}")
        val patternToExpansion = mapOf(
                buildOperator(buildCommand("\\function:on{A}to{B}")) to "\\cdot : A? \\rightarrow B?"
        )
        val expanded = expandAsWritten(node, patternToExpansion)
        assertThat(expanded).isEqualTo("\\cdot : X \\rightarrow Y")
    }

    @Test
    fun testVarArgInfixExpandAsWritten() {
        val node = buildNode("\\and{a > 0}{b < 0}{c = 0} + \\and{A = 0}{B < 0}")
        val patternToExpansion = mapOf(
            buildOperator(buildCommand("\\and{form}...")) to "form{... \\textrm{and} ...}?"
        )
        val expanded = expandAsWritten(node, patternToExpansion)
        assertThat(expanded).isEqualTo(
                "a > 0 \\textrm{and} b < 0 \\textrm{and} c = 0 + A = 0 \\textrm{and} B < 0")
    }

    @Test
    fun testVarArgInfixWithPrefixExpandAsWritten() {
        val node = buildNode("\\and{a > 0}{b < 0}{c = 0} + \\and{A = 0}{B < 0}")
        val patternToExpansion = mapOf(
            buildOperator(buildCommand("\\and{form}...")) to "form{prefix ... \\textrm{and} ...}?"
        )
        val expanded = expandAsWritten(node, patternToExpansion)
        assertThat(expanded).isEqualTo(
            "prefix a > 0 \\textrm{and} b < 0 \\textrm{and} c = 0 + prefix A = 0 \\textrm{and} B < 0")
    }

    @Test
    fun testVarArgPrefixExpandAsWritten() {
        val node = buildNode("\\and{a > 0}{b < 0}{c = 0} + \\and{A = 0}{B < 0}")
        val patternToExpansion = mapOf(
            buildOperator(buildCommand("\\and{form}...")) to "form{ \\textrm{and} ...}?"
        )
        val expanded = expandAsWritten(node, patternToExpansion)
        assertThat(expanded).isEqualTo(
                " \\textrm{and} a > 0 \\textrm{and} b < 0 \\textrm{and} c = 0 +  \\textrm{and} A = 0 \\textrm{and} B < 0")
    }

    @Test
    fun testVarArgInfixWithSuffixExpandAsWritten() {
        val node = buildNode("\\and{a > 0}{b < 0}{c = 0} + \\and{A = 0}{B < 0}")
        val patternToExpansion = mapOf(
            buildOperator(buildCommand("\\and{form}...")) to "form{... \\textrm{and} ... suffix}?"
        )
        val expanded = expandAsWritten(node, patternToExpansion)
        assertThat(expanded).isEqualTo(
            "a > 0 \\textrm{and} b < 0 \\textrm{and} c = 0 suffix + A = 0 \\textrm{and} B < 0 suffix")
    }

    @Test
    fun testVarArgInfixWithPrefixAndSuffixExpandAsWritten() {
        val node = buildNode("\\and{a > 0}{b < 0}{c = 0} + \\and{A = 0}{B < 0}")
        val patternToExpansion = mapOf(
            buildOperator(buildCommand("\\and{form}...")) to "form{prefix ... \\textrm{and} ... suffix}?"
        )
        val expanded = expandAsWritten(node, patternToExpansion)
        assertThat(expanded).isEqualTo(
            "prefix a > 0 \\textrm{and} b < 0 \\textrm{and} c = 0 suffix + prefix A = 0 \\textrm{and} B < 0 suffix")
    }

    @Test
    fun testVarArgInfixExpectedUsageExpandAsWritten() {
        val node = buildNode("\\sequence{x}{y}{z} + \\sequence{X}{Y}")
        val patternToExpansion = mapOf(
            buildOperator(buildCommand("\\sequence{form}...")) to "form{... , ... \\cdots}?"
        )
        val expanded = expandAsWritten(node, patternToExpansion)
        assertThat(expanded).isEqualTo(
            "x , y , z \\cdots + X , Y \\cdots")
    }

    @Test
    fun testVarArgSuffixExpandAsWritten() {
        val node = buildNode("\\and{a > 0}{b < 0}{c = 0} + \\and{A = 0}{B < 0}")
        val patternToExpansion = mapOf(
            buildOperator(buildCommand("\\and{form}...")) to "form{... \\textrm{and} }?"
        )
        val expanded = expandAsWritten(node, patternToExpansion)
        assertThat(expanded).isEqualTo(
                "a > 0 \\textrm{and} b < 0 \\textrm{and} c = 0 \\textrm{and}  + A = 0 \\textrm{and} B < 0 \\textrm{and} ")
    }
}

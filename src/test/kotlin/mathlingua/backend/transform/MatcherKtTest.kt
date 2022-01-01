/*
 * Copyright 2020 The MathLingua Authors
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

package mathlingua.mathlingua.transform

import assertk.assertThat
import assertk.assertions.isEmpty
import assertk.assertions.isEqualTo
import mathlingua.backend.WrittenAsForm
import mathlingua.backend.transform.expandAsWritten
import mathlingua.frontend.textalk.Command
import mathlingua.frontend.textalk.ExpressionTexTalkNode
import mathlingua.frontend.textalk.OperatorTexTalkNode
import mathlingua.frontend.textalk.TexTalkNodeType
import mathlingua.frontend.textalk.TexTalkTokenType
import mathlingua.frontend.textalk.TextTexTalkNode
import mathlingua.frontend.textalk.newTexTalkLexer
import mathlingua.frontend.textalk.newTexTalkParser
import org.junit.jupiter.api.Test

private fun buildNode(code: String): ExpressionTexTalkNode {
    val result = newTexTalkParser().parse(newTexTalkLexer(code))
    if (result.errors.isNotEmpty()) {
        throw RuntimeException("Failed to build statement: ${result.errors.joinToString("\n")}")
    }
    return result.root
}

private fun buildCommand(code: String) = buildNode(code).children[0] as Command

private fun buildOperator(cmd: Command) = OperatorTexTalkNode(lhs = null, command = cmd, rhs = null)

private fun buildText(text: String) =
    TextTexTalkNode(
        text = text,
        type = TexTalkNodeType.Identifier,
        tokenType = TexTalkTokenType.Identifier,
        isVarArg = false)

class MatcherKtTest {
    @Test
    fun testSimpleOperatorExpandAsWritten() {
        val patternToExpansion =
            mapOf(
                OperatorTexTalkNode(
                    lhs = buildText("A"),
                    command = buildCommand("\\set.in/"),
                    rhs = buildText("B")) to WrittenAsForm(target = null, form = "A? \\in B?"))
        val node = buildNode("X \\set.in/ Y")
        val expanded = expandAsWritten(null, node, patternToExpansion)
        assertThat(expanded.errors).isEmpty()
        assertThat(expanded.text).isEqualTo("X \\in Y")
    }

    @Test
    fun testSimpleOperatorExpandAsWrittenAddParens() {
        val patternToExpansion =
            mapOf(
                OperatorTexTalkNode(
                    lhs = buildText("A"),
                    command = buildCommand("\\set.in/"),
                    rhs = buildText("B")) to WrittenAsForm(target = null, form = "A+? \\in B+?"))
        val node = buildNode("{X + Y} \\set.in/ {A + B}")
        val expanded = expandAsWritten(null, node, patternToExpansion)
        assertThat(expanded.errors).isEmpty()
        assertThat(expanded.text)
            .isEqualTo("\\left ( {X + Y} \\right ) \\in \\left ( {A + B} \\right )")
    }

    @Test
    fun testSimpleOperatorExpandAsWrittenDoNotAddParensForSimpleExpressions() {
        val patternToExpansion =
            mapOf(
                OperatorTexTalkNode(
                    lhs = buildText("A"),
                    command = buildCommand("\\set.in/"),
                    rhs = buildText("B")) to WrittenAsForm(target = null, form = "A+? \\in B+?"))
        val node = buildNode("X \\set.in/ Y")
        val expanded = expandAsWritten(null, node, patternToExpansion)
        assertThat(expanded.errors).isEmpty()
        assertThat(expanded.text).isEqualTo("X \\in Y")
    }

    @Test
    fun testSimpleOperatorExpandAsWrittenRemoveParens() {
        val patternToExpansion =
            mapOf(
                OperatorTexTalkNode(
                    lhs = buildText("A"),
                    command = buildCommand("\\over/"),
                    rhs = buildText("B")) to
                    WrittenAsForm(target = null, form = "\\frac{A-?}{B-?}"))
        val node = buildNode("(X + Y) \\over/ (A + B)")
        val expanded = expandAsWritten(null, node, patternToExpansion)
        assertThat(expanded.errors).isEmpty()
        assertThat(expanded.text).isEqualTo("\\frac{X + Y}{A + B}")
    }

    @Test
    fun testOperatorWithArgumentExpandAsWritten() {
        val patternToExpansion =
            mapOf(
                OperatorTexTalkNode(
                    lhs = buildText("A"),
                    command = buildCommand("\\set.in{T}/"),
                    rhs = buildText("B")) to
                    WrittenAsForm(target = null, form = "A? \\in/ B? (with respect to T?)"))
        val node = buildNode("X \\set.in{Z}/ Y")
        val expanded = expandAsWritten(null, node, patternToExpansion)
        assertThat(expanded.errors).isEmpty()
        assertThat(expanded.text).isEqualTo("X \\in/ Y (with respect to Z)")
    }

    @Test
    fun testParenCommandValueWithParenExpandAsWritten() {
        val patternToExpression =
            mapOf(
                buildOperator(buildCommand("\\function(x)")) to
                    WrittenAsForm(target = null, form = "f(x?)"))
        val node = buildNode("\\function(y)")
        val expanded = expandAsWritten(null, node, patternToExpression)
        assertThat(expanded.errors).isEmpty()
        assertThat(expanded.text).isEqualTo("f(y)")
    }

    @Test
    fun testParenCommandValueWithoutParenExpandAsWritten() {
        val patternToExpression =
            mapOf(
                buildOperator(buildCommand("\\function(x)")) to
                    WrittenAsForm(target = null, form = "f(x?)"))
        val node = buildNode("\\function")
        val expanded = expandAsWritten(null, node, patternToExpression)
        assertThat(expanded.errors).isEmpty()
        assertThat(expanded.text).isEqualTo("f(x?)")
    }

    @Test
    fun testParenCommandValueWithParenVarargExpandAsWritten() {
        val patternToExpression =
            mapOf(
                buildOperator(buildCommand("\\function(x...)")) to
                    WrittenAsForm(target = null, form = "f(x{...;...}?)"))
        val node = buildNode("\\function(a, b, c)")
        val expanded = expandAsWritten(null, node, patternToExpression)
        assertThat(expanded.errors).isEmpty()
        assertThat(expanded.text).isEqualTo("f(a;b;c)")
    }

    @Test
    fun testParenCommandValueWithParenAndCurlyExpandAsWritten() {
        val patternToExpression =
            mapOf(
                buildOperator(buildCommand("\\function{a}(x)")) to
                    WrittenAsForm(target = null, form = "f_{a?}(x?)"))
        val node = buildNode("\\function{b}(y)")
        val expanded = expandAsWritten(null, node, patternToExpression)
        assertThat(expanded.errors).isEmpty()
        assertThat(expanded.text).isEqualTo("f_{b}(y)")
    }

    @Test
    fun testParenCommandValueWithParenAndNamedGroupExpandAsWritten() {
        val patternToExpression =
            mapOf(
                buildOperator(buildCommand("\\function(x):given{a}")) to
                    WrittenAsForm(target = null, form = "f_{a?}(x?)"))
        val node = buildNode("\\function(y):given{b}")
        val expanded = expandAsWritten(null, node, patternToExpression)
        assertThat(expanded.errors).isEmpty()
        assertThat(expanded.text).isEqualTo("f_{b}(y)")
    }

    @Test
    fun testParenCommandNoMatchExpandAsWritten() {
        val patternToExpression =
            mapOf(
                buildOperator(buildCommand("\\function(x)")) to
                    WrittenAsForm(target = null, form = "f(x?)"))
        val node = buildNode("\\function(a,b)")
        val expanded = expandAsWritten(null, node, patternToExpression)
        assertThat(expanded.errors)
            .isEqualTo(listOf("Expected exactly 1 arguments but found 2 for '(a, b)'"))
        assertThat(expanded.text).isEqualTo("\\function(a, b)")
    }

    @Test
    fun testSimpleCommandExpandAsWritten() {
        val patternToExpansion =
            mapOf(
                buildOperator(buildCommand("\\function:on{A}:to{B}")) to
                    WrittenAsForm(target = null, form = "\\cdot : A? \\rightarrow B?"))
        val node = buildNode("\\function:on{X}:to{Y}")
        val expanded = expandAsWritten(null, node, patternToExpansion)
        assertThat(expanded.errors).isEmpty()
        assertThat(expanded.text).isEqualTo("\\cdot : X \\rightarrow Y")
    }

    @Test
    fun testSimpleCommandMatchingTargetExpandAsWritten() {
        val patternToExpansion =
            mapOf(
                buildOperator(buildCommand("\\function:on{A}:to{B}")) to
                    WrittenAsForm(target = "f", form = "f? : A? \\rightarrow B?"))
        val node = buildNode("\\function:on{X}:to{Y}")
        val expanded = expandAsWritten("g", node, patternToExpansion)
        assertThat(expanded.errors).isEmpty()
        assertThat(expanded.text).isEqualTo("g : X \\rightarrow Y")
    }

    @Test
    fun testVarArgInfixExpandAsWritten() {
        val patternToExpansion =
            mapOf(
                buildOperator(buildCommand("\\and{form}...")) to
                    WrittenAsForm(target = null, form = "form{... \\textrm{and} ...}?"))
        val node = buildNode("\\and{a > 0}{b < 0}{c = 0} + \\and{A = 0}{B < 0}")
        val expanded = expandAsWritten(null, node, patternToExpansion)
        assertThat(expanded.errors).isEmpty()
        assertThat(expanded.text)
            .isEqualTo("a > 0 \\textrm{and} b < 0 \\textrm{and} c = 0 + A = 0 \\textrm{and} B < 0")
    }

    @Test
    fun testVarArgInfixWithPrefixExpandAsWritten() {
        val patternToExpansion =
            mapOf(
                buildOperator(buildCommand("\\and{form}...")) to
                    WrittenAsForm(target = null, form = "form{prefix ... \\textrm{and} ...}?"))
        val node = buildNode("\\and{a > 0}{b < 0}{c = 0} + \\and{A = 0}{B < 0}")
        val expanded = expandAsWritten(null, node, patternToExpansion)
        assertThat(expanded.errors).isEmpty()
        assertThat(expanded.text)
            .isEqualTo(
                "prefix a > 0 \\textrm{and} b < 0 \\textrm{and} c = 0 + prefix A = 0 \\textrm{and} B < 0")
    }

    @Test
    fun testVarArgPrefixExpandAsWritten() {
        val patternToExpansion =
            mapOf(
                buildOperator(buildCommand("\\and{form}...")) to
                    WrittenAsForm(target = null, form = "form{ \\textrm{and} ...}?"))
        val node = buildNode("\\and{a > 0}{b < 0}{c = 0} + \\and{A = 0}{B < 0}")
        val expanded = expandAsWritten(null, node, patternToExpansion)
        assertThat(expanded.errors).isEmpty()
        assertThat(expanded.text)
            .isEqualTo(
                " \\textrm{and} a > 0 \\textrm{and} b < 0 \\textrm{and} c = 0 +  \\textrm{and} A = 0 \\textrm{and} B < 0")
    }

    @Test
    fun testVarArgInfixWithSuffixExpandAsWritten() {
        val patternToExpansion =
            mapOf(
                buildOperator(buildCommand("\\and{form}...")) to
                    WrittenAsForm(target = null, form = "form{... \\textrm{and} ... suffix}?"))
        val node = buildNode("\\and{a > 0}{b < 0}{c = 0} + \\and{A = 0}{B < 0}")
        val expanded = expandAsWritten(null, node, patternToExpansion)
        assertThat(expanded.errors).isEmpty()
        assertThat(expanded.text)
            .isEqualTo(
                "a > 0 \\textrm{and} b < 0 \\textrm{and} c = 0 suffix + A = 0 \\textrm{and} B < 0 suffix")
    }

    @Test
    fun testVarArgInfixWithPrefixAndSuffixExpandAsWritten() {
        val patternToExpansion =
            mapOf(
                buildOperator(buildCommand("\\and{form}...")) to
                    WrittenAsForm(
                        target = null, form = "form{prefix ... \\textrm{and} ... suffix}?"))
        val node = buildNode("\\and{a > 0}{b < 0}{c = 0} + \\and{A = 0}{B < 0}")
        val expanded = expandAsWritten(null, node, patternToExpansion)
        assertThat(expanded.errors).isEmpty()
        assertThat(expanded.text)
            .isEqualTo(
                "prefix a > 0 \\textrm{and} b < 0 \\textrm{and} c = 0 suffix + prefix A = 0 \\textrm{and} B < 0 suffix")
    }

    @Test
    fun testVarArgInfixWithPrefixAndSuffixExpandAsWrittenAddParens() {
        val patternToExpansion =
            mapOf(
                buildOperator(buildCommand("\\and{form}...")) to
                    WrittenAsForm(
                        target = null, form = "form{prefix ... \\textrm{and} ... suffix}+?"))
        val node = buildNode("\\and{a > 0}{b < 0}{c = 0} + \\and{A = 0}{B < 0}")
        val expanded = expandAsWritten(null, node, patternToExpansion)
        assertThat(expanded.errors).isEmpty()
        assertThat(expanded.text)
            .isEqualTo(
                "prefix \\left ( a > 0 \\right ) \\textrm{and} \\left ( b < 0 \\right ) \\textrm{and} \\left ( c = 0 \\right ) suffix + prefix \\left ( A = 0 \\right ) \\textrm{and} \\left ( B < 0 \\right ) suffix")
    }

    @Test
    fun testVarArgInfixWithPrefixAndSuffixExpandAsWrittenRemoveParens() {
        val patternToExpansion =
            mapOf(
                buildOperator(buildCommand("\\and{form}...")) to
                    WrittenAsForm(
                        target = null, form = "form{prefix ... \\textrm{and} ... suffix}-?"))
        val node =
            buildNode("\\and{(a > 0)}{   (b < 0)   }{  (c = 0)  } + \\and{ (A = 0)}{(B < 0) }")
        val expanded = expandAsWritten(null, node, patternToExpansion)
        assertThat(expanded.errors).isEmpty()
        assertThat(expanded.text)
            .isEqualTo(
                "prefix a > 0 \\textrm{and} b < 0 \\textrm{and} c = 0 suffix + prefix A = 0 \\textrm{and} B < 0 suffix")
    }

    @Test
    fun testVarArgInfixExpectedUsageExpandAsWritten() {
        val patternToExpansion =
            mapOf(
                buildOperator(buildCommand("\\sequence{form}...")) to
                    WrittenAsForm(target = null, form = "form{... , ... \\cdots}?"))
        val node = buildNode("\\sequence{x}{y}{z} + \\sequence{X}{Y}")
        val expanded = expandAsWritten(null, node, patternToExpansion)
        assertThat(expanded.errors).isEmpty()
        assertThat(expanded.text).isEqualTo("x , y , z \\cdots + X , Y \\cdots")
    }

    @Test
    fun testVarArgSuffixExpandAsWritten() {
        val patternToExpansion =
            mapOf(
                buildOperator(buildCommand("\\and{form}...")) to
                    WrittenAsForm(target = null, form = "form{... \\textrm{and} }?"))
        val node = buildNode("\\and{a > 0}{b < 0}{c = 0} + \\and{A = 0}{B < 0}")
        val expanded = expandAsWritten(null, node, patternToExpansion)
        assertThat(expanded.errors).isEmpty()
        assertThat(expanded.text)
            .isEqualTo(
                "a > 0 \\textrm{and} b < 0 \\textrm{and} c = 0 \\textrm{and}  + A = 0 \\textrm{and} B < 0 \\textrm{and} ")
    }

    @Test
    fun testSetOfWhere() {
        val patternToExpansion =
            mapOf(
                buildOperator(buildCommand("\\set[x...]:of{form...}:where{condition}...")) to
                    WrittenAsForm(
                        target = null,
                        form =
                            "\\left \\{ form{... , ...}? \\: : \\: condition{... \\text{ and } ...}? \\right \\}"))
        val node = buildNode("\\set[x, y]:of{f(x), g(y)}:where{x > 0}{f(x) > 0}{y < 0}{g(y) < 0}")
        val expanded = expandAsWritten(null, node, patternToExpansion)
        assertThat(expanded.errors).isEmpty()
        assertThat(expanded.text)
            .isEqualTo(
                "\\left \\{ f(x) , g(y) \\: : \\: x > 0 \\text{ and } f(x) > 0 \\text{ and } y < 0 \\text{ and } g(y) < 0 \\right \\}")
    }
}

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

package mathlingua.lib.frontend.chalktalk

import kotlin.test.Test
import strikt.api.expect
import strikt.api.expectThat
import strikt.assertions.isEmpty
import strikt.assertions.isEqualTo
import strikt.assertions.isFalse
import strikt.assertions.isTrue

internal class ChalkTalkTokenLexerTest {
    @Test
    fun `correctly identifies tokens`() {
        val prefixLexer = newChalkTalkTokenLexer(PREFIX)
        var tokenCount = 0
        while (prefixLexer.hasNext()) {
            prefixLexer.next()
            tokenCount++
        }
        expect {
            that(tokenCount).isEqualTo(NUM_PREFIX_TOKENS)
            that(prefixLexer.diagnostics()).isEmpty()
        }

        for (case in TEST_CASES) {
            val lexer = newChalkTalkTokenLexer(PREFIX + case.input)
            // move past the prefix tokens
            for (i in 0 until NUM_PREFIX_TOKENS) {
                lexer.next()
            }
            expectThat(lexer.hasNext())
                .describedAs("Case '${case.input}' doesn't leave a remaining token")
                .isTrue()
            val peek = lexer.peek()
            val next = lexer.next()
            expect {
                that(next).isEqualTo(peek)
                that(lexer.hasNext()).isFalse()
                that(next)
                    .isEqualTo(
                        ChalkTalkToken(
                            type = case.expectedType,
                            text = case.expectedText,
                            row = 0,
                            column = PREFIX.length))
                that(lexer.diagnostics()).isEmpty()
            }
        }
    }

    @Test
    fun `correctly identifies multiple aligned sections`() =
        runIndentUnIndentTest(
            """
        a:
        b:
    """.trimIndent(),
            listOf(
                ChalkTalkToken(type = ChalkTalkTokenType.Name, text = "a", row = 0, column = 0),
                ChalkTalkToken(type = ChalkTalkTokenType.Colon, text = ":", row = 0, column = 1),
                ChalkTalkToken(
                    type = ChalkTalkTokenType.Newline, text = "<newline>", row = 0, column = 2),
                ChalkTalkToken(type = ChalkTalkTokenType.Name, text = "b", row = 1, column = 0),
                ChalkTalkToken(type = ChalkTalkTokenType.Colon, text = ":", row = 1, column = 1)))

    @Test
    fun `correctly handles single indented arg`() =
        runIndentUnIndentTest(
            """
        a:
        . b:
    """.trimIndent(),
            listOf(
                ChalkTalkToken(type = ChalkTalkTokenType.Name, text = "a", row = 0, column = 0),
                ChalkTalkToken(type = ChalkTalkTokenType.Colon, text = ":", row = 0, column = 1),
                ChalkTalkToken(
                    type = ChalkTalkTokenType.Newline, text = "<newline>", row = 0, column = 2),
                ChalkTalkToken(
                    type = ChalkTalkTokenType.Indent, text = "<indent>", row = 1, column = 0),
                ChalkTalkToken(
                    type = ChalkTalkTokenType.DotSpace, text = ". ", row = 1, column = 0),
                ChalkTalkToken(type = ChalkTalkTokenType.Name, text = "b", row = 1, column = 2),
                ChalkTalkToken(type = ChalkTalkTokenType.Colon, text = ":", row = 1, column = 3)))

    @Test
    fun `correctly handles multiple indented arg`() =
        runIndentUnIndentTest(
            """
        a:
        . b:
        . c:
    """.trimIndent(),
            listOf(
                ChalkTalkToken(type = ChalkTalkTokenType.Name, text = "a", row = 0, column = 0),
                ChalkTalkToken(type = ChalkTalkTokenType.Colon, text = ":", row = 0, column = 1),
                ChalkTalkToken(
                    type = ChalkTalkTokenType.Newline, text = "<newline>", row = 0, column = 2),
                ChalkTalkToken(
                    type = ChalkTalkTokenType.Indent, text = "<indent>", row = 1, column = 0),
                ChalkTalkToken(
                    type = ChalkTalkTokenType.DotSpace, text = ". ", row = 1, column = 0),
                ChalkTalkToken(type = ChalkTalkTokenType.Name, text = "b", row = 1, column = 2),
                ChalkTalkToken(type = ChalkTalkTokenType.Colon, text = ":", row = 1, column = 3),
                ChalkTalkToken(
                    type = ChalkTalkTokenType.Newline, text = "<newline>", row = 1, column = 4),
                ChalkTalkToken(
                    type = ChalkTalkTokenType.DotSpace, text = ". ", row = 2, column = 0),
                ChalkTalkToken(type = ChalkTalkTokenType.Name, text = "c", row = 2, column = 2),
                ChalkTalkToken(type = ChalkTalkTokenType.Colon, text = ":", row = 2, column = 3)))

    @Test
    fun `correctly handles nested indented args`() =
        runIndentUnIndentTest(
            """
        a:
        . b:
          . c:
    """.trimIndent(),
            listOf(
                ChalkTalkToken(type = ChalkTalkTokenType.Name, text = "a", row = 0, column = 0),
                ChalkTalkToken(type = ChalkTalkTokenType.Colon, text = ":", row = 0, column = 1),
                ChalkTalkToken(
                    type = ChalkTalkTokenType.Newline, text = "<newline>", row = 0, column = 2),
                ChalkTalkToken(
                    type = ChalkTalkTokenType.Indent, text = "<indent>", row = 1, column = 0),
                ChalkTalkToken(
                    type = ChalkTalkTokenType.DotSpace, text = ". ", row = 1, column = 0),
                ChalkTalkToken(type = ChalkTalkTokenType.Name, text = "b", row = 1, column = 2),
                ChalkTalkToken(type = ChalkTalkTokenType.Colon, text = ":", row = 1, column = 3),
                ChalkTalkToken(
                    type = ChalkTalkTokenType.Newline, text = "<newline>", row = 1, column = 4),
                ChalkTalkToken(
                    type = ChalkTalkTokenType.Indent, text = "<indent>", row = 2, column = 0),
                ChalkTalkToken(
                    type = ChalkTalkTokenType.DotSpace, text = ". ", row = 2, column = 2),
                ChalkTalkToken(type = ChalkTalkTokenType.Name, text = "c", row = 2, column = 4),
                ChalkTalkToken(type = ChalkTalkTokenType.Colon, text = ":", row = 2, column = 5)))

    @Test
    fun `correctly handles single indented arg with follow-up section`() =
        runIndentUnIndentTest(
            """
        a:
        . b:
        c:
    """.trimIndent(),
            listOf(
                ChalkTalkToken(type = ChalkTalkTokenType.Name, text = "a", row = 0, column = 0),
                ChalkTalkToken(type = ChalkTalkTokenType.Colon, text = ":", row = 0, column = 1),
                ChalkTalkToken(
                    type = ChalkTalkTokenType.Newline, text = "<newline>", row = 0, column = 2),
                ChalkTalkToken(
                    type = ChalkTalkTokenType.Indent, text = "<indent>", row = 1, column = 0),
                ChalkTalkToken(
                    type = ChalkTalkTokenType.DotSpace, text = ". ", row = 1, column = 0),
                ChalkTalkToken(type = ChalkTalkTokenType.Name, text = "b", row = 1, column = 2),
                ChalkTalkToken(type = ChalkTalkTokenType.Colon, text = ":", row = 1, column = 3),
                ChalkTalkToken(
                    type = ChalkTalkTokenType.Newline, text = "<newline>", row = 1, column = 4),
                ChalkTalkToken(
                    type = ChalkTalkTokenType.UnIndent, text = "<unindent>", row = 2, column = 2),
                ChalkTalkToken(type = ChalkTalkTokenType.Name, text = "c", row = 2, column = 0),
                ChalkTalkToken(type = ChalkTalkTokenType.Colon, text = ":", row = 2, column = 1),
            ))

    @Test
    fun `correctly handles nested indented args with a follow-up section`() =
        runIndentUnIndentTest(
            """
        a:
        . b:
          . c:
        d:
    """.trimIndent(),
            listOf(
                ChalkTalkToken(type = ChalkTalkTokenType.Name, text = "a", row = 0, column = 0),
                ChalkTalkToken(type = ChalkTalkTokenType.Colon, text = ":", row = 0, column = 1),
                ChalkTalkToken(
                    type = ChalkTalkTokenType.Newline, text = "<newline>", row = 0, column = 2),
                ChalkTalkToken(
                    type = ChalkTalkTokenType.Indent, text = "<indent>", row = 1, column = 0),
                ChalkTalkToken(
                    type = ChalkTalkTokenType.DotSpace, text = ". ", row = 1, column = 0),
                ChalkTalkToken(type = ChalkTalkTokenType.Name, text = "b", row = 1, column = 2),
                ChalkTalkToken(type = ChalkTalkTokenType.Colon, text = ":", row = 1, column = 3),
                ChalkTalkToken(
                    type = ChalkTalkTokenType.Newline, text = "<newline>", row = 1, column = 4),
                ChalkTalkToken(
                    type = ChalkTalkTokenType.Indent, text = "<indent>", row = 2, column = 0),
                ChalkTalkToken(
                    type = ChalkTalkTokenType.DotSpace, text = ". ", row = 2, column = 2),
                ChalkTalkToken(type = ChalkTalkTokenType.Name, text = "c", row = 2, column = 4),
                ChalkTalkToken(type = ChalkTalkTokenType.Colon, text = ":", row = 2, column = 5),
                ChalkTalkToken(
                    type = ChalkTalkTokenType.Newline, text = "<newline>", row = 2, column = 6),
                ChalkTalkToken(
                    type = ChalkTalkTokenType.UnIndent, text = "<unindent>", row = 3, column = 4),
                ChalkTalkToken(
                    type = ChalkTalkTokenType.UnIndent, text = "<unindent>", row = 3, column = 2),
                ChalkTalkToken(type = ChalkTalkTokenType.Name, text = "d", row = 3, column = 0),
                ChalkTalkToken(type = ChalkTalkTokenType.Colon, text = ":", row = 3, column = 1),
            ))

    @Test
    fun `correctly handles nested indented args with a follow-up un-indented arg`() =
        runIndentUnIndentTest(
            """
        a:
        . b:
          . c:
            . d:
          . e:
    """.trimIndent(),
            listOf(
                ChalkTalkToken(type = ChalkTalkTokenType.Name, text = "a", row = 0, column = 0),
                ChalkTalkToken(type = ChalkTalkTokenType.Colon, text = ":", row = 0, column = 1),
                ChalkTalkToken(
                    type = ChalkTalkTokenType.Newline, text = "<newline>", row = 0, column = 2),
                ChalkTalkToken(
                    type = ChalkTalkTokenType.Indent, text = "<indent>", row = 1, column = 0),
                ChalkTalkToken(
                    type = ChalkTalkTokenType.DotSpace, text = ". ", row = 1, column = 0),
                ChalkTalkToken(type = ChalkTalkTokenType.Name, text = "b", row = 1, column = 2),
                ChalkTalkToken(type = ChalkTalkTokenType.Colon, text = ":", row = 1, column = 3),
                ChalkTalkToken(
                    type = ChalkTalkTokenType.Newline, text = "<newline>", row = 1, column = 4),
                ChalkTalkToken(
                    type = ChalkTalkTokenType.Indent, text = "<indent>", row = 2, column = 0),
                ChalkTalkToken(
                    type = ChalkTalkTokenType.DotSpace, text = ". ", row = 2, column = 2),
                ChalkTalkToken(type = ChalkTalkTokenType.Name, text = "c", row = 2, column = 4),
                ChalkTalkToken(type = ChalkTalkTokenType.Colon, text = ":", row = 2, column = 5),
                ChalkTalkToken(
                    type = ChalkTalkTokenType.Newline, text = "<newline>", row = 2, column = 6),
                ChalkTalkToken(
                    type = ChalkTalkTokenType.Indent, text = "<indent>", row = 3, column = 0),
                ChalkTalkToken(
                    type = ChalkTalkTokenType.DotSpace, text = ". ", row = 3, column = 4),
                ChalkTalkToken(type = ChalkTalkTokenType.Name, text = "d", row = 3, column = 6),
                ChalkTalkToken(type = ChalkTalkTokenType.Colon, text = ":", row = 3, column = 7),
                ChalkTalkToken(
                    type = ChalkTalkTokenType.Newline, text = "<newline>", row = 3, column = 8),
                ChalkTalkToken(
                    type = ChalkTalkTokenType.UnIndent, text = "<unindent>", row = 4, column = 6),
                ChalkTalkToken(
                    type = ChalkTalkTokenType.DotSpace, text = ". ", row = 4, column = 2),
                ChalkTalkToken(type = ChalkTalkTokenType.Name, text = "e", row = 4, column = 4),
                ChalkTalkToken(type = ChalkTalkTokenType.Colon, text = ":", row = 4, column = 5)))

    @Test
    fun `correctly handles nested indented args with a follow-up parallel arg`() =
        runIndentUnIndentTest(
            """
        a:
        . b:
          . c:
            . d:
            . e:
    """.trimIndent(),
            listOf(
                ChalkTalkToken(type = ChalkTalkTokenType.Name, text = "a", row = 0, column = 0),
                ChalkTalkToken(type = ChalkTalkTokenType.Colon, text = ":", row = 0, column = 1),
                ChalkTalkToken(
                    type = ChalkTalkTokenType.Newline, text = "<newline>", row = 0, column = 2),
                ChalkTalkToken(
                    type = ChalkTalkTokenType.Indent, text = "<indent>", row = 1, column = 0),
                ChalkTalkToken(
                    type = ChalkTalkTokenType.DotSpace, text = ". ", row = 1, column = 0),
                ChalkTalkToken(type = ChalkTalkTokenType.Name, text = "b", row = 1, column = 2),
                ChalkTalkToken(type = ChalkTalkTokenType.Colon, text = ":", row = 1, column = 3),
                ChalkTalkToken(
                    type = ChalkTalkTokenType.Newline, text = "<newline>", row = 1, column = 4),
                ChalkTalkToken(
                    type = ChalkTalkTokenType.Indent, text = "<indent>", row = 2, column = 0),
                ChalkTalkToken(
                    type = ChalkTalkTokenType.DotSpace, text = ". ", row = 2, column = 2),
                ChalkTalkToken(type = ChalkTalkTokenType.Name, text = "c", row = 2, column = 4),
                ChalkTalkToken(type = ChalkTalkTokenType.Colon, text = ":", row = 2, column = 5),
                ChalkTalkToken(
                    type = ChalkTalkTokenType.Newline, text = "<newline>", row = 2, column = 6),
                ChalkTalkToken(
                    type = ChalkTalkTokenType.Indent, text = "<indent>", row = 3, column = 0),
                ChalkTalkToken(
                    type = ChalkTalkTokenType.DotSpace, text = ". ", row = 3, column = 4),
                ChalkTalkToken(type = ChalkTalkTokenType.Name, text = "d", row = 3, column = 6),
                ChalkTalkToken(type = ChalkTalkTokenType.Colon, text = ":", row = 3, column = 7),
                ChalkTalkToken(
                    type = ChalkTalkTokenType.Newline, text = "<newline>", row = 3, column = 8),
                ChalkTalkToken(
                    type = ChalkTalkTokenType.DotSpace, text = ". ", row = 4, column = 4),
                ChalkTalkToken(type = ChalkTalkTokenType.Name, text = "e", row = 4, column = 6),
                ChalkTalkToken(type = ChalkTalkTokenType.Colon, text = ":", row = 4, column = 7)))
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

private data class TestCase(
    val input: String, val expectedType: ChalkTalkTokenType, val expectedText: String)

private const val PREFIX =
    "abc_123 \"prefix text\" `prefix stmt 1` 'prefix stmt 2'123*+:=" +
        " = ( : ){ }_ , :: prefix text block ::. [id] "

private const val NUM_PREFIX_TOKENS = 18

private val TEST_CASES =
    listOf(
        TestCase(":=", ChalkTalkTokenType.ColonEqual, ":="),
        TestCase("(", ChalkTalkTokenType.LParen, "("),
        TestCase(")", ChalkTalkTokenType.RParen, ")"),
        TestCase("{", ChalkTalkTokenType.LCurly, "{"),
        TestCase("}", ChalkTalkTokenType.RCurly, "}"),
        TestCase(",", ChalkTalkTokenType.Comma, ","),
        TestCase(":", ChalkTalkTokenType.Colon, ":"),
        TestCase("_", ChalkTalkTokenType.Underscore, "_"),
        TestCase("\"some \\\" text\"", ChalkTalkTokenType.Text, "some \\\" text"),
        TestCase("'some statement'", ChalkTalkTokenType.Statement, "some statement"),
        TestCase("`another statement`", ChalkTalkTokenType.Statement, "another statement"),
        TestCase(":: some {::} text ::", ChalkTalkTokenType.TextBlock, " some {::} text "),
        TestCase("name", ChalkTalkTokenType.Name, "name"),
        TestCase("name_123", ChalkTalkTokenType.Name, "name_123"),
        TestCase("name_abc", ChalkTalkTokenType.Name, "name_abc"),
        TestCase("*", ChalkTalkTokenType.Operator, "*"),
        TestCase("*+", ChalkTalkTokenType.Operator, "*+"),
        TestCase("*+_ab", ChalkTalkTokenType.Operator, "*+_ab"),
        TestCase("*+_12", ChalkTalkTokenType.Operator, "*+_12"),
        TestCase(". ", ChalkTalkTokenType.DotSpace, ". "),
        TestCase("[some[id[text]]]", ChalkTalkTokenType.Id, "some[id[text]]"),
        TestCase("\n", ChalkTalkTokenType.Newline, "<newline>"),
        TestCase("...", ChalkTalkTokenType.DotDotDot, "..."))

private fun runIndentUnIndentTest(input: String, expected: List<ChalkTalkToken>) {
    val lexer = newChalkTalkTokenLexer(input)
    val actual = mutableListOf<ChalkTalkToken>()
    while (lexer.hasNext()) {
        actual.add(lexer.next())
    }
    expect {
        that(actual.toList()).isEqualTo(expected)
        that(lexer.diagnostics()).isEmpty()
    }
}

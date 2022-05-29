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

package mathlingua.lib.frontend.textalk

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertTrue
import mathlingua.lib.frontend.ast.TexTalkToken
import mathlingua.lib.frontend.ast.TexTalkTokenType

internal class TexTalkLexerTest {
    @Test
    fun `correctly identifies tokens`() {
        val prefixLexer = newTexTalkLexer(PREFIX)
        var tokenCount = 0
        while (prefixLexer.hasNext()) {
            prefixLexer.next()
            tokenCount++
        }
        assertEquals(expected = NUM_PREFIX_TOKENS, actual = tokenCount)
        assertEquals(expected = 0, actual = prefixLexer.diagnostics().size)

        for (case in TEST_CASES) {
            val lexer = newTexTalkLexer(PREFIX + case.input)
            // move past the prefix tokens
            for (i in 0 until NUM_PREFIX_TOKENS) {
                lexer.next()
            }
            assertTrue(
                actual = lexer.hasNext(),
                message = "Case '${case.input}' doesn't leave a remaining token")
            val peek = lexer.peek()
            val next = lexer.next()

            assertEquals(expected = peek, actual = next)
            assertFalse(actual = lexer.hasNext())
            assertEquals(
                expected =
                    TexTalkToken(
                        type = case.expectedType,
                        text = case.input,
                        row = 0,
                        column = PREFIX.length),
                actual = next)
            assertEquals(expected = 0, actual = lexer.diagnostics().size)
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

private const val PREFIX = "\\some.function[x]_{a}^{b}{c}:on{A}:to{B}+ "

private const val NUM_PREFIX_TOKENS = 29

private data class TestCase(val input: String, val expectedType: TexTalkTokenType)

private val TEST_CASES =
    listOf(
        TestCase("(", TexTalkTokenType.LParen),
        TestCase(")", TexTalkTokenType.RParen),
        TestCase("[", TexTalkTokenType.LSquare),
        TestCase("]", TexTalkTokenType.RSquare),
        TestCase("{", TexTalkTokenType.LCurly),
        TestCase("}", TexTalkTokenType.RCurly),
        TestCase("[:", TexTalkTokenType.LSquareColon),
        TestCase(":]", TexTalkTokenType.ColonRSquare),
        TestCase(",", TexTalkTokenType.Comma),
        TestCase(":=", TexTalkTokenType.ColonEqual),
        TestCase("is", TexTalkTokenType.Is),
        TestCase("in", TexTalkTokenType.In),
        TestCase("notin", TexTalkTokenType.NotIn),
        TestCase("as", TexTalkTokenType.As),
        TestCase("_", TexTalkTokenType.Underscore),
        TestCase("^", TexTalkTokenType.Caret),
        TestCase("...", TexTalkTokenType.DotDotDot),
        TestCase(".", TexTalkTokenType.Dot),
        TestCase("\\", TexTalkTokenType.Backslash),
        TestCase("/", TexTalkTokenType.Slash),
        TestCase(":", TexTalkTokenType.Colon),
        TestCase("=", TexTalkTokenType.Equals),
        TestCase("!=", TexTalkTokenType.NotEqual),
        TestCase("x", TexTalkTokenType.Name),
        TestCase("someName", TexTalkTokenType.Name),
        TestCase("someName_1", TexTalkTokenType.Name),
        TestCase("someName_123", TexTalkTokenType.Name),
        TestCase("someName_a", TexTalkTokenType.Name),
        TestCase("someName_abc", TexTalkTokenType.Name),
        TestCase("*", TexTalkTokenType.Operator),
        TestCase("*+", TexTalkTokenType.Operator),
        TestCase("*+_1", TexTalkTokenType.Operator),
        TestCase("*+_123", TexTalkTokenType.Operator),
        TestCase("*+_a", TexTalkTokenType.Operator),
        TestCase("*+_abc", TexTalkTokenType.Operator),
    )

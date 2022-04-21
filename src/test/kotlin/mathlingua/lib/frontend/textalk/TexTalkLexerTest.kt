package mathlingua.lib.frontend.textalk

import kotlin.test.Test
import strikt.api.expect
import strikt.api.expectThat
import strikt.assertions.isEmpty
import strikt.assertions.isEqualTo
import strikt.assertions.isFalse
import strikt.assertions.isTrue

internal class TexTalkLexerTest {
    @Test
    fun `correctly identifies tokens`() {
        val prefixLexer = newTexTalkLexer(PREFIX)
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
            val lexer = newTexTalkLexer(PREFIX + case.input)
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
                        TexTalkToken(
                            type = case.expectedType,
                            text = case.input,
                            row = 0,
                            column = PREFIX.length))
                that(lexer.diagnostics()).isEmpty()
            }
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
        TestCase(",", TexTalkTokenType.Comma),
        TestCase(":=", TexTalkTokenType.ColonEquals),
        TestCase("is", TexTalkTokenType.Is),
        TestCase("in", TexTalkTokenType.In),
        TestCase("notin", TexTalkTokenType.NotIn),
        TestCase("as", TexTalkTokenType.As),
        TestCase(":Type:", TexTalkTokenType.ColonTypeColon),
        TestCase(":Statement:", TexTalkTokenType.ColonStatementColon),
        TestCase(":Expression:", TexTalkTokenType.ColonExpressionColon),
        TestCase("_", TexTalkTokenType.Underscore),
        TestCase("^", TexTalkTokenType.Caret),
        TestCase("...", TexTalkTokenType.DotDotDot),
        TestCase(".", TexTalkTokenType.Dot),
        TestCase("\\", TexTalkTokenType.Backslash),
        TestCase("/", TexTalkTokenType.Slash),
        TestCase(":", TexTalkTokenType.Colon),
        TestCase("=", TexTalkTokenType.Equals),
        TestCase("!=", TexTalkTokenType.NotEquals),
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
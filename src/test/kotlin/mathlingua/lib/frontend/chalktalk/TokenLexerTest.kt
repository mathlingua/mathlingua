package mathlingua.lib.frontend.chalktalk

import kotlin.test.Test
import strikt.api.expect
import strikt.api.expectThat
import strikt.assertions.isEmpty
import strikt.assertions.isEqualTo
import strikt.assertions.isFalse
import strikt.assertions.isTrue

internal class TokenLexerTest {
    @Test
    fun `correctly identifies tokens`() {
        val prefixLexer = newTokenLexer(PREFIX)
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
            val lexer = newTokenLexer(PREFIX + case.input)
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
                        Token(
                            type = case.expectedType,
                            text = case.expectedText,
                            row = 0,
                            column = PREFIX.length))
                that(lexer.diagnostics()).isEmpty()
            }
        }
    }

    @Test
    fun `correctly identifies indents and un-indents`() {
        val lexer = newTokenLexer(INDENT_UN_INDENT_TEST_INPUT)
        val actual = mutableListOf<Token>()
        while (lexer.hasNext()) {
            actual.add(lexer.next())
        }
        expect {
            that(actual.toList()).isEqualTo(INDENT_UN_INDENT_EXPECTED)
            that(lexer.diagnostics()).isEmpty()
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

private data class TestCase(
    val input: String, val expectedType: TokenType, val expectedText: String)

private const val PREFIX =
    "abc_123 \"prefix text\" `prefix stmt 1` 'prefix stmt 2'123*+:=" +
        " = ( : ){ }_ , :: prefix text block ::. [id] "

private const val NUM_PREFIX_TOKENS = 18

private val TEST_CASES =
    listOf(
        TestCase(":=", TokenType.ColonEqual, ":="),
        TestCase("(", TokenType.LParen, "("),
        TestCase(")", TokenType.RParen, ")"),
        TestCase("{", TokenType.LCurly, "{"),
        TestCase("}", TokenType.RCurly, "}"),
        TestCase(",", TokenType.Comma, ","),
        TestCase(":", TokenType.Colon, ":"),
        TestCase("_", TokenType.Underscore, "_"),
        TestCase("\"some \\\" text\"", TokenType.Text, "some \\\" text"),
        TestCase("'some statement'", TokenType.Statement, "some statement"),
        TestCase("`another statement`", TokenType.Statement, "another statement"),
        TestCase(":: some {::} text ::", TokenType.TextBlock, " some {::} text "),
        TestCase("name", TokenType.Name, "name"),
        TestCase("name_123", TokenType.Name, "name_123"),
        TestCase("name_abc", TokenType.Name, "name_abc"),
        TestCase("*", TokenType.Operator, "*"),
        TestCase("*+", TokenType.Operator, "*+"),
        TestCase("*+_ab", TokenType.Operator, "*+_ab"),
        TestCase("*+_12", TokenType.Operator, "*+_12"),
        TestCase(". ", TokenType.DotSpace, ". "),
        TestCase("[some[id[text]]]", TokenType.Id, "some[id[text]]"),
        TestCase("\n", TokenType.Newline, "<newline>"),
        TestCase("...", TokenType.DotDotDot, "..."))

private val INDENT_UN_INDENT_TEST_INPUT =
    """
            Header: 1
            secondHeader: 2
            thirdHeader: 3
            . x: y
              y: 'something'
              . iff: a
                then: b
              . a:
                b:
              z: c
            fourthHeader: "x"
            . a:
              . b:
                . c:
                . d:
                  . e:
            fifthHeader:
        """.trimIndent()

private val INDENT_UN_INDENT_EXPECTED =
    listOf(
        Token(type = TokenType.Name, text = "Header", row = 0, column = 0),
        Token(type = TokenType.Colon, text = ":", row = 0, column = 6),
        Token(type = TokenType.Name, text = "1", row = 0, column = 8),
        Token(type = TokenType.Newline, text = "<newline>", row = 0, column = 9),
        Token(type = TokenType.Name, text = "secondHeader", row = 1, column = 0),
        Token(type = TokenType.Colon, text = ":", row = 1, column = 12),
        Token(type = TokenType.Name, text = "2", row = 1, column = 14),
        Token(type = TokenType.Newline, text = "<newline>", row = 1, column = 15),
        Token(type = TokenType.Name, text = "thirdHeader", row = 2, column = 0),
        Token(type = TokenType.Colon, text = ":", row = 2, column = 11),
        Token(type = TokenType.Name, text = "3", row = 2, column = 13),
        Token(type = TokenType.Newline, text = "<newline>", row = 2, column = 14),
        Token(type = TokenType.DotSpace, text = ". ", row = 3, column = 2),
        Token(type = TokenType.Name, text = "x", row = 3, column = 2),
        Token(type = TokenType.Colon, text = ":", row = 3, column = 3),
        Token(type = TokenType.Name, text = "y", row = 3, column = 5),
        Token(type = TokenType.Newline, text = "<newline>", row = 3, column = 6),
        Token(type = TokenType.Name, text = "y", row = 4, column = 2),
        Token(type = TokenType.Colon, text = ":", row = 4, column = 3),
        Token(type = TokenType.Statement, text = "something", row = 4, column = 5),
        Token(type = TokenType.Newline, text = "<newline>", row = 4, column = 16),
        Token(type = TokenType.DotSpace, text = ". ", row = 5, column = 4),
        Token(type = TokenType.Name, text = "iff", row = 5, column = 4),
        Token(type = TokenType.Colon, text = ":", row = 5, column = 7),
        Token(type = TokenType.Name, text = "a", row = 5, column = 9),
        Token(type = TokenType.Newline, text = "<newline>", row = 5, column = 10),
        Token(type = TokenType.Name, text = "then", row = 6, column = 4),
        Token(type = TokenType.Colon, text = ":", row = 6, column = 8),
        Token(type = TokenType.Name, text = "b", row = 6, column = 10),
        Token(type = TokenType.Newline, text = "<newline>", row = 6, column = 11),
        Token(type = TokenType.DotSpace, text = ". ", row = 7, column = 4),
        Token(type = TokenType.Name, text = "a", row = 7, column = 4),
        Token(type = TokenType.Colon, text = ":", row = 7, column = 5),
        Token(type = TokenType.Newline, text = "<newline>", row = 7, column = 6),
        Token(type = TokenType.Name, text = "b", row = 8, column = 4),
        Token(type = TokenType.Colon, text = ":", row = 8, column = 5),
        Token(type = TokenType.Newline, text = "<newline>", row = 8, column = 6),
        Token(type = TokenType.UnIndent, text = "<unindent>", row = 9, column = 2),
        Token(type = TokenType.Name, text = "z", row = 9, column = 2),
        Token(type = TokenType.Colon, text = ":", row = 9, column = 3),
        Token(type = TokenType.Name, text = "c", row = 9, column = 5),
        Token(type = TokenType.Newline, text = "<newline>", row = 9, column = 6),
        Token(type = TokenType.UnIndent, text = "<unindent>", row = 10, column = 0),
        Token(type = TokenType.Name, text = "fourthHeader", row = 10, column = 0),
        Token(type = TokenType.Colon, text = ":", row = 10, column = 12),
        Token(type = TokenType.Text, text = "x", row = 10, column = 14),
        Token(type = TokenType.Newline, text = "<newline>", row = 10, column = 17),
        Token(type = TokenType.DotSpace, text = ". ", row = 11, column = 2),
        Token(type = TokenType.Name, text = "a", row = 11, column = 2),
        Token(type = TokenType.Colon, text = ":", row = 11, column = 3),
        Token(type = TokenType.Newline, text = "<newline>", row = 11, column = 4),
        Token(type = TokenType.DotSpace, text = ". ", row = 12, column = 4),
        Token(type = TokenType.Name, text = "b", row = 12, column = 4),
        Token(type = TokenType.Colon, text = ":", row = 12, column = 5),
        Token(type = TokenType.Newline, text = "<newline>", row = 12, column = 6),
        Token(type = TokenType.DotSpace, text = ". ", row = 13, column = 6),
        Token(type = TokenType.Name, text = "c", row = 13, column = 6),
        Token(type = TokenType.Colon, text = ":", row = 13, column = 7),
        Token(type = TokenType.Newline, text = "<newline>", row = 13, column = 8),
        Token(type = TokenType.DotSpace, text = ". ", row = 14, column = 6),
        Token(type = TokenType.Name, text = "d", row = 14, column = 6),
        Token(type = TokenType.Colon, text = ":", row = 14, column = 7),
        Token(type = TokenType.Newline, text = "<newline>", row = 14, column = 8),
        Token(type = TokenType.DotSpace, text = ". ", row = 15, column = 8),
        Token(type = TokenType.Name, text = "e", row = 15, column = 8),
        Token(type = TokenType.Colon, text = ":", row = 15, column = 9),
        Token(type = TokenType.Newline, text = "<newline>", row = 15, column = 10),
        Token(type = TokenType.UnIndent, text = "<unindent>", row = 16, column = 0),
        Token(type = TokenType.UnIndent, text = "<unindent>", row = 16, column = 0),
        Token(type = TokenType.UnIndent, text = "<unindent>", row = 16, column = 0),
        Token(type = TokenType.UnIndent, text = "<unindent>", row = 16, column = 0),
        Token(type = TokenType.Name, text = "fifthHeader", row = 16, column = 0),
        Token(type = TokenType.Colon, text = ":", row = 16, column = 11))

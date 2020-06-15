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

package mathlingua.common.chalktalk.phase1

import assertk.assertThat
import assertk.assertions.isEqualTo
import mathlingua.common.ParseError
import mathlingua.common.chalktalk.phase1.ast.Phase1Token
import mathlingua.common.chalktalk.phase1.ast.ChalkTalkTokenType
import org.junit.jupiter.api.Test

internal class ChalkTalkLexerTest {
    @Test
    fun `correctly identifies names`() {
        val text = "name1 name2... name3#123 name4...#name5..."
        val lexer = newChalkTalkLexer(text)
        val actual: MutableList<Phase1Token> = ArrayList()
        while (lexer.hasNext()) {
            actual.add(lexer.next())
        }

        val expected = listOf(
                Phase1Token(
                        text = "name1",
                        type = ChalkTalkTokenType.Name,
                        row = 0,
                        column = 0),
                Phase1Token(
                        text = "name2...",
                        type = ChalkTalkTokenType.Name,
                        row = 0,
                        column = 6),
                Phase1Token(
                        text = "name3#123",
                        type = ChalkTalkTokenType.Name,
                        row = 0,
                        column = 15),
                Phase1Token(
                        text = "name4...#name5...",
                        type = ChalkTalkTokenType.Name,
                        row = 0,
                        column = 25),
                Phase1Token(
                        text = "<Indent>",
                        type = ChalkTalkTokenType.Begin,
                        row = 1,
                        column = 0),
                Phase1Token(
                        text = "<Unindent>",
                        type = ChalkTalkTokenType.End,
                        row = 1,
                        column = 0
                )
        )

        assertThat(actual).isEqualTo(expected)
        assertThat(lexer.errors()).isEqualTo(emptyList<ParseError>())
    }

    @Test
    fun `correctly identifies tokens non-name`() {
        val text = "someName:'some statement'\"some text\". [some id],:="
        val lexer = newChalkTalkLexer(text)
        val actual: MutableList<Phase1Token> = ArrayList()
        while (lexer.hasNext()) {
            actual.add(lexer.next())
        }

        val expected = listOf(
            Phase1Token(text = "someName", type = ChalkTalkTokenType.Name, row = 0, column = 0),
            Phase1Token(text = ":", type = ChalkTalkTokenType.Colon, row = 0, column = 8),
            Phase1Token(
                text = "'some statement'",
                type = ChalkTalkTokenType.Statement,
                row = 0,
                column = 9
            ),
            Phase1Token(text = "\"some text\"", type = ChalkTalkTokenType.String, row = 0, column = 25),
            Phase1Token(text = ". ", type = ChalkTalkTokenType.DotSpace, row = 0, column = 36),
            Phase1Token(text = "[some id]", type = ChalkTalkTokenType.Id, row = 0, column = 38),
            Phase1Token(text = ",", type = ChalkTalkTokenType.Comma, row = 0, column = 47),
            Phase1Token(text = ":=", type = ChalkTalkTokenType.ColonEquals, row = 0, column = 48),
            Phase1Token(text = "<Indent>", type = ChalkTalkTokenType.Begin, row = 1, column = 0),
            Phase1Token(text = "<Unindent>", type = ChalkTalkTokenType.End, row = 1, column = 0)
        )

        assertThat(actual).isEqualTo(expected)
        assertThat(lexer.errors()).isEqualTo(emptyList<ParseError>())
    }

    @Test
    fun `correctly reports Being and End tokens`() {
        val text = """
            x:
            . a:
              . p:
            . b:
              . q
              . r
              . s:
                . t
            . c:
              . u
        """.trimIndent()
        val lexer = newChalkTalkLexer(text)
        val actual: MutableList<Phase1Token> = ArrayList()
        while (lexer.hasNext()) {
            actual.add(lexer.next())
        }

        val expected = listOf(
            Phase1Token(text = "x", type = ChalkTalkTokenType.Name, row = 0, column = 0),
            Phase1Token(text = ":", type = ChalkTalkTokenType.Colon, row = 0, column = 1),
            Phase1Token(text = "<Indent>", type = ChalkTalkTokenType.Begin, row = 1, column = 0),
            Phase1Token(text = ". ", type = ChalkTalkTokenType.DotSpace, row = 1, column = 1),
            Phase1Token(text = "a", type = ChalkTalkTokenType.Name, row = 1, column = 3),
            Phase1Token(text = ":", type = ChalkTalkTokenType.Colon, row = 1, column = 4),
            Phase1Token(text = "<Indent>", type = ChalkTalkTokenType.Begin, row = 2, column = 2),
            Phase1Token(text = ". ", type = ChalkTalkTokenType.DotSpace, row = 2, column = 3),
            Phase1Token(text = "p", type = ChalkTalkTokenType.Name, row = 2, column = 5),
            Phase1Token(text = ":", type = ChalkTalkTokenType.Colon, row = 2, column = 6),
            Phase1Token(text = "<Indent>", type = ChalkTalkTokenType.Begin, row = 3, column = 0),
            Phase1Token(text = "<Unindent>", type = ChalkTalkTokenType.End, row = 3, column = 0),
            Phase1Token(text = "<Unindent>", type = ChalkTalkTokenType.End, row = 3, column = 0),
            Phase1Token(text = ". ", type = ChalkTalkTokenType.DotSpace, row = 3, column = 1),
            Phase1Token(text = "b", type = ChalkTalkTokenType.Name, row = 3, column = 3),
            Phase1Token(text = ":", type = ChalkTalkTokenType.Colon, row = 3, column = 4),
            Phase1Token(text = "<Indent>", type = ChalkTalkTokenType.Begin, row = 4, column = 2),
            Phase1Token(text = ". ", type = ChalkTalkTokenType.DotSpace, row = 4, column = 3),
            Phase1Token(text = "q", type = ChalkTalkTokenType.Name, row = 4, column = 5),
            Phase1Token(text = "<Indent>", type = ChalkTalkTokenType.Begin, row = 5, column = 2),
            Phase1Token(text = "<Unindent>", type = ChalkTalkTokenType.End, row = 5, column = 2),
            Phase1Token(text = ". ", type = ChalkTalkTokenType.DotSpace, row = 5, column = 3),
            Phase1Token(text = "r", type = ChalkTalkTokenType.Name, row = 5, column = 5),
            Phase1Token(text = "<Indent>", type = ChalkTalkTokenType.Begin, row = 6, column = 2),
            Phase1Token(text = "<Unindent>", type = ChalkTalkTokenType.End, row = 6, column = 2),
            Phase1Token(text = ". ", type = ChalkTalkTokenType.DotSpace, row = 6, column = 3),
            Phase1Token(text = "s", type = ChalkTalkTokenType.Name, row = 6, column = 5),
            Phase1Token(text = ":", type = ChalkTalkTokenType.Colon, row = 6, column = 6),
            Phase1Token(text = "<Indent>", type = ChalkTalkTokenType.Begin, row = 7, column = 4),
            Phase1Token(text = ". ", type = ChalkTalkTokenType.DotSpace, row = 7, column = 5),
            Phase1Token(text = "t", type = ChalkTalkTokenType.Name, row = 7, column = 7),
            Phase1Token(text = "<Indent>", type = ChalkTalkTokenType.Begin, row = 8, column = 0),
            Phase1Token(text = "<Unindent>", type = ChalkTalkTokenType.End, row = 8, column = 0),
            Phase1Token(text = "<Unindent>", type = ChalkTalkTokenType.End, row = 8, column = 0),
            Phase1Token(text = "<Unindent>", type = ChalkTalkTokenType.End, row = 8, column = 0),
            Phase1Token(text = ". ", type = ChalkTalkTokenType.DotSpace, row = 8, column = 1),
            Phase1Token(text = "c", type = ChalkTalkTokenType.Name, row = 8, column = 3),
            Phase1Token(text = ":", type = ChalkTalkTokenType.Colon, row = 8, column = 4),
            Phase1Token(text = "<Indent>", type = ChalkTalkTokenType.Begin, row = 9, column = 2),
            Phase1Token(text = ". ", type = ChalkTalkTokenType.DotSpace, row = 9, column = 3),
            Phase1Token(text = "u", type = ChalkTalkTokenType.Name, row = 9, column = 5),
            Phase1Token(text = "<Indent>", type = ChalkTalkTokenType.Begin, row = 10, column = 0),
            Phase1Token(text = "<Unindent>", type = ChalkTalkTokenType.End, row = 10, column = 0),
            Phase1Token(text = "<Unindent>", type = ChalkTalkTokenType.End, row = 10, column = 0),
            Phase1Token(text = "<Unindent>", type = ChalkTalkTokenType.End, row = 10, column = 0)
        )

        assertThat(actual).isEqualTo(expected)
        assertThat(lexer.errors()).isEqualTo(emptyList<ParseError>())
    }

    @Test
    fun `correctly handles ids with square braces`() {
        val text = "[\\f[x, y]{a, b}]"
        val lexer = newChalkTalkLexer(text)
        val actual: MutableList<Phase1Token> = ArrayList()
        while (lexer.hasNext()) {
            actual.add(lexer.next())
        }

        val expected = listOf(
            Phase1Token(text = "[\\f[x, y]{a, b}]", type = ChalkTalkTokenType.Id, row = 0, column = 0),
            Phase1Token(text = "<Indent>", type = ChalkTalkTokenType.Begin, row = 1, column = 0),
            Phase1Token(text = "<Unindent>", type = ChalkTalkTokenType.End, row = 1, column = 0)
        )

        assertThat(actual).isEqualTo(expected)
        assertThat(lexer.errors()).isEqualTo(emptyList<ParseError>())
    }

    @Test
    fun `correctly handles ids without square braces`() {
        val text = "[abc]"
        val lexer = newChalkTalkLexer(text)
        val actual: MutableList<Phase1Token> = ArrayList()
        while (lexer.hasNext()) {
            actual.add(lexer.next())
        }

        val expected = listOf(
            Phase1Token(text = "[abc]", type = ChalkTalkTokenType.Id, row = 0, column = 0),
            Phase1Token(text = "<Indent>", type = ChalkTalkTokenType.Begin, row = 1, column = 0),
            Phase1Token(text = "<Unindent>", type = ChalkTalkTokenType.End, row = 1, column = 0)
        )

        assertThat(actual).isEqualTo(expected)
        assertThat(lexer.errors()).isEqualTo(emptyList<ParseError>())
    }
}

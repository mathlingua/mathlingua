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

package mathlingua.mathlingua.textalk

import assertk.assertThat
import assertk.assertions.isEqualTo
import mathlingua.textalk.TexTalkToken
import mathlingua.textalk.TexTalkTokenType
import mathlingua.textalk.newTexTalkLexer
import org.junit.jupiter.api.Test

internal class TexTalkLexerTest {
    @Test
    fun `correctly identifies tokens`() {
        val text = "G := (X, *, 0) + B, X is \\some[x]_a^b{x, y}.thing:on{A} $1 #2 abc...xyz +... ...+::="
        val lexer = newTexTalkLexer(text)
        val actual = mutableListOf<TexTalkToken>()
        while (lexer.hasNext()) {
            actual.add(lexer.next())
        }

        val expected = listOf(
            TexTalkToken(text = "G", tokenType = TexTalkTokenType.Identifier, row = 0, column = 0),
            TexTalkToken(text = ":=", tokenType = TexTalkTokenType.ColonEquals, row = 0, column = 2),
            TexTalkToken(text = "(", tokenType = TexTalkTokenType.LParen, row = 0, column = 5),
            TexTalkToken(text = "X", tokenType = TexTalkTokenType.Identifier, row = 0, column = 6),
            TexTalkToken(text = ",", tokenType = TexTalkTokenType.Comma, row = 0, column = 7),
            TexTalkToken(text = "*", tokenType = TexTalkTokenType.Operator, row = 0, column = 9),
            TexTalkToken(text = ",", tokenType = TexTalkTokenType.Comma, row = 0, column = 10),
            TexTalkToken(text = "0", tokenType = TexTalkTokenType.Identifier, row = 0, column = 12),
            TexTalkToken(text = ")", tokenType = TexTalkTokenType.RParen, row = 0, column = 13),
            TexTalkToken(text = "+", tokenType = TexTalkTokenType.Operator, row = 0, column = 15),
            TexTalkToken(text = "B", tokenType = TexTalkTokenType.Identifier, row = 0, column = 17),
            TexTalkToken(text = ",", tokenType = TexTalkTokenType.Comma, row = 0, column = 18),
            TexTalkToken(text = "X", tokenType = TexTalkTokenType.Identifier, row = 0, column = 20),
            TexTalkToken(text = "is", tokenType = TexTalkTokenType.Is, row = 0, column = 22),
            TexTalkToken(text = "\\", tokenType = TexTalkTokenType.Backslash, row = 0, column = 25),
            TexTalkToken(text = "some", tokenType = TexTalkTokenType.Identifier, row = 0, column = 26),
            TexTalkToken(text = "[", tokenType = TexTalkTokenType.LSquare, row = 0, column = 30),
            TexTalkToken(text = "x", tokenType = TexTalkTokenType.Identifier, row = 0, column = 31),
            TexTalkToken(text = "]", tokenType = TexTalkTokenType.RSquare, row = 0, column = 32),
            TexTalkToken(text = "_", tokenType = TexTalkTokenType.Underscore, row = 0, column = 33),
            TexTalkToken(text = "a", tokenType = TexTalkTokenType.Identifier, row = 0, column = 34),
            TexTalkToken(text = "^", tokenType = TexTalkTokenType.Caret, row = 0, column = 35),
            TexTalkToken(text = "b", tokenType = TexTalkTokenType.Identifier, row = 0, column = 36),
            TexTalkToken(text = "{", tokenType = TexTalkTokenType.LCurly, row = 0, column = 37),
            TexTalkToken(text = "x", tokenType = TexTalkTokenType.Identifier, row = 0, column = 38),
            TexTalkToken(text = ",", tokenType = TexTalkTokenType.Comma, row = 0, column = 39),
            TexTalkToken(text = "y", tokenType = TexTalkTokenType.Identifier, row = 0, column = 41),
            TexTalkToken(text = "}", tokenType = TexTalkTokenType.RCurly, row = 0, column = 42),
            TexTalkToken(text = ".", tokenType = TexTalkTokenType.Period, row = 0, column = 43),
            TexTalkToken(text = "thing", tokenType = TexTalkTokenType.Identifier, row = 0, column = 44),
            TexTalkToken(text = ":", tokenType = TexTalkTokenType.Colon, row = 0, column = 49),
            TexTalkToken(text = "on", tokenType = TexTalkTokenType.Identifier, row = 0, column = 50),
            TexTalkToken(text = "{", tokenType = TexTalkTokenType.LCurly, row = 0, column = 52),
            TexTalkToken(text = "A", tokenType = TexTalkTokenType.Identifier, row = 0, column = 53),
            TexTalkToken(text = "}", tokenType = TexTalkTokenType.RCurly, row = 0, column = 54),
            TexTalkToken(text = "$1", tokenType = TexTalkTokenType.Identifier, row = 0, column = 56),
            TexTalkToken(text = "#2", tokenType = TexTalkTokenType.Identifier, row = 0, column = 59),
            TexTalkToken(text = "abc", tokenType = TexTalkTokenType.Identifier, row = 0, column = 62),
            TexTalkToken(text = "...", tokenType = TexTalkTokenType.DotDotDot, row = 0, column = 65),
            TexTalkToken(text = "xyz", tokenType = TexTalkTokenType.Identifier, row = 0, column = 68),
            TexTalkToken(text = "+...", tokenType = TexTalkTokenType.Operator, row = 0, column = 72),
            TexTalkToken(text = "...+", tokenType = TexTalkTokenType.Operator, row = 0, column = 77),
            TexTalkToken(text = "::=", tokenType = TexTalkTokenType.ColonColonEquals, row = 0, column = 81)
        )

        assertThat(actual).isEqualTo(expected)
        assertThat(lexer.errors.size).isEqualTo(0)
    }
}
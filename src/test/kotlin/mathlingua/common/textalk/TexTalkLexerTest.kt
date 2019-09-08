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

package mathlingua.mathlingua.common.textalk

import assertk.assertThat
import assertk.assertions.isEqualTo
import mathlingua.common.textalk.TexTalkToken
import mathlingua.common.textalk.TexTalkTokenType
import mathlingua.common.textalk.newTexTalkLexer
import org.junit.jupiter.api.Test

internal class TexTalkLexerTest {
    @Test
    fun `correctly identifies tokens`() {
        val text = "G := (X, *, 0) + B, X is \\some[x]_a^b{x, y}.thing:on{A}"
        val lexer = newTexTalkLexer(text)
        val actual = mutableListOf<TexTalkToken>()
        while (lexer.hasNext()) {
            actual.add(lexer.next())
        }

        val expected = listOf(
            TexTalkToken(parent = null, text = "G", tokenType = TexTalkTokenType.Identifier, row = 0, column = 0),
            TexTalkToken(parent = null, text = ":=", tokenType = TexTalkTokenType.ColonEquals, row = 0, column = 2),
            TexTalkToken(parent = null, text = "(", tokenType = TexTalkTokenType.LParen, row = 0, column = 5),
            TexTalkToken(parent = null, text = "X", tokenType = TexTalkTokenType.Identifier, row = 0, column = 6),
            TexTalkToken(parent = null, text = ",", tokenType = TexTalkTokenType.Comma, row = 0, column = 7),
            TexTalkToken(parent = null, text = "*", tokenType = TexTalkTokenType.Operator, row = 0, column = 9),
            TexTalkToken(parent = null, text = ",", tokenType = TexTalkTokenType.Comma, row = 0, column = 10),
            TexTalkToken(parent = null, text = "0", tokenType = TexTalkTokenType.Identifier, row = 0, column = 12),
            TexTalkToken(parent = null, text = ")", tokenType = TexTalkTokenType.RParen, row = 0, column = 13),
            TexTalkToken(parent = null, text = "+", tokenType = TexTalkTokenType.Operator, row = 0, column = 15),
            TexTalkToken(parent = null, text = "B", tokenType = TexTalkTokenType.Identifier, row = 0, column = 17),
            TexTalkToken(parent = null, text = ",", tokenType = TexTalkTokenType.Comma, row = 0, column = 18),
            TexTalkToken(parent = null, text = "X", tokenType = TexTalkTokenType.Identifier, row = 0, column = 20),
            TexTalkToken(parent = null, text = "is", tokenType = TexTalkTokenType.Is, row = 0, column = 22),
            TexTalkToken(parent = null, text = "\\", tokenType = TexTalkTokenType.Backslash, row = 0, column = 25),
            TexTalkToken(parent = null, text = "some", tokenType = TexTalkTokenType.Identifier, row = 0, column = 29),
            TexTalkToken(parent = null, text = "[", tokenType = TexTalkTokenType.LSquare, row = 0, column = 30),
            TexTalkToken(parent = null, text = "x", tokenType = TexTalkTokenType.Identifier, row = 0, column = 31),
            TexTalkToken(parent = null, text = "]", tokenType = TexTalkTokenType.RSquare, row = 0, column = 32),
            TexTalkToken(parent = null, text = "_", tokenType = TexTalkTokenType.Underscore, row = 0, column = 33),
            TexTalkToken(parent = null, text = "a", tokenType = TexTalkTokenType.Identifier, row = 0, column = 34),
            TexTalkToken(parent = null, text = "^", tokenType = TexTalkTokenType.Caret, row = 0, column = 35),
            TexTalkToken(parent = null, text = "b", tokenType = TexTalkTokenType.Identifier, row = 0, column = 36),
            TexTalkToken(parent = null, text = "{", tokenType = TexTalkTokenType.LCurly, row = 0, column = 37),
            TexTalkToken(parent = null, text = "x", tokenType = TexTalkTokenType.Identifier, row = 0, column = 38),
            TexTalkToken(parent = null, text = ",", tokenType = TexTalkTokenType.Comma, row = 0, column = 39),
            TexTalkToken(parent = null, text = "y", tokenType = TexTalkTokenType.Identifier, row = 0, column = 41),
            TexTalkToken(parent = null, text = "}", tokenType = TexTalkTokenType.RCurly, row = 0, column = 42),
            TexTalkToken(parent = null, text = ".", tokenType = TexTalkTokenType.Period, row = 0, column = 43),
            TexTalkToken(parent = null, text = "thing", tokenType = TexTalkTokenType.Identifier, row = 0, column = 48),
            TexTalkToken(parent = null, text = ":", tokenType = TexTalkTokenType.Colon, row = 0, column = 49),
            TexTalkToken(parent = null, text = "on", tokenType = TexTalkTokenType.Identifier, row = 0, column = 51),
            TexTalkToken(parent = null, text = "{", tokenType = TexTalkTokenType.LCurly, row = 0, column = 52),
            TexTalkToken(parent = null, text = "A", tokenType = TexTalkTokenType.Identifier, row = 0, column = 53),
            TexTalkToken(parent = null, text = "}", tokenType = TexTalkTokenType.RCurly, row = 0, column = 54)
        )

        assertThat(actual).isEqualTo(expected)
        assertThat(lexer.errors.size).isEqualTo(0)
    }
}
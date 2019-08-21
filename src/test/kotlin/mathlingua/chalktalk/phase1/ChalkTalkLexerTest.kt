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

package mathlingua.chalktalk.phase1

import mathlingua.chalktalk.phase1.ast.ChalkTalkToken
import org.junit.Test

internal class ChalkTalkLexerTest {
    @Test
    fun `correctly idvarentifies tokens`() {
        val text = "someName:'some statement'\"some text\". [some id],:="
        val lexer = newChalkTalkLexer(text)
        val actual: MutableList<ChalkTalkToken> = ArrayList()
        while (lexer.hasNext()) {
            actual.add(lexer.next())
        }
        /*
        val expected = listOf(
          ChalkTalkToken(text = "someName", type = ChalkTalkTokenType.Name, row = 0, column = 7),
          ChalkTalkToken(text = ":", type = ChalkTalkTokenType.Colon, row = 0, column = 8),
          ChalkTalkToken(
            text = "'some statement'",
            type = ChalkTalkTokenType.Statement,
            row = 0,
            column = 24
          ),
          ChalkTalkToken(text = "\"some text\"", type = ChalkTalkTokenType.String, row = 0, column = 35),
          ChalkTalkToken(text = ". ", type = ChalkTalkTokenType.DotSpace, row = 0, column = 36),
          ChalkTalkToken(text = "[some id]", type = ChalkTalkTokenType.Id, row = 0, column = 46),
          ChalkTalkToken(text = ",", type = ChalkTalkTokenType.Comma, row = 0, column = 47),
          ChalkTalkToken(text = ":=", type = ChalkTalkTokenType.ColonEquals, row = 0, column = 48),
          ChalkTalkToken(text = "(", type = ChalkTalkTokenType.Begin, row = 1, column = 0),
          ChalkTalkToken(text = ")", type = ChalkTalkTokenType.End, row = 1, column = 0)
        )
         */
//        Assertions.assertIterableEquals(expected, actual)
//        Assertions.assertEquals(0, lexer.errors().size)
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
        val actual: MutableList<ChalkTalkToken> = ArrayList()
        while (lexer.hasNext()) {
            actual.add(lexer.next())
        }
        /*
        val expected = listOf(
          ChalkTalkToken(text = "x", type = ChalkTalkTokenType.Name, row = 0, column = 0),
          ChalkTalkToken(text = ":", type = ChalkTalkTokenType.Colon, row = 0, column = 1),
          ChalkTalkToken(text = "(", type = ChalkTalkTokenType.Begin, row = 1, column = 0),
          ChalkTalkToken(text = ". ", type = ChalkTalkTokenType.DotSpace, row = 1, column = 1),
          ChalkTalkToken(text = "a", type = ChalkTalkTokenType.Name, row = 1, column = 3),
          ChalkTalkToken(text = ":", type = ChalkTalkTokenType.Colon, row = 1, column = 4),
          ChalkTalkToken(text = "(", type = ChalkTalkTokenType.Begin, row = 2, column = 2),
          ChalkTalkToken(text = ". ", type = ChalkTalkTokenType.DotSpace, row = 2, column = 3),
          ChalkTalkToken(text = "p", type = ChalkTalkTokenType.Name, row = 2, column = 5),
          ChalkTalkToken(text = ":", type = ChalkTalkTokenType.Colon, row = 2, column = 6),
          ChalkTalkToken(text = "(", type = ChalkTalkTokenType.Begin, row = 3, column = 0),
          ChalkTalkToken(text = ")", type = ChalkTalkTokenType.End, row = 3, column = 0),
          ChalkTalkToken(text = ")", type = ChalkTalkTokenType.End, row = 3, column = 0),
          ChalkTalkToken(text = ". ", type = ChalkTalkTokenType.DotSpace, row = 3, column = 1),
          ChalkTalkToken(text = "b", type = ChalkTalkTokenType.Name, row = 3, column = 3),
          ChalkTalkToken(text = ":", type = ChalkTalkTokenType.Colon, row = 3, column = 4),
          ChalkTalkToken(text = "(", type = ChalkTalkTokenType.Begin, row = 4, column = 2),
          ChalkTalkToken(text = ". ", type = ChalkTalkTokenType.DotSpace, row = 4, column = 3),
          ChalkTalkToken(text = "q", type = ChalkTalkTokenType.Name, row = 4, column = 5),
          ChalkTalkToken(text = "(", type = ChalkTalkTokenType.Begin, row = 5, column = 2),
          ChalkTalkToken(text = ")", type = ChalkTalkTokenType.End, row = 5, column = 2),
          ChalkTalkToken(text = ". ", type = ChalkTalkTokenType.DotSpace, row = 5, column = 3),
          ChalkTalkToken(text = "r", type = ChalkTalkTokenType.Name, row = 5, column = 5),
          ChalkTalkToken(text = "(", type = ChalkTalkTokenType.Begin, row = 6, column = 2),
          ChalkTalkToken(text = ")", type = ChalkTalkTokenType.End, row = 6, column = 2),
          ChalkTalkToken(text = ". ", type = ChalkTalkTokenType.DotSpace, row = 6, column = 3),
          ChalkTalkToken(text = "s", type = ChalkTalkTokenType.Name, row = 6, column = 5),
          ChalkTalkToken(text = ":", type = ChalkTalkTokenType.Colon, row = 6, column = 6),
          ChalkTalkToken(text = "(", type = ChalkTalkTokenType.Begin, row = 7, column = 4),
          ChalkTalkToken(text = ". ", type = ChalkTalkTokenType.DotSpace, row = 7, column = 5),
          ChalkTalkToken(text = "t", type = ChalkTalkTokenType.Name, row = 7, column = 7),
          ChalkTalkToken(text = "(", type = ChalkTalkTokenType.Begin, row = 8, column = 0),
          ChalkTalkToken(text = ")", type = ChalkTalkTokenType.End, row = 8, column = 0),
          ChalkTalkToken(text = ")", type = ChalkTalkTokenType.End, row = 8, column = 0),
          ChalkTalkToken(text = ")", type = ChalkTalkTokenType.End, row = 8, column = 0),
          ChalkTalkToken(text = ". ", type = ChalkTalkTokenType.DotSpace, row = 8, column = 1),
          ChalkTalkToken(text = "c", type = ChalkTalkTokenType.Name, row = 8, column = 3),
          ChalkTalkToken(text = ":", type = ChalkTalkTokenType.Colon, row = 8, column = 4),
          ChalkTalkToken(text = "(", type = ChalkTalkTokenType.Begin, row = 9, column = 2),
          ChalkTalkToken(text = ". ", type = ChalkTalkTokenType.DotSpace, row = 9, column = 3),
          ChalkTalkToken(text = "u", type = ChalkTalkTokenType.Name, row = 9, column = 5),
          ChalkTalkToken(text = "(", type = ChalkTalkTokenType.Begin, row = 10, column = 0),
          ChalkTalkToken(text = ")", type = ChalkTalkTokenType.End, row = 10, column = 0),
          ChalkTalkToken(text = ")", type = ChalkTalkTokenType.End, row = 10, column = 0),
          ChalkTalkToken(text = ")", type = ChalkTalkTokenType.End, row = 10, column = 0)
        )
         */
//        Assertions.assertIterableEquals(expected, actual)
//        Assertions.assertEquals(0, lexer.errors().size)
    }
}

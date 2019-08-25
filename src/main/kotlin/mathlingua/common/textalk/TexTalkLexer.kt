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

package mathlingua.common.textalk

import mathlingua.common.ParseError

interface TexTalkLexer {

    val errors: List<ParseError>

    operator fun hasNext(): Boolean

    fun hasNextNext(): Boolean

    fun peek(): TexTalkToken

    fun peekPeek(): TexTalkToken

    operator fun next(): TexTalkToken
}

fun newTexTalkLexer(text: String): TexTalkLexer {
    return TexTalkLexerImpl(text)
}

private class TexTalkLexerImpl(text: String) : TexTalkLexer {

    override val errors: MutableList<ParseError>
    private val tokens: MutableList<TexTalkToken>
    private var index: Int = 0

    init {
        this.errors = ArrayList()
        this.tokens = ArrayList()
        this.index = 0

        var i = 0

        var line = 0
        var column = -1

        while (i < text.length) {
            val c = text[i++]
            column++
            if (c == '\n') {
                line++
                column = 0
            } else if (c == '\\') {
                this.tokens.add(TexTalkToken("" + c, TexTalkTokenType.Backslash, line, column))
            } else if (c == ':') {
                this.tokens.add(TexTalkToken("" + c, TexTalkTokenType.Colon, line, column))
            } else if (c == '.') {
                this.tokens.add(TexTalkToken("" + c, TexTalkTokenType.Period, line, column))
            } else if (c == '(') {
                this.tokens.add(TexTalkToken("" + c, TexTalkTokenType.LParen, line, column))
            } else if (c == ')') {
                this.tokens.add(TexTalkToken("" + c, TexTalkTokenType.RParen, line, column))
            } else if (c == '[') {
                this.tokens.add(TexTalkToken("" + c, TexTalkTokenType.LSquare, line, column))
            } else if (c == ']') {
                this.tokens.add(TexTalkToken("" + c, TexTalkTokenType.RSquare, line, column))
            } else if (c == '{') {
                this.tokens.add(TexTalkToken("" + c, TexTalkTokenType.LCurly, line, column))
            } else if (c == '}') {
                this.tokens.add(TexTalkToken("" + c, TexTalkTokenType.RCurly, line, column))
            } else if (c == '_') {
                this.tokens.add(TexTalkToken("" + c, TexTalkTokenType.Underscore, line, column))
            } else if (c == '^') {
                this.tokens.add(TexTalkToken("" + c, TexTalkTokenType.Caret, line, column))
            } else if (c == ',') {
                this.tokens.add(TexTalkToken("" + c, TexTalkTokenType.Comma, line, column))
            } else if (isLetterOrDigit(c)) {
                val id = StringBuilder("" + c)
                while (i < text.length && isLetterOrDigit(text[i])) {
                    id.append(text[i++])
                    column++
                }
                this.tokens.add(TexTalkToken(id.toString(), TexTalkTokenType.Identifier, line, column))
            } else if (isOpChar(c)) {
                val op = StringBuilder("" + c)
                while (i < text.length && isOpChar(text[i])) {
                    op.append(text[i++])
                    column++
                }
                this.tokens.add(TexTalkToken(op.toString(), TexTalkTokenType.Operator, line, column))
            } else if (c != ' ') {
                this.errors.add(
                        ParseError(
                                "Unrecognized character $c", line, column
                        )
                )
            }
        }
    }

    override fun hasNext(): Boolean {
        return this.index < this.tokens.size
    }

    override fun hasNextNext(): Boolean {
        return this.index + 1 < this.tokens.size
    }

    override fun peek(): TexTalkToken {
        return this.tokens[this.index]
    }

    override fun peekPeek(): TexTalkToken {
        return this.tokens[this.index + 1]
    }

    override fun next(): TexTalkToken {
        val result = peek()
        this.index++
        return result
    }

    private fun isOpChar(c: Char): Boolean {
        return (c == '!' || c == '@' || c == '%' || c == '&' || c == '*' || c == '-' || c == '+'
                || c == '=' || c == '|' || c == '/' || c == '<' || c == '>')
    }

    private fun isLetterOrDigit(c: Char): Boolean {
        return Regex("[a-zA-Z0-9]+").matches("$c")
    }
}

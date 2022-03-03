/*
 * Copyright 2019 The MathLingua Authors
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

package mathlingua.frontend.textalk

import mathlingua.frontend.support.ParseError

// ------------------------------------------------------------------------------------------------------------------ //

internal interface TexTalkLexer {
    fun hasNext(): Boolean
    fun hasNextNext(): Boolean
    fun peek(): TexTalkToken
    fun peekPeek(): TexTalkToken
    fun next(): TexTalkToken
    val errors: List<ParseError>
}

internal fun newTexTalkLexer(text: String): TexTalkLexer {
    return TexTalkLexerImpl(text)
}

// ------------------------------------------------------------------------------------------------------------------ //

const val TYPE = ":Type:"

const val STATEMENT = ":Statement:"

const val EXPRESSION = ":Expression:"

private class TexTalkLexerImpl(text: String) : TexTalkLexer {
    override val errors = mutableListOf<ParseError>()
    private val tokens = mutableListOf<TexTalkToken>()
    private var index = 0

    init {
        var i = 0

        var line = 0
        var column = -1

        while (i < text.length) {
            val c = text[i++]
            column++
            if (c == '\n') {
                line++
                column = 0
            } else if (c == ':' &&
                i < text.length &&
                text[i] == 'T' &&
                i + 1 < text.length &&
                text[i + 1] == 'y' &&
                i + 2 < text.length &&
                text[i + 2] == 'p' &&
                i + 3 < text.length &&
                text[i + 3] == 'e' &&
                i + 4 < text.length &&
                text[i + 4] == ':') {
                val startLine = line
                val startCol = column
                column += TYPE.length
                i += TYPE.length - 1
                this.tokens.add(
                    TexTalkToken(TYPE, TexTalkTokenType.Identifier, startLine, startCol))
            } else if (c == ':' &&
                i < text.length &&
                text[i] == 'S' &&
                i + 1 < text.length &&
                text[i + 1] == 't' &&
                i + 2 < text.length &&
                text[i + 2] == 'a' &&
                i + 3 < text.length &&
                text[i + 3] == 't' &&
                i + 4 < text.length &&
                text[i + 4] == 'e' &&
                i + 5 < text.length &&
                text[i + 5] == 'm' &&
                i + 6 < text.length &&
                text[i + 6] == 'e' &&
                i + 7 < text.length &&
                text[i + 7] == 'n' &&
                i + 8 < text.length &&
                text[i + 8] == 't' &&
                i + 9 < text.length &&
                text[i + 9] == ':') {
                val startLine = line
                val startCol = column
                column += STATEMENT.length
                i += STATEMENT.length - 1
                this.tokens.add(
                    TexTalkToken(STATEMENT, TexTalkTokenType.Identifier, startLine, startCol))
            } else if (c == ':' &&
                i < text.length &&
                text[i] == 'E' &&
                i + 1 < text.length &&
                text[i + 1] == 'x' &&
                i + 2 < text.length &&
                text[i + 2] == 'p' &&
                i + 3 < text.length &&
                text[i + 3] == 'r' &&
                i + 4 < text.length &&
                text[i + 4] == 'e' &&
                i + 5 < text.length &&
                text[i + 5] == 's' &&
                i + 6 < text.length &&
                text[i + 6] == 's' &&
                i + 7 < text.length &&
                text[i + 7] == 'i' &&
                i + 8 < text.length &&
                text[i + 8] == 'o' &&
                i + 9 < text.length &&
                text[i + 9] == 'n' &&
                i + 10 < text.length &&
                text[i + 10] == ':') {
                val startLine = line
                val startCol = column
                column += EXPRESSION.length
                i += EXPRESSION.length - 1
                this.tokens.add(
                    TexTalkToken(EXPRESSION, TexTalkTokenType.Identifier, startLine, startCol))
            } else if (c == '`') {
                val startLine = line
                val startCol = column
                val builder = StringBuilder()
                while (i < text.length && text[i] != '`') {
                    val nextChar = text[i++]
                    builder.append(nextChar)
                    column++
                    if (nextChar == '\n') {
                        line++
                    }
                }
                if (i < text.length && text[i] == '`') {
                    // move past the trailing `
                    i++
                    column++
                } else {
                    this.errors.add(
                        ParseError(
                            message = "Expected terminating `", row = startLine, column = startCol))
                }
                this.tokens.add(
                    TexTalkToken(
                        builder.toString(), TexTalkTokenType.Identifier, startLine, startCol))
            } else if (c == '\\') {
                this.tokens.add(TexTalkToken("" + c, TexTalkTokenType.Backslash, line, column))
                // make sure not to match words that start with 'is' such as 'isomorphic'
            } else if (c == 'i' &&
                i < text.length &&
                text[i] == 's' &&
                (i + 1 >= text.length || !isIdentifierChar(text[i + 1]))) {
                this.tokens.add(TexTalkToken("is", TexTalkTokenType.Is, line, column))
                // skip the 's'
                i++
                column++
                // make sure not to match words that start with 'in' such as 'integers'
            } else if (c == 'a' &&
                i < text.length &&
                text[i] == 's' &&
                (i + 1 >= text.length || !isIdentifierChar(text[i + 1]))) {
                this.tokens.add(TexTalkToken("as", TexTalkTokenType.As, line, column))
                // skip the 's'
                i++
                column++
                // make sure not to match words that start with 'in' such as 'integers'
            } else if (c == 'i' &&
                i < text.length &&
                text[i] == 'n' &&
                (i + 1 >= text.length || !isIdentifierChar(text[i + 1]))) {
                this.tokens.add(TexTalkToken("in", TexTalkTokenType.In, line, column))
                // skip the 'n'
                i++
                column++
            } else if (c == 'n' &&
                i < text.length &&
                text[i] == 'o' &&
                i + 1 < text.length &&
                text[i + 1] == 't' &&
                i + 2 < text.length &&
                text[i + 2] == 'i' &&
                i + 3 < text.length &&
                text[i + 3] == 'n' &&
                (i + 4 >= text.length || !isIdentifierChar(text[i + 4]))) {
                this.tokens.add(TexTalkToken("notin", TexTalkTokenType.NotIn, line, column))
                // skip the 'otin'
                i += 4
                column += 4
            } else if (c == ':' && i < text.length && text[i] == '=') {
                this.tokens.add(TexTalkToken(":=", TexTalkTokenType.ColonEquals, line, column))
                // skip the =
                i++
                column++
            } else if (c == '!' && i < text.length && text[i] == '=') {
                this.tokens.add(TexTalkToken("!=", TexTalkTokenType.NotEquals, line, column))
                // skip the =
                i++
                column++
            } else if (c == '=') {
                this.tokens.add(TexTalkToken("=", TexTalkTokenType.Equals, line, column))
            } else if (c == '.' &&
                i < text.length &&
                text[i] == '.' &&
                i + 1 < text.length &&
                text[i + 1] == '.') {
                val startLine = line
                val startColumn = column
                // skip past the next two '.' characters
                i += 2
                column += 2

                val builder = StringBuilder("...")
                var isOp = false
                while (i < text.length && isOpChar(text[i])) {
                    isOp = true
                    builder.append(text[i++])
                    column++
                }

                this.tokens.add(
                    TexTalkToken(
                        builder.toString(),
                        if (isOp) {
                            TexTalkTokenType.Operator
                        } else {
                            TexTalkTokenType.DotDotDot
                        },
                        startLine,
                        startColumn))
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
            } else if (c == '?') {
                this.tokens.add(TexTalkToken("$c", TexTalkTokenType.Identifier, line, column))
            } else if (isDigitIdentifierChar(c)) {
                val startLine = line
                val startColumn = column

                val builder = StringBuilder()
                builder.append(c)
                while (i < text.length && isDigitIdentifierChar(text[i])) {
                    builder.append(text[i++])
                    column++
                }

                var hasDot = false
                if (i < text.length && text[i] == '.') {
                    hasDot = true
                    builder.append(text[i++])
                    column++
                }

                if (hasDot) {
                    if (i >= text.length || !isDigitIdentifierChar(text[i])) {
                        this.errors.add(
                            ParseError(
                                message =
                                    "A number literal with a decimal point must have at least one digit after the decimal place",
                                row = startLine,
                                column = startColumn))
                    } else {
                        while (i < text.length && isDigitIdentifierChar(text[i])) {
                            builder.append(text[i++])
                            column++
                        }
                    }
                }

                this.tokens.add(
                    TexTalkToken(
                        text = builder.toString(),
                        tokenType = TexTalkTokenType.Identifier,
                        row = startLine,
                        column = startColumn))
            } else if (isNonDigitIdentifierChar(c)) {
                val startLine = line
                val startColumn = column

                val id = StringBuilder("" + c)
                while (i < text.length && isIdentifierChar(text[i])) {
                    id.append(text[i++])
                    column++
                }
                if (i < text.length && text[i] == '?') {
                    id.append(text[i++])
                    column++
                }
                this.tokens.add(
                    TexTalkToken(
                        id.toString(), TexTalkTokenType.Identifier, startLine, startColumn))
            } else if (isOpChar(c)) {
                val startLine = line
                val startColumn = column

                val op = StringBuilder("" + c)
                while (i < text.length && isOpChar(text[i])) {
                    op.append(text[i++])
                    column++
                }
                if (i < text.length &&
                    text[i] == '.' &&
                    i + 1 < text.length &&
                    text[i + 1] == '.' &&
                    i + 2 < text.length &&
                    text[i + 2] == '.') {
                    i += 3
                    column += 3
                    op.append("...")
                }

                this.tokens.add(
                    TexTalkToken(op.toString(), TexTalkTokenType.Operator, startLine, startColumn))
            } else if (c != ' ') {
                this.errors.add(ParseError("Unrecognized character $c", line, column))
            }
        }
    }

    override fun hasNext() = this.index < this.tokens.size

    override fun hasNextNext() = this.index + 1 < this.tokens.size

    override fun peek() = this.tokens[this.index]

    override fun peekPeek() = this.tokens[this.index + 1]

    override fun next(): TexTalkToken {
        val result = peek()
        this.index++
        return result
    }

    override fun toString(): String {
        val builder = StringBuilder()
        var i = this.index
        while (i < this.tokens.size) {
            builder.append(this.tokens[i++].text)
        }
        return builder.toString()
    }

    private fun isIdentifierChar(c: Char) = isNonDigitIdentifierChar(c) || isDigitIdentifierChar(c)

    private fun isNonDigitIdentifierChar(c: Char) = Regex("[$#a-zA-Z]+").matches("$c")

    private fun isDigitIdentifierChar(c: Char) = Regex("[0-9]+").matches("$c")
}

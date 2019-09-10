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

import mathlingua.common.ParseError
import mathlingua.common.Stack
import mathlingua.common.chalktalk.phase1.ast.Phase1Token
import mathlingua.common.chalktalk.phase1.ast.ChalkTalkTokenType

interface ChalkTalkLexer {
    fun hasNext(): Boolean
    fun hasNextNext(): Boolean
    fun peek(): Phase1Token
    fun peekPeek(): Phase1Token
    fun next(): Phase1Token
    fun errors(): List<ParseError>
}

fun newChalkTalkLexer(text: String): ChalkTalkLexer {
    return ChalkTalkLexerImpl(text)
}

private class ChalkTalkLexerImpl(private var text: String) :
    ChalkTalkLexer {

    private val errors = mutableListOf<ParseError>()
    private var chalkTalkTokens: MutableList<Phase1Token>? = null
    private var index = 0

    private fun ensureInitialized() {
        if (this.chalkTalkTokens == null) {
            initialize()
        }
    }

    private fun initialize() {
        this.chalkTalkTokens = mutableListOf()

        if (!this.text.endsWith("\n")) {
            this.text += "\n"
        }

        var i = 0
        var line = 0
        var column = -1

        val levStack = Stack<Int>()
        levStack.push(0)

        var numOpen = 0

        while (i < text.length) {
            if (text[i] == '-' && i + 1 < text.length && text[i + 1] == '-') {
                // it is a comment and should be ignored
                while (i < text.length && text[i] != '\n') {
                    i++
                    column++
                }
                if (i < text.length && text[i] == '\n') {
                    i++
                    column = 0
                    line++
                }
                continue
            }

            val c = text[i++]
            column++
            if (c == '=') {
                this.chalkTalkTokens!!.add(
                    Phase1Token("=", ChalkTalkTokenType.Equals, line, column)
                )
            } else if (c == '(') {
                this.chalkTalkTokens!!.add(
                    Phase1Token("(", ChalkTalkTokenType.LParen, line, column)
                )
            } else if (c == ')') {
                this.chalkTalkTokens!!.add(
                    Phase1Token(")", ChalkTalkTokenType.RParen, line, column)
                )
            } else if (c == '{') {
                this.chalkTalkTokens!!.add(
                    Phase1Token("{", ChalkTalkTokenType.LCurly, line, column)
                )
            } else if (c == '}') {
                this.chalkTalkTokens!!.add(
                    Phase1Token("}", ChalkTalkTokenType.RCurly, line, column)
                )
            } else if (c == ':') {
                if (i < text.length && text[i] == '=') {
                    this.chalkTalkTokens!!.add(
                        Phase1Token(
                            ":=",
                            ChalkTalkTokenType.ColonEquals,
                            line,
                            column
                        )
                    )
                    i++ // move past the =
                } else {
                    this.chalkTalkTokens!!.add(Phase1Token(":", ChalkTalkTokenType.Colon, line, column))
                }
            } else if (c == ',') {
                this.chalkTalkTokens!!.add(Phase1Token(",", ChalkTalkTokenType.Comma, line, column))
            } else if (c == '.' && i < text.length && text[i] == ' ') {
                this.chalkTalkTokens!!.add(Phase1Token(". ", ChalkTalkTokenType.DotSpace, line, column))
                i++ // move past space
                column++
            } else if (c == '\n') {
                line++
                column = 0

                // text[i-1] == c since i was incremented
                // text[i-2] is the character before c
                if (i - 2 < 0 || text[i - 2] == '\n') {
                    while (i < text.length && text[i] == '\n') {
                        i++
                        column++
                        line++
                    }
                    this.chalkTalkTokens!!.add(
                        Phase1Token(
                            "-",
                            ChalkTalkTokenType.Linebreak,
                            line,
                            column
                        )
                    )
                    continue
                }

                var indentCount = 0
                while (i < text.length &&
                    i + 1 < text.length &&
                    text[i] == ' ' &&
                    text[i + 1] == ' '
                ) {
                    indentCount++
                    i += 2
                    column += 2
                }

                // treat '. ' like another indent
                if (i < text.length &&
                    text[i] == '.' &&
                    i + 1 < text.length &&
                    text[i + 1] == ' '
                ) {
                    indentCount++
                }

                this.chalkTalkTokens!!.add(Phase1Token("<Indent>", ChalkTalkTokenType.Begin, line, column))
                numOpen++

                val level = levStack.peek()
                if (indentCount <= level) {
                    while (numOpen > 0 && !levStack.isEmpty() && indentCount <= levStack.peek()) {
                        this.chalkTalkTokens!!.add(Phase1Token("<Unindent>", ChalkTalkTokenType.End, line, column))
                        numOpen--
                        levStack.pop()
                    }
                    // if the level stack is empty re-initialize it
                    if (levStack.isEmpty()) {
                        levStack.push(0)
                    }
                }
                levStack.push(indentCount)
            } else if (isOperatorChar(c)) {
                var name = "" + c
                while (i < text.length && isOperatorChar(text[i])) {
                    name += text[i++]
                    column++
                }
                this.chalkTalkTokens!!.add(Phase1Token(name, ChalkTalkTokenType.Name, line, column))
            } else if (isNameChar(c)) {
                var name = "" + c
                while (i < text.length && isNameChar(text[i])) {
                    name += text[i++]
                    column++
                }
                this.chalkTalkTokens!!.add(Phase1Token(name, ChalkTalkTokenType.Name, line, column))
            } else if (c == '"') {
                var str = "" + c
                while (i < text.length && text[i] != '"') {
                    str += text[i++]
                    column++
                }
                if (i == text.length) {
                    errors.add(
                        ParseError("Expected a terminating \"", line, column)
                    )
                    str += "\""
                } else {
                    // include the terminating "
                    str += text[i++]
                    column++
                }
                this.chalkTalkTokens!!.add(Phase1Token(str, ChalkTalkTokenType.String, line, column))
            } else if (c == '\'') {
                var stmt = "" + c
                while (i < text.length && text[i] != '\'') {
                    stmt += text[i++]
                    column++
                }
                if (i == text.length) {
                    errors.add(
                        ParseError("Expected a terminating '", line, column)
                    )
                    stmt += "'"
                } else {
                    // include the terminating '
                    stmt += text[i++]
                    column++
                }
                this.chalkTalkTokens!!.add(Phase1Token(stmt, ChalkTalkTokenType.Statement, line, column))
            } else if (c == '[') {
                val startLine = line
                val startColumn = column

                var id = "" + c
                var braceCount = 1
                while (i < text.length && text[i] != '\n') {
                    val next = text[i++]
                    id += next
                    column++

                    if (next == '[') {
                        braceCount++
                    } else if (next == ']') {
                        braceCount--
                    }

                    if (braceCount == 0) {
                        break
                    }
                }

                if (i == text.length) {
                    errors.add(
                        ParseError("Expected a terminating ]", line, column)
                    )
                    id += "]"
                }
                this.chalkTalkTokens!!.add(Phase1Token(id, ChalkTalkTokenType.Id, startLine, startColumn))
            } else if (c != ' ') { // spaces are ignored
                errors.add(
                    ParseError(
                        "Unrecognized character $c", line, column
                    )
                )
            }
        }

        // numOpen must be used to determine the number of open Begin chalkTalkTokens
        // (as apposed to checking if levStack.isNotEmpty()) since whenever
        // the levStack is empty in the above code, it is re-initialized to
        // contain a level of 0
        while (numOpen > 0) {
            this.chalkTalkTokens!!.add(Phase1Token("<Unindent>", ChalkTalkTokenType.End, line, column))
            numOpen--
        }
    }

    private fun isOperatorChar(c: Char): Boolean {
        return "~!@%^&*-+<>\\/=".contains(c)
    }

    private fun isNameChar(c: Char): Boolean {
        return Regex("[$#a-zA-Z0-9]+").matches("$c")
    }

    override fun hasNext(): Boolean {
        ensureInitialized()
        return this.index < this.chalkTalkTokens!!.size
    }

    override fun hasNextNext(): Boolean {
        ensureInitialized()
        return this.index + 1 < this.chalkTalkTokens!!.size
    }

    override fun peek(): Phase1Token {
        ensureInitialized()
        return this.chalkTalkTokens!![this.index]
    }

    override fun peekPeek(): Phase1Token {
        ensureInitialized()
        return this.chalkTalkTokens!![this.index + 1]
    }

    override fun next(): Phase1Token {
        ensureInitialized()
        val tok = peek()
        this.index++
        return tok
    }

    override fun errors(): List<ParseError> {
        return this.errors
    }
}

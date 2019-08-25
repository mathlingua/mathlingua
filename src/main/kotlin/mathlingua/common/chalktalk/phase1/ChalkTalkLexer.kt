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
import mathlingua.common.chalktalk.phase1.ast.ChalkTalkToken
import mathlingua.common.chalktalk.phase1.ast.ChalkTalkTokenType

interface ChalkTalkLexer {
    fun hasNext(): Boolean
    fun hasNextNext(): Boolean
    fun peek(): ChalkTalkToken
    fun peekPeek(): ChalkTalkToken
    fun next(): ChalkTalkToken
    fun errors(): List<ParseError>
}

fun newChalkTalkLexer(text: String): ChalkTalkLexer {
    return ChalkTalkLexerImpl(text)
}

private class ChalkTalkLexerImpl(private var text: String) :
        ChalkTalkLexer {

    private val errors: MutableList<ParseError>
    private var chalkTalkTokens: MutableList<ChalkTalkToken>? = null
    private var index: Int = 0

    init {
        this.errors = ArrayList()
        this.chalkTalkTokens = null
        this.index = 0
    }

    private fun ensureInitialized() {
        if (this.chalkTalkTokens == null) {
            initialize()
        }
    }

    private fun initialize() {
        this.chalkTalkTokens = ArrayList()

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
                        ChalkTalkToken("=", ChalkTalkTokenType.Equals, line, column)
                )
            } else if (c == '(') {
                this.chalkTalkTokens!!.add(
                        ChalkTalkToken("(", ChalkTalkTokenType.LParen, line, column)
                )
            } else if (c == ')') {
                this.chalkTalkTokens!!.add(
                        ChalkTalkToken(")", ChalkTalkTokenType.RParen, line, column)
                )
            } else if (c == '{') {
                this.chalkTalkTokens!!.add(
                        ChalkTalkToken("{", ChalkTalkTokenType.LCurly, line, column)
                )
            } else if (c == '}') {
                this.chalkTalkTokens!!.add(
                        ChalkTalkToken("}", ChalkTalkTokenType.RCurly, line, column)
                )
            } else if (c == ':') {
                if (i < text.length && text[i] == '=') {
                    this.chalkTalkTokens!!.add(
                            ChalkTalkToken(
                                    ":=",
                                    ChalkTalkTokenType.ColonEquals,
                                    line,
                                    column
                            )
                    )
                    i++ // move past the =
                } else {
                    this.chalkTalkTokens!!.add(ChalkTalkToken(":", ChalkTalkTokenType.Colon, line, column))
                }
            } else if (c == ',') {
                this.chalkTalkTokens!!.add(ChalkTalkToken(",", ChalkTalkTokenType.Comma, line, column))
            } else if (c == '.' && i < text.length && text[i] == ' ') {
                this.chalkTalkTokens!!.add(ChalkTalkToken(". ", ChalkTalkTokenType.DotSpace, line, column))
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
                            ChalkTalkToken(
                                    "-",
                                    ChalkTalkTokenType.Linebreak,
                                    line,
                                    column
                            )
                    )
                    continue
                }

                var indentCount = 0
                while (i < text.length
                        && i + 1 < text.length
                        && text[i] == ' '
                        && text[i + 1] == ' '
                ) {
                    indentCount++
                    i += 2
                    column += 2
                }

                // treat '. ' like another indent
                if (i < text.length
                        && text[i] == '.'
                        && i + 1 < text.length
                        && text[i + 1] == ' '
                ) {
                    indentCount++
                }

                this.chalkTalkTokens!!.add(ChalkTalkToken("<Indent>", ChalkTalkTokenType.Begin, line, column))
                numOpen++

                val level = levStack.peek()
                if (indentCount <= level) {
                    while (numOpen > 0 && !levStack.isEmpty() && indentCount <= levStack.peek()) {
                        this.chalkTalkTokens!!.add(ChalkTalkToken("<Unindent>", ChalkTalkTokenType.End, line, column))
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
                this.chalkTalkTokens!!.add(ChalkTalkToken(name, ChalkTalkTokenType.Name, line, column))
            } else if (isLetterOrDigit(c)) {
                var name = "" + c
                while (i < text.length && isLetterOrDigit(text[i])) {
                    name += text[i++]
                    column++
                }
                this.chalkTalkTokens!!.add(ChalkTalkToken(name, ChalkTalkTokenType.Name, line, column))
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
                this.chalkTalkTokens!!.add(ChalkTalkToken(str, ChalkTalkTokenType.String, line, column))
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
                this.chalkTalkTokens!!.add(ChalkTalkToken(stmt, ChalkTalkTokenType.Statement, line, column))
            } else if (c == '[') {
                var id = "" + c
                while (i < text.length && text[i] != ']') {
                    id += text[i++]
                    column++
                }
                if (i == text.length) {
                    errors.add(
                            ParseError("Expected a terminating ]", line, column)
                    )
                    id += "]"
                } else {
                    // include the terminating ]
                    id += text[i++]
                    column++
                }
                this.chalkTalkTokens!!.add(ChalkTalkToken(id, ChalkTalkTokenType.Id, line, column))
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
            this.chalkTalkTokens!!.add(ChalkTalkToken("<Unindent>", ChalkTalkTokenType.End, line, column))
            numOpen--
        }
    }

    private fun isOperatorChar(c: Char): Boolean {
        return "~!@#%^&*-+<>\\/".contains(c)
    }

    override fun hasNext(): Boolean {
        ensureInitialized()
        return this.index < this.chalkTalkTokens!!.size
    }

    override fun hasNextNext(): Boolean {
        ensureInitialized()
        return this.index + 1 < this.chalkTalkTokens!!.size
    }

    override fun peek(): ChalkTalkToken {
        ensureInitialized()
        return this.chalkTalkTokens!![this.index]
    }

    override fun peekPeek(): ChalkTalkToken {
        ensureInitialized()
        return this.chalkTalkTokens!![this.index + 1]
    }

    override fun next(): ChalkTalkToken {
        ensureInitialized()
        val tok = peek()
        this.index++
        return tok
    }

    override fun errors(): List<ParseError> {
        return this.errors
    }

    private fun isLetterOrDigit(c: Char): Boolean {
        return Regex("[a-zA-Z0-9]+").matches("$c")
    }
}

private class Stack<T> {
    private val data = ArrayList<T>()

    fun push(item: T) {
        data.add(item)
    }

    fun pop(): T {
        return data.removeAt(data.size - 1)
    }

    fun peek(): T {
        return data.elementAt(data.size - 1)
    }

    fun isEmpty(): Boolean {
        return data.isEmpty()
    }
}

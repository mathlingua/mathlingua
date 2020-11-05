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

import java.util.Stack
import mathlingua.chalktalk.phase1.ast.ChalkTalkTokenType
import mathlingua.chalktalk.phase1.ast.Phase1Token
import mathlingua.support.ParseError

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

// ------------------------------------------------------------------------------------------------------------------ //

// this function is needed to support transpiling to JavaScript since kotlinc-js states
// that Character.isDigit() is an unresolved reference
private fun isDigit(c: Char): Boolean {
    return c in '0'..'9'
}

private class ChalkTalkLexerImpl(private var text: String) : ChalkTalkLexer {

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
            if (c == '.' &&
                i < text.length &&
                text[i] == '.' &&
                i + 1 < text.length &&
                text[i + 1] == '.') {
                val startColumn = column
                // move past the ...
                i += 2
                column += 2
                this.chalkTalkTokens!!.add(
                    Phase1Token("...", ChalkTalkTokenType.DotDotDot, line, startColumn))
            } else if (c == '=') {
                this.chalkTalkTokens!!.add(
                    Phase1Token("=", ChalkTalkTokenType.Equals, line, column))
            } else if (c == '_') {
                this.chalkTalkTokens!!.add(
                    Phase1Token("_", ChalkTalkTokenType.Underscore, line, column))
            } else if (c == '(') {
                this.chalkTalkTokens!!.add(
                    Phase1Token("(", ChalkTalkTokenType.LParen, line, column))
            } else if (c == ')') {
                this.chalkTalkTokens!!.add(
                    Phase1Token(")", ChalkTalkTokenType.RParen, line, column))
            } else if (c == '{') {
                this.chalkTalkTokens!!.add(
                    Phase1Token("{", ChalkTalkTokenType.LCurly, line, column))
            } else if (c == '}') {
                this.chalkTalkTokens!!.add(
                    Phase1Token("}", ChalkTalkTokenType.RCurly, line, column))
            } else if (c == ':') {
                if (i < text.length && text[i] == '=') {
                    this.chalkTalkTokens!!.add(
                        Phase1Token(":=", ChalkTalkTokenType.ColonEquals, line, column))
                    i++ // move past the =
                    column++
                } else {
                    this.chalkTalkTokens!!.add(
                        Phase1Token(":", ChalkTalkTokenType.Colon, line, column))
                }
            } else if (c == ',') {
                this.chalkTalkTokens!!.add(Phase1Token(",", ChalkTalkTokenType.Comma, line, column))
            } else if (c == '.' && i < text.length && text[i] == ' ') {
                this.chalkTalkTokens!!.add(
                    Phase1Token(". ", ChalkTalkTokenType.DotSpace, line, column))
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
                        Phase1Token("-", ChalkTalkTokenType.Linebreak, line, column))
                    continue
                }

                var indentCount = 0
                while (i < text.length &&
                    i + 1 < text.length &&
                    text[i] == ' ' &&
                    text[i + 1] == ' ') {
                    indentCount++
                    i += 2
                    column += 2
                }

                // treat '. ' like another indent
                if (i < text.length &&
                    text[i] == '.' &&
                    i + 1 < text.length &&
                    text[i + 1] == ' ') {
                    indentCount++
                }

                this.chalkTalkTokens!!.add(
                    Phase1Token("<Indent>", ChalkTalkTokenType.Begin, line, column))
                numOpen++

                val level = levStack.peek()
                if (indentCount <= level) {
                    while (numOpen > 0 && !levStack.isEmpty() && indentCount <= levStack.peek()) {
                        this.chalkTalkTokens!!.add(
                            Phase1Token("<Unindent>", ChalkTalkTokenType.End, line, column))
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
                val startLine = line
                val startColumn = column
                var name = "" + c
                while (i < text.length && isOperatorChar(text[i])) {
                    name += text[i++]
                    column++
                }
                this.chalkTalkTokens!!.add(
                    Phase1Token(name, ChalkTalkTokenType.Name, startLine, startColumn))
            } else if (isNameChar(c) || c == '?') {
                // a name can be of the form
                //   ?
                //   name
                //   name?
                //   name...
                //   name#123
                //   name...#other...
                val startLine = line
                val startColumn = column
                var name = "" + c
                var isComplete = false

                // get the text portion
                while (i < text.length && isNameChar(text[i])) {
                    name += text[i++]
                    column++
                }

                var hasQuestionMark = false
                if (i < text.length && text[i] == '?') {
                    hasQuestionMark = true
                    name += text[i++]
                    column++
                }

                if (!hasQuestionMark) {
                    // process the name#123 case and if matching mark the match as complete
                    if (i < text.length &&
                        text[i] == '#' &&
                        i + 1 < text.length &&
                        isDigit(text[i + 1])) {
                        name += text[i++] // append #
                        column++
                        while (i < text.length && isDigit(text[i])) {
                            name += text[i++]
                            column++
                        }
                        isComplete = true
                    }

                    // if it is not complete, that means it is not of the form name#123
                    // so check if it is of the form name...
                    if (!isComplete &&
                        i < text.length &&
                        text[i] == '.' &&
                        i + 1 < text.length &&
                        text[i + 1] == '.' &&
                        i + 2 < text.length &&
                        text[i + 2] == '.') {
                        for (tmp in 0 until "...".length) {
                            name += text[i++]
                            column++
                        }
                        // it is not necessarily complete if it is of the form name...
                        // at this point because it could actually be of the form name...#other...
                    }

                    // check if it is of the form name...#other...
                    if (!isComplete && i < text.length && text[i] == '#') {
                        name += text[i++] // append the #
                        column++
                        // get the name portion
                        while (i < text.length && isNameChar(text[i])) {
                            name += text[i++]
                            column++
                        }
                        // error if a name after # wasn't specified
                        if (name.endsWith("#")) {
                            errors.add(
                                ParseError(
                                    "If a name contains a # is must be of the form " +
                                        "<identifier>...#<identifier>... but found '$name' " +
                                        " (missing the name after '#')",
                                    startLine,
                                    startColumn))
                        }
                        // get the ... portion
                        if (i < text.length &&
                            text[i] == '.' &&
                            i + 1 < text.length &&
                            text[i + 1] == '.' &&
                            i + 2 < text.length &&
                            text[i + 2] == '.') {
                            for (tmp in 0 until "...".length) {
                                name += text[i++]
                                column++
                            }
                        }
                        // error if it is of the form <name>...#<name>
                        // without the trailing ...
                        if (!name.endsWith("...")) {
                            errors.add(
                                ParseError(
                                    "If a name contains a # is must be of the form " +
                                        "<identifier>...#<identifier>... but found '$name' " +
                                        "(missing the trailing '...')",
                                    startLine,
                                    startColumn))
                        }
                    }
                }

                this.chalkTalkTokens!!.add(
                    Phase1Token(name, ChalkTalkTokenType.Name, startLine, startColumn))
            } else if (c == '"') {
                val startLine = line
                val startColumn = column
                var str = "" + c
                while (i < text.length && text[i] != '"') {
                    str += text[i++]
                    column++
                }
                if (i == text.length) {
                    errors.add(ParseError("Expected a terminating \"", line, column))
                    str += "\""
                } else {
                    // include the terminating "
                    str += text[i++]
                    column++
                }
                this.chalkTalkTokens!!.add(
                    Phase1Token(str, ChalkTalkTokenType.String, startLine, startColumn))
            } else if (c == '\'') {
                val startLine = line
                val startColumn = column
                var stmt = "" + c
                while (i < text.length && text[i] != '\'') {
                    stmt += text[i++]
                    column++
                }
                if (i == text.length) {
                    errors.add(ParseError("Expected a terminating '", line, column))
                    stmt += "'"
                } else {
                    // include the terminating '
                    stmt += text[i++]
                    column++
                }
                this.chalkTalkTokens!!.add(
                    Phase1Token(stmt, ChalkTalkTokenType.Statement, startLine, startColumn))
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
                    errors.add(ParseError("Expected a terminating ]", line, column))
                    id += "]"
                }
                this.chalkTalkTokens!!.add(
                    Phase1Token(id, ChalkTalkTokenType.Id, startLine, startColumn))
            } else if (c != ' ') { // spaces are ignored
                errors.add(ParseError("Unrecognized character $c", line, column))
            }
        }

        // numOpen must be used to determine the number of open Begin chalkTalkTokens
        // (as apposed to checking if levStack.isNotEmpty()) since whenever
        // the levStack is empty in the above code, it is re-initialized to
        // contain a level of 0
        while (numOpen > 0) {
            this.chalkTalkTokens!!.add(
                Phase1Token("<Unindent>", ChalkTalkTokenType.End, line, column))
            numOpen--
        }
    }

    private fun isOperatorChar(c: Char) = "~!@%^&*-+<>\\/=".contains(c)

    private fun isNameChar(c: Char) = Regex("[a-zA-Z0-9]+").matches("$c")

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

    override fun errors() = this.errors
}

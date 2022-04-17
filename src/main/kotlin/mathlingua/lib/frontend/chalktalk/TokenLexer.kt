package mathlingua.lib.frontend.chalktalk

import java.util.Stack
import mathlingua.lib.frontend.Diagnostic
import mathlingua.lib.frontend.DiagnosticType
import java.util.LinkedList

internal interface TokenLexer {
    fun hasNext(): Boolean
    fun peek(): Token
    fun next(): Token

    fun hasNextNext(): Boolean
    fun peekPeek(): Token
    fun nextNext(): Token

    fun diagnostics(): List<Diagnostic>
}

internal fun newTokenLexer(text: String): TokenLexer {
    return TokenLexerImpl(text)
}

internal enum class TokenType {
    ColonEqual,
    LParen,
    RParen,
    LCurly,
    RCurly,
    Comma,
    Colon,
    Underscore,
    Text,
    Statement,
    TextBlock,
    Name,
    Operator,
    UnIndent,
    DotSpace,
    LineBreak,
    Id,
    Newline,
    DotDotDot
}

internal data class Token(val type: TokenType, val text: String, val row: Int, val column: Int)

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

private class TokenLexerImpl(text: String) : TokenLexer {
    private val tokens = LinkedList<Token>()
    private val diagnostics = mutableListOf<Diagnostic>()

    init {
        var row = 0
        var column = 0
        var i = 0

        val indentStack = Stack<Int>()
        while (i < text.length) {
            // handle line breaks
            if (text[i] == '\n' && i + 1 < text.length && text[i + 1] == '\n') {
                // there is a newline followed by a line break
                tokens.add(
                    Token(type = TokenType.Newline, text = "<newline>", row = row, column = column))
                tokens.add(
                    Token(
                        type = TokenType.LineBreak,
                        text = "<linebreak>",
                        row = row,
                        column = column))
                while (i < text.length && text[i] == '\n') {
                    row++
                    column = 0
                    i++
                }
                continue
            }

            // handle indents and un-indents
            if (text[i] == '\n') {
                tokens.add(
                    Token(type = TokenType.Newline, text = "<newline>", row = row, column = column))

                // move past the newline character
                i++
                row++
                column = 0

                val indent = text.getIndent(i)
                i += indent.size
                column += indent.size

                if (indentStack.isEmpty()) {
                    if (indent.size > 0 && !indent.endsWithDotSpace) {
                        // the only indent possible is a '. ' if no indentation
                        // has occurred yet
                        diagnostics.add(
                            Diagnostic(
                                type = DiagnosticType.Error,
                                message = "Expected a '. '",
                                row = row,
                                column = column))
                    }
                    if (indent.size > 0) {
                        indentStack.push(indent.size)
                    }
                } else {
                    val prevIndent = indentStack.peek()
                    if (indent.size > prevIndent) {
                        if (!indent.endsWithDotSpace) {
                            diagnostics.add(
                                Diagnostic(
                                    type = DiagnosticType.Error,
                                    message = "Unexpected indent",
                                    row = row,
                                    column = column))
                        }
                        indentStack.push(indent.size)
                    } else if (indent.size < prevIndent) {
                        while (indentStack.isNotEmpty() && indent.size < indentStack.peek()) {
                            indentStack.pop()
                            tokens.add(
                                Token(
                                    type = TokenType.UnIndent,
                                    text = "<unindent>",
                                    row = row,
                                    column = column))
                        }
                        // after unrolling the stack while the current indent is smaller than
                        // any previously seen indents, if the last indent in the stack doesn't
                        // match the current indent, then the current indent has an additional
                        // space, which is an error
                        if ((indentStack.isNotEmpty() && indent.size != indentStack.peek()) ||
                            (indentStack.isEmpty() && indent.size > 0)) {
                            diagnostics.add(
                                Diagnostic(
                                    type = DiagnosticType.Error,
                                    message = "Misaligned indent",
                                    row = row,
                                    column = column))
                        }
                    }
                    // else: If the new line is indented the same as the previous indent,
                    // then nothing needs to be done since no change in indentation has occurred
                }

                // report a '. ' regardless of whether the indentation has increased, decreased,
                // or stayed the same
                if (indent.endsWithDotSpace) {
                    tokens.add(
                        Token(type = TokenType.DotSpace, text = ". ", row = row, column = column))
                }

                continue
            }

            // handle ids
            if (text[i] == '[') {
                val startRow = row
                val startColumn = column

                i++
                column++

                // the number of open [ brackets
                var numOpen = 1
                val buffer = StringBuilder()
                while (i < text.length) {
                    val c = text[i]
                    i++
                    column++
                    if (c == '[') {
                        numOpen++
                        buffer.append(c)
                    } else if (c == ']') {
                        numOpen--
                        if (numOpen > 0) {
                            buffer.append(c)
                        } else {
                            break
                        }
                    } else if (c == '\n') {
                        row++
                        buffer.append(c)
                    } else {
                        buffer.append(c)
                    }
                }
                tokens.add(
                    Token(
                        type = TokenType.Id,
                        text = buffer.toString(),
                        row = startRow,
                        column = startColumn))
                continue
            }

            // handle "...", '...', and `...`
            val streamResult = text.checkStreamAndType(i)
            if (streamResult != null) {
                tokens.add(
                    Token(
                        type = streamResult.type,
                        text =
                            streamResult.text.removeSurrounding(
                                streamResult.prefix, streamResult.suffix),
                        row = row,
                        column = column))

                row += streamResult.numNewlines
                i += streamResult.text.length
                if (streamResult.numNewlines == 0) {
                    column += streamResult.text.length
                } else {
                    val index = streamResult.text.lastIndexOf("\n")
                    column = streamResult.text.length - index - 1
                }
                continue
            }

            // handle keyword tokens
            val textAndType = text.checkTextAndType(i)
            if (textAndType != null) {
                tokens.add(
                    Token(
                        type = textAndType.type,
                        text = textAndType.text,
                        row = row,
                        column = column))
                column += textAndType.text.length
                i += textAndType.text.length
                continue
            }

            // handle identifiers
            val identifier = text.checkName(i)
            if (identifier != null) {
                tokens.add(
                    Token(type = TokenType.Name, text = identifier, row = row, column = column))
                column += identifier.length
                i += identifier.length
                continue
            }

            // handle operators
            val operator = text.checkOperator(i)
            if (operator != null) {
                tokens.add(
                    Token(type = TokenType.Operator, text = operator, row = row, column = column))
                column += operator.length
                i += operator.length
                continue
            }

            // ignore whitespace
            val c = text[i]
            if (c == ' ') {
                i++
                column++
                continue
            }

            diagnostics.add(
                Diagnostic(
                    type = DiagnosticType.Error,
                    message = "Unrecognized token '$c'",
                    row = row,
                    column = column))

            if (c == '\n') {
                row++
            }
            column++
            i++
        }

        while (indentStack.isNotEmpty()) {
            val indent = indentStack.pop()
            if (indent > 0) {
                diagnostics.add(
                    Diagnostic(
                        type = DiagnosticType.Error,
                        message = "Unexpected indent of size $indent",
                        row = row,
                        column = column))
            }
        }
    }

    override fun hasNext() = tokens.isNotEmpty()

    override fun peek() = tokens.element()!!

    override fun next() = tokens.remove()!!

    override fun hasNextNext() = tokens.size >= 2

    override fun peekPeek() = tokens[1]

    override fun nextNext(): Token {
        tokens.remove()
        return tokens.remove()
    }

    override fun diagnostics(): List<Diagnostic> = diagnostics
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

private data class TextAndType(val text: String, val type: TokenType)

private val TEXT_AND_TYPES =
    mutableListOf(
        TextAndType(":=", TokenType.ColonEqual),
        TextAndType("(", TokenType.LParen),
        TextAndType(")", TokenType.RParen),
        TextAndType("{", TokenType.LCurly),
        TextAndType("}", TokenType.RCurly),
        TextAndType(",", TokenType.Comma),
        TextAndType(":", TokenType.Colon),
        TextAndType("_", TokenType.Underscore),
        TextAndType("...", TokenType.DotDotDot),
        TextAndType(". ", TokenType.DotSpace))

private data class StreamAndType(
    val prefix: String, val suffix: String, val escape: String?, val type: TokenType)

private val STREAMS_AND_TYPES =
    mutableListOf(
        StreamAndType("'", "'", null, TokenType.Statement),
        StreamAndType("`", "`", null, TokenType.Statement),
        StreamAndType("\"", "\"", "\\\"", TokenType.Text),
        StreamAndType("::", "::", "{::}", TokenType.TextBlock))

private fun String.checkTextAndType(index: Int): TextAndType? {
    for (textAndType in TEXT_AND_TYPES) {
        if (this.startsWith(textAndType.text, index)) {
            return textAndType
        }
    }
    return null
}

private data class CheckStreamAndTypeResult(
    val type: TokenType,
    val text: String,
    val numNewlines: Int,
    val prefix: String,
    val suffix: String)

private fun String.checkStreamAndType(index: Int): CheckStreamAndTypeResult? {
    var i = index
    for (streamAndType in STREAMS_AND_TYPES) {
        if (this.startsWith(streamAndType.prefix, i)) {
            val buffer = StringBuilder()
            buffer.append(streamAndType.prefix)
            i += streamAndType.prefix.length

            var numNewlines = 0
            while (i < this.length) {
                if (streamAndType.escape != null && this.startsWith(streamAndType.escape, i)) {
                    buffer.append(streamAndType.escape)
                    i += streamAndType.escape.length
                } else if (this.startsWith(streamAndType.suffix, i)) {
                    buffer.append(streamAndType.suffix)
                    i += streamAndType.suffix.length
                    break
                } else {
                    val c = this[i++]
                    buffer.append(c)
                    if (c == '\n') {
                        numNewlines++
                    }
                }
            }

            return CheckStreamAndTypeResult(
                type = streamAndType.type,
                text = buffer.toString(),
                numNewlines = numNewlines,
                prefix = streamAndType.prefix,
                suffix = streamAndType.suffix)
        }
    }
    return null
}

private fun String.checkName(index: Int): String? {
    val buffer = StringBuilder()
    var i = index
    while (i < this.length && this[i].isNameChar()) {
        buffer.append(this[i++])
    }
    if (buffer.isNotEmpty() &&
        i < this.length &&
        this[i] == '_' &&
        i + 1 < this.length &&
        this[i + 1].isNameChar()) {
        buffer.append(this[i++])
        while (i < this.length && this[i].isNameChar()) {
            buffer.append(this[i++])
        }
    }
    return if (buffer.isEmpty()) {
        null
    } else {
        buffer.toString()
    }
}

private fun Char.isNameChar() =
    (this in 'a'..'z') || (this in 'A'..'Z') || (this in '0'..'9') || (this in "`'\"")

private fun String.checkOperator(index: Int): String? {
    val buffer = StringBuilder()
    var i = index
    while (i < this.length && this[i].isOperatorChar()) {
        buffer.append(this[i++])
    }
    if (buffer.isNotEmpty() && i < this.length && this[i] == '_') {
        buffer.append(this[i++])
        while (i < this.length && this[i].isNameChar()) {
            buffer.append(this[i++])
        }
    }
    return if (buffer.isEmpty()) {
        null
    } else {
        buffer.toString()
    }
}

private fun Char.isOperatorChar() = (this in "~!@#$%^&*-+=|<>?'`\"")

private data class Indent(val size: Int, val endsWithDotSpace: Boolean, val error: String?)

private fun String.getIndent(index: Int): Indent {
    var size = 0
    var endsWithDotSpace = false
    var error: String? = null
    var i = index
    while (i < this.length && this[i] == ' ') {
        size++
        i++
    }
    if (i + 1 < this.length && this[i] == '.' && this[i + 1] == ' ') {
        i += 2
        size += 2
        endsWithDotSpace = true
    }
    if (i < this.length && this[i] == ' ') {
        error = "Unexpected whitespace after '. '"
        while (i < this.length && this[i] == ' ') {
            i++
            size++
        }
    }
    return Indent(size, endsWithDotSpace, error)
}

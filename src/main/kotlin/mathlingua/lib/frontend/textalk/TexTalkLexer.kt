package mathlingua.lib.frontend.textalk

import java.util.LinkedList
import mathlingua.lib.frontend.Diagnostic
import mathlingua.lib.frontend.DiagnosticType

internal interface TexTalkLexer {
    fun hasNext(): Boolean
    fun peek(): TexTalkToken
    fun next(): TexTalkToken

    fun hasNextNext(): Boolean
    fun peekPeek(): TexTalkToken
    fun nextNext(): TexTalkToken

    fun diagnostics(): List<Diagnostic>
}

internal fun newTexTalkLexer(text: String): TexTalkLexer {
    return TexTalkLexerImpl(text)
}

internal enum class TexTalkTokenType {
    LParen,
    RParen,
    LSquare,
    RSquare,
    LCurly,
    RCurly,
    Comma,
    ColonEquals,
    Is,
    In,
    NotIn,
    As,
    ColonTypeColon,
    ColonStatementColon,
    ColonExpressionColon,
    Underscore,
    Caret,
    DotDotDot,
    Dot,
    Backslash,
    Slash,
    Name,
    Operator,
    Colon,
    Equals,
    NotEquals
}

internal data class TexTalkToken(
    val type: TexTalkTokenType, val text: String, val row: Int, val column: Int)

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

private class TexTalkLexerImpl(text: String) : TexTalkLexer {
    private val tokens = LinkedList<TexTalkToken>()
    private val diagnostics = mutableListOf<Diagnostic>()

    init {
        var row = 0
        var column = 0
        var i = 0

        while (i < text.length) {
            // handle keyword tokens
            val textAndType = text.checkTextAndType(i)
            if (textAndType != null) {
                tokens.add(
                    TexTalkToken(
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
                    TexTalkToken(
                        type = TexTalkTokenType.Name,
                        text = identifier,
                        row = row,
                        column = column))
                column += identifier.length
                i += identifier.length
                continue
            }

            // handle operators
            val operator = text.checkOperator(i)
            if (operator != null) {
                tokens.add(
                    TexTalkToken(
                        type = TexTalkTokenType.Operator,
                        text = operator,
                        row = row,
                        column = column))
                column += operator.length
                i += operator.length
                continue
            }

            // ignore whitespace
            val c = text[i]
            if (c == ' ' || c == '\t' || c == '\n') {
                i++
                column++
                if (c == '\n') {
                    row++
                }
                continue
            }

            diagnostics.add(
                Diagnostic(
                    type = DiagnosticType.Error,
                    message = "Unrecognized token '$c'",
                    row = row,
                    column = column))

            column++
            i++
        }
    }

    override fun hasNext() = tokens.isNotEmpty()

    override fun peek() = tokens.element()!!

    override fun next() = tokens.remove()!!

    override fun hasNextNext() = tokens.size >= 2

    override fun peekPeek() = tokens[1]

    override fun nextNext(): TexTalkToken {
        tokens.remove()
        return tokens.remove()
    }

    override fun diagnostics(): List<Diagnostic> = diagnostics
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

private data class TextAndType(val text: String, val type: TexTalkTokenType)

private val TEXT_AND_TYPES =
    mutableListOf(
        TextAndType("(", TexTalkTokenType.LParen),
        TextAndType(")", TexTalkTokenType.RParen),
        TextAndType("[", TexTalkTokenType.LSquare),
        TextAndType("]", TexTalkTokenType.RSquare),
        TextAndType("{", TexTalkTokenType.LCurly),
        TextAndType("}", TexTalkTokenType.RCurly),
        TextAndType(",", TexTalkTokenType.Comma),
        TextAndType("is", TexTalkTokenType.Is),
        TextAndType("in", TexTalkTokenType.In),
        TextAndType("notin", TexTalkTokenType.NotIn),
        TextAndType("as", TexTalkTokenType.As),
        TextAndType("_", TexTalkTokenType.Underscore),
        TextAndType("^", TexTalkTokenType.Caret),
        TextAndType("...", TexTalkTokenType.DotDotDot),
        TextAndType(".", TexTalkTokenType.Dot),
        TextAndType("\\", TexTalkTokenType.Backslash),
        TextAndType("/", TexTalkTokenType.Slash),
        TextAndType("=", TexTalkTokenType.Equals),
        TextAndType("!=", TexTalkTokenType.NotEquals),
        TextAndType(":=", TexTalkTokenType.ColonEquals),
        TextAndType(":Type:", TexTalkTokenType.ColonTypeColon),
        TextAndType(":Statement:", TexTalkTokenType.ColonStatementColon),
        TextAndType(":Expression:", TexTalkTokenType.ColonExpressionColon),
        TextAndType(":", TexTalkTokenType.Colon))

private fun String.checkTextAndType(index: Int): TextAndType? {
    for (textAndType in TEXT_AND_TYPES) {
        if (this.startsWith(textAndType.text, index)) {
            return textAndType
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
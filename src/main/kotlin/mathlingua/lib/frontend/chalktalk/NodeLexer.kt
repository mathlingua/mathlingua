package mathlingua.lib.frontend.chalktalk

import mathlingua.lib.frontend.MetaData
import mathlingua.lib.frontend.ParseError

internal interface NodeLexer {
    fun hasNext(): Boolean
    fun peek(): ChalkTalkNode
    fun next(): ChalkTalkNode
    fun errors(): List<ParseError>
}

internal fun newNodeLexer(lexer: TokenLexer): NodeLexer {
    return NodeLexerImpl(lexer)
}

//////////////////////////////////////////////////////

private class NodeLexerImpl(private val lexer: TokenLexer) : NodeLexer {
    private val nodes = mutableListOf<ChalkTalkNode>()
    private var index = 0
    private val errors = mutableListOf<ParseError>()

    init {
        var inArg = false
        var inGroup = false
        var inSection = false
        var wasPrevDotSpace = false
        while (lexer.hasNext()) {
            val next = lexer.peek()
            when (next.type) {
                TokenType.LParen -> {
                    nodes.add(BeginArgument)
                    val arg = argument(isInline = !wasPrevDotSpace)
                    if (arg != null) {
                        nodes.add(arg)
                    } else {
                        errors.add(
                            ParseError(message = "Expected an argument", row = -1, column = -1))
                    }
                    nodes.add(EndArgument)
                }
                TokenType.LCurly -> {
                    nodes.add(BeginArgument)
                    val arg = argument(isInline = !wasPrevDotSpace)
                    if (arg != null) {
                        nodes.add(arg)
                    } else {
                        errors.add(
                            ParseError(message = "Expected an argument", row = -1, column = -1))
                    }
                    nodes.add(EndArgument)
                }
                TokenType.Text -> {
                    nodes.add(
                        Text(
                            text = next.text,
                            metadata =
                                MetaData(
                                    row = next.row,
                                    column = next.column,
                                    isInline = !wasPrevDotSpace)))
                    lexer.next()
                }
                TokenType.Statement -> {
                    nodes.add(
                        Statement(
                            text = next.text,
                            metadata =
                                MetaData(
                                    row = next.row,
                                    column = next.column,
                                    isInline = !wasPrevDotSpace),
                        ))
                    lexer.next()
                }
                TokenType.TextBlock -> {
                    nodes.add(
                        TextBlock(
                            text = next.text,
                            metadata =
                                MetaData(
                                    row = next.row,
                                    column = next.column,
                                    isInline = !wasPrevDotSpace),
                        ))
                    lexer.next()
                }
                TokenType.Name -> {
                    if (lexer.hasNextNext() && lexer.peekPeek().type == TokenType.Colon) {
                        if (!inGroup) {
                            nodes.add(BeginGroup)
                        }
                        if (!inSection) {
                            nodes.add(BeginSection(name = next.text))
                        }
                        lexer.next() // move past the name
                        lexer.next() // move past the colon
                        inGroup = true
                        inSection = true
                    } else {
                        nodes.add(BeginArgument)
                        val arg = argument(isInline = !wasPrevDotSpace)
                        if (arg != null) {
                            nodes.add(arg)
                        } else {
                            errors.add(
                                ParseError(message = "Expected an argument", row = -1, column = -1))
                        }
                        nodes.add(EndArgument)
                    }
                }
                TokenType.Operator -> {
                    nodes.add(BeginArgument)
                    val arg = argument(isInline = !wasPrevDotSpace)
                    if (arg != null) {
                        nodes.add(arg)
                    } else {
                        errors.add(
                            ParseError(message = "Expected an argument", row = -1, column = -1))
                    }
                    nodes.add(EndArgument)
                }
                TokenType.Unindent -> {
                    if (inArg) {
                        nodes.add(EndArgument)
                    }
                    if (inSection) {
                        nodes.add(EndSection)
                    }
                    inArg = false
                    inSection = false
                    lexer.next()
                }
                TokenType.DotSpace -> {
                    if (inArg) {
                        nodes.add(EndArgument)
                    } else {
                        nodes.add(BeginArgument)
                    }
                    inArg = true
                    lexer.next()
                }
                TokenType.LineBreak -> {
                    if (inGroup) {
                        nodes.add(EndGroup)
                    }
                    inGroup = false
                    lexer.next()
                }
                TokenType.Id -> {
                    if (!inGroup) {
                        nodes.add(BeginGroup)
                    }
                    inGroup = true
                    nodes.add(
                        Id(
                            text = next.text,
                            metadata =
                                MetaData(row = next.row, column = next.column, isInline = false)))
                    lexer.next()
                }
                TokenType.Newline -> {
                    if (inSection) {
                        nodes.add(EndSection)
                    }
                    inSection = false
                    lexer.next()
                }
                // the token types below should never be encountered directly
                // instead they should be encountered while creating nodes
                // where construction began through encountering a different
                // starting token
                TokenType.ColonEqual -> {
                    errors.add(
                        ParseError(
                            message = "Unexpected token ${next.text}",
                            row = next.row,
                            column = next.column))
                    lexer.next()
                }
                TokenType.RParen -> {
                    errors.add(
                        ParseError(
                            message = "Unexpected token ${next.text}",
                            row = next.row,
                            column = next.column))
                    lexer.next()
                }
                TokenType.RCurly -> {
                    errors.add(
                        ParseError(
                            message = "Unexpected token ${next.text}",
                            row = next.row,
                            column = next.column))
                    lexer.next()
                }
                TokenType.Comma -> {
                    errors.add(
                        ParseError(
                            message = "Unexpected token ${next.text}",
                            row = next.row,
                            column = next.column))
                    lexer.next()
                }
                TokenType.Colon -> {
                    errors.add(
                        ParseError(
                            message = "Unexpected token ${next.text}",
                            row = next.row,
                            column = next.column))
                    lexer.next()
                }
                TokenType.Underscore -> {
                    errors.add(
                        ParseError(
                            message = "Unexpected token ${next.text}",
                            row = next.row,
                            column = next.column))
                    lexer.next()
                }
                TokenType.DotDotDot -> {
                    errors.add(
                        ParseError(
                            message = "Unexpected token ${next.text}",
                            row = next.row,
                            column = next.column))
                    lexer.next()
                }
            }

            wasPrevDotSpace = next.type == TokenType.DotSpace
        }
    }

    override fun hasNext() = index < nodes.size

    override fun peek() = nodes[index]

    override fun next() = nodes[index++]

    override fun errors(): List<ParseError> = errors

    private fun has(type: TokenType) = lexer.hasNext() && lexer.peek().type == type

    private fun hasHas(type1: TokenType, type2: TokenType) =
        lexer.hasNext() &&
            lexer.hasNextNext() &&
            lexer.peek().type == type1 &&
            lexer.peekPeek().type == type2

    private fun expect(type: TokenType): Token? {
        if (!lexer.hasNext()) {
            errors.add(
                ParseError(
                    message = "Expected a $type token but found the end of text",
                    row = -1,
                    column = -1))
            return null
        }
        val next = lexer.next()
        if (next.type != type) {
            errors.add(
                ParseError(
                    message = "Expected a $type but found ${next.type}",
                    row = next.row,
                    column = next.column))
        }
        return next
    }

    private fun name(isInline: Boolean): Name? {
        if (!has(TokenType.Name)) {
            return null
        }
        val next = lexer.next()
        return Name(
            text = next.text,
            metadata = MetaData(row = next.row, column = next.column, isInline = isInline))
    }

    private fun nameParam(isInline: Boolean): NameParam? {
        val name = name(isInline) ?: return null
        val hasDotDotDot =
            if (has(TokenType.DotDotDot)) {
                expect(TokenType.DotDotDot)
                true
            } else {
                false
            }
        return NameParam(name = name, isVarArgs = hasDotDotDot)
    }

    // returns a list of names if the input is of the form `<name>, ...` or `<name><end>`
    // and returns `null` if the input is only a single `<name>` not followed by a <comma>
    // or <end>.  Thus, the function makes a <name> appearing at the start of the input.
    // is actually part of a list.
    private fun nameParamList(end: TokenType, isInline: Boolean): List<NameParam>? {
        if (!hasHas(TokenType.Name, TokenType.Comma) && !hasHas(TokenType.Name, end)) {
            return null
        }

        val result = mutableListOf<NameParam>()
        while (lexer.hasNext()) {
            val nameParam = nameParam(isInline) ?: break
            result.add(nameParam)
            if (has(end)) {
                break
            }
            if (has(TokenType.Comma)) {
                lexer.next() // move past the comma
            }
        }
        while (lexer.hasNext() && lexer.peek().type != end) {
            val next = lexer.next()
            errors.add(
                ParseError(
                    message = "Expected a $end but found ${next.type}",
                    row = next.row,
                    column = next.column))
        }
        expect(end)
        return result
    }

    private fun operator(isInline: Boolean): Name? {
        if (!has(TokenType.Operator)) {
            return null
        }
        val next = lexer.next()
        return Name(
            text = next.text,
            metadata = MetaData(row = next.row, column = next.column, isInline = isInline))
    }

    private fun argument(isInline: Boolean): ChalkTalkNode? {
        return null!!
    }
}

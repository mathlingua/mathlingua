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

package mathlingua.textalk

import mathlingua.support.ParseError

// ------------------------------------------------------------------------------------------------------------------ //

interface TexTalkParser {
    fun parse(texTalkLexer: TexTalkLexer): TexTalkParseResult
}

data class TexTalkParseResult(val root: ExpressionTexTalkNode, val errors: List<ParseError>)

fun newTexTalkParser(): TexTalkParser {
    return TexTalkParserImpl()
}

// ------------------------------------------------------------------------------------------------------------------ //

private val INVALID = TexTalkToken("INVALID", TexTalkTokenType.Invalid, -1, -1)

private class TexTalkParserImpl : TexTalkParser {
    override fun parse(texTalkLexer: TexTalkLexer): TexTalkParseResult {
        val worker = ParserWorker(texTalkLexer)
        val root = worker.parse()
        val errors = worker.getErrors()
        val result = TexTalkParseResult(root, errors)
        if (errors.isNotEmpty()) {
            return result
        }
        val operatorResult = parseOperators(result.root)
        if (operatorResult.errors.isNotEmpty()) {
            return TexTalkParseResult(
                root = result.root, errors = result.errors.plus(operatorResult.errors))
        }
        return operatorResult
    }

    private class ParserWorker(private val texTalkLexer: TexTalkLexer) {
        private val errors = mutableListOf<ParseError>()

        fun getErrors(): List<ParseError> {
            return errors
        }

        fun parse(): ExpressionTexTalkNode {
            val exp = expression(null) ?: ExpressionTexTalkNode(emptyList())
            return resolveColonOrColonColonEqualsNode(resolveIsNode(exp)) as ExpressionTexTalkNode
        }

        private fun resolveIsNode(texTalkNode: TexTalkNode): TexTalkNode {
            if (texTalkNode !is ExpressionTexTalkNode) {
                return texTalkNode
            }

            var isIndex = -1
            for (i in texTalkNode.children.indices) {
                val child = texTalkNode.children[i]
                if (child is TextTexTalkNode && child.type == TexTalkNodeType.Is) {
                    if (isIndex < 0) {
                        isIndex = i
                    } else {
                        addError("A statement can only contain one 'is' statement")
                    }
                }
            }

            if (isIndex < 0) {
                return texTalkNode
            }

            val lhs = parameters(texTalkNode.children, 0, isIndex)
            val rhs = parameters(texTalkNode.children, isIndex + 1, texTalkNode.children.size)
            return ExpressionTexTalkNode(listOf(IsTexTalkNode(lhs, rhs)))
        }

        private fun resolveColonOrColonColonEqualsNode(texTalkNode: TexTalkNode): TexTalkNode {
            if (texTalkNode !is ExpressionTexTalkNode) {
                return texTalkNode
            }

            var colonEqualsIndex = -1
            var colonColonEqualsIndex = -1
            for (i in texTalkNode.children.indices) {
                val child = texTalkNode.children[i]
                if (child is TextTexTalkNode && child.type == TexTalkNodeType.ColonEquals) {
                    if (colonEqualsIndex < 0) {
                        colonEqualsIndex = i
                    } else {
                        addError("A statement can only contain one ':='")
                    }
                } else if (child is TextTexTalkNode &&
                    child.type == TexTalkNodeType.ColonColonEquals) {
                    if (colonColonEqualsIndex < 0) {
                        colonColonEqualsIndex = i
                    } else {
                        addError("A statement can only contain one '::='")
                    }
                }
            }

            if (colonEqualsIndex >= 0 && colonColonEqualsIndex >= 0) {
                addError("A statement can only contain ':=' or '::=', but not both.")
            }

            return when {
                colonEqualsIndex >= 0 -> {
                    val lhs = parameters(texTalkNode.children, 0, colonEqualsIndex)
                    val rhs =
                        parameters(
                            texTalkNode.children, colonEqualsIndex + 1, texTalkNode.children.size)
                    ExpressionTexTalkNode(listOf(ColonEqualsTexTalkNode(lhs, rhs)))
                }
                colonColonEqualsIndex >= 0 -> {
                    val lhs = parameters(texTalkNode.children, 0, colonColonEqualsIndex)
                    val rhs =
                        parameters(
                            texTalkNode.children,
                            colonColonEqualsIndex + 1,
                            texTalkNode.children.size)
                    ExpressionTexTalkNode(listOf(ColonColonEqualsTexTalkNode(lhs, rhs)))
                }
                else -> {
                    texTalkNode
                }
            }
        }

        private fun parameters(
            texTalkNodes: List<TexTalkNode>, startInc: Int, endEx: Int
        ): ParametersTexTalkNode {
            val parts = mutableListOf<ExpressionTexTalkNode>()
            var i = startInc
            while (i < endEx) {
                val items = mutableListOf<TexTalkNode>()
                while (i < endEx && texTalkNodes[i].type != TexTalkNodeType.Comma) {
                    items.add(resolveIsNode(texTalkNodes[i++]))
                }
                if (i < endEx && texTalkNodes[i].type !== TexTalkNodeType.Comma) {
                    addError("Expected a Comma but found ${texTalkNodes[i].type}")
                } else {
                    i++ // move past the comma
                }
                parts.add(ExpressionTexTalkNode(items))
            }
            return ParametersTexTalkNode(parts)
        }

        private fun command(): Command? {
            if (!has(TexTalkTokenType.Backslash)) {
                return null
            }

            val backSlash = expect(TexTalkTokenType.Backslash)

            val parts = mutableListOf<CommandPart>()
            while (hasNext()) {
                val part = commandPart()
                if (part == null) {
                    addError("Missing a command part", backSlash)
                } else {
                    parts.add(part)
                }
                if (has(TexTalkTokenType.Period)) {
                    expect(TexTalkTokenType.Period) // move past the .
                    if (has(TexTalkTokenType.Operator)) {
                        val operator = expect(TexTalkTokenType.Operator)
                        parts.add(
                            CommandPart(
                                name =
                                    TextTexTalkNode(
                                        type = TexTalkNodeType.Identifier,
                                        tokenType = TexTalkTokenType.Operator,
                                        text = operator.text,
                                        isVarArg = false),
                                square = null,
                                subSup = null,
                                groups = emptyList(),
                                paren = null,
                                namedGroups = emptyList()))
                        break
                    } else if (has(TexTalkTokenType.Caret)) {
                        val caret = expect(TexTalkTokenType.Caret)
                        parts.add(
                            CommandPart(
                                name =
                                    TextTexTalkNode(
                                        type = TexTalkNodeType.Identifier,
                                        tokenType = TexTalkTokenType.Caret,
                                        text = caret.text,
                                        isVarArg = false),
                                square = null,
                                subSup = null,
                                groups = emptyList(),
                                paren = null,
                                namedGroups = emptyList()))
                        break
                    }
                } else {
                    break
                }
            }

            if (parts.isEmpty()) {
                addError("Expected at least one command part", backSlash)
            }

            return Command(parts)
        }

        private fun commandPart(): CommandPart? {
            if (!has(TexTalkTokenType.Identifier)) {
                return null
            }

            val name = text(TexTalkTokenType.Identifier, TexTalkNodeType.Identifier, false)
            val square = group(TexTalkNodeType.SquareGroup)
            val subSup = subSup()
            val groups = mutableListOf<GroupTexTalkNode>()

            while (texTalkLexer.hasNext()) {
                val grp = group(TexTalkNodeType.CurlyGroup)
                grp ?: break
                groups.add(grp)
            }

            val paren = group(TexTalkNodeType.ParenGroup)
            val namedGroups = mutableListOf<NamedGroupTexTalkNode>()
            if (has(TexTalkTokenType.Colon)) {
                expect(TexTalkTokenType.Colon) // absorb the colon
                while (hasNext()) {
                    val namedGrp = namedGroup()
                    namedGrp ?: break
                    namedGroups.add(namedGrp)
                }
            }

            return CommandPart(name!!, square, subSup, groups, paren, namedGroups)
        }

        private fun subSup(): SubSupTexTalkNode? {
            val sub = sub()
            val sup = sup()
            return if (sub == null && sup == null) {
                null
            } else SubSupTexTalkNode(sub, sup)
        }

        private fun sub(): GroupTexTalkNode? {
            if (!has(TexTalkTokenType.Underscore)) {
                return null
            }

            val (_, _, row, column) = expect(TexTalkTokenType.Underscore)

            var grp: GroupTexTalkNode? = null
            if (has(TexTalkTokenType.Identifier)) {
                val id = next()
                grp =
                    GroupTexTalkNode(
                        type = TexTalkNodeType.CurlyGroup,
                        parameters =
                            ParametersTexTalkNode(
                                items =
                                    listOf(
                                        ExpressionTexTalkNode(
                                            children =
                                                listOf(
                                                    TextTexTalkNode(
                                                        type = TexTalkNodeType.Identifier,
                                                        tokenType = id.tokenType,
                                                        text = id.text,
                                                        isVarArg = false))))),
                        isVarArg = false)
            }

            if (grp == null) {
                val curly = group(TexTalkNodeType.CurlyGroup)
                if (curly != null) {
                    grp = curly
                }
            }

            if (grp == null) {
                val paren = group(TexTalkNodeType.ParenGroup)
                if (paren != null) {
                    grp = paren
                }
            }

            if (grp == null) {
                addError("Expected a value with an underscore", row, column)
                grp =
                    GroupTexTalkNode(
                        TexTalkNodeType.CurlyGroup, ParametersTexTalkNode(emptyList()), false)
            }

            return grp
        }

        private fun sup(): GroupTexTalkNode? {
            if (!has(TexTalkTokenType.Caret)) {
                return null
            }

            val (_, _, row, column) = expect(TexTalkTokenType.Caret)

            var grp: GroupTexTalkNode? = null
            if (has(TexTalkTokenType.Identifier)) {
                val id = next()
                grp =
                    GroupTexTalkNode(
                        type = TexTalkNodeType.CurlyGroup,
                        parameters =
                            ParametersTexTalkNode(
                                items =
                                    listOf(
                                        ExpressionTexTalkNode(
                                            children =
                                                listOf(
                                                    TextTexTalkNode(
                                                        type = TexTalkNodeType.Identifier,
                                                        tokenType = id.tokenType,
                                                        text = id.text,
                                                        isVarArg = false))))),
                        isVarArg = false)
            }

            if (grp == null) {
                val curly = group(TexTalkNodeType.CurlyGroup)
                if (curly != null) {
                    grp = curly
                }
            }

            if (grp == null) {
                val paren = group(TexTalkNodeType.ParenGroup)
                if (paren != null) {
                    grp = paren
                }
            }

            if (grp == null) {
                addError("Expected a value with a caret", row, column)
                grp =
                    GroupTexTalkNode(
                        TexTalkNodeType.CurlyGroup, ParametersTexTalkNode(emptyList()), false)
            }

            return grp
        }

        private fun group(nodeType: TexTalkNodeType): GroupTexTalkNode? {
            val startType: TexTalkTokenType
            val endType: TexTalkTokenType
            when (nodeType) {
                TexTalkNodeType.ParenGroup -> {
                    startType = TexTalkTokenType.LParen
                    endType = TexTalkTokenType.RParen
                }
                TexTalkNodeType.SquareGroup -> {
                    startType = TexTalkTokenType.LSquare
                    endType = TexTalkTokenType.RSquare
                }
                TexTalkNodeType.CurlyGroup -> {
                    startType = TexTalkTokenType.LCurly
                    endType = TexTalkTokenType.RCurly
                }
                else -> throw RuntimeException("Unrecognized group type $nodeType")
            }

            if (!has(startType)) {
                return null
            }

            val expressions = mutableListOf<ExpressionTexTalkNode>()
            expect(startType)

            val terminators = HashSet<TexTalkTokenType>()
            terminators.add(TexTalkTokenType.Comma)
            terminators.add(endType)

            val firstExp = expression(terminators)
            if (firstExp != null) {
                expressions.add(firstExp)
            }

            while (has(TexTalkTokenType.Comma)) {
                next() // absorb the comma
                val exp = expression(terminators)
                exp ?: break
                expressions.add(exp)
            }

            expect(endType)

            val isVarArg = has(TexTalkTokenType.DotDotDot)
            if (isVarArg) {
                next() // move past the ...
            }

            return GroupTexTalkNode(
                type = nodeType,
                parameters = ParametersTexTalkNode(expressions),
                isVarArg = isVarArg)
        }

        private fun namedGroup(): NamedGroupTexTalkNode? {
            if (!hasHas(TexTalkTokenType.Identifier, TexTalkTokenType.LCurly)) {
                return null
            }

            val rawText = text(TexTalkTokenType.Identifier, TexTalkNodeType.Identifier, false)
            val text =
                if (rawText != null) {
                    rawText
                } else {
                    addError("Expected an identifier in a named group")
                    TextTexTalkNode(
                        TexTalkNodeType.Identifier, TexTalkTokenType.Invalid, "INVALID", false)
                }

            val rawGroup = group(TexTalkNodeType.CurlyGroup)
            val first =
                if (rawGroup != null) {
                    rawGroup
                } else {
                    addError("Expected a group in a named group")
                    GroupTexTalkNode(
                        TexTalkNodeType.CurlyGroup, ParametersTexTalkNode(emptyList()), false)
                }
            val groups = mutableListOf<GroupTexTalkNode>()
            groups.add(first)
            while (true) {
                val grp = group(TexTalkNodeType.CurlyGroup)
                grp ?: break
                groups.add(grp)
            }
            return NamedGroupTexTalkNode(text, groups)
        }

        private fun text(
            tokenType: TexTalkTokenType, nodeType: TexTalkNodeType, canBeVarArg: Boolean
        ): TextTexTalkNode? {
            if (!has(tokenType)) {
                return null
            }

            val textToken = next()
            val nextIsDotDotDot =
                has(TexTalkTokenType.DotDotDot) &&
                    !hasHas(TexTalkTokenType.DotDotDot, TexTalkTokenType.Identifier)

            var isVarArg = false
            if (canBeVarArg) {
                if (nextIsDotDotDot) {
                    isVarArg = true
                }
            } else {
                if (nextIsDotDotDot) {
                    addError("Unexpected ... suffix", textToken)
                }
            }

            if (nextIsDotDotDot) {
                next() // move past the ...
            }

            return TextTexTalkNode(
                type = nodeType, tokenType = tokenType, text = textToken.text, isVarArg = isVarArg)
        }

        private fun expression(terminators: Set<TexTalkTokenType>?): ExpressionTexTalkNode? {
            val nodes = mutableListOf<TexTalkNode>()
            while (hasNext() &&
                (terminators == null || !terminators.contains(texTalkLexer.peek().tokenType))) {
                val child =
                    command()
                        ?: group(TexTalkNodeType.ParenGroup) ?: group(TexTalkNodeType.CurlyGroup)
                            ?: text(TexTalkTokenType.Is, TexTalkNodeType.Is, false)
                            ?: text(TexTalkTokenType.Identifier, TexTalkNodeType.Identifier, true)
                            ?: text(TexTalkTokenType.Operator, TexTalkNodeType.Operator, false)
                            ?: text(TexTalkTokenType.Comma, TexTalkNodeType.Comma, false)
                            ?: text(TexTalkTokenType.Caret, TexTalkNodeType.Operator, false)
                            ?: text(TexTalkTokenType.Underscore, TexTalkNodeType.Operator, false)
                            ?: text(
                            TexTalkTokenType.ColonEquals, TexTalkNodeType.ColonEquals, false)
                            ?: text(
                            TexTalkTokenType.ColonColonEquals,
                            TexTalkNodeType.ColonColonEquals,
                            false)
                            ?: text(TexTalkTokenType.DotDotDot, TexTalkNodeType.Operator, false)

                if (child == null) {
                    val peek = texTalkLexer.peek()
                    addError("Unexpected token ${peek.text}", peek)
                    next() // move past the unrecognized token
                } else {
                    nodes.add(child)
                }
            }

            return if (nodes.isEmpty()) {
                null
            } else ExpressionTexTalkNode(nodes)
        }

        private fun expect(tokenType: TexTalkTokenType): TexTalkToken {
            if (has(tokenType)) {
                return next() // absorb the token
            } else {
                val message =
                    if (hasNext()) {
                        "Expected a token of type '$tokenType' but found type " +
                            "'${texTalkLexer.peek().type}' for text '${texTalkLexer.peek().text}' " +
                            "(Line: ${texTalkLexer.peek().row + 1}, Column: ${texTalkLexer.peek().column + 1})"
                    } else {
                        "Expected a token of type $tokenType but found the end of input"
                    }
                addError(message)
                return INVALID
            }
        }

        private fun hasNext() = texTalkLexer.hasNext()

        private fun next() = texTalkLexer.next()

        private fun has(tokenType: TexTalkTokenType) =
            hasNext() && texTalkLexer.peek().tokenType === tokenType

        private fun hasHas(tokenType1: TexTalkTokenType, tokenType2: TexTalkTokenType) =
            has(tokenType1) &&
                texTalkLexer.hasNextNext() &&
                texTalkLexer.peekPeek().tokenType == tokenType2

        private fun addError(message: String, token: TexTalkToken) =
            addError(message, token.row, token.column)

        private fun addError(message: String) = addError(message, -1, -1)

        private fun addError(message: String, row: Int, column: Int) =
            errors.add(ParseError(message, row, column))
    }
}
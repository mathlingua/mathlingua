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

interface TexTalkParser {
    fun parse(texTalkLexer: TexTalkLexer): TexTalkParseResult
}

data class TexTalkParseResult(
    val root: ExpressionTexTalkNode,
    val errors: List<ParseError>
)

fun newTexTalkParser(): TexTalkParser {
    return TexTalkParserImpl()
}

fun <T : TexTalkNode> populateParents(node: T): T {
    node.forEach {
        it.parent = node
        populateParents(it)
    }
    return node
}

private val INVALID = TexTalkToken(null, "INVALID", TexTalkTokenType.Invalid, -1, -1)

class TexTalkParserImpl : TexTalkParser {

    override fun parse(texTalkLexer: TexTalkLexer): TexTalkParseResult {
        val worker = ParserWorker(texTalkLexer)
        val root = worker.parse()
        val errors = worker.getErrors()
        return TexTalkParseResult(root, errors)
    }

    private class ParserWorker(private val texTalkLexer: TexTalkLexer) {
        private val errors = mutableListOf<ParseError>()

        fun getErrors(): List<ParseError> {
            return errors
        }

        fun parse(): ExpressionTexTalkNode {
            val exp = expression(null) ?: ExpressionTexTalkNode(null, emptyList())
            val result = resolveColonEqualsNode(resolveIsNode(exp)) as ExpressionTexTalkNode
            populateParents(result)
            return result
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
            return ExpressionTexTalkNode(null, listOf(IsTexTalkNode(null, lhs, rhs)))
        }

        private fun resolveColonEqualsNode(texTalkNode: TexTalkNode): TexTalkNode {
            if (texTalkNode !is ExpressionTexTalkNode) {
                return texTalkNode
            }

            var colonEqualsIndex = -1
            for (i in texTalkNode.children.indices) {
                val child = texTalkNode.children[i]
                if (child is TextTexTalkNode && child.type == TexTalkNodeType.ColonEquals) {
                    if (colonEqualsIndex < 0) {
                        colonEqualsIndex = i
                    } else {
                        addError("A statement can only contain one ':='")
                    }
                }
            }

            if (colonEqualsIndex < 0) {
                return texTalkNode
            }

            val lhs = parameters(texTalkNode.children, 0, colonEqualsIndex)
            val rhs = parameters(texTalkNode.children, colonEqualsIndex + 1, texTalkNode.children.size)
            return ExpressionTexTalkNode(null, listOf(ColonEqualsTexTalkNode(null, lhs, rhs)))
        }

        private fun parameters(texTalkNodes: List<TexTalkNode>, startInc: Int, endEx: Int): ParametersTexTalkNode {
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
                parts.add(ExpressionTexTalkNode(null, items))
            }
            return ParametersTexTalkNode(null, parts)
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
                } else {
                    break
                }
            }

            if (parts.isEmpty()) {
                addError("Expected at least one command part", backSlash)
            }

            return Command(null, parts)
        }

        private fun commandPart(): CommandPart? {
            if (!has(TexTalkTokenType.Identifier)) {
                return null
            }

            val name = text(TexTalkTokenType.Identifier, TexTalkNodeType.Identifier)
            val square = group(TexTalkNodeType.SquareGroup)
            val subSup = subSup()
            val groups = mutableListOf<GroupTexTalkNode>()

            var startGroup: GroupTexTalkNode? = null
            val paren = group(TexTalkNodeType.ParenGroup)
            if (paren != null) {
                startGroup = paren
            }

            if (startGroup == null) {
                val curly = group(TexTalkNodeType.CurlyGroup)
                if (curly != null) {
                    startGroup = curly
                }
            }

            if (startGroup != null) {
                groups.add(startGroup)

                while (hasNext()) {
                    val grp = group(startGroup.type)
                    grp ?: break
                    groups.add(grp)
                }
            }

            val namedGroups = mutableListOf<NamedGroupTexTalkNode>()
            if (has(TexTalkTokenType.Colon)) {
                expect(TexTalkTokenType.Colon) // absorb the colon
                while (hasNext()) {
                    val namedGrp = namedGroup()
                    namedGrp ?: break
                    namedGroups.add(namedGrp)
                }
            }

            return CommandPart(
                null,
                name!!,
                square,
                subSup,
                groups,
                namedGroups
            )
        }

        private fun subSup(): SubSupTexTalkNode? {
            val sub = sub()
            val sup = sup()
            return if (sub == null && sup == null) {
                null
            } else SubSupTexTalkNode(null, sub, sup)
        }

        private fun sub(): GroupTexTalkNode? {
            if (!has(TexTalkTokenType.Underscore)) {
                return null
            }

            val (_, _, _, row, column) = expect(TexTalkTokenType.Underscore)

            var grp: GroupTexTalkNode? = null
            val curly = group(TexTalkNodeType.CurlyGroup)
            if (curly != null) {
                grp = curly
            }

            if (grp == null) {
                val paren = group(TexTalkNodeType.ParenGroup)
                if (paren != null) {
                    grp = paren
                }
            }

            if (grp == null) {
                addError("Expected a value with an underscore", row, column)
                grp = GroupTexTalkNode(null, TexTalkNodeType.CurlyGroup, ParametersTexTalkNode(null, emptyList()))
            }

            return grp
        }

        private fun sup(): GroupTexTalkNode? {
            if (!has(TexTalkTokenType.Caret)) {
                return null
            }

            val (_, _, _, row, column) = expect(TexTalkTokenType.Caret)
            var grp: GroupTexTalkNode? = null

            val curly = group(TexTalkNodeType.CurlyGroup)
            if (curly != null) {
                grp = curly
            }

            if (grp == null) {
                val paren = group(TexTalkNodeType.ParenGroup)
                if (paren != null) {
                    grp = paren
                }
            }

            if (grp == null) {
                addError("Expected a value with a caret", row, column)
                grp = GroupTexTalkNode(null, TexTalkNodeType.CurlyGroup, ParametersTexTalkNode(null, emptyList()))
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
            return GroupTexTalkNode(null, nodeType, ParametersTexTalkNode(null, expressions))
        }

        private fun namedGroup(): NamedGroupTexTalkNode? {
            if (!hasHas(TexTalkTokenType.Identifier, TexTalkTokenType.LCurly)) {
                return null
            }

            val rawText = text(TexTalkTokenType.Identifier, TexTalkNodeType.Identifier)
            val text = if (rawText != null) {
                rawText
            } else {
                addError("Expected an identifier in a named group")
                TextTexTalkNode(null, TexTalkNodeType.Identifier, "INVALID")
            }

            val rawGroup = group(TexTalkNodeType.CurlyGroup)
            val group = if (rawGroup != null) {
                rawGroup
            } else {
                addError("Expected a group in a named group")
                GroupTexTalkNode(null, TexTalkNodeType.CurlyGroup, ParametersTexTalkNode(null, emptyList()))
            }
            return NamedGroupTexTalkNode(null, text, group)
        }

        private fun text(tokenType: TexTalkTokenType, nodeType: TexTalkNodeType): TextTexTalkNode? {
            return if (!has(tokenType)) {
                null
            } else TextTexTalkNode(null, nodeType, next().text)
        }

        private fun expression(terminators: Set<TexTalkTokenType>?): ExpressionTexTalkNode? {
            val nodes = mutableListOf<TexTalkNode>()
            while (hasNext() && (terminators == null ||
                    !terminators.contains(texTalkLexer.peek().tokenType))
            ) {
                val child = command()
                    ?: group(TexTalkNodeType.ParenGroup)
                    ?: group(TexTalkNodeType.CurlyGroup)
                    ?: text(TexTalkTokenType.Is, TexTalkNodeType.Is)
                    ?: text(TexTalkTokenType.Identifier, TexTalkNodeType.Identifier)
                    ?: text(TexTalkTokenType.Operator, TexTalkNodeType.Operator)
                    ?: text(TexTalkTokenType.Comma, TexTalkNodeType.Comma)
                    ?: text(TexTalkTokenType.Caret, TexTalkNodeType.Operator)
                    ?: text(TexTalkTokenType.Underscore, TexTalkNodeType.Operator)
                    ?: text(TexTalkTokenType.ColonEquals, TexTalkNodeType.ColonEquals)

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
            } else ExpressionTexTalkNode(null, nodes)
        }

        private fun expect(tokenType: TexTalkTokenType): TexTalkToken {
            if (has(tokenType)) {
                return next() // absorb the token
            } else {
                val message = if (hasNext()) {
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

        private fun has(tokenType: TexTalkTokenType): Boolean {
            return hasNext() && texTalkLexer.peek().tokenType === tokenType
        }

        private fun hasHas(tokenType1: TexTalkTokenType, tokenType2: TexTalkTokenType): Boolean {
            return has(tokenType1) && texTalkLexer.hasNextNext() && texTalkLexer.peekPeek().tokenType == tokenType2
        }

        private fun addError(message: String, token: TexTalkToken) {
            addError(message, token.row, token.column)
        }

        private fun addError(message: String) {
            addError(message, -1, -1)
        }

        private fun addError(message: String, row: Int, column: Int) {
            errors.add(ParseError(message, row, column))
        }
    }
}

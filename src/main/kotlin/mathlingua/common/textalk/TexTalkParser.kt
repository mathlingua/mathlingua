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
    val root: ExpressionNode,
    val errors: List<ParseError>
)

fun newTexTalkParser(): TexTalkParser {
    return TexTalkParserImpl()
}

private val INVALID = TexTalkToken("INVALID", TexTalkTokenType.Invalid, -1, -1)

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

        fun parse(): ExpressionNode {
            val exp = expression(null) ?: ExpressionNode(emptyList())
            return resolveColonEqualsNode(resolveIsNode(exp)) as ExpressionNode
        }

        private fun resolveIsNode(node: Node): Node {
            if (node !is ExpressionNode) {
                return node
            }

            var isIndex = -1
            for (i in node.children.indices) {
                val child = node.children[i]
                if (child is TextNode && child.type == NodeType.Is) {
                    if (isIndex < 0) {
                        isIndex = i
                    } else {
                        addError("A statement can only contain one 'is' statement")
                    }
                }
            }

            if (isIndex < 0) {
                return node
            }

            val lhs = parameters(node.children, 0, isIndex)
            val rhs = parameters(node.children, isIndex + 1, node.children.size)
            return ExpressionNode(listOf(IsNode(lhs, rhs)))
        }

        private fun resolveColonEqualsNode(node: Node): Node {
            if (node !is ExpressionNode) {
                return node
            }

            var colonEqualsIndex = -1
            for (i in node.children.indices) {
                val child = node.children[i]
                if (child is TextNode && child.type == NodeType.ColonEquals) {
                    if (colonEqualsIndex < 0) {
                        colonEqualsIndex = i
                    } else {
                        addError("A statement can only contain one ':='")
                    }
                }
            }

            if (colonEqualsIndex < 0) {
                return node
            }

            val lhs = parameters(node.children, 0, colonEqualsIndex)
            val rhs = parameters(node.children, colonEqualsIndex + 1, node.children.size)
            return ExpressionNode(listOf(ColonEqualsNode(lhs, rhs)))
        }

        private fun parameters(nodes: List<Node>, startInc: Int, endEx: Int): ParametersNode {
            val parts = mutableListOf<ExpressionNode>()
            var i = startInc
            while (i < endEx) {
                val items = mutableListOf<Node>()
                while (i < endEx && nodes[i].type != NodeType.Comma) {
                    items.add(resolveIsNode(nodes[i++]))
                }
                if (i < endEx && nodes[i].type !== NodeType.Comma) {
                    addError("Expected a Comma but found ${nodes[i].type}")
                } else {
                    i++ // move past the comma
                }
                parts.add(ExpressionNode(items))
            }
            return ParametersNode(parts)
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

            return Command(parts)
        }

        private fun commandPart(): CommandPart? {
            if (!has(TexTalkTokenType.Identifier)) {
                return null
            }

            val name = text(TexTalkTokenType.Identifier, NodeType.Identifier)
            val square = group(NodeType.SquareGroup)
            val subSup = subSup()
            val groups = mutableListOf<GroupNode>()

            var startGroup: GroupNode? = null
            val paren = group(NodeType.ParenGroup)
            if (paren != null) {
                startGroup = paren
            }

            if (startGroup == null) {
                val curly = group(NodeType.CurlyGroup)
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

            val namedGroups = mutableListOf<NamedGroupNode>()
            if (has(TexTalkTokenType.Colon)) {
                expect(TexTalkTokenType.Colon) // absorb the colon
                while (hasNext()) {
                    val namedGrp = namedGroup()
                    namedGrp ?: break
                    namedGroups.add(namedGrp)
                }
            }

            return CommandPart(
                name!!,
                square,
                subSup,
                groups,
                namedGroups
            )
        }

        private fun subSup(): SubSupNode? {
            val sub = sub()
            val sup = sup()
            return if (sub == null && sup == null) {
                null
            } else SubSupNode(sub, sup)
        }

        private fun sub(): GroupNode? {
            if (!has(TexTalkTokenType.Underscore)) {
                return null
            }

            val (_, _, row, column) = expect(TexTalkTokenType.Underscore)

            var grp: GroupNode? = null
            val curly = group(NodeType.CurlyGroup)
            if (curly != null) {
                grp = curly
            }

            if (grp == null) {
                val paren = group(NodeType.ParenGroup)
                if (paren != null) {
                    grp = paren
                }
            }

            if (grp == null) {
                addError("Expected a value with an underscore", row, column)
                grp = GroupNode(NodeType.CurlyGroup, ParametersNode(emptyList()))
            }

            return grp
        }

        private fun sup(): GroupNode? {
            if (!has(TexTalkTokenType.Caret)) {
                return null
            }

            val (_, _, row, column) = expect(TexTalkTokenType.Caret)
            var grp: GroupNode? = null

            val curly = group(NodeType.CurlyGroup)
            if (curly != null) {
                grp = curly
            }

            if (grp == null) {
                val paren = group(NodeType.ParenGroup)
                if (paren != null) {
                    grp = paren
                }
            }

            if (grp == null) {
                addError("Expected a value with a caret", row, column)
                grp = GroupNode(NodeType.CurlyGroup, ParametersNode(emptyList()))
            }

            return grp
        }

        private fun group(nodeType: NodeType): GroupNode? {
            val startType: TexTalkTokenType
            val endType: TexTalkTokenType
            when (nodeType) {
                NodeType.ParenGroup -> {
                    startType = TexTalkTokenType.LParen
                    endType = TexTalkTokenType.RParen
                }
                NodeType.SquareGroup -> {
                    startType = TexTalkTokenType.LSquare
                    endType = TexTalkTokenType.RSquare
                }
                NodeType.CurlyGroup -> {
                    startType = TexTalkTokenType.LCurly
                    endType = TexTalkTokenType.RCurly
                }
                else -> throw RuntimeException("Unrecognized group type $nodeType")
            }

            if (!has(startType)) {
                return null
            }

            val expressions = mutableListOf<ExpressionNode>()
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
            return GroupNode(nodeType, ParametersNode(expressions))
        }

        private fun namedGroup(): NamedGroupNode? {
            if (!hasHas(TexTalkTokenType.Identifier, TexTalkTokenType.LCurly)) {
                return null
            }

            val rawText = text(TexTalkTokenType.Identifier, NodeType.Identifier)
            val text = if (rawText != null) {
                rawText
            } else {
                addError("Expected an identifier in a named group")
                TextNode(NodeType.Identifier, "INVALID")
            }

            val rawGroup = group(NodeType.CurlyGroup)
            val group = if (rawGroup != null) {
                rawGroup
            } else {
                addError("Expected a group in a named group")
                GroupNode(NodeType.CurlyGroup, ParametersNode(emptyList()))
            }
            return NamedGroupNode(text, group)
        }

        private fun text(tokenType: TexTalkTokenType, nodeType: NodeType): TextNode? {
            return if (!has(tokenType)) {
                null
            } else TextNode(nodeType, next().text)
        }

        private fun expression(terminators: Set<TexTalkTokenType>?): ExpressionNode? {
            val nodes = mutableListOf<Node>()
            while (hasNext() && (terminators == null ||
                    !terminators.contains(texTalkLexer.peek().tokenType))
            ) {
                val child = command()
                    ?: group(NodeType.ParenGroup)
                    ?: group(NodeType.CurlyGroup)
                    ?: text(TexTalkTokenType.Is, NodeType.Is)
                    ?: text(TexTalkTokenType.Identifier, NodeType.Identifier)
                    ?: text(TexTalkTokenType.Operator, NodeType.Operator)
                    ?: text(TexTalkTokenType.Comma, NodeType.Comma)
                    ?: text(TexTalkTokenType.Caret, NodeType.Operator)
                    ?: text(TexTalkTokenType.Underscore, NodeType.Operator)
                    ?: text(TexTalkTokenType.ColonEquals, NodeType.ColonEquals)

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
            } else ExpressionNode(nodes)
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

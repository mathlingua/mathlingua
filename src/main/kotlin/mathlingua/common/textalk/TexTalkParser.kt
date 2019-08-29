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

class TexTalkParserImpl : TexTalkParser {

    override fun parse(texTalkLexer: TexTalkLexer): TexTalkParseResult {
        val worker = ParserWorker(texTalkLexer)
        val root = worker.parse()
        val errors = worker.getErrors()
        return TexTalkParseResult(root, errors)
    }

    private class ParserWorker(private val texTalkLexer: TexTalkLexer) {
        private val errors: MutableList<ParseError>

        init {
            this.errors = ArrayList()
        }

        fun getErrors(): List<ParseError> {
            return this.errors
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
            for (i in 0 until node.children.size) {
                val child = node.children[i]
                if (child is TextNode && child.type == NodeType.Is) {
                    if (isIndex < 0) {
                        isIndex = i
                    } else {
                        errors.add(
                            ParseError(
                                "A statement can only contain one 'is' statement",
                                -1, -1
                            )
                        )
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
            for (i in 0 until node.children.size) {
                val child = node.children[i]
                if (child is TextNode && child.type == NodeType.ColonEquals) {
                    if (colonEqualsIndex < 0) {
                        colonEqualsIndex = i
                    } else {
                        errors.add(
                            ParseError(
                                "A statement can only contain one ':='",
                                -1, -1
                            )
                        )
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
            val parts = ArrayList<ExpressionNode>()
            var i = startInc
            while (i < endEx) {
                val items = ArrayList<Node>()
                while (i < endEx && nodes[i].type != NodeType.Comma) {
                    items.add(resolveIsNode(nodes[i++]))
                }
                if (i < endEx && nodes[i].type !== NodeType.Comma) {
                    throw RuntimeException("Expected a Comma but found ${nodes[i].type}")
                }
                i++ // move past the comma
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
            while (this.texTalkLexer.hasNext()) {
                val part = commandPart()
                if (part == null) {
                    errors.add(
                        ParseError(
                            "Missing a command part",
                            backSlash.row, backSlash.column
                        )
                    )
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
                errors.add(
                    ParseError(
                        "Expected at least one command part",
                        backSlash.row, backSlash.column
                    )
                )
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
            val groups = ArrayList<GroupNode>()

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

                while (this.texTalkLexer.hasNext()) {
                    val grp = group(startGroup.type)
                    if (grp == null) {
                        break
                    }
                    groups.add(grp)
                }
            }

            val namedGroups = ArrayList<NamedGroupNode>()
            if (has(TexTalkTokenType.Colon)) {
                expect(TexTalkTokenType.Colon) // absorb the colon
                while (this.texTalkLexer.hasNext()) {
                    val namedGrp = namedGroup()
                    if (namedGrp == null) {
                        break
                    }
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
                this.errors.add(
                    ParseError(
                        "Expected a value with an underscore", row, column
                    )
                )
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
                this.errors.add(
                    ParseError(
                        "Expected a value with a caret", row, column
                    )
                )
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

            val expressions = ArrayList<ExpressionNode>()
            expect(startType)

            val terminators = HashSet<TexTalkTokenType>()
            terminators.add(TexTalkTokenType.Comma)
            terminators.add(endType)

            val firstExp = expression(terminators)
            if (firstExp != null) {
                expressions.add(firstExp)
            }

            while (has(TexTalkTokenType.Comma)) {
                this.texTalkLexer.next() // absorb the comma
                val exp = expression(terminators)
                exp ?: break
                expressions.add(exp)
            }

            expect(endType)
            return GroupNode(nodeType, ParametersNode(expressions))
        }

        private fun namedGroup(): NamedGroupNode? {
            val isNamedGroup = (this.texTalkLexer.hasNext() &&
                this.texTalkLexer.hasNextNext() &&
                this.texTalkLexer.peek().tokenType === TexTalkTokenType.Identifier &&
                this.texTalkLexer.peekPeek().tokenType === TexTalkTokenType.LCurly)
            if (!isNamedGroup) {
                return null
            }

            val rawText = text(TexTalkTokenType.Identifier, NodeType.Identifier)
            val text: TextNode
            if (rawText != null) {
                text = rawText
            } else {
                this.errors.add(
                    ParseError("Expected an identifier in a named group", -1, -1)
                )
                text = TextNode(NodeType.Identifier, "INVALID")
            }

            val rawGroup = group(NodeType.CurlyGroup)
            val group: GroupNode
            if (rawGroup != null) {
                group = rawGroup
            } else {
                this.errors.add(ParseError("Expected a group in a named group", -1, -1))
                group = GroupNode(NodeType.CurlyGroup, ParametersNode(emptyList()))
            }
            return NamedGroupNode(text, group)
        }

        private fun text(tokenType: TexTalkTokenType, nodeType: NodeType): TextNode? {
            return if (!has(tokenType)) {
                null
            } else TextNode(nodeType, this.texTalkLexer.next().text)
        }

        private fun expression(terminators: Set<TexTalkTokenType>?): ExpressionNode? {
            val nodes = ArrayList<Node>()
            while (this.texTalkLexer.hasNext() && (terminators == null ||
                    !terminators.contains(this.texTalkLexer.peek().tokenType))
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
                    val peek = this.texTalkLexer.peek()
                    this.errors.add(
                        ParseError(
                            "Unexpected token ${peek.text}", peek.row, peek.column
                        )
                    )
                    this.texTalkLexer.next() // move past the unrecognized token
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
                return this.texTalkLexer.next() // absorb the token
            } else {
                val message = if (this.texTalkLexer.hasNext()) {
                    "Expected a token of type '" +
                        tokenType +
                        "' but " +
                        "found type '" +
                        this.texTalkLexer.peek().type +
                        "' for text '" +
                        this.texTalkLexer.peek().text + "' (Line: " +
                        (this.texTalkLexer.peek().row + 1) +
                        ", Column: " +
                        (this.texTalkLexer.peek().column + 1) +
                        ")"
                } else {
                    "Expected a token of type $tokenType but found the end of input"
                }
                this.errors.add(ParseError(message, -1, -1))
                return TexTalkToken("INVALID", TexTalkTokenType.Invalid, -1, -1)
            }
        }

        private fun has(tokenType: TexTalkTokenType): Boolean {
            return this.texTalkLexer.hasNext() && this.texTalkLexer.peek().tokenType === tokenType
        }
    }
}

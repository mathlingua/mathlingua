/*
 * Copyright 2022 Dominic Kramer
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

package mathlingua.lib.frontend.textalk

import java.util.LinkedList
import mathlingua.lib.frontend.ast.TexTalkToken
import mathlingua.lib.frontend.ast.TexTalkTokenType

internal sealed interface TreeNode {
    fun toString(indent: String): String
    fun row(): Int?
    fun column(): Int?
    fun startsWith(type: TexTalkTokenType): Boolean
}

internal data class AtomTreeNode(var token: TexTalkToken) : TreeNode {
    override fun toString(indent: String) = "$indent${token.text}"
    override fun row() = token.row
    override fun column() = token.column
    override fun startsWith(type: TexTalkTokenType) = token.type == type
}

internal data class ParenTreeNode(
    var prefix: TexTalkToken?, var content: TreeNode?, var suffix: TexTalkToken?
) : TreeNode {
    override fun toString(indent: String) =
        "<$indent${prefix?.text} ${content?.toString("")} ${suffix?.text}>"
    override fun row() = prefix?.row
    override fun column() = prefix?.column
    override fun startsWith(type: TexTalkTokenType) = prefix?.type == type
}

internal data class ListTreeNode(val nodes: MutableList<TreeNode>) : TreeNode {
    override fun toString(indent: String) = "$indent${nodes.map { it.toString("") }}"
    override fun row() = nodes.firstOrNull()?.row()
    override fun column() = nodes.firstOrNull()?.column()
    override fun startsWith(type: TexTalkTokenType) = nodes.firstOrNull()?.startsWith(type) ?: false
}

internal data class SplitTreeNode(var lhs: TreeNode?, var center: TreeNode?, var rhs: TreeNode?) :
    TreeNode {
    override fun toString(indent: String): String {
        val builder = StringBuilder()
        builder.append("\n")
        builder.append(center?.toString(indent))
        builder.append("\n")
        builder.append(lhs?.toString("$indent  "))
        builder.append("\n")
        builder.append("$indent  _____\n")
        builder.append(rhs?.toString("$indent  "))
        builder.append("\n")
        return builder.toString()
    }
    override fun row() = lhs?.row()
    override fun column() = lhs?.column()
    override fun startsWith(type: TexTalkTokenType) = lhs?.startsWith(type) ?: false
}

internal data class UnitTreeNode(val nodes: MutableList<TreeNode>, val terminator: TexTalkToken?) :
    TreeNode {
    override fun toString(indent: String) =
        "$indent${nodes.map { it.toString("") }}${terminator?.text ?: ""}"
    override fun row() = nodes.firstOrNull()?.row()
    override fun column() = nodes.firstOrNull()?.column()
    override fun startsWith(type: TexTalkTokenType) = nodes.firstOrNull()?.startsWith(type) ?: false
}

internal fun lexerToTree(lexer: TexTalkLexer) =
    groupByParens(lexer)
        ?.splitByNodesMatching(
            setOf(TexTalkTokenType.Is, TexTalkTokenType.In, TexTalkTokenType.NotIn))
        ?.splitByNodesMatching(setOf(TexTalkTokenType.Equals, TexTalkTokenType.NotEqual))
        ?.splitByNodesMatching(setOf(TexTalkTokenType.As))
        ?.splitForUnitNodes()

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

private fun listToTreeNode(nodes: MutableList<TreeNode>): TreeNode? =
    if (nodes.isEmpty()) {
        null
    } else if (nodes.size == 1) {
        nodes.first()
    } else {
        ListTreeNode(nodes = LinkedList(nodes))
    }

private fun TreeNode.splitByNodesMatching(splitTypes: Set<TexTalkTokenType>): TreeNode =
    when (this) {
        is AtomTreeNode -> this
        is ParenTreeNode -> {
            ParenTreeNode(
                prefix = this.prefix,
                content = this.content?.splitByNodesMatching(splitTypes),
                suffix = this.suffix)
        }
        is UnitTreeNode ->
            throw Exception("Unit nodes shouldn't exist when splitting by $splitTypes")
        is ListTreeNode -> {
            val index =
                this.nodes.indexOfFirst { it is AtomTreeNode && it.token.type in splitTypes }
            if (index == -1) {
                this
            } else {
                val nodeList = this.nodes.toMutableList()
                SplitTreeNode(
                        lhs =
                            listToTreeNode(nodeList.subList(0, index))
                                ?.splitByNodesMatching(splitTypes),
                        center = nodeList[index].splitByNodesMatching(splitTypes),
                        rhs = listToTreeNode(nodeList.subList(index + 1, this.nodes.size)))
                    .splitByNodesMatching(splitTypes)
            }
        }
        is SplitTreeNode -> {
            SplitTreeNode(
                lhs = this.lhs?.splitByNodesMatching(splitTypes),
                center = this.center?.splitByNodesMatching(splitTypes),
                rhs = this.rhs?.splitByNodesMatching(splitTypes))
        }
    }

private fun groupByParens(lexer: TexTalkLexer): TreeNode? {
    val nodes = mutableListOf<TreeNode>()
    while (lexer.hasNext() && !lexer.peek().type.isRightParenType()) {
        val next = lexer.next()
        if (next.type.isLeftParenType()) {
            val endType = next.type.getExpectedEndType()
            val content = groupByParens(lexer)
            val suffix =
                if (lexer.hasNext() && lexer.peek().type == endType) {
                    lexer.next()
                } else {
                    null
                }
            nodes.add(ParenTreeNode(prefix = next, content = content, suffix = suffix))
        } else {
            nodes.add(AtomTreeNode(token = next))
        }
    }
    return listToTreeNode(nodes)
}

private fun TreeNode.splitForUnitNodes(): TreeNode =
    when (this) {
        is SplitTreeNode -> {
            SplitTreeNode(
                lhs = lhs?.splitForUnitNodes(),
                center = center?.splitForUnitNodes(),
                rhs = rhs?.splitForUnitNodes())
        }
        is UnitTreeNode ->
            throw Exception("Unit nodes shouldn't be encountered when splitting for Unit nodes")
        is AtomTreeNode -> this
        is ParenTreeNode -> {
            ParenTreeNode(
                prefix = this.prefix,
                content = this.content?.splitForUnitNodes(),
                suffix = this.suffix)
        }
        is ListTreeNode -> {
            val newNodes = mutableListOf<TreeNode>()
            var i = 0
            while (i < this.nodes.size) {
                val subNodes = mutableListOf<TreeNode>()
                while (i < this.nodes.size && !this.nodes.has(TexTalkTokenType.Comma)) {
                    subNodes.add(this.nodes[i++])
                }
                val commaNode =
                    if (this.nodes.has(TexTalkTokenType.Comma)) {
                        this.nodes[i++] // move past the comma
                    } else {
                        null
                    }
                val comma =
                    if (commaNode is AtomTreeNode) {
                        commaNode.token
                    } else {
                        null
                    }
                if (subNodes.isNotEmpty()) {
                    newNodes.add(UnitTreeNode(nodes = subNodes, terminator = comma))
                }
            }
            if (newNodes.size == 1) {
                newNodes[0]
            } else {
                ListTreeNode(newNodes)
            }
        }
    }

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

private fun TexTalkTokenType.isLeftParenType() =
    this == TexTalkTokenType.LParen ||
        this == TexTalkTokenType.LCurly ||
        this == TexTalkTokenType.LSquare ||
        this == TexTalkTokenType.LSquareColon

private fun TexTalkTokenType.isRightParenType() =
    this == TexTalkTokenType.RParen ||
        this == TexTalkTokenType.RCurly ||
        this == TexTalkTokenType.RSquare ||
        this == TexTalkTokenType.ColonRSquare

private fun TexTalkTokenType.getExpectedEndType() =
    if (this == TexTalkTokenType.LParen) {
        TexTalkTokenType.RParen
    } else if (this == TexTalkTokenType.LCurly) {
        TexTalkTokenType.RCurly
    } else if (this == TexTalkTokenType.LSquare) {
        TexTalkTokenType.RSquare
    } else if (this == TexTalkTokenType.LSquareColon) {
        TexTalkTokenType.ColonRSquare
    } else {
        null
    }

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

import java.lang.RuntimeException
import mathlingua.lib.frontend.Diagnostic
import mathlingua.lib.frontend.ast.TexTalkNode
import mathlingua.lib.frontend.ast.TexTalkToken
import mathlingua.lib.frontend.ast.TexTalkTokenType

internal interface TexTalkParser {
    fun parse(): TexTalkNode
    fun diagnostics(): List<Diagnostic>
}

internal fun newTexTalkParser(lexer: TexTalkLexer): TexTalkParser = null!!

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

private sealed interface TreeNode {
    fun toString(indent: String): String
}

private data class AtomTreeNode(var token: TexTalkToken) : TreeNode {
    override fun toString(indent: String) = "$indent${token.text}"
}

private data class ListTreeNode(val nodes: List<TreeNode>) : TreeNode {
    override fun toString(indent: String) = "$indent${nodes.map { it.toString("") }}"
}

private data class SplitTreeNode(var lhs: TreeNode?, var center: TreeNode, var rhs: TreeNode?) :
    TreeNode {
    override fun toString(indent: String): String {
        val builder = StringBuilder()
        builder.append("\n")
        builder.append(center.toString(indent))
        builder.append("\n")
        builder.append(lhs?.toString("$indent  "))
        builder.append("\n")
        builder.append("$indent  _____\n")
        builder.append(rhs?.toString("$indent  "))
        builder.append("\n")
        return builder.toString()
    }
}

private fun listToTreeNode(nodes: List<TreeNode>): TreeNode? =
    if (nodes.isEmpty()) {
        null
    } else if (nodes.size == 1) {
        nodes.first()
    } else {
        ListTreeNode(nodes = nodes)
    }

private fun splitByIsInOrNotInNodes(lexer: TexTalkLexer): TreeNode? {
    val lhsNodes = mutableListOf<TreeNode>()
    while (lexer.hasNext() &&
        lexer.peek().type != TexTalkTokenType.Is &&
        lexer.peek().type != TexTalkTokenType.In &&
        lexer.peek().type != TexTalkTokenType.NotIn) {
        lhsNodes.add(AtomTreeNode(token = lexer.next()))
    }
    val lhs = listToTreeNode(lhsNodes)
    if (!lexer.hasNext()) {
        return lhs
    }
    val peek = lexer.peek()
    if (peek.type != TexTalkTokenType.Is &&
        peek.type != TexTalkTokenType.In &&
        peek.type != TexTalkTokenType.NotIn) {
        throw RuntimeException("Expected an 'is', 'in', or 'notin' node but found ${lexer.peek()}")
    }
    val isNode = lexer.next()
    val rhs = splitByIsInOrNotInNodes(lexer)
    return SplitTreeNode(lhs = lhs, center = AtomTreeNode(token = isNode), rhs = rhs)
}

private fun TreeNode?.splitByEqualsOrNotEqualsNodes(): TreeNode? {
    if (this == null) {
        return null
    }

    return when (this) {
        is AtomTreeNode -> this
        is ListTreeNode -> {
            val index =
                this.nodes.indexOfFirst {
                    it is AtomTreeNode &&
                        (it.token.type == TexTalkTokenType.Equals ||
                            it.token.type == TexTalkTokenType.NotEqual)
                }
            if (index == -1) {
                this
            } else {
                SplitTreeNode(
                    lhs = listToTreeNode(this.nodes.subList(0, index)),
                    center = this.nodes[index],
                    rhs = listToTreeNode(this.nodes.subList(index + 1, this.nodes.size)))
            }
        }
        is SplitTreeNode -> {
            SplitTreeNode(
                lhs = this.lhs.splitByEqualsOrNotEqualsNodes(),
                center = this.center,
                rhs = this.rhs.splitByEqualsOrNotEqualsNodes())
        }
    }
}

private fun TreeNode?.splitByAsNodes(): TreeNode? {
    if (this == null) {
        return null
    }

    return when (this) {
        is AtomTreeNode -> this
        is ListTreeNode -> {
            val index =
                this.nodes.indexOfFirst {
                    it is AtomTreeNode && it.token.type == TexTalkTokenType.As
                }
            if (index == -1) {
                this
            } else {
                SplitTreeNode(
                    lhs = listToTreeNode(this.nodes.subList(0, index)),
                    center = this.nodes[index],
                    rhs = listToTreeNode(this.nodes.subList(index + 1, this.nodes.size)))
            }
        }
        is SplitTreeNode -> {
            SplitTreeNode(
                lhs = this.lhs.splitByAsNodes(),
                center = this.center,
                rhs = this.rhs.splitByAsNodes())
        }
    }
}

private fun lexerToTree(lexer: TexTalkLexer) =
    splitByIsInOrNotInNodes(lexer)?.splitByEqualsOrNotEqualsNodes()?.splitByAsNodes()

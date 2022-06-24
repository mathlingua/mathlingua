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

import mathlingua.lib.frontend.Diagnostic
import mathlingua.lib.frontend.MetaData
import mathlingua.lib.frontend.ast.AsExpression
import mathlingua.lib.frontend.ast.DEFAULT_VARIADIC_IS_RHS
import mathlingua.lib.frontend.ast.DEFAULT_VARIADIC_TARGET
import mathlingua.lib.frontend.ast.EqualsExpression
import mathlingua.lib.frontend.ast.InExpression
import mathlingua.lib.frontend.ast.IsExpression
import mathlingua.lib.frontend.ast.Name
import mathlingua.lib.frontend.ast.NotEqualsExpression
import mathlingua.lib.frontend.ast.NotInExpression
import mathlingua.lib.frontend.ast.TexTalkNode
import mathlingua.lib.frontend.ast.TexTalkTokenType
import mathlingua.lib.frontend.ast.VariadicIsExpression
import mathlingua.lib.frontend.ast.VariadicIsRhs
import mathlingua.lib.frontend.ast.VariadicRhs
import mathlingua.lib.frontend.ast.VariadicTarget
import mathlingua.lib.util.BiUnion
import mathlingua.lib.util.BiUnionFirst
import mathlingua.lib.util.BiUnionSecond

internal fun AtomTreeNode.atomTreeNodeToTexTalkNode(
    diagnostics: MutableList<Diagnostic>
): TexTalkNode? {
    return if (this.token.type == TexTalkTokenType.Name) {
        Name(
            text = this.token.text,
            metadata = MetaData(row = this.token.row, column = this.token.column, isInline = null))
    } else {
        diagnostics.add(
            error(message = "Unexpected token", row = this.token.row, column = this.token.column))
        null
    }
}

internal fun SplitTreeNode.toTexTalkNode(diagnostics: MutableList<Diagnostic>): TexTalkNode? =
    this.toIsExpressionOrVariadicIsExpression(diagnostics)?.value
        ?: this.toInExpression(diagnostics) ?: this.toNotInExpression(diagnostics)
            ?: this.toAsExpression(diagnostics) ?: this.toEqualsExpression(diagnostics)
            ?: this.toNotEqualsExpression(diagnostics)

internal fun SplitTreeNode.toIsExpressionOrVariadicIsExpression(
    diagnostics: MutableList<Diagnostic>
): BiUnion<IsExpression, VariadicIsExpression, TexTalkNode>? {
    val row = this.center?.row() ?: -1
    val column = this.center?.column() ?: -1

    val lhs = this.lhs?.treeNodeToTexTalkNodeList(diagnostics) ?: emptyList()
    val rhs = this.rhs?.treeNodeToTexTalkNodeList(diagnostics) ?: emptyList()

    if (lhs.isEmpty()) {
        diagnostics.add(
            error(
                message = "The left-hand-side of an 'is' expression cannot be empty",
                row = row,
                column = column))
        return BiUnionFirst(
            IsExpression(
                lhs = emptyList(),
                rhs = emptyList(),
                metadata = MetaData(row = row, column = column, isInline = null)))
    }

    if (rhs.isEmpty()) {
        diagnostics.add(
            error(
                message = "The right-hand-side of an 'is' expression cannot be empty",
                row = row,
                column = column))
        return BiUnionFirst(
            IsExpression(
                lhs = emptyList(),
                rhs = emptyList(),
                metadata = MetaData(row = row, column = column, isInline = null)))
    }

    if (lhs.size == 1 && rhs.size == 1) {
        // determine if it is a VariadicIsExpression or an IsExpression
        val left = lhs.first()
        val right = rhs.first()

        if (left is VariadicTarget || right is VariadicIsRhs) {
            // parse it as a VariadicIsExpression
            if (left !is VariadicTarget) {
                diagnostics.add(
                    error(
                        message = "The left-hand-side must be a VariadicTarget",
                        row = row,
                        column = column))
            }

            if (right !is VariadicRhs) {
                diagnostics.add(
                    error(
                        message = "The right-hand-side must be a VariadicRhs",
                        row = row,
                        column = column))
            }

            return BiUnionSecond(
                VariadicIsExpression(
                    lhs = (left as? VariadicTarget) ?: DEFAULT_VARIADIC_TARGET,
                    rhs = (right as? VariadicIsRhs) ?: DEFAULT_VARIADIC_IS_RHS,
                    metadata = MetaData(row = row, column = column, isInline = null)))
        } else {
            // parse it as an IsExpression
            return BiUnionFirst(
                IsExpression(
                    lhs = lhs.filterAndError(diagnostics),
                    rhs = rhs.filterAndError(diagnostics),
                    metadata = MetaData(row = row, column = column, isInline = null)))
        }
    }

    return null
}

internal fun SplitTreeNode.toInExpression(diagnostics: MutableList<Diagnostic>): InExpression? {
    return null
}

internal fun SplitTreeNode.toNotInExpression(
    diagnostics: MutableList<Diagnostic>
): NotInExpression? {
    return null
}

internal fun SplitTreeNode.toAsExpression(diagnostics: MutableList<Diagnostic>): AsExpression? {
    return null
}

internal fun SplitTreeNode.toEqualsExpression(
    diagnostics: MutableList<Diagnostic>
): EqualsExpression? {
    return null
}

internal fun SplitTreeNode.toNotEqualsExpression(
    diagnostics: MutableList<Diagnostic>
): NotEqualsExpression? {
    return null
}

/*
 * Takes a TreeNode and converts it to a list of TexTalkNodes.  There will be a list of more than one element
 * if the TreeNode is a ListTreeNode or ParenTreeNode with content of the form `..., ..., ...` in which case there
 * will be an element for each comma separated element that was successfully parsed.  There will be an empty list
 * if the TreeNode doesn't parse to a TexTalkNode or if it is a comma-separated list, none parse correctly.
 */
internal fun TreeNode.treeNodeToTexTalkNodeList(
    diagnostics: MutableList<Diagnostic>
): MutableList<TexTalkNode> =
    when (this) {
        is AtomTreeNode -> {
            mutableListOfNotNull(this.atomTreeNodeToTexTalkNode(diagnostics))
        }
        is ParenTreeNode -> {
            when (this.content) {
                null -> {
                    emptyMutableList()
                }
                is ListTreeNode -> {
                    toTexTalkNodeList(
                        diagnostics,
                        this.content as ListTreeNode,
                        this.prefix?.row ?: -1,
                        this.prefix?.column ?: -1)
                }
                else -> {
                    this.content!!.treeNodeToTexTalkNodeList(diagnostics)
                }
            }
        }
        is ListTreeNode -> {
            toTexTalkNodeList(diagnostics, this, -1, -1)
        }
        is SplitTreeNode -> {
            mutableListOfNotNull(this.toTexTalkNode(diagnostics))
        }
    }

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

private fun toTexTalkNodeList(
    diagnostics: MutableList<Diagnostic>,
    treeNode: ListTreeNode,
    fallbackRow: Int,
    fallbackColumn: Int
): MutableList<TexTalkNode> {
    val nodes = treeNode.nodes
    val result = mutableListOf<TexTalkNode>()
    while (nodes.isNotEmpty()) {
        val subQueue = mutableListOf<TreeNode>()
        while (nodes.isNotEmpty() && !nodes.has(TexTalkTokenType.Comma)) {
            subQueue.add(nodes.removeAt(0))
        }
        val comma =
            if (nodes.has(TexTalkTokenType.Comma)) {
                (nodes.removeAt(0) as AtomTreeNode).token
            } else {
                null
            }
        val param = newTexTalkSubParser(subQueue, diagnostics).parse()
        if (param == null) {
            diagnostics.add(
                error(
                    message = "Unexpected empty parameter",
                    row = comma?.row ?: fallbackRow,
                    column = comma?.column ?: fallbackColumn))
        } else {
            result.add(param)
        }
    }
    return result
}

private fun <T> mutableListOfNotNull(vararg args: T?): MutableList<T> {
    val result = mutableListOf<T>()
    for (a in args) {
        if (a != null) {
            result.add(a)
        }
    }
    return result
}

private fun <T> emptyMutableList(): MutableList<T> = mutableListOf()

/*
 * Given a list of TexTalkNodes returns the first node, and for any subsequent nodes reports a diagnostic that they
 * are not expected.  If the list is empty `null` is returned.  Calling code is responsible for recording a diagnostic
 * if required.
 */
private fun MutableList<TexTalkNode>.pollFirstAndError(
    diagnostics: MutableList<Diagnostic>
): TexTalkNode? =
    if (this.isEmpty()) {
        null
    } else {
        for (i in 1 until this.size) {
            diagnostics.add(
                error(
                    message = "Unexpected item",
                    row = this[i].metadata.row,
                    column = this[i].metadata.column))
        }
        this.first()
    }

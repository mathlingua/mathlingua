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
import mathlingua.lib.frontend.ast.AssignmentIsFormItem
import mathlingua.lib.frontend.ast.DefinitionIsFormItem
import mathlingua.lib.frontend.ast.EqualsExpression
import mathlingua.lib.frontend.ast.ExpressionIsFormItem
import mathlingua.lib.frontend.ast.InExpression
import mathlingua.lib.frontend.ast.IsExpression
import mathlingua.lib.frontend.ast.MetaIsForm
import mathlingua.lib.frontend.ast.MetaIsFormItem
import mathlingua.lib.frontend.ast.Name
import mathlingua.lib.frontend.ast.NodeList
import mathlingua.lib.frontend.ast.NonBracketNodeList
import mathlingua.lib.frontend.ast.NotEqualsExpression
import mathlingua.lib.frontend.ast.NotInExpression
import mathlingua.lib.frontend.ast.SpecificationIsFormItem
import mathlingua.lib.frontend.ast.SquareColonNodeList
import mathlingua.lib.frontend.ast.StatementIsFormItem
import mathlingua.lib.frontend.ast.TexTalkNode
import mathlingua.lib.frontend.ast.TexTalkTokenNode
import mathlingua.lib.frontend.ast.TexTalkTokenType
import mathlingua.lib.frontend.ast.VariadicIsExpression
import mathlingua.lib.util.BiUnion

internal fun AtomTreeNode.atomTreeNodeToTexTalkNode(): TexTalkNode {
    return if (this.token.type == TexTalkTokenType.Name) {
        val metadata = MetaData(row = this.token.row, column = this.token.column, isInline = null)
        when (val text = this.token.text
        ) {
            "statement" -> {
                StatementIsFormItem(metadata = metadata)
            }
            "assignment" -> {
                AssignmentIsFormItem(metadata = metadata)
            }
            "specification" -> {
                SpecificationIsFormItem(metadata = metadata)
            }
            "expression" -> {
                ExpressionIsFormItem(metadata = metadata)
            }
            "definition" -> {
                DefinitionIsFormItem(metadata = metadata)
            }
            else -> {
                Name(text = text, metadata = metadata)
            }
        }
    } else {
        TexTalkTokenNode(token = this.token)
    }
}

internal fun BracketTreeNode.bracketTreeNodeToTexTalkNode(
    diagnostics: MutableList<Diagnostic>
): TexTalkNode? {
    // check if it's a tuple or a grouping
    val row = this.prefix?.row ?: -1
    val column = this.prefix?.column ?: -1
    return when (this.prefix?.type) {
        TexTalkTokenType.LParen -> {
            null
        }
        TexTalkTokenType.LCurly -> {
            null
        }
        TexTalkTokenType.LSquare -> {
            null
        }
        TexTalkTokenType.LSquareColon -> {
            if (this.content == null) {
                diagnostics.add(
                    error(
                        message =
                            "Expected at least one statement, assignment, specification, expression, or definition.",
                        row = row,
                        column = column,
                    ))
                null
            } else {
                val content = this.content!!.toTexTalkNode(diagnostics)
                if (content !is NodeList<*>) {
                    diagnostics.add(
                        error(
                            message =
                                "Expected a list of statement, assignment, specification, expression, and/or definition.",
                            row = row,
                            column = column,
                        ))
                    null
                } else {
                    val items =
                        content.nodes.mapNotNull {
                            if (it is MetaIsFormItem) {
                                it
                            } else {
                                diagnostics.add(
                                    error(
                                        message =
                                            "Expected 'statement', 'assignment', 'specification', 'expression', and/or 'definition'",
                                        row = it.metadata.row,
                                        column = it.metadata.column,
                                    ))
                                null
                            }
                        }
                    MetaIsForm(
                        items =
                            SquareColonNodeList(
                                nodes = items,
                                metadata = MetaData(row = row, column = column, isInline = null)),
                        metadata = MetaData(row = row, column = column, isInline = null))
                }
            }
        }
        else -> {
            diagnostics.add(
                error(
                    message = "Expected a (, [, {, or [:",
                    row = row,
                    column = column,
                ))
            null
        }
    }
}

internal fun ListTreeNode.listTreeNodeToTexTalkNodes(
    diagnostics: MutableList<Diagnostic>
): TexTalkNode? {
    val nodes = this.nodes.mapNotNull { it.toTexTalkNode(diagnostics) }
    return if (nodes.isEmpty()) {
        null
    } else if (nodes.size == 1) {
        nodes[0]
    } else {
        NonBracketNodeList(nodes = nodes, metadata = nodes[0].metadata.copy())
    }
}

internal fun SplitTreeNode.splitTreeNodeToTexTalkNode(
    diagnostics: MutableList<Diagnostic>
): TexTalkNode? =
    this.toIsExpressionOrVariadicIsExpression(diagnostics)?.value
        ?: this.toInExpression(diagnostics) ?: this.toNotInExpression(diagnostics)
            ?: this.toAsExpression(diagnostics) ?: this.toEqualsExpression(diagnostics)
            ?: this.toNotEqualsExpression(diagnostics)

internal fun UnitTreeNode.unitTreeNodeToTexTalkNode(
    diagnostics: MutableList<Diagnostic>
): TexTalkNode? = newTexTalkSubParser(this.nodes, diagnostics).parse()

internal fun TreeNode.toTexTalkNode(diagnostics: MutableList<Diagnostic>): TexTalkNode? =
    when (this) {
        is AtomTreeNode -> this.atomTreeNodeToTexTalkNode()
        is BracketTreeNode -> this.bracketTreeNodeToTexTalkNode(diagnostics)
        is ListTreeNode -> this.listTreeNodeToTexTalkNodes(diagnostics)
        is SplitTreeNode -> this.splitTreeNodeToTexTalkNode(diagnostics)
        is UnitTreeNode -> this.unitTreeNodeToTexTalkNode(diagnostics)
    }

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

private fun SplitTreeNode.toIsExpressionOrVariadicIsExpression(
    diagnostics: MutableList<Diagnostic>
): BiUnion<IsExpression, VariadicIsExpression, TexTalkNode>? {
    TODO()
}

private fun SplitTreeNode.toInExpression(diagnostics: MutableList<Diagnostic>): InExpression? {
    return null
}

private fun SplitTreeNode.toNotInExpression(
    diagnostics: MutableList<Diagnostic>
): NotInExpression? {
    return null
}

private fun SplitTreeNode.toAsExpression(diagnostics: MutableList<Diagnostic>): AsExpression? {
    return null
}

private fun SplitTreeNode.toEqualsExpression(
    diagnostics: MutableList<Diagnostic>
): EqualsExpression? {
    return null
}

private fun SplitTreeNode.toNotEqualsExpression(
    diagnostics: MutableList<Diagnostic>
): NotEqualsExpression? {
    return null
}

/*
 * Copyright 2019 The MathLingua Authors
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

package mathlingua.backend.transform

import mathlingua.frontend.chalktalk.phase2.ast.clause.Clause
import mathlingua.frontend.chalktalk.phase2.ast.clause.ClauseListNode
import mathlingua.frontend.chalktalk.phase2.ast.clause.Statement
import mathlingua.frontend.chalktalk.phase2.ast.common.Phase2Node
import mathlingua.frontend.chalktalk.phase2.hasChild
import mathlingua.frontend.support.Location
import mathlingua.frontend.support.ValidationFailure
import mathlingua.frontend.support.ValidationSuccess
import mathlingua.frontend.textalk.Command
import mathlingua.frontend.textalk.ExpressionTexTalkNode
import mathlingua.frontend.textalk.IsTexTalkNode
import mathlingua.frontend.textalk.ParametersTexTalkNode
import mathlingua.frontend.textalk.TexTalkNode
import mathlingua.frontend.textalk.TexTalkNodeType
import mathlingua.frontend.textalk.TexTalkTokenType
import mathlingua.frontend.textalk.TextTexTalkNode

internal fun Phase2Node.normalize(): Phase2Node {
    var result = this
    result = result.commaSeparateCompoundCommands(result).root
    result = result.separateIsStatements(result).root
    result = result.separateInfixOperatorStatements(result).root
    return result.glueCommands(result).root
}

// replaces anything of the form `x is \a \b \c` as `x \a.b.c`
internal fun TexTalkNode.normalize(location: Location): TexTalkNode {
    return this.transform {
        if (it is ExpressionTexTalkNode) {
            val allCmd = it.children.all { it is Command }
            if (allCmd) {
                ExpressionTexTalkNode(children = it.getCommandsToGlue(location))
            } else {
                it
            }
        } else {
            it
        }
    }
}

internal fun Phase2Node.separateInfixOperatorStatements(
    follow: Phase2Node
): RootTarget<Phase2Node, Phase2Node> {
    var newFollow: Phase2Node? = null
    val newRoot =
        this.transform {
            val result =
                if (it is ClauseListNode) {
                    val newClauses = mutableListOf<Clause>()
                    for (c in it.clauses) {
                        if (c is Statement) {
                            when (val validation = c.texTalkRoot
                            ) {
                                is ValidationSuccess -> {
                                    val expRoot = validation.value
                                    for (expanded in getExpandedInfixOperators(expRoot)) {
                                        val stmt =
                                            Statement(
                                                text = expanded.toCode(),
                                                texTalkRoot = ValidationSuccess(expanded),
                                                row = c.row,
                                                column = c.column,
                                                isInline = c.isInline)
                                        newClauses.add(stmt)
                                    }
                                }
                                is ValidationFailure -> newClauses.add(c)
                            }
                        } else {
                            newClauses.add(c)
                        }
                    }
                    val result =
                        ClauseListNode(clauses = newClauses, row = it.row, column = it.column)
                    if (newFollow == null && it.hasChild(follow)) {
                        newFollow = result
                    }
                    result
                } else {
                    it
                }
            result
        }
    return RootTarget(root = newRoot, target = newFollow ?: follow)
}

internal fun Phase2Node.commaSeparateCompoundCommands(
    follow: Phase2Node
): RootTarget<Phase2Node, Phase2Node> {
    var newFollow: Phase2Node? = null
    val newRoot =
        this.transform {
            val result =
                if (it is Statement) {
                    if (it.texTalkRoot is ValidationSuccess) {
                        val newRoot =
                            it.texTalkRoot.value.transform { texTalkNode ->
                                if (texTalkNode is IsTexTalkNode) {
                                    val newExpressions = mutableListOf<ExpressionTexTalkNode>()
                                    val location = Location(it.row, it.column)
                                    for (exp in texTalkNode.rhs.items) {
                                        newExpressions.addAll(
                                            exp.getCommandsToGlue(location).map { cmd ->
                                                ExpressionTexTalkNode(children = listOf(cmd))
                                            })
                                    }
                                    IsTexTalkNode(
                                        lhs = texTalkNode.lhs,
                                        rhs = texTalkNode.rhs.copy(items = newExpressions))
                                } else {
                                    texTalkNode
                                }
                            }
                        val newStatement =
                            Statement(
                                text = newRoot.toCode(),
                                texTalkRoot = ValidationSuccess(newRoot as ExpressionTexTalkNode),
                                row = it.row,
                                column = it.column,
                                isInline = it.isInline)
                        if (newFollow == null && it.hasChild(follow)) {
                            newFollow = newStatement
                        }
                        newStatement
                    } else {
                        it
                    }
                } else {
                    it
                }
            result
        }
    return RootTarget(root = newRoot, target = newFollow ?: follow)
}

internal fun Phase2Node.separateIsStatements(
    follow: Phase2Node
): RootTarget<Phase2Node, Phase2Node> {
    var newFollow: Phase2Node? = null
    val newRoot =
        this.transform {
            val result =
                if (it is ClauseListNode) {
                    val newClauses = mutableListOf<Clause>()
                    for (clause in it.clauses) {
                        if (clause is Statement) {
                            val separated = findSeparatedIsNodes(clause)
                            if (separated == null) {
                                newClauses.add(clause)
                            } else {
                                newClauses.addAll(
                                    separated.map {
                                        val expRoot = ExpressionTexTalkNode(children = listOf(it))
                                        val stmt =
                                            Statement(
                                                text = expRoot.toCode(),
                                                texTalkRoot = ValidationSuccess(expRoot),
                                                row = clause.row,
                                                column = clause.column,
                                                isInline = clause.isInline)
                                        stmt
                                    })
                            }
                        } else {
                            newClauses.add(clause)
                        }
                    }
                    val result =
                        ClauseListNode(clauses = newClauses, row = it.row, column = it.column)
                    if (newFollow == null && it.hasChild(follow)) {
                        newFollow = result
                    }
                    result
                } else {
                    it
                }
            result
        }
    return RootTarget(root = newRoot, target = newFollow ?: follow)
}

internal fun TexTalkNode.replaceSignatures(signature: String, replacement: String) =
    this.transform {
        if (it is Command && it.signature() == signature) {
            TextTexTalkNode(
                type = TexTalkNodeType.Identifier,
                tokenType = TexTalkTokenType.Identifier,
                text = replacement,
                isVarArg = false)
        } else {
            this
        }
    }

// -----------------------------------------------------------------------------

private fun findSeparatedIsNodes(node: Statement) =
    when (val validation = node.texTalkRoot
    ) {
        is ValidationFailure -> null
        is ValidationSuccess -> {
            val root = validation.value
            if (root.children.size == 1 && root.children[0] is IsTexTalkNode) {
                val isNode = root.children[0] as IsTexTalkNode
                separateIsStatementsUnder(isNode)
            } else {
                null
            }
        }
    }

private fun separateIsStatementsUnder(isNode: IsTexTalkNode): List<IsTexTalkNode> {
    val result = mutableListOf<IsTexTalkNode>()
    for (left in isNode.lhs.items) {
        for (right in isNode.rhs.items) {
            result.add(
                IsTexTalkNode(
                    lhs = ParametersTexTalkNode(items = listOf(left)),
                    rhs = ParametersTexTalkNode(items = listOf(right))))
        }
    }
    return result
}

private fun getSingleInfixOperatorIndex(exp: ExpressionTexTalkNode): Int {
    for (i in 1 until exp.children.size - 1) {
        val prev = exp.children[i - 1]
        val cur = exp.children[i]
        val next = exp.children[i + 1]
        if (!isOperator(prev) && cur is Command && !isOperator(next)) {
            return i
        }
    }

    return -1
}

private fun isComma(node: TexTalkNode) = node is TextTexTalkNode && node.text == ","

private fun isOperator(node: TexTalkNode): Boolean {
    if (node !is TextTexTalkNode) {
        return false
    }

    if (node.text.isBlank()) {
        return false
    }

    for (c in node.text) {
        if (!isOpChar(c)) {
            return false
        }
    }

    return true
}

private fun isOpChar(c: Char) =
    (c == '!' ||
        c == '@' ||
        c == '%' ||
        c == '&' ||
        c == '*' ||
        c == '-' ||
        c == '+' ||
        c == '=' ||
        c == '|' ||
        c == '/' ||
        c == '<' ||
        c == '>')

private fun getArguments(exp: ExpressionTexTalkNode, start: Int, end: Int): List<TexTalkNode> {
    val result = mutableListOf<TexTalkNode>()
    var i = start
    while (i < end) {
        val argChildren = mutableListOf<TexTalkNode>()
        while (i < end && !isComma(exp.children[i])) {
            argChildren.add(exp.children[i++])
        }

        if (i < end && isComma(exp.children[i])) {
            i++ // skip the comma
        }

        if (argChildren.size == 1) {
            result.add(argChildren[0])
        } else {
            result.add(ExpressionTexTalkNode(children = argChildren))
        }
    }
    return result
}

private fun getExpandedInfixOperators(exp: ExpressionTexTalkNode): List<ExpressionTexTalkNode> {
    val opIndex = getSingleInfixOperatorIndex(exp)
    if (opIndex < 0) {
        return listOf(exp)
    }

    val leftArgs = getArguments(exp, 0, opIndex)
    val rightArgs = getArguments(exp, opIndex + 1, exp.children.size)

    val result = mutableListOf<ExpressionTexTalkNode>()

    val op = exp.children[opIndex]
    for (left in leftArgs) {
        for (right in rightArgs) {
            result.add(ExpressionTexTalkNode(children = listOf(left, op, right)))
        }
    }

    return result
}

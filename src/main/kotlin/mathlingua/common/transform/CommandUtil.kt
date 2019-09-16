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

package mathlingua.common.transform

import mathlingua.common.ValidationFailure
import mathlingua.common.ValidationSuccess
import mathlingua.common.chalktalk.phase2.Clause
import mathlingua.common.chalktalk.phase2.ClauseListNode
import mathlingua.common.chalktalk.phase2.Phase2Node
import mathlingua.common.chalktalk.phase2.Statement
import mathlingua.common.textalk.Command
import mathlingua.common.textalk.CommandPart
import mathlingua.common.textalk.ExpressionTexTalkNode
import mathlingua.common.textalk.IsTexTalkNode
import mathlingua.common.textalk.ParametersTexTalkNode
import mathlingua.common.textalk.TexTalkNode
import mathlingua.common.textalk.TexTalkNodeType
import mathlingua.common.textalk.TextTexTalkNode
import kotlin.system.exitProcess

data class RootTarget<R, T>(val root: R, val target: T)

fun findCommands(texTalkNode: TexTalkNode): List<Command> {
    val commands = mutableListOf<Command>()
    findCommandsImpl(texTalkNode, commands)
    return commands.distinct()
}

fun replaceSignatures(texTalkNode: TexTalkNode, signature: String, replacement: String): TexTalkNode {
    return texTalkNode.transform {
        if (it is Command && getCommandSignature(it).toCode() == signature) {
            TextTexTalkNode(type = TexTalkNodeType.Identifier, text = replacement)
        } else {
            texTalkNode
        }
    }
}

fun replaceCommands(
    node: Phase2Node,
    cmdToReplacement: Map<Command, String>,
    shouldProcessChalk: (node: Phase2Node) -> Boolean,
    shouldProcessTex: (root: TexTalkNode, node: TexTalkNode) -> Boolean
): Phase2Node {
    return node.transform {
        if (!shouldProcessChalk(it) || it !is Statement) {
            it
        } else {
            when (val validation = it.texTalkRoot) {
                is ValidationFailure -> it
                is ValidationSuccess -> {
                    val root = validation.value
                    val newRoot = replaceCommands(root, root, cmdToReplacement, shouldProcessTex) as ExpressionTexTalkNode
                    Statement(
                        text = newRoot.toCode(),
                        texTalkRoot = ValidationSuccess(newRoot),
                        row = -1,
                        column = -1
                    )
                }
            }
        }
    }
}

fun replaceCommands(
    texTalkNode: TexTalkNode,
    root: TexTalkNode,
    cmdToReplacement: Map<Command, String>,
    shouldProcess: (root: TexTalkNode, node: TexTalkNode) -> Boolean
): TexTalkNode {
    return texTalkNode.transform {
        if (!shouldProcess(root, it) || it !is Command) {
            it
        } else {
            if (!cmdToReplacement.containsKey(it)) {
                it
            } else {
                val name = cmdToReplacement[it]
                TextTexTalkNode(type = TexTalkNodeType.Identifier, text = name!!)
            }
        }
    }
}

private fun findCommandsImpl(texTalkNode: TexTalkNode, commands: MutableList<Command>) {
    if (texTalkNode is Command) {
        commands.add(texTalkNode)
    }

    texTalkNode.forEach { findCommandsImpl(it, commands) }
}

fun separateIsStatements(root: Phase2Node, follow: Phase2Node): RootTarget<Phase2Node, Phase2Node?> {
    var newFollow: Phase2Node? = null
    val newRoot = root.transform {
        val result = if (it is ClauseListNode) {
            val newClauses = mutableListOf<Clause>()
            for (clause in it.clauses) {
                if (clause is Statement) {
                    val separated = findSeparatedIsNodes(clause)
                    if (separated == null) {
                        newClauses.add(clause)
                    } else {
                        newClauses.addAll(separated.map {
                            val root = ExpressionTexTalkNode(
                                children = listOf(it)
                            )
                            Statement(
                                text = root.toCode(),
                                texTalkRoot = ValidationSuccess(root),
                                    row = -1,
                                    column = -1
                            )
                        })
                    }
                } else {
                    newClauses.add(clause)
                }
            }
            ClauseListNode(
                clauses = newClauses,
                    row = -1,
                    column = -1
            )
        } else {
            it
        }
        if (it === follow) {
            newFollow = result
        }
        result
    }
    return RootTarget(
            root = newRoot,
            target = newFollow
    )
}

private fun findSeparatedIsNodes(node: Statement): List<IsTexTalkNode>? {
    return when (val validation = node.texTalkRoot) {
        is ValidationFailure -> null
        is ValidationSuccess -> {
            val root = validation.value
            return if (root.children.size == 1 && root.children[0] is IsTexTalkNode) {
                val isNode = root.children[0] as IsTexTalkNode
                separateIsStatementsUnder(isNode)
            } else {
                null
            }
        }
    }
}

private fun separateIsStatementsUnder(isNode: IsTexTalkNode): List<IsTexTalkNode> {
    val result = mutableListOf<IsTexTalkNode>()
    for (left in isNode.lhs.items) {
        for (right in isNode.rhs.items) {
            result.add(
                IsTexTalkNode(
                    lhs = ParametersTexTalkNode(
                        items = listOf(left)
                    ),
                    rhs = ParametersTexTalkNode(
                        items = listOf(right)
                    )
                )
            )
        }
    }
    return result
}

// this function requires that `is` nodes are separated
// that is 'x is \a, \b' is separated as 'x is \a' and
// 'x is \b'
fun glueCommands(root: Phase2Node, follow: Phase2Node): RootTarget<Phase2Node, Phase2Node?> {
    var newFollow: Phase2Node? = null
    val newRoot = root.transform {
        val result = if (it is Statement &&
            it.texTalkRoot is ValidationSuccess &&
            it.texTalkRoot.value.children.all { c -> c is Command }) {
            val exp = it.texTalkRoot.value
            val cmds = getCommandsToGlue(exp)
            val gluedCmds = glueCommands(cmds)
            if (gluedCmds.size != 1) {
                throw Error("Expected id $it to only contain a single glued command")
            }
            val newExp = ExpressionTexTalkNode(
                children = listOf(
                    gluedCmds[0]
                )
            )
            Statement(
                text = newExp.toCode(),
                texTalkRoot = ValidationSuccess(newExp),
                    row = -1,
                    column = -1
            )
        } else if (it is Statement &&
            it.texTalkRoot is ValidationSuccess &&
            it.texTalkRoot.value.children.size == 1 &&
            it.texTalkRoot.value.children[0] is IsTexTalkNode) {
            val isNode = it.texTalkRoot.value.children[0] as IsTexTalkNode
            if (isNode.rhs.items.size != 1) {
                throw Error("Expected 'is' node $isNode to only contain a single rhs item")
            }
            val cmds = getCommandsToGlue(isNode.rhs.items[0])
            val gluedCmds = glueCommands(cmds)
            if (gluedCmds.size != 1) {
                throw Error("Expected 'is' node $isNode to have only one glued rhs command")
            }
            val newExp = ExpressionTexTalkNode(
                children = listOf(
                    IsTexTalkNode(
                        lhs = isNode.lhs,
                        rhs = ParametersTexTalkNode(
                            items = listOf(
                                ExpressionTexTalkNode(
                                    children = listOf(
                                        gluedCmds[0]
                                    )
                                )
                            )
                        )
                    )
                )
            )
            Statement(
                text = newExp.toCode(),
                texTalkRoot = ValidationSuccess(newExp),
                    row = -1,
                    column = -1
            )
        } else {
            it
        }
        if (it === follow) {
            newFollow = result
        }
        result
    }
    return RootTarget(
            root = newRoot,
            target = newFollow
    )
}

private fun getCommandsToGlue(node: ExpressionTexTalkNode): List<Command> {
    val cmds = mutableListOf<Command>()
    for (n in node.children) {
        if (n !is Command) {
            throw Error("Unexpected non-Command node")
        }
        cmds.add(n)
    }
    return glueCommands(cmds)
}

fun glueCommands(commands: List<Command>): List<Command> {
    if (commands.isEmpty()) {
        return emptyList()
    }

    if (commands.size == 1) {
        return listOf(commands.first())
    }

    val last = commands.last()
    val newCommands = mutableListOf<Command>()
    for (i in 0 until commands.size - 1) {
        val cmd = commands[i]
        val parts = mutableListOf<CommandPart>()
        parts.addAll(cmd.parts)
        parts.addAll(last.parts)
        newCommands.add(Command(parts = parts))
    }
    return newCommands
}

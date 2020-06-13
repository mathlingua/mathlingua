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
import mathlingua.common.chalktalk.phase2.*
import mathlingua.common.chalktalk.phase2.ast.Phase2Node
import mathlingua.common.chalktalk.phase2.ast.clause.Clause
import mathlingua.common.chalktalk.phase2.ast.clause.ClauseListNode
import mathlingua.common.chalktalk.phase2.ast.clause.Statement
import mathlingua.common.textalk.Command
import mathlingua.common.textalk.CommandPart
import mathlingua.common.textalk.ExpressionTexTalkNode
import mathlingua.common.textalk.IsTexTalkNode
import mathlingua.common.textalk.ParametersTexTalkNode
import mathlingua.common.textalk.TexTalkNode
import mathlingua.common.textalk.TexTalkNodeType
import mathlingua.common.textalk.TextTexTalkNode
import mathlingua.common.validationSuccess

data class RootTarget<R, T>(val root: R, val target: T)

internal fun locateAllCommands(phase2Node: Phase2Node): List<Command> {
    var root = phase2Node
    root = separateIsStatements(root, root).root
    root = separateInfixOperatorStatements(root, root).root
    root = glueCommands(root, root).root

    val commands = mutableListOf<Command>()
    findCommandsImpl(root, commands)
    return commands
}

private fun findCommandsImpl(phase2Node: Phase2Node, commands: MutableList<Command>) {
    phase2Node.forEach {
        if (it is Statement) {
            if (it.texTalkRoot is ValidationSuccess) {
                findCommandsImpl(it.texTalkRoot.value, commands)
            }
        } else {
            findCommandsImpl(it, commands)
        }
    }
}

internal fun findCommands(texTalkNode: TexTalkNode): List<Command> {
    val commands = mutableListOf<Command>()
    findCommandsImpl(texTalkNode, commands)
    return commands.distinct()
}

internal fun replaceSignatures(
    texTalkNode: TexTalkNode,
    signature: String,
    replacement: String
) = texTalkNode.transform {
    if (it is Command && it.signature() == signature) {
        TextTexTalkNode(type = TexTalkNodeType.Identifier, text = replacement, isVarArg = false)
    } else {
        texTalkNode
    }
}

internal fun replaceCommands(
    node: Phase2Node,
    cmdToReplacement: Map<Command, String>,
    shouldProcessChalk: (node: Phase2Node) -> Boolean,
    shouldProcessTex: (root: TexTalkNode, node: TexTalkNode) -> Boolean
) = node.transform {
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
                        texTalkRoot = validationSuccess(newRoot)
                )
            }
        }
    }
}

internal fun replaceCommands(
    texTalkNode: TexTalkNode,
    root: TexTalkNode,
    cmdToReplacement: Map<Command, String>,
    shouldProcess: (root: TexTalkNode, node: TexTalkNode) -> Boolean
) = texTalkNode.transform {
    if (!shouldProcess(root, it) || it !is Command) {
        it
    } else {
        if (!cmdToReplacement.containsKey(it)) {
            it
        } else {
            val name = cmdToReplacement[it]
            TextTexTalkNode(type = TexTalkNodeType.Identifier, text = name!!, isVarArg = false)
        }
    }
}

private fun findCommandsImpl(texTalkNode: TexTalkNode, commands: MutableList<Command>) {
    if (texTalkNode is Command) {
        commands.add(texTalkNode)
    }

    texTalkNode.forEach { findCommandsImpl(it, commands) }
}

internal fun separateIsStatements(root: Phase2Node, follow: Phase2Node): RootTarget<Phase2Node, Phase2Node> {
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
                            val expRoot = ExpressionTexTalkNode(
                                children = listOf(it)
                            )
                            Statement(
                                    text = expRoot.toCode(),
                                    texTalkRoot = validationSuccess(expRoot)
                            )
                        })
                    }
                } else {
                    newClauses.add(clause)
                }
            }
            val result = ClauseListNode(clauses = newClauses)
            if (newFollow == null && hasChild(it, follow)) {
                newFollow = result
            }
            result
        } else {
            it
        }
        result
    }
    return RootTarget(
            root = newRoot,
            target = newFollow ?: follow
    )
}

private fun findSeparatedIsNodes(node: Statement) = when (val validation = node.texTalkRoot) {
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
internal fun glueCommands(root: Phase2Node, follow: Phase2Node): RootTarget<Phase2Node, Phase2Node> {
    var newFollow: Phase2Node? = null
    val newRoot = root.transform {
        val result = if (it is Statement &&
            it.texTalkRoot is ValidationSuccess &&
            it.texTalkRoot.value.children.isNotEmpty() &&
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
            val result = Statement(
                    text = newExp.toCode(),
                    texTalkRoot = validationSuccess(newExp)
            )
            if (newFollow == null && hasChild(it, follow)) {
                newFollow = result
            }
            result
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
            val result = Statement(
                    text = newExp.toCode(),
                    texTalkRoot = validationSuccess(newExp)
            )
            if (newFollow == null && hasChild(it, follow)) {
                newFollow = result
            }
            result
        } else {
            it
        }
        result
    }
    return RootTarget(
            root = newRoot,
            target = newFollow ?: follow
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

internal fun glueCommands(commands: List<Command>): List<Command> {
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

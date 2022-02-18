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

import mathlingua.frontend.chalktalk.phase2.ast.clause.Statement
import mathlingua.frontend.chalktalk.phase2.ast.common.Phase2Node
import mathlingua.frontend.chalktalk.phase2.hasChild
import mathlingua.frontend.support.Location
import mathlingua.frontend.support.ValidationFailure
import mathlingua.frontend.support.ValidationSuccess
import mathlingua.frontend.textalk.Command
import mathlingua.frontend.textalk.CommandPart
import mathlingua.frontend.textalk.ExpressionTexTalkNode
import mathlingua.frontend.textalk.IsTexTalkNode
import mathlingua.frontend.textalk.ParametersTexTalkNode
import mathlingua.frontend.textalk.TexTalkNode
import mathlingua.frontend.textalk.TexTalkNodeType
import mathlingua.frontend.textalk.TexTalkTokenType
import mathlingua.frontend.textalk.TextTexTalkNode

internal data class RootTarget<R, T>(val root: R, val target: T)

internal fun getCommandsToGlue(node: ExpressionTexTalkNode, location: Location): List<Command> {
    val cmds = mutableListOf<Command>()
    for (n in node.children) {
        if (n !is Command) {
            if (n !is TextTexTalkNode || (n.text != ":Defines:" && n.text != ":States:")) {
                println(
                    "Unexpected non-Command node: ${n.toCode()} (${location.row + 1}, ${location.column + 1})")
            }
        } else {
            cmds.add(n)
        }
    }
    return glueCommands(cmds)
}

internal fun locateAllCommands(phase2Node: Phase2Node): List<Command> {
    var root = phase2Node
    root = separateIsStatements(root, root).root
    root = separateInfixOperatorStatements(root, root).root
    root = glueCommands(root, root).root

    val commands = mutableListOf<Command>()
    findCommandsImpl(root, commands)
    return commands
}

internal fun findCommands(texTalkNode: TexTalkNode): List<Command> {
    val commands = mutableListOf<Command>()
    findCommandsImpl(texTalkNode, commands)
    return commands.distinct()
}

internal fun replaceCommands(
    node: Phase2Node,
    cmdToReplacement: Map<Command, String>,
    shouldProcessChalk: (node: Phase2Node) -> Boolean,
    shouldProcessTex: (root: TexTalkNode, node: TexTalkNode) -> Boolean
) =
    node.transform {
        if (!shouldProcessChalk(it) || it !is Statement) {
            it
        } else {
            when (val validation = it.texTalkRoot
            ) {
                is ValidationFailure -> it
                is ValidationSuccess -> {
                    val root = validation.value
                    val newRoot =
                        replaceCommands(
                            root, root, cmdToReplacement, shouldProcessTex) as ExpressionTexTalkNode
                    Statement(
                        text = newRoot.toCode(),
                        texTalkRoot = ValidationSuccess(newRoot),
                        it.row,
                        it.column)
                }
            }
        }
    }

internal fun replaceCommands(
    texTalkNode: TexTalkNode,
    root: TexTalkNode,
    cmdToReplacement: Map<Command, String>,
    shouldProcess: (root: TexTalkNode, node: TexTalkNode) -> Boolean
) =
    texTalkNode.transform {
        if (!shouldProcess(root, it) || it !is Command) {
            it
        } else {
            if (!cmdToReplacement.containsKey(it)) {
                it
            } else {
                val name = cmdToReplacement[it]
                TextTexTalkNode(
                    type = TexTalkNodeType.Identifier,
                    tokenType = TexTalkTokenType.Identifier,
                    text = name!!,
                    isVarArg = false)
            }
        }
    }

// this function requires that `is` nodes are separated
// that is 'x is \a, \b' is separated as 'x is \a' and
// 'x is \b'
internal fun glueCommands(
    root: Phase2Node, follow: Phase2Node
): RootTarget<Phase2Node, Phase2Node> {
    var newFollow: Phase2Node? = null
    val newRoot =
        root.transform {
            val result =
                if (it is Statement &&
                    it.texTalkRoot is ValidationSuccess &&
                    it.texTalkRoot.value.children.size == 1 &&
                    it.texTalkRoot.value.children[0] is IsTexTalkNode &&
                    (it.texTalkRoot.value.children[0] as IsTexTalkNode).rhs.items.getOrNull(0)
                        ?.children
                        ?.all { it is Command } == true) {
                    val isNode = it.texTalkRoot.value.children[0] as IsTexTalkNode
                    if (isNode.rhs.items.size != 1) {
                        throw Error("Expected 'is' node $isNode to only contain a single rhs item")
                    }
                    val cmds =
                        getCommandsToGlue(isNode.rhs.items[0], Location(root.row, root.column))
                    val gluedCmds = glueCommands(cmds)
                    if (gluedCmds.size != 1) {
                        throw Error("Expected 'is' node $isNode to have only one glued rhs command")
                    }
                    val newExp =
                        ExpressionTexTalkNode(
                            children =
                                listOf(
                                    IsTexTalkNode(
                                        lhs = isNode.lhs,
                                        rhs =
                                            ParametersTexTalkNode(
                                                items =
                                                    listOf(
                                                        ExpressionTexTalkNode(
                                                            children = listOf(gluedCmds[0])))))))
                    val result =
                        Statement(
                            text = newExp.toCode(),
                            texTalkRoot = ValidationSuccess(newExp),
                            it.row,
                            it.column)
                    if (newFollow == null && hasChild(it, follow)) {
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

// -----------------------------------------------------------------------------

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

private fun findCommandsImpl(texTalkNode: TexTalkNode, commands: MutableList<Command>) {
    if (texTalkNode is Command) {
        commands.add(texTalkNode)
    }

    texTalkNode.forEach { findCommandsImpl(it, commands) }
}

private fun glueCommands(commands: List<Command>): List<Command> {
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
        newCommands.add(Command(parts = parts, false))
    }
    return newCommands
}

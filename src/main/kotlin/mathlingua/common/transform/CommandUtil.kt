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

import mathlingua.common.Validation
import mathlingua.common.chalktalk.phase2.Phase2Node
import mathlingua.common.chalktalk.phase2.Statement
import mathlingua.common.textalk.Command
import mathlingua.common.textalk.CommandPart
import mathlingua.common.textalk.ExpressionTexTalkNode
import mathlingua.common.textalk.TexTalkNode
import mathlingua.common.textalk.TexTalkNodeType
import mathlingua.common.textalk.TextTexTalkNode
import mathlingua.common.textalk.populateParents

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
    sigToReplacement: Map<String, String>,
    commandsFound: MutableList<Command>,
    shouldProcessChalk: (node: Phase2Node) -> Boolean,
    shouldProcessTex: (node: TexTalkNode) -> Boolean
): Phase2Node {
    return node.transform {
        if (!shouldProcessChalk(it) || it !is Statement) {
            it
        } else {
            val validation = it.texTalkRoot
            if (!validation.isSuccessful) {
                it
            } else {
                val root = it.texTalkRoot.value!!
                val newRoot = replaceCommands(root, sigToReplacement, commandsFound, shouldProcessTex) as ExpressionTexTalkNode
                Statement(
                    text = newRoot.toCode(),
                    texTalkRoot = Validation.success(newRoot)
                )
            }
        }
    }
}

fun replaceCommands(
    texTalkNode: TexTalkNode,
    sigToReplacement: Map<String, String>,
    commandsFound: MutableList<Command>,
    shouldProcess: (node: TexTalkNode) -> Boolean
): TexTalkNode {
    return populateParents(texTalkNode).transform {
        if (!shouldProcess(it) || it !is Command) {
            it
        } else {
            commandsFound.add(it)
            val sig = getCommandSignature(it).toCode()
            if (!sigToReplacement.containsKey(sig)) {
                it
            } else {
                val name = sigToReplacement[sig]
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

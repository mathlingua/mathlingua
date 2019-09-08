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

import mathlingua.common.textalk.Command
import mathlingua.common.textalk.TexTalkNode
import mathlingua.common.textalk.TexTalkNodeType
import mathlingua.common.textalk.TextTexTalkNode

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

fun replaceCommands(texTalkNode: TexTalkNode, sigToReplacement: Map<String, String>): TexTalkNode {
    return texTalkNode.transform {
        if (it !is Command) {
            it
        } else {
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

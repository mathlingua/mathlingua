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
import mathlingua.common.chalktalk.phase2.Phase2Node
import mathlingua.common.chalktalk.phase2.Statement
import mathlingua.common.textalk.Command
import mathlingua.common.textalk.CommandPart
import mathlingua.common.textalk.ExpressionTexTalkNode
import mathlingua.common.textalk.GroupTexTalkNode
import mathlingua.common.textalk.IsTexTalkNode
import mathlingua.common.textalk.NamedGroupTexTalkNode
import mathlingua.common.textalk.ParametersTexTalkNode
import mathlingua.common.textalk.SubSupTexTalkNode
import mathlingua.common.textalk.TexTalkNode
import mathlingua.common.textalk.TexTalkNodeType
import mathlingua.common.textalk.TextTexTalkNode

fun getSignature(stmt: Statement): String? {
    val sigs = findAllStatementSignatures(stmt)
    return if (sigs.size == 1) {
        sigs.first()
    } else null
}

fun findAllStatementSignatures(stmt: Statement) = when (val rootValidation = stmt.texTalkRoot) {
    is ValidationSuccess -> {
        val expressionNode = rootValidation.value
        val signatures = mutableSetOf<String>()
        findAllSignaturesImpl(expressionNode, signatures)
        signatures
    }
    is ValidationFailure -> emptySet<String>()
}

fun getMergedCommandSignature(expressionNode: ExpressionTexTalkNode): String? {
    val commandParts = mutableListOf<CommandPart>()
    for (child in expressionNode.children) {
        if (child is Command) {
            commandParts.addAll(child.parts)
        }
    }

    if (commandParts.isNotEmpty()) {
        return getCommandSignature(Command(parts = commandParts)).toCode()
    }

    return null
}

fun getCommandSignature(command: Command) = Command(
    parts = command.parts.map { getCommandPartForSignature(it) }
)

fun locateAllSignatures(node: Phase2Node): Set<String> {
    val signatures = mutableSetOf<String>()
    findAllSignaturesImpl(node, signatures)
    return signatures
}

private fun findAllSignaturesImpl(node: Phase2Node, signatures: MutableSet<String>) {
    if (node is Statement) {
        val sigs = findAllStatementSignatures(node)
        signatures.addAll(sigs)
    }

    node.forEach { findAllSignaturesImpl(it, signatures) }
}

private fun findAllSignaturesImpl(texTalkNode: TexTalkNode, signatures: MutableSet<String>) {
    if (texTalkNode is IsTexTalkNode) {
        for (expNode in texTalkNode.rhs.items) {
            val sig = getMergedCommandSignature(expNode)
            if (sig != null) {
                signatures.add(sig)
            }
        }
        return
    } else if (texTalkNode is Command) {
        val sig = getCommandSignature(texTalkNode).toCode()
        signatures.add(sig)
    }

    texTalkNode.forEach { findAllSignaturesImpl(it, signatures) }
}

private fun <T> callOrNull(input: T?, fn: (t: T) -> T): T? = if (input == null) {
    null
} else {
    fn(input)
}

private fun getCommandPartForSignature(node: CommandPart) = CommandPart(
    name = node.name,
    square = callOrNull(node.square, ::getGroupNodeForSignature),
    subSup = callOrNull(node.subSup, ::getSubSupForSignature),
    groups = node.groups.map { getGroupNodeForSignature(it) },
    namedGroups = node.namedGroups.map { getNamedGroupNodeForSignature(it) }
)

private fun getSubSupForSignature(node: SubSupTexTalkNode) = SubSupTexTalkNode(
    sub = callOrNull(node.sub, ::getGroupNodeForSignature),
    sup = callOrNull(node.sup, ::getGroupNodeForSignature)
)

private fun getGroupNodeForSignature(node: GroupTexTalkNode) = GroupTexTalkNode(
    type = node.type,
    parameters = getParametersNodeForSignature(node.parameters)
)

private fun getParametersNodeForSignature(node: ParametersTexTalkNode) = ParametersTexTalkNode(
    items = node.items.map {
        ExpressionTexTalkNode(
            children = listOf(
                TextTexTalkNode(
                    type = TexTalkNodeType.Identifier,
                    text = "?"
                )
            )
        )
    }
)

private fun getNamedGroupNodeForSignature(node: NamedGroupTexTalkNode) = NamedGroupTexTalkNode(
    name = node.name,
    group = getGroupNodeForSignature(node.group)
)

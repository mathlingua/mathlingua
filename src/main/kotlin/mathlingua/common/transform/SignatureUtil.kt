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
import mathlingua.common.chalktalk.phase2.ast.Phase2Node
import mathlingua.common.chalktalk.phase2.ast.clause.IdStatement
import mathlingua.common.chalktalk.phase2.ast.clause.Statement
import mathlingua.common.chalktalk.phase2.ast.section.TextSection
import mathlingua.common.chalktalk.phase2.ast.toplevel.DefinesGroup
import mathlingua.common.chalktalk.phase2.ast.toplevel.ProtoGroup
import mathlingua.common.chalktalk.phase2.ast.toplevel.RepresentsGroup
import mathlingua.common.chalktalk.phase2.ast.toplevel.TopLevelGroup
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

internal fun getSignature(group: TopLevelGroup): String? {
    return when (group) {
        is DefinesGroup -> getSignature(group.id)
        is RepresentsGroup -> getSignature(group.id)
        is ProtoGroup -> getSignature(group.textSection)
        else -> null
    }
}

internal fun getSignature(id: IdStatement) = getSignature(id.toStatement())

internal fun getSignature(stmt: Statement): String? {
    val sigs = findAllStatementSignatures(stmt)
    return if (sigs.size == 1) {
        sigs.first()
    } else null
}

internal fun findAllStatementSignatures(stmt: Statement): Set<String> {
    val gluedStmt = glueCommands(stmt, stmt).root as Statement
    return when (val rootValidation = gluedStmt.texTalkRoot) {
        is ValidationSuccess -> {
            val expressionNode = rootValidation.value
            val signatures = mutableSetOf<String>()
            findAllSignaturesImpl(expressionNode, signatures)
            signatures
        }
        is ValidationFailure -> emptySet()
    }
}

internal fun getMergedCommandSignature(expressionNode: ExpressionTexTalkNode): String? {
    val commandParts = mutableListOf<CommandPart>()
    for (child in expressionNode.children) {
        if (child is Command) {
            commandParts.addAll(child.parts)
        }
    }

    if (commandParts.isNotEmpty()) {
        return getCommandSignature(Command(parts = commandParts))
    }

    return null
}

internal fun getCommandSignature(command: Command) = flattenSignature(Command(
    parts = command.parts.map { getCommandPartForSignature(it) }
).toCode())

internal fun locateAllSignatures(node: Phase2Node): Set<String> {
    val signatures = mutableSetOf<String>()
    findAllSignaturesImpl(node, signatures)
    return signatures
}

private fun findAllSignaturesImpl(node: Phase2Node, signatures: MutableSet<String>) {
    if (node is Statement) {
        signatures.addAll(findAllStatementSignatures(node))
    } else if (node is IdStatement) {
        findAllSignaturesImpl(node.toStatement(), signatures)
    } else if (node is TextSection) {
        val sig = getSignature(node)
        if (sig != null) {
            signatures.add(sig)
        }
    }

    node.forEach { findAllSignaturesImpl(it, signatures) }
}

private fun getSignature(section: TextSection): String? {
    val match = Regex("\\\\term\\{(.*?)\\}").find(section.text)
    if (match == null || match.groupValues.size < 2) {
        return null
    }
    return match.groupValues[1]
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
        val sig = getCommandSignature(texTalkNode)
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
    parameters = getParametersNodeForSignature(node.parameters),
    isVarArg = node.isVarArg
)

private fun getParametersNodeForSignature(node: ParametersTexTalkNode) = ParametersTexTalkNode(
    items = node.items.map {
        if (it.children.size == 1 &&
            it.children[0] is TextTexTalkNode &&
            (it.children[0] as TextTexTalkNode).isVarArg) {
            ExpressionTexTalkNode(
                    children = listOf(
                            TextTexTalkNode(
                                    type = TexTalkNodeType.Identifier,
                                    text = "?",
                                    isVarArg = true
                            )
                    )
            )
        } else {
            ExpressionTexTalkNode(
                    children = listOf(
                            TextTexTalkNode(
                                    type = TexTalkNodeType.Identifier,
                                    text = "?",
                                    isVarArg = false
                            )
                    )
            )
        }
    }
)

private fun getNamedGroupNodeForSignature(node: NamedGroupTexTalkNode) = NamedGroupTexTalkNode(
    name = node.name,
    group = getGroupNodeForSignature(node.group)
)

internal fun flattenSignature(signature: String): String {
    /*
     * This converts \f[?]{?, ?, ?}{} to \f[]{}{}.  That is, the ?, are
     * removed.  In addition \f{}{}{}... is replaced with \f{}...
     * That is, a sequence of {} followed by ... is replaced with a
     * single {}...
     */
    return signature.replace(Regex("\\?\\.\\.\\."), "?")
                    .replace(Regex("\\s*\\??(\\s*,\\s*\\?\\s*)*"), "")
                    .replace(" ", "")
                    .replace(Regex("(\\{\\})*\\{\\}\\.\\.\\."), "{}...")
}

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

import mathlingua.common.*
import mathlingua.common.chalktalk.phase2.ast.Phase2Node
import mathlingua.common.chalktalk.phase2.ast.clause.IdStatement
import mathlingua.common.chalktalk.phase2.ast.clause.Statement
import mathlingua.common.chalktalk.phase2.ast.section.TextSection
import mathlingua.common.chalktalk.phase2.ast.toplevel.DefinesGroup
import mathlingua.common.chalktalk.phase2.ast.toplevel.ProtoGroup
import mathlingua.common.chalktalk.phase2.ast.toplevel.RepresentsGroup
import mathlingua.common.chalktalk.phase2.ast.toplevel.TopLevelGroup
import mathlingua.common.textalk.*

internal fun locateAllSignatures(node: Phase2Node, locationTracker: LocationTracker): Set<Signature> {
    val signatures = mutableSetOf<Signature>()
    findAllSignaturesImpl(node, signatures, locationTracker)
    return signatures
}

private fun findAllSignaturesImpl(node: Phase2Node, signatures: MutableSet<Signature>, locationTracker: LocationTracker) {
    if (node is Statement) {
        signatures.addAll(findAllStatementSignatures(node, locationTracker))
    } else if (node is IdStatement) {
        findAllSignaturesImpl(node.toStatement(), signatures, locationTracker)
    } else if (node is TextSection) {
        val sig = getSignature(node)
        if (sig != null) {
            signatures.add(
                    Signature(
                            form = sig,
                            location = locationTracker.getLocationOf(node) ?: Location(-1, -1)
                    )
            )
        }
    }

    node.forEach { findAllSignaturesImpl(it, signatures, locationTracker) }
}

private fun getSignature(section: TextSection): String? {
    val match = Regex("\\\\term\\{(.*?)\\}").find(section.text)
    if (match == null || match.groupValues.size < 2) {
        return null
    }
    return match.groupValues[1]
}

internal fun findAllStatementSignatures(stmt: Statement, locationTracker: LocationTracker): Set<Signature> {
    val gluedStmt = glueCommands(stmt, stmt).root as Statement
    return when (val rootValidation = gluedStmt.texTalkRoot) {
        is ValidationSuccess -> {
            val expressionNode = rootValidation.value
            val signatures = mutableSetOf<Signature>()
            findAllSignaturesImpl(expressionNode, signatures, locationTracker.getLocationOf(stmt) ?: Location(-1, -1))
            signatures
        }
        is ValidationFailure -> emptySet()
    }
}

private fun findAllSignaturesImpl(texTalkNode: TexTalkNode, signatures: MutableSet<Signature>, location: Location) {
    if (texTalkNode is IsTexTalkNode) {
        for (expNode in texTalkNode.rhs.items) {
            val sig = getMergedCommandSignature(expNode)
            if (sig != null) {
                signatures.add(
                        Signature(
                                form = sig,
                                location = location
                        ))
            }
        }
        return
    } else if (texTalkNode is Command) {
        val sig = texTalkNode.signature()
        signatures.add(
                Signature(
                        form = sig,
                        location = location
                ))
    }

    texTalkNode.forEach { findAllSignaturesImpl(it, signatures, location) }
}

internal fun getMergedCommandSignature(expressionNode: ExpressionTexTalkNode): String? {
    val commandParts = mutableListOf<CommandPart>()
    for (child in expressionNode.children) {
        if (child is Command) {
            commandParts.addAll(child.parts)
        }
    }

    if (commandParts.isNotEmpty()) {
        return Command(parts = commandParts).signature()
    }

    return null
}

internal fun getSignature(group: TopLevelGroup): String? {
    return when (group) {
        is DefinesGroup -> group.id.signature()
        is RepresentsGroup -> group.id.signature()
        is ProtoGroup -> getSignature(group.textSection)
        else -> null
    }
}

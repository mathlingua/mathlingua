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

package mathlingua.backend.transform

import mathlingua.Signature
import mathlingua.frontend.chalktalk.phase2.ast.clause.IdStatement
import mathlingua.frontend.chalktalk.phase2.ast.clause.Statement
import mathlingua.frontend.chalktalk.phase2.ast.common.Phase2Node
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.TopLevelGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.defines.DefinesGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.states.StatesGroup
import mathlingua.frontend.chalktalk.phase2.ast.section.TextSection
import mathlingua.frontend.support.Location
import mathlingua.frontend.support.MutableLocationTracker
import mathlingua.frontend.support.ValidationFailure
import mathlingua.frontend.support.ValidationSuccess
import mathlingua.frontend.textalk.Command
import mathlingua.frontend.textalk.CommandPart
import mathlingua.frontend.textalk.ExpressionTexTalkNode
import mathlingua.frontend.textalk.IsTexTalkNode
import mathlingua.frontend.textalk.TexTalkNode

internal fun locateAllSignatures(
    node: Phase2Node, locationTracker: MutableLocationTracker
): Set<Signature> {
    val signatures = mutableSetOf<Signature>()
    findAllSignaturesImpl(normalize(node, locationTracker), signatures, locationTracker)
    return signatures
}

// -----------------------------------------------------------------------------

private fun findAllSignaturesImpl(
    node: Phase2Node, signatures: MutableSet<Signature>, locationTracker: MutableLocationTracker
) {
    if (node is Statement) {
        signatures.addAll(findAllStatementSignatures(node, locationTracker))
    } else if (node is IdStatement) {
        findAllSignaturesImpl(node.toStatement(), signatures, locationTracker)
    } else if (node is TextSection) {
        val sig = getSignature(node)
        if (sig != null) {
            signatures.add(
                Signature(
                    form = sig, location = locationTracker.getLocationOf(node) ?: Location(-1, -1)))
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

internal fun findAllStatementSignatures(
    stmt: Statement, locationTracker: MutableLocationTracker
): Set<Signature> {
    val gluedStmt = glueCommands(stmt, stmt, locationTracker).root as Statement
    return when (val rootValidation = gluedStmt.texTalkRoot
    ) {
        is ValidationSuccess -> {
            val expressionNode = rootValidation.value
            val signatures = mutableSetOf<Signature>()
            findAllSignaturesImpl(
                expressionNode, signatures, locationTracker.getLocationOf(stmt) ?: Location(-1, -1))
            signatures
        }
        is ValidationFailure -> emptySet()
    }
}

private fun findAllSignaturesImpl(
    texTalkNode: TexTalkNode, signatures: MutableSet<Signature>, location: Location
) {
    if (texTalkNode is IsTexTalkNode) {
        for (expNode in texTalkNode.rhs.items) {
            val sig = getMergedCommandSignature(expNode)
            if (sig != null) {
                signatures.add(Signature(form = sig, location = location))
            }
        }
        return
    } else if (texTalkNode is Command) {
        val sig = texTalkNode.signature()
        signatures.add(Signature(form = sig, location = location))
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
        return Command(parts = commandParts, false).signature()
    }

    return null
}

internal fun getSignature(group: TopLevelGroup): String? {
    return when (group) {
        is DefinesGroup -> group.id.signature()
        is StatesGroup -> group.id.signature()
        else -> null
    }
}

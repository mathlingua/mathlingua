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

import mathlingua.frontend.chalktalk.phase2.ast.clause.IdStatement
import mathlingua.frontend.chalktalk.phase2.ast.clause.Statement
import mathlingua.frontend.chalktalk.phase2.ast.common.Phase2Node
import mathlingua.frontend.chalktalk.phase2.ast.section.TextSection
import mathlingua.frontend.support.Location
import mathlingua.frontend.support.MutableLocationTracker
import mathlingua.frontend.support.ValidationFailure
import mathlingua.frontend.support.ValidationSuccess
import mathlingua.frontend.textalk.ColonEqualsTexTalkNode
import mathlingua.frontend.textalk.Command
import mathlingua.frontend.textalk.CommandPart
import mathlingua.frontend.textalk.ExpressionTexTalkNode
import mathlingua.frontend.textalk.InTexTalkNode
import mathlingua.frontend.textalk.IsTexTalkNode
import mathlingua.frontend.textalk.OperatorTexTalkNode
import mathlingua.frontend.textalk.TexTalkNode
import mathlingua.frontend.textalk.TexTalkNodeType
import mathlingua.frontend.textalk.TexTalkTokenType
import mathlingua.frontend.textalk.TextTexTalkNode

data class Signature(val form: String, val location: Location)

internal fun locateAllSignatures(
    node: Phase2Node, ignoreLhsEqual: Boolean, locationTracker: MutableLocationTracker
): Set<Signature> {
    val signatures = mutableSetOf<Signature>()
    findAllSignaturesImpl(
        normalize(node, locationTracker), ignoreLhsEqual, signatures, locationTracker)
    return signatures
}

// -----------------------------------------------------------------------------

private fun findAllSignaturesImpl(
    node: Phase2Node,
    ignoreLhsEqual: Boolean,
    signatures: MutableSet<Signature>,
    locationTracker: MutableLocationTracker
) {
    if (node is Statement) {
        signatures.addAll(findAllStatementSignatures(node, ignoreLhsEqual, locationTracker))
    } else if (node is IdStatement) {
        findAllSignaturesImpl(node.toStatement(), ignoreLhsEqual, signatures, locationTracker)
    } else if (node is TextSection) {
        val sig = getSignature(node)
        if (sig != null) {
            signatures.add(
                Signature(
                    form = sig, location = locationTracker.getLocationOf(node) ?: Location(-1, -1)))
        }
    }

    node.forEach { findAllSignaturesImpl(it, ignoreLhsEqual, signatures, locationTracker) }
}

private fun getSignature(section: TextSection): String? {
    val match = Regex("\\\\term\\{(.*?)\\}").find(section.text)
    if (match == null || match.groupValues.size < 2) {
        return null
    }
    return match.groupValues[1]
}

internal fun findAllStatementSignatures(
    stmt: Statement, ignoreLhsEqual: Boolean, locationTracker: MutableLocationTracker
): Set<Signature> {
    val gluedStmt = glueCommands(stmt, stmt, locationTracker).root as Statement
    return when (val rootValidation = gluedStmt.texTalkRoot
    ) {
        is ValidationSuccess -> {
            val expressionNode = rootValidation.value
            val signatures = mutableSetOf<Signature>()
            findAllSignaturesImpl(
                expressionNode,
                ignoreLhsEqual,
                false,
                signatures,
                locationTracker.getLocationOf(stmt) ?: Location(-1, -1))
            signatures
        }
        is ValidationFailure -> emptySet()
    }
}

private fun findAllSignaturesImpl(
    texTalkNode: TexTalkNode,
    ignoreLhsEqual: Boolean,
    isInLhsEqual: Boolean,
    signatures: MutableSet<Signature>,
    location: Location
) {
    if (texTalkNode is ColonEqualsTexTalkNode) {
        texTalkNode.lhs.forEach {
            findAllSignaturesImpl(it, ignoreLhsEqual, isInLhsEqual = true, signatures, location)
        }
        texTalkNode.rhs.forEach {
            findAllSignaturesImpl(it, ignoreLhsEqual, isInLhsEqual = false, signatures, location)
        }
        return
    }

    if (isInLhsEqual && ignoreLhsEqual) {
        return
    }

    if (texTalkNode is IsTexTalkNode) {
        for (expNode in texTalkNode.rhs.items) {
            val sig = getMergedCommandSignature(expNode)
            if (sig != null) {
                signatures.add(Signature(form = sig, location = location))
            }
        }
    } else if (texTalkNode is InTexTalkNode) {
        for (expNode in texTalkNode.rhs.items) {
            val sig = getMergedCommandSignature(expNode)
            if (sig != null) {
                signatures.add(Signature(form = sig, location = location))
            }
        }
    } else if (texTalkNode is Command) {
        val sig = texTalkNode.signature()
        signatures.add(Signature(form = sig, location = location))
    } else if (texTalkNode is TextTexTalkNode &&
        texTalkNode.type == TexTalkNodeType.Operator &&
        texTalkNode.text != "=") {
        signatures.add(Signature(form = texTalkNode.text, location = location))
    } else if (texTalkNode is OperatorTexTalkNode &&
        texTalkNode.lhs == null &&
        texTalkNode.command is TextTexTalkNode &&
        texTalkNode.command.tokenType == TexTalkTokenType.Operator &&
        texTalkNode.command.text != "=" &&
        texTalkNode.rhs != null) {
        signatures.add(Signature(form = texTalkNode.command.text, location = location))
    } else if (texTalkNode is OperatorTexTalkNode &&
        texTalkNode.lhs != null &&
        texTalkNode.command is TextTexTalkNode &&
        texTalkNode.command.tokenType == TexTalkTokenType.Operator &&
        texTalkNode.command.text != "=" &&
        texTalkNode.rhs == null) {
        signatures.add(Signature(form = texTalkNode.command.text, location = location))
    } else if (texTalkNode is OperatorTexTalkNode &&
        texTalkNode.lhs != null &&
        texTalkNode.command is TextTexTalkNode &&
        texTalkNode.command.tokenType == TexTalkTokenType.Operator &&
        texTalkNode.command.text != "=" &&
        texTalkNode.rhs != null) {
        signatures.add(Signature(form = texTalkNode.command.text, location = location))
    }

    texTalkNode.forEach {
        findAllSignaturesImpl(it, ignoreLhsEqual, isInLhsEqual, signatures, location)
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
        return Command(parts = commandParts, false).signature()
    }

    return null
}

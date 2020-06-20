/*
 * Copyright 2020 Google LLC
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

package mathlingua.common.chalktalk.phase2.ast.section

import mathlingua.common.*
import mathlingua.common.chalktalk.phase1.ast.Phase1Node
import mathlingua.common.chalktalk.phase2.*
import mathlingua.common.chalktalk.phase2.ast.Phase2Node
import mathlingua.common.chalktalk.phase2.ast.clause.AbstractionNode
import mathlingua.common.chalktalk.phase2.ast.clause.Target
import mathlingua.common.chalktalk.phase2.ast.clause.validateTargetList

data class ExpandsSection(val targets: List<AbstractionNode>) : Phase2Node {
    override fun forEach(fn: (node: Phase2Node) -> Unit) = targets.forEach(fn)

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter): CodeWriter {
        writer.writeIndent(isArg, indent)
        writer.writeHeader("expands")
        appendTargetArgs(writer, targets, indent + 2)
        return writer
    }

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) =
            chalkTransformer(ExpandsSection(
                    targets = targets.map { it.transform(chalkTransformer) as AbstractionNode }
            ))
}

private data class PseudoExpandsSection(val targets: List<Target>) : Phase2Node {
    override fun forEach(fn: (node: Phase2Node) -> Unit) {
        throw RuntimeException("PseudoExpandsSection.forEach() should never be invoked")
    }

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter): CodeWriter {
        throw RuntimeException("PseudoExpandsSection.toCode() should never be invoked")
    }

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node): Phase2Node {
        throw RuntimeException("PseudoExpandsSection.transform() should never be invoked")
    }
}

// only x... or {x}... is valid
private fun isValidAbstraction(node: AbstractionNode) =
    node.abstraction.subParams == null &&
    node.abstraction.parts.size == 1 &&
    node.abstraction.parts[0].subParams == null &&
    node.abstraction.parts[0].params == null &&
    ((!node.abstraction.isEnclosed &&
        node.abstraction.parts[0].name.text.endsWith("...")) ||
        (node.abstraction.isEnclosed &&
        node.abstraction.isVarArgs &&
        !node.abstraction.parts[0].name.text.endsWith("..."))
    )

fun validateExpandsSection(node: Phase1Node, tracker: MutableLocationTracker): Validation<ExpandsSection> =
    when (val validation = validateTargetList(tracker,
        node,
        "expands",
        ::PseudoExpandsSection)) {
        is ValidationFailure -> validationFailure(validation.errors)
        is ValidationSuccess -> {
            val newErrors = mutableListOf<ParseError>()
            val targets = mutableListOf<AbstractionNode>()
            for (target in validation.value.targets) {
                val id = if (target is AbstractionNode && isValidAbstraction(target)) {
                    target
                } else {
                    null
                }
                if (id == null) {
                    newErrors.add(ParseError(
                            message = "an 'expands' section can only contain <name>... or {<name>}...",
                            row = tracker.getLocationOf(target)?.row ?: -1,
                            column = tracker.getLocationOf(target)?.column ?: -1
                    ))
                } else {
                    targets.add(id)
                }
            }

            if (newErrors.isEmpty()) {
                validationSuccess(tracker, node, ExpandsSection(
                        targets = targets
                ))
            } else {
                validationFailure(newErrors)
            }
        }
    }

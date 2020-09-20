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

package mathlingua.common.chalktalk.phase2.ast

import mathlingua.common.*
import mathlingua.common.chalktalk.phase1.ast.*
import mathlingua.common.chalktalk.phase2.CodeWriter
import mathlingua.common.chalktalk.phase2.ast.toplevel.*

data class Document(
    val groups: List<TopLevelGroup>
) : Phase2Node {

    fun defines() = groups.filterIsInstance<DefinesGroup>()
    fun represents() = groups.filterIsInstance<RepresentsGroup>()
    fun theorems() = groups.filterIsInstance<TheoremGroup>()
    fun axioms() = groups.filterIsInstance<AxiomGroup>()
    fun conjectures() = groups.filterIsInstance<ConjectureGroup>()
    fun resources() = groups.filterIsInstance<ResourceGroup>()

    override fun forEach(fn: (node: Phase2Node) -> Unit) {
        groups.forEach(fn)
    }

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter): CodeWriter {
        for (grp in groups) {
            writer.append(grp, false, 0)
            writer.writeNewline(3)
        }
        return writer
    }

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node): Phase2Node {
        return chalkTransformer(
                Document(
                        groups = groups.map { it.transform(chalkTransformer) as TopLevelGroup }
                )
        )
    }
}

fun validateDocument(rawNode: Phase1Node, tracker: MutableLocationTracker): Validation<Document> {
    val node = rawNode.resolve()

    val errors = ArrayList<ParseError>()
    if (node !is Root) {
        errors.add(
            ParseError(
                "Expected a Root",
                getRow(node),
                getColumn(node)
            )
        )
        return validationFailure(errors)
    }

    val allGroups = mutableListOf<TopLevelGroup>()

    for (group in node.groups) {
        when {
            isTheoremGroup(group) -> {
                when (val resultValidation = validateTheoremGroup(group, tracker)) {
                    is ValidationSuccess -> allGroups.add(resultValidation.value)
                    is ValidationFailure -> errors.addAll(resultValidation.errors)
                }
            }
            isAxiomGroup(group) -> {
                when (val axiomValidation = validateAxiomGroup(group, tracker)) {
                    is ValidationSuccess -> allGroups.add(axiomValidation.value)
                    is ValidationFailure -> errors.addAll(axiomValidation.errors)
                }
            }
            isConjectureGroup(group) -> {
                when (val conjectureValidation = validateConjectureGroup(group, tracker)) {
                    is ValidationSuccess -> allGroups.add(conjectureValidation.value)
                    is ValidationFailure -> errors.addAll(conjectureValidation.errors)
                }
            }
            isDefinesGroup(group) -> {
                when (val definesValidation = validateDefinesGroup(group, tracker)) {
                    is ValidationSuccess -> allGroups.add(definesValidation.value)
                    is ValidationFailure -> errors.addAll(definesValidation.errors)
                }
            }
            isRepresentsGroup(group) -> {
                when (val representsValidation = validateRepresentsGroup(group, tracker)) {
                    is ValidationSuccess -> allGroups.add(representsValidation.value)
                    is ValidationFailure -> errors.addAll(representsValidation.errors)
                }
            }
            isFoundationGroup(group) -> {
                when (val foundationValidation = validateFoundationGroup(group, tracker)) {
                    is ValidationSuccess -> allGroups.add(foundationValidation.value)
                    is ValidationFailure -> errors.addAll(foundationValidation.errors)
                }
            }
            isEntryGroup(group) -> {
                when (val entryValidation = validateEntryGroup(group, tracker)) {
                    is ValidationSuccess -> allGroups.add(entryValidation.value)
                    is ValidationFailure -> errors.addAll(entryValidation.errors)
                }
            }
            isResourceGroup(group) -> {
                when (val resourceValidation = validateResourceGroup(group, tracker)) {
                    is ValidationSuccess -> allGroups.add(resourceValidation.value)
                    is ValidationFailure -> errors.addAll(resourceValidation.errors)
                }
            }
            isDefinitionGroup(group) -> {
                when (val validation = validateDefinitionGroup(group, tracker)) {
                    is ValidationSuccess -> allGroups.add(validation.value)
                    is ValidationFailure -> errors.addAll(validation.errors)
                }
            }
            isNoteGroup(group) -> {
                when (val validation = validateNoteGroup(group, tracker)) {
                    is ValidationSuccess -> allGroups.add(validation.value)
                    is ValidationFailure -> errors.addAll(validation.errors)
                }
            }
            isProblemGroup(group) -> {
                when (val validation = validateProblemGroup(group, tracker)) {
                    is ValidationSuccess -> allGroups.add(validation.value)
                    is ValidationFailure -> errors.addAll(validation.errors)
                }
            }
            else -> {
                errors.add(
                    ParseError(
                        "Expected a top level group but found " + group.toCode(),
                        getRow(group), getColumn(group)
                    )
                )
            }
        }
    }

    return if (errors.isNotEmpty()) {
        validationFailure(errors)
    } else validationSuccess(
            tracker,
            rawNode,
            Document(groups = allGroups)
    )
}

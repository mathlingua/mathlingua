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

package mathlingua.chalktalk.phase2.ast

import mathlingua.chalktalk.phase1.ast.Phase1Node
import mathlingua.chalktalk.phase1.ast.Root
import mathlingua.chalktalk.phase1.ast.getColumn
import mathlingua.chalktalk.phase1.ast.getRow
import mathlingua.chalktalk.phase2.CodeWriter
import mathlingua.chalktalk.phase2.ast.common.Phase2Node
import mathlingua.chalktalk.phase2.ast.group.toplevel.TopLevelGroup
import mathlingua.chalktalk.phase2.ast.group.toplevel.defineslike.defines.DefinesGroup
import mathlingua.chalktalk.phase2.ast.group.toplevel.defineslike.defines.isDefinesGroup
import mathlingua.chalktalk.phase2.ast.group.toplevel.defineslike.defines.neoValidateDefinesGroup
import mathlingua.chalktalk.phase2.ast.group.toplevel.defineslike.evaluates.isEvaluatesGroup
import mathlingua.chalktalk.phase2.ast.group.toplevel.defineslike.evaluates.neoValidateEvaluatesGroup
import mathlingua.chalktalk.phase2.ast.group.toplevel.defineslike.foundation.FoundationGroup
import mathlingua.chalktalk.phase2.ast.group.toplevel.defineslike.foundation.isFoundationGroup
import mathlingua.chalktalk.phase2.ast.group.toplevel.defineslike.foundation.neoValidateFoundationGroup
import mathlingua.chalktalk.phase2.ast.group.toplevel.defineslike.mutually.MutuallyGroup
import mathlingua.chalktalk.phase2.ast.group.toplevel.defineslike.mutually.isMutuallyGroup
import mathlingua.chalktalk.phase2.ast.group.toplevel.defineslike.mutually.neoValidateMutuallyGroup
import mathlingua.chalktalk.phase2.ast.group.toplevel.defineslike.states.StatesGroup
import mathlingua.chalktalk.phase2.ast.group.toplevel.defineslike.states.isStatesGroup
import mathlingua.chalktalk.phase2.ast.group.toplevel.defineslike.states.neoValidateStatesGroup
import mathlingua.chalktalk.phase2.ast.group.toplevel.defineslike.views.isViewsGroup
import mathlingua.chalktalk.phase2.ast.group.toplevel.defineslike.views.neoValidateViewsGroup
import mathlingua.chalktalk.phase2.ast.group.toplevel.entry.isEntryGroup
import mathlingua.chalktalk.phase2.ast.group.toplevel.entry.neoValidateEntryGroup
import mathlingua.chalktalk.phase2.ast.group.toplevel.resource.ResourceGroup
import mathlingua.chalktalk.phase2.ast.group.toplevel.resource.isResourceGroup
import mathlingua.chalktalk.phase2.ast.group.toplevel.resource.neoValidateResourceGroup
import mathlingua.chalktalk.phase2.ast.group.toplevel.resultlike.axiom.AxiomGroup
import mathlingua.chalktalk.phase2.ast.group.toplevel.resultlike.axiom.isAxiomGroup
import mathlingua.chalktalk.phase2.ast.group.toplevel.resultlike.axiom.neoValidateAxiomGroup
import mathlingua.chalktalk.phase2.ast.group.toplevel.resultlike.conjecture.ConjectureGroup
import mathlingua.chalktalk.phase2.ast.group.toplevel.resultlike.conjecture.isConjectureGroup
import mathlingua.chalktalk.phase2.ast.group.toplevel.resultlike.conjecture.neoValidateConjectureGroup
import mathlingua.chalktalk.phase2.ast.group.toplevel.resultlike.theorem.TheoremGroup
import mathlingua.chalktalk.phase2.ast.group.toplevel.resultlike.theorem.isTheoremGroup
import mathlingua.chalktalk.phase2.ast.group.toplevel.resultlike.theorem.neoValidateTheoremGroup
import mathlingua.support.MutableLocationTracker
import mathlingua.support.ParseError
import mathlingua.support.Validation
import mathlingua.support.validationFailure
import mathlingua.support.validationSuccess

data class Document(val groups: List<TopLevelGroup>) : Phase2Node {

    fun defines() = groups.filterIsInstance<DefinesGroup>()
    fun states() = groups.filterIsInstance<StatesGroup>()
    fun foundations() = groups.filterIsInstance<FoundationGroup>()
    fun mutually() = groups.filterIsInstance<MutuallyGroup>()
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
            Document(groups = groups.map { it.transform(chalkTransformer) as TopLevelGroup }))
    }
}

fun validateDocument(rawNode: Phase1Node, tracker: MutableLocationTracker): Validation<Document> {
    val node = rawNode.resolve()

    val errors = ArrayList<ParseError>()
    if (node !is Root) {
        errors.add(ParseError("Expected a Root", getRow(node), getColumn(node)))
        return validationFailure(errors)
    }

    val allGroups = mutableListOf<TopLevelGroup>()

    for (group in node.groups) {
        when {
            isTheoremGroup(group) -> {
                allGroups.add(neoValidateTheoremGroup(group, errors, tracker))
            }
            isAxiomGroup(group) -> {
                allGroups.add(neoValidateAxiomGroup(group, errors, tracker))
            }
            isConjectureGroup(group) -> {
                allGroups.add(neoValidateConjectureGroup(group, errors, tracker))
            }
            isDefinesGroup(group) -> {
                allGroups.add(neoValidateDefinesGroup(group, errors, tracker))
            }
            isStatesGroup(group) -> {
                allGroups.add(neoValidateStatesGroup(group, errors, tracker))
            }
            isFoundationGroup(group) -> {
                allGroups.add(neoValidateFoundationGroup(group, errors, tracker))
            }
            isViewsGroup(group) -> {
                allGroups.add(neoValidateViewsGroup(group, errors, tracker))
            }
            isMutuallyGroup(group) -> {
                allGroups.add(neoValidateMutuallyGroup(group, errors, tracker))
            }
            isEntryGroup(group) -> {
                allGroups.add(neoValidateEntryGroup(group, errors, tracker))
            }
            isResourceGroup(group) -> {
                allGroups.add(neoValidateResourceGroup(group, errors, tracker))
            }
            isEvaluatesGroup(group) -> {
                allGroups.add(neoValidateEvaluatesGroup(group, errors, tracker))
            }
            else -> {
                errors.add(
                    ParseError(
                        "Expected a top level group but found " + group.toCode(),
                        getRow(group),
                        getColumn(group)))
            }
        }
    }

    return if (errors.isNotEmpty()) {
        validationFailure(errors)
    } else validationSuccess(tracker, rawNode, Document(groups = allGroups))
}

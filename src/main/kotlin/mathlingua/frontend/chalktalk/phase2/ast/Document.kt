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

package mathlingua.frontend.chalktalk.phase2.ast

import mathlingua.backend.transform.normalize
import mathlingua.frontend.chalktalk.phase1.ast.Phase1Node
import mathlingua.frontend.chalktalk.phase1.ast.Root
import mathlingua.frontend.chalktalk.phase1.ast.getColumn
import mathlingua.frontend.chalktalk.phase1.ast.getRow
import mathlingua.frontend.chalktalk.phase2.CodeWriter
import mathlingua.frontend.chalktalk.phase2.ast.common.Phase2Node
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.TopLevelGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.defines.DefinesGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.defines.isDefinesGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.defines.validateDefinesGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.evaluates.isEvaluatesGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.evaluates.validateEvaluatesGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.foundation.FoundationGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.foundation.isFoundationGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.foundation.validateFoundationGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.mutually.MutuallyGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.mutually.isMutuallyGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.mutually.validateMutuallyGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.states.StatesGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.states.isStatesGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.states.validateStatesGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.entry.isEntryGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.entry.validateEntryGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.resource.ResourceGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.resource.isResourceGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.resource.validateResourceGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.resultlike.axiom.AxiomGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.resultlike.axiom.isAxiomGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.resultlike.axiom.validateAxiomGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.resultlike.conjecture.ConjectureGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.resultlike.conjecture.isConjectureGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.resultlike.conjecture.validateConjectureGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.resultlike.theorem.TheoremGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.resultlike.theorem.isTheoremGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.resultlike.theorem.validateTheoremGroup
import mathlingua.frontend.support.Location
import mathlingua.frontend.support.MutableLocationTracker
import mathlingua.frontend.support.ParseError
import mathlingua.frontend.support.Validation
import mathlingua.frontend.support.validationFailure
import mathlingua.frontend.support.validationSuccess
import mathlingua.frontend.textalk.newTexTalkLexer
import mathlingua.frontend.textalk.newTexTalkParser

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
                allGroups.add(validateTheoremGroup(group, errors, tracker))
            }
            isAxiomGroup(group) -> {
                allGroups.add(validateAxiomGroup(group, errors, tracker))
            }
            isConjectureGroup(group) -> {
                allGroups.add(validateConjectureGroup(group, errors, tracker))
            }
            isDefinesGroup(group) -> {
                allGroups.add(validateDefinesGroup(group, errors, tracker))
            }
            isStatesGroup(group) -> {
                allGroups.add(validateStatesGroup(group, errors, tracker))
            }
            isFoundationGroup(group) -> {
                allGroups.add(validateFoundationGroup(group, errors, tracker))
            }
            isMutuallyGroup(group) -> {
                allGroups.add(validateMutuallyGroup(group, errors, tracker))
            }
            isEntryGroup(group) -> {
                allGroups.add(validateEntryGroup(group, errors, tracker))
            }
            isResourceGroup(group) -> {
                allGroups.add(validateResourceGroup(group, errors, tracker))
            }
            isEvaluatesGroup(group) -> {
                allGroups.add(validateEvaluatesGroup(group, errors, tracker))
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

    for (group in node.groups) {
        val id = group.id
        if (id != null) {
            val lexer = newTexTalkLexer(id.text)
            val parse = newTexTalkParser().parse(lexer)
            val idBefore = parse.root.toCode()
            val idAfter =
                normalize(parse.root, Location(row = getRow(group), column = getColumn(group)))
                    .toCode()
            if (idBefore != idAfter) {
                errors.add(
                    ParseError(
                        message = "A command in an id cannot be of the form \\x \\y ...",
                        row = getRow(group),
                        column = getColumn(group)))
            }
        }
    }

    return if (errors.isNotEmpty()) {
        validationFailure(errors)
    } else validationSuccess(tracker, rawNode, Document(groups = allGroups))
}

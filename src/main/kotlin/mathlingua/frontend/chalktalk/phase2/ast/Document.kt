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
import mathlingua.frontend.chalktalk.phase1.ast.BlockComment
import mathlingua.frontend.chalktalk.phase1.ast.Group
import mathlingua.frontend.chalktalk.phase1.ast.Phase1Node
import mathlingua.frontend.chalktalk.phase1.ast.Root
import mathlingua.frontend.chalktalk.phase1.ast.getColumn
import mathlingua.frontend.chalktalk.phase1.ast.getRow
import mathlingua.frontend.chalktalk.phase2.CodeWriter
import mathlingua.frontend.chalktalk.phase2.ast.common.Phase2Node
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.TopLevelBlockComment
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.TopLevelGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.defines.DefinesGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.defines.isDefinesGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.defines.validateDefinesGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.states.StatesGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.states.isStatesGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.states.validateStatesGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.isBlockComment
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.note.NoteGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.note.isNoteGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.note.validateNoteGroup
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
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.topic.TopicGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.topic.isTopicGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.topic.validateTopicGroup
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
    fun theorems() = groups.filterIsInstance<TheoremGroup>()
    fun axioms() = groups.filterIsInstance<AxiomGroup>()
    fun conjectures() = groups.filterIsInstance<ConjectureGroup>()
    fun resources() = groups.filterIsInstance<ResourceGroup>()
    fun topics() = groups.filterIsInstance<TopicGroup>()
    fun notes() = groups.filterIsInstance<NoteGroup>()
    fun blockComments() = groups.filterIsInstance<TopLevelBlockComment>()

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
            isTopicGroup(group) -> {
                allGroups.add(validateTopicGroup(group, errors, tracker))
            }
            isNoteGroup(group) -> {
                allGroups.add(validateNoteGroup(group, errors, tracker))
            }
            isResourceGroup(group) -> {
                allGroups.add(validateResourceGroup(group, errors, tracker))
            }
            isBlockComment(group) -> {
                allGroups.add(TopLevelBlockComment(group as BlockComment))
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
        if (group is Group) {
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
    }

    return if (errors.isNotEmpty()) {
        validationFailure(errors)
    } else validationSuccess(tracker, rawNode, Document(groups = allGroups))
}

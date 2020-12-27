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

package mathlingua.chalktalk.phase2.ast.clause

import mathlingua.chalktalk.phase1.ast.Group
import mathlingua.chalktalk.phase1.ast.Phase1Node
import mathlingua.chalktalk.phase1.ast.Section
import mathlingua.chalktalk.phase2.CodeWriter
import mathlingua.chalktalk.phase2.ast.common.Phase2Node
import mathlingua.chalktalk.phase2.ast.group.clause.If.isIfGroup
import mathlingua.chalktalk.phase2.ast.group.clause.If.validateIfGroup
import mathlingua.chalktalk.phase2.ast.group.clause.and.isAndGroup
import mathlingua.chalktalk.phase2.ast.group.clause.and.validateAndGroup
import mathlingua.chalktalk.phase2.ast.group.clause.collection.isCollectionGroup
import mathlingua.chalktalk.phase2.ast.group.clause.collection.validateCollectionGroup
import mathlingua.chalktalk.phase2.ast.group.clause.exists.isExistsGroup
import mathlingua.chalktalk.phase2.ast.group.clause.exists.validateExistsGroup
import mathlingua.chalktalk.phase2.ast.group.clause.expands.isExpandsGroup
import mathlingua.chalktalk.phase2.ast.group.clause.expands.validateExpandsGroup
import mathlingua.chalktalk.phase2.ast.group.clause.forAll.isForGroup
import mathlingua.chalktalk.phase2.ast.group.clause.forAll.validateForGroup
import mathlingua.chalktalk.phase2.ast.group.clause.iff.isIffGroup
import mathlingua.chalktalk.phase2.ast.group.clause.iff.validateIffGroup
import mathlingua.chalktalk.phase2.ast.group.clause.inductively.isConstantGroup
import mathlingua.chalktalk.phase2.ast.group.clause.inductively.isConstructorGroup
import mathlingua.chalktalk.phase2.ast.group.clause.inductively.isInductivelyGroup
import mathlingua.chalktalk.phase2.ast.group.clause.inductively.validateConstantGroup
import mathlingua.chalktalk.phase2.ast.group.clause.inductively.validateConstructorGroup
import mathlingua.chalktalk.phase2.ast.group.clause.inductively.validateInductivelyGroup
import mathlingua.chalktalk.phase2.ast.group.clause.mapping.isMappingGroup
import mathlingua.chalktalk.phase2.ast.group.clause.mapping.validateMappingGroup
import mathlingua.chalktalk.phase2.ast.group.clause.matching.isMatchingGroup
import mathlingua.chalktalk.phase2.ast.group.clause.matching.validateMatchingGroup
import mathlingua.chalktalk.phase2.ast.group.clause.not.isNotGroup
import mathlingua.chalktalk.phase2.ast.group.clause.not.validateNotGroup
import mathlingua.chalktalk.phase2.ast.group.clause.or.isOrGroup
import mathlingua.chalktalk.phase2.ast.group.clause.or.validateOrGroup
import mathlingua.chalktalk.phase2.ast.group.clause.piecewise.isPiecewiseGroup
import mathlingua.chalktalk.phase2.ast.group.clause.piecewise.validatePiecewiseGroup
import mathlingua.chalktalk.phase2.ast.group.toplevel.defineslike.defines.isDefinesGroup
import mathlingua.chalktalk.phase2.ast.group.toplevel.defineslike.defines.validateDefinesGroup
import mathlingua.chalktalk.phase2.ast.group.toplevel.defineslike.evaluates.isEvaluatesGroup
import mathlingua.chalktalk.phase2.ast.group.toplevel.defineslike.evaluates.validateEvaluatesGroup
import mathlingua.chalktalk.phase2.ast.group.toplevel.defineslike.states.isStatesGroup
import mathlingua.chalktalk.phase2.ast.group.toplevel.defineslike.states.validateStatesGroup
import mathlingua.chalktalk.phase2.ast.group.toplevel.defineslike.views.isViewsGroup
import mathlingua.chalktalk.phase2.ast.group.toplevel.defineslike.views.validateViewsGroup
import mathlingua.support.MutableLocationTracker
import mathlingua.support.ParseError
import mathlingua.support.Validation

interface Clause : Phase2Node

interface Target : Clause

fun firstSectionMatchesName(node: Phase1Node, name: String): Boolean {
    if (node !is Group) {
        return false
    }

    val (sections) = node
    return if (sections.isEmpty()) {
        false
    } else sections[0].name.text == name
}

fun sectionsMatchNames(node: Phase1Node, name1: String, name2: String): Boolean {
    if (node !is Group) {
        return false
    }

    val name2Index = node.sections.indexOfFirst { it.name.text == name2 }
    return node.sections.isNotEmpty() && node.sections[0].name.text == name1 && name2Index >= 0
}

fun toCode(writer: CodeWriter, isArg: Boolean, indent: Int, phase1Node: Phase1Node): CodeWriter {
    writer.writeIndent(isArg, indent)
    writer.writePhase1Node(phase1Node)
    return writer
}

fun toCode(
    writer: CodeWriter, isArg: Boolean, indent: Int, vararg sections: Phase2Node?
): CodeWriter {
    for (i in sections.indices) {
        val sect = sections[i]
        if (sect != null) {
            writer.append(sect, isArg && i == 0, indent)
            if (i != sections.size - 1) {
                writer.writeNewline()
            }
        }
    }
    return writer
}

data class Validator(
    val name: String,
    val optional: Boolean,
    val validate: (section: Section, tracker: MutableLocationTracker) -> Validation<Phase2Node>)

private data class NeoValidationPair<T>(
    val matches: (node: Phase1Node) -> Boolean,
    val validate:
        (node: Phase1Node, errors: MutableList<ParseError>, tracker: MutableLocationTracker) -> T)

private val NEO_CLAUSE_VALIDATORS =
    listOf(
        NeoValidationPair<Clause>(::isAbstraction, ::validateAbstractionNode),
        NeoValidationPair(::isTuple, ::validateTupleNode),
        NeoValidationPair(::isAssignment, ::validateAssignmentNode),
        NeoValidationPair(::isIdentifier, ::validateIdentifier),
        NeoValidationPair(::isStatement, ::validateStatement),
        NeoValidationPair(::isText, ::validateText),
        NeoValidationPair(::isForGroup, ::validateForGroup),
        NeoValidationPair(::isExistsGroup, ::validateExistsGroup),
        NeoValidationPair(::isNotGroup, ::validateNotGroup),
        NeoValidationPair(::isOrGroup, ::validateOrGroup),
        NeoValidationPair(::isAndGroup, ::validateAndGroup),
        NeoValidationPair(::isIfGroup, ::validateIfGroup),
        NeoValidationPair(::isIffGroup, ::validateIffGroup),
        NeoValidationPair(::isExpandsGroup, ::validateExpandsGroup),
        NeoValidationPair(::isMappingGroup, ::validateMappingGroup),
        NeoValidationPair(::isCollectionGroup, ::validateCollectionGroup),
        NeoValidationPair(::isMatchingGroup, ::validateMatchingGroup),
        NeoValidationPair(::isConstantGroup, ::validateConstantGroup),
        NeoValidationPair(::isConstructorGroup, ::validateConstructorGroup),
        NeoValidationPair(::isInductivelyGroup, ::validateInductivelyGroup),
        NeoValidationPair(::isPiecewiseGroup, ::validatePiecewiseGroup),
        NeoValidationPair(::isEvaluatesGroup, ::validateEvaluatesGroup),
        NeoValidationPair(::isDefinesGroup, ::validateDefinesGroup),
        NeoValidationPair(::isStatesGroup, ::validateStatesGroup),
        NeoValidationPair(::isViewsGroup, ::validateViewsGroup))

fun validateClause(
    rawNode: Phase1Node, errors: MutableList<ParseError>, tracker: MutableLocationTracker
): Clause {
    val node = rawNode.resolve()

    for (pair in NEO_CLAUSE_VALIDATORS) {
        if (pair.matches(node)) {
            return pair.validate(node, errors, tracker) as Clause
        }
    }

    throw RuntimeException("Unrecognized clause $node")
}

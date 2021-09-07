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

package mathlingua.frontend.chalktalk.phase2.ast.clause

import mathlingua.frontend.chalktalk.phase1.ast.Group
import mathlingua.frontend.chalktalk.phase1.ast.Phase1Node
import mathlingua.frontend.chalktalk.phase1.ast.getColumn
import mathlingua.frontend.chalktalk.phase1.ast.getRow
import mathlingua.frontend.chalktalk.phase2.CodeWriter
import mathlingua.frontend.chalktalk.phase2.ast.DEFAULT_IF_GROUP
import mathlingua.frontend.chalktalk.phase2.ast.common.Phase2Node
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.If.isIfGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.If.validateIfGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.and.isAndGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.and.validateAndGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.exists.isExistsGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.exists.validateExistsGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.existsUnique.isExistsUniqueGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.existsUnique.validateExistsUniqueGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.forAll.isForGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.forAll.validateForGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.iff.isIffGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.iff.validateIffGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.not.isNotGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.not.validateNotGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.or.isOrGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.or.validateOrGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.piecewise.isPiecewiseGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.piecewise.validatePiecewiseGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.defines.isDefinesGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.defines.validateDefinesGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.states.isStatesGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.states.validateStatesGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.viewing.equality.isEqualityGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.viewing.equality.validateEqualityGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.viewing.membership.isMembershipGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.viewing.membership.validateMembershipGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.viewing.viewingas.isViewingAsGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.viewing.viewingas.validateViewingAsGroup
import mathlingua.frontend.support.MutableLocationTracker
import mathlingua.frontend.support.ParseError

interface Clause : Phase2Node

interface Target : Clause

interface DefinesStatesOrViews : Clause

fun firstSectionMatchesName(node: Phase1Node, name: String): Boolean {
    if (node !is Group) {
        return false
    }

    val (sections) = node
    return if (sections.isEmpty()) {
        false
    } else sections[0].name.text == name
}

fun secondSectionMatchesName(node: Phase1Node, name: String): Boolean {
    if (node !is Group) {
        return false
    }

    val (sections) = node
    return if (sections.size < 2) {
        false
    } else sections[1].name.text == name
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
    val nonNullSections = sections.filterNotNull()
    for (i in nonNullSections.indices) {
        val sect = nonNullSections[i]
        if (i != 0) {
            writer.writeNewline()
        }
        writer.append(sect, isArg && i == 0, indent)
    }
    return writer
}

private data class ValidationPair<T>(
    val matches: (node: Phase1Node) -> Boolean,
    val validate:
        (node: Phase1Node, errors: MutableList<ParseError>, tracker: MutableLocationTracker) -> T)

private val CLAUSE_VALIDATORS =
    listOf(
        ValidationPair<Clause>(::isAbstraction, ::validateAbstractionNode),
        ValidationPair(::isTuple, ::validateTupleNode),
        ValidationPair(::isAssignment, ::validateAssignmentNode),
        ValidationPair(::isIdentifier, ::validateIdentifier),
        ValidationPair(::isStatement, ::validateStatement),
        ValidationPair(::isText, ::validateText),
        ValidationPair(::isForGroup, ::validateForGroup),
        ValidationPair(::isExistsGroup, ::validateExistsGroup),
        ValidationPair(::isExistsUniqueGroup, ::validateExistsUniqueGroup),
        ValidationPair(::isNotGroup, ::validateNotGroup),
        ValidationPair(::isOrGroup, ::validateOrGroup),
        ValidationPair(::isAndGroup, ::validateAndGroup),
        ValidationPair(::isIfGroup, ::validateIfGroup),
        ValidationPair(::isIffGroup, ::validateIffGroup),
        ValidationPair(::isPiecewiseGroup, ::validatePiecewiseGroup),
        ValidationPair(::isDefinesGroup, ::validateDefinesGroup),
        ValidationPair(::isStatesGroup, ::validateStatesGroup),
        ValidationPair(::isMembershipGroup, ::validateMembershipGroup),
        ValidationPair(::isViewingAsGroup, ::validateViewingAsGroup),
        ValidationPair(::isEqualityGroup, ::validateEqualityGroup))

fun validateClause(
    rawNode: Phase1Node, errors: MutableList<ParseError>, tracker: MutableLocationTracker
): Clause {
    val node = rawNode.resolve()

    for (pair in CLAUSE_VALIDATORS) {
        if (pair.matches(node)) {
            return pair.validate(node, errors, tracker) as Clause
        }
    }

    errors.add(
        ParseError(
            message = "Unrecognized clause.  Perhaps there is a typo.",
            row = getRow(rawNode),
            column = getColumn(rawNode)))
    return DEFAULT_IF_GROUP
}

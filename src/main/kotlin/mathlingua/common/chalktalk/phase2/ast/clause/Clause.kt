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

package mathlingua.common.chalktalk.phase2.ast.clause

import mathlingua.common.chalktalk.phase1.ast.Phase1Node
import mathlingua.common.chalktalk.phase1.ast.Group
import mathlingua.common.chalktalk.phase1.ast.Section
import mathlingua.common.chalktalk.phase1.ast.getColumn
import mathlingua.common.chalktalk.phase1.ast.getRow
import mathlingua.common.chalktalk.phase2.CodeWriter
import mathlingua.common.chalktalk.phase2.ast.common.Phase2Node
import mathlingua.common.chalktalk.phase2.ast.group.clause.`for`.isForGroup
import mathlingua.common.chalktalk.phase2.ast.group.clause.`for`.validateForGroup
import mathlingua.common.chalktalk.phase2.ast.group.clause.`if`.isIfGroup
import mathlingua.common.chalktalk.phase2.ast.group.clause.`if`.validateIfGroup
import mathlingua.common.chalktalk.phase2.ast.group.clause.collection.isCollectionGroup
import mathlingua.common.chalktalk.phase2.ast.group.clause.collection.validateCollectionGroup
import mathlingua.common.chalktalk.phase2.ast.group.clause.exists.isExistsGroup
import mathlingua.common.chalktalk.phase2.ast.group.clause.exists.validateExistsGroup
import mathlingua.common.chalktalk.phase2.ast.group.clause.expands.isExpandsGroup
import mathlingua.common.chalktalk.phase2.ast.group.clause.expands.validateExpandsGroup
import mathlingua.common.chalktalk.phase2.ast.group.clause.iff.isIffGroup
import mathlingua.common.chalktalk.phase2.ast.group.clause.iff.validateIffGroup
import mathlingua.common.chalktalk.phase2.ast.group.clause.mapping.isMappingGroup
import mathlingua.common.chalktalk.phase2.ast.group.clause.mapping.validateMappingGroup
import mathlingua.common.chalktalk.phase2.ast.group.clause.matching.isMatchingGroup
import mathlingua.common.chalktalk.phase2.ast.group.clause.matching.validateMatchingGroup
import mathlingua.common.chalktalk.phase2.ast.group.clause.not.isNotGroup
import mathlingua.common.chalktalk.phase2.ast.group.clause.not.validateNotGroup
import mathlingua.common.chalktalk.phase2.ast.group.clause.or.isOrGroup
import mathlingua.common.chalktalk.phase2.ast.group.clause.or.validateOrGroup
import mathlingua.common.chalktalk.phase2.ast.section.identifySections
import mathlingua.common.support.MutableLocationTracker
import mathlingua.common.support.ParseError
import mathlingua.common.support.Validation
import mathlingua.common.support.ValidationFailure
import mathlingua.common.support.ValidationSuccess
import mathlingua.common.support.validationFailure
import mathlingua.common.support.validationSuccess

private data class ValidationPair<T>(
    val matches: (node: Phase1Node) -> Boolean,
    val validate: (node: Phase1Node, tracker: MutableLocationTracker) -> Validation<T>
)

interface Clause : Phase2Node
interface Target : Clause

private val CLAUSE_VALIDATORS = listOf(
        ValidationPair<Clause>(
                ::isAbstraction,
                ::validateAbstractionNode
        ),
        ValidationPair(
                ::isTuple,
                ::validateTupleNode
        ),
        ValidationPair(
                ::isAssignment,
                ::validateAssignmentNode
        ),
        ValidationPair(
                ::isIdentifier,
                ::validateIdentifier
        ),
        ValidationPair(
                ::isStatement,
                ::validateStatement
        ),
        ValidationPair(
                ::isText,
                ::validateText
        ),
        ValidationPair(
                ::isForGroup,
                ::validateForGroup
        ),
        ValidationPair(
                ::isExistsGroup,
                ::validateExistsGroup
        ),
        ValidationPair(
                ::isNotGroup,
                ::validateNotGroup
        ),
        ValidationPair(
                ::isOrGroup,
                ::validateOrGroup
        ),
        ValidationPair(
                ::isIfGroup,
                ::validateIfGroup
        ),
        ValidationPair(
                ::isIffGroup,
                ::validateIffGroup
        ),
        ValidationPair(
                ::isExpandsGroup,
                ::validateExpandsGroup
        ),
        ValidationPair(
            ::isMappingGroup,
            ::validateMappingGroup
        ),
        ValidationPair(
            ::isCollectionGroup,
            ::validateCollectionGroup
        ),
        ValidationPair(
            ::isMatchingGroup,
            ::validateMatchingGroup
        )
)

fun validateClause(rawNode: Phase1Node, tracker: MutableLocationTracker): Validation<Clause> {
    val node = rawNode.resolve()

    for (pair in CLAUSE_VALIDATORS) {
        if (pair.matches(node)) {
            return when (val validation = pair.validate(node, tracker)) {
                is ValidationSuccess -> validationSuccess(tracker, rawNode, validation.value)
                is ValidationFailure -> validationFailure(validation.errors)
            }
        }
    }

    return validationFailure(
        listOf(
            ParseError(
                "Expected a Target",
                getRow(node), getColumn(node)
            )
        )
    )
}

fun firstSectionMatchesName(node: Phase1Node, name: String): Boolean {
    if (node !is Group) {
        return false
    }

    val (sections) = node
    return if (sections.isEmpty()) {
        false
    } else sections[0].name.text == name
}

fun <G : Phase2Node, S> validateSingleSectionGroup(
    tracker: MutableLocationTracker,
    rawNode: Phase1Node,
    sectionName: String,
    buildGroup: (sect: S) -> G,
    validateSection: (section: Section, tracker: MutableLocationTracker) -> Validation<S>
): Validation<G> {
    val node = rawNode.resolve()

    val errors = ArrayList<ParseError>()
    if (node !is Group) {
        errors.add(
            ParseError(
                "Expected a Group",
                getRow(node), getColumn(node)
            )
        )
        return validationFailure(errors)
    }

    val (sections) = node
    val sectionMap: Map<String, List<Section>>
    try {
        sectionMap = identifySections(
                sections,
                sectionName
        )
    } catch (e: ParseError) {
        errors.add(ParseError(e.message, e.row, e.column))
        return validationFailure(errors)
    }

    var section: S? = null
    val sect = sectionMap[sectionName]
    when (val validation = validateSection(sect!![0], tracker)) {
        is ValidationSuccess -> section = validation.value
        is ValidationFailure -> errors.addAll(validation.errors)
    }

    return if (errors.isNotEmpty()) {
        validationFailure(errors)
    } else validationSuccess(tracker, rawNode, buildGroup(section!!))
}

fun <G : Phase2Node, S1, S2> validateDoubleSectionGroup(
    tracker: MutableLocationTracker,
    rawNode: Phase1Node,
    section1Name: String,
    validateSection1: (section: Section, tracker: MutableLocationTracker) -> Validation<S1>,
    section2Name: String,
    validateSection2: (section: Section, tracker: MutableLocationTracker) -> Validation<S2>,
    buildGroup: (sect1: S1, sect2: S2) -> G
): Validation<G> {
    val node = rawNode.resolve()

    val errors = ArrayList<ParseError>()
    if (node !is Group) {
        errors.add(
            ParseError(
                "Expected a Group",
                getRow(node), getColumn(node)
            )
        )
        return validationFailure(errors)
    }

    val (sections) = node

    val sectionMap: Map<String, List<Section>>
    try {
        sectionMap = identifySections(
                sections, section1Name, section2Name
        )
    } catch (e: ParseError) {
        errors.add(ParseError(e.message, e.row, e.column))
        return validationFailure(errors)
    }

    var section1: S1? = null
    val sect1 = sectionMap[section1Name]
    when (val section1Validation = validateSection1(sect1!![0], tracker)) {
        is ValidationSuccess -> section1 = section1Validation.value
        is ValidationFailure -> errors.addAll(section1Validation.errors)
    }

    var section2: S2? = null
    val sect2 = sectionMap[section2Name]
    when (val section2Validation = validateSection2(sect2!![0], tracker)) {
        is ValidationSuccess -> section2 = section2Validation.value
        is ValidationFailure -> errors.addAll(section2Validation.errors)
    }

    return if (errors.isNotEmpty()) {
        validationFailure(errors)
    } else validationSuccess(tracker, rawNode, buildGroup(section1!!, section2!!))
}

fun <Wrapped : Phase2Node, Base> validateWrappedNode(
    tracker: MutableLocationTracker,
    rawNode: Phase1Node,
    expectedType: String,
    checkType: (node: Phase1Node) -> Base?,
    build: (base: Base) -> Wrapped
): Validation<Wrapped> {
    val node = rawNode.resolve()

    val base = checkType(node)
    if (base == null) {
        return validationFailure(
            listOf(
                ParseError(
                    "Cannot convert ${node.toCode()} to a $expectedType",
                    getRow(node), getColumn(node)
                )
            )
        )
    }

    return validationSuccess(tracker, rawNode, build(base))
}

fun toCode(writer: CodeWriter, isArg: Boolean, indent: Int, phase1Node: Phase1Node): CodeWriter {
    writer.writeIndent(isArg, indent)
    writer.writePhase1Node(phase1Node)
    return writer
}

fun toCode(writer: CodeWriter, isArg: Boolean, indent: Int, vararg sections: Phase2Node?): CodeWriter {
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

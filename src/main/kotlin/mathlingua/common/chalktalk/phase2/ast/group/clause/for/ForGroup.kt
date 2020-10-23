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

package mathlingua.common.chalktalk.phase2.ast.group.clause.`for`

import mathlingua.common.chalktalk.phase1.ast.*
import mathlingua.common.chalktalk.phase2.ast.common.ThreePartNode
import mathlingua.common.chalktalk.phase2.ast.clause.Clause
import mathlingua.common.chalktalk.phase2.ast.clause.firstSectionMatchesName
import mathlingua.common.chalktalk.phase2.ast.group.clause.`if`.ThenSection
import mathlingua.common.chalktalk.phase2.ast.group.clause.`if`.validateThenSection
import mathlingua.common.chalktalk.phase2.ast.group.toplevel.shared.WhereSection
import mathlingua.common.chalktalk.phase2.ast.group.toplevel.shared.validateWhereSection
import mathlingua.common.chalktalk.phase2.ast.section.identifySections
import mathlingua.common.support.MutableLocationTracker
import mathlingua.common.support.ParseError
import mathlingua.common.support.Validation
import mathlingua.common.support.ValidationFailure
import mathlingua.common.support.ValidationSuccess
import mathlingua.common.support.validationFailure
import mathlingua.common.support.validationSuccess

data class ForGroup(
    val forSection: ForSection,
    val whereSection: WhereSection?,
    val thenSection: ThenSection
) : ThreePartNode<ForSection, WhereSection?, ThenSection>(
    forSection,
    whereSection,
    thenSection,
    ::ForGroup
), Clause

fun isForGroup(node: Phase1Node) = firstSectionMatchesName(node, "for")

fun validateForGroup(rawNode: Phase1Node, tracker: MutableLocationTracker): Validation<ForGroup> {
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

    val sectionMap: Map<String, Section>
    try {
        sectionMap = identifySections(
                sections,
                "for", "where?", "then"
        )
    } catch (e: ParseError) {
        errors.add(ParseError(e.message, e.row, e.column))
        return validationFailure(errors)
    }

    var forSection: ForSection? = null
    val forNode = sectionMap["for"]

    when (val forEvaluation = validateForSection(forNode!!, tracker)) {
        is ValidationSuccess -> forSection = forEvaluation.value
        is ValidationFailure -> errors.addAll(forEvaluation.errors)
    }

    var whereSection: WhereSection? = null
    if (sectionMap.containsKey("where")) {
        val where = sectionMap["where"]!!
        when (val whereValidation = validateWhereSection(where, tracker)) {
            is ValidationSuccess -> whereSection = whereValidation.value
            is ValidationFailure -> errors.addAll(whereValidation.errors)
        }
    }

    var thenSection: ThenSection? = null
    val then = sectionMap["then"]
    when (val thenValidation = validateThenSection(then!!, tracker)) {
        is ValidationSuccess -> thenSection = thenValidation.value
        is ValidationFailure -> errors.addAll(thenValidation.errors)
    }

    return if (!errors.isEmpty()) {
        validationFailure(errors)
    } else validationSuccess(tracker, rawNode, ForGroup(forSection!!, whereSection, thenSection!!))
}

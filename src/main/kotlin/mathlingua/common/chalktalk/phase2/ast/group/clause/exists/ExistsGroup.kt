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

package mathlingua.common.chalktalk.phase2.ast.group.clause.exists

import mathlingua.common.chalktalk.phase1.ast.Group
import mathlingua.common.support.MutableLocationTracker
import mathlingua.common.chalktalk.phase1.ast.Phase1Node
import mathlingua.common.chalktalk.phase1.ast.Section
import mathlingua.common.chalktalk.phase1.ast.getColumn
import mathlingua.common.chalktalk.phase1.ast.getRow
import mathlingua.common.chalktalk.phase2.ast.clause.Clause
import mathlingua.common.chalktalk.phase2.ast.clause.firstSectionMatchesName
import mathlingua.common.chalktalk.phase2.ast.common.ThreePartNode
import mathlingua.common.chalktalk.phase2.ast.group.toplevel.shared.WhereSection
import mathlingua.common.chalktalk.phase2.ast.group.toplevel.shared.validateWhereSection
import mathlingua.common.chalktalk.phase2.ast.section.identifySections
import mathlingua.common.support.ParseError
import mathlingua.common.support.Validation
import mathlingua.common.support.ValidationFailure
import mathlingua.common.support.ValidationSuccess
import mathlingua.common.support.validationFailure
import mathlingua.common.support.validationSuccess

data class ExistsGroup(
    val existsSection: ExistsSection,
    val whereSection: WhereSection?,
    val suchThatSection: SuchThatSection
) : ThreePartNode<ExistsSection, WhereSection?, SuchThatSection>(
    existsSection,
    whereSection,
    suchThatSection,
    ::ExistsGroup
), Clause

fun isExistsGroup(node: Phase1Node) = firstSectionMatchesName(node, "exists")

fun validateExistsGroup(rawNode: Phase1Node, tracker: MutableLocationTracker): Validation<ExistsGroup> {
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
            "exists", "where?", "suchThat"
        )
    } catch (e: ParseError) {
        errors.add(ParseError(e.message, e.row, e.column))
        return validationFailure(errors)
    }

    var existsSection: ExistsSection? = null
    val existsNode = sectionMap["exists"]

    when (val existsEvaluation = validateExistsSection(existsNode!!, tracker)) {
        is ValidationSuccess -> existsSection = existsEvaluation.value
        is ValidationFailure -> errors.addAll(existsEvaluation.errors)
    }

    var whereSection: WhereSection? = null
    if (sectionMap.containsKey("where")) {
        val where = sectionMap["where"]!!
        when (val whereValidation = validateWhereSection(where, tracker)) {
            is ValidationSuccess -> whereSection = whereValidation.value
            is ValidationFailure -> errors.addAll(whereValidation.errors)
        }
    }

    var suchThatSection: SuchThatSection? = null
    val suchThat = sectionMap["suchThat"]
    when (val suchThatValidation = validateSuchThatSection(suchThat!!, tracker)) {
        is ValidationSuccess -> suchThatSection = suchThatValidation.value
        is ValidationFailure -> errors.addAll(suchThatValidation.errors)
    }

    return if (errors.isNotEmpty()) {
        validationFailure(errors)
    } else validationSuccess(tracker, rawNode, ExistsGroup(
        existsSection!!, whereSection, suchThatSection!!))
}

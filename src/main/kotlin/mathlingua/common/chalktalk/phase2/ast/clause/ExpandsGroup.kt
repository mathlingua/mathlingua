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

package mathlingua.common.chalktalk.phase2.ast.clause

import mathlingua.common.chalktalk.phase1.ast.*
import mathlingua.common.chalktalk.phase2.ast.TwoPartNode
import mathlingua.common.chalktalk.phase2.ast.section.identifySections
import mathlingua.common.chalktalk.phase2.ast.section.*
import mathlingua.common.support.MutableLocationTracker
import mathlingua.common.support.ParseError
import mathlingua.common.support.Validation
import mathlingua.common.support.ValidationFailure
import mathlingua.common.support.ValidationSuccess
import mathlingua.common.support.validationFailure
import mathlingua.common.support.validationSuccess

data class ExpandsGroup(
    val expandsSection: ExpandsSection,
    val asSection: AsSection
) : TwoPartNode<ExpandsSection, AsSection>(
    expandsSection,
    asSection,
    ::ExpandsGroup
), Clause

fun isExpandsGroup(node: Phase1Node) = firstSectionMatchesName(node, "expands")

fun validateExpandsGroup(rawNode: Phase1Node, tracker: MutableLocationTracker): Validation<ExpandsGroup> {
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
                "expands", "as"
        )
    } catch (e: ParseError) {
        errors.add(ParseError(e.message, e.row, e.column))
        return validationFailure(errors)
    }

    var expandsSection: ExpandsSection? = null
    val expandsNode = sectionMap["expands"]

    when (val expandsEvaluation = validateExpandsSection(expandsNode!![0], tracker)) {
        is ValidationSuccess -> expandsSection = expandsEvaluation.value
        is ValidationFailure -> errors.addAll(expandsEvaluation.errors)
    }

    var asSection: AsSection? = null
    val asNode = sectionMap["as"]
    when (val asValidation = validateAsSection(asNode!![0], tracker)) {
        is ValidationSuccess -> asSection = asValidation.value
        is ValidationFailure -> errors.addAll(asValidation.errors)
    }

    if (expandsSection == null) {
        errors.add(
            ParseError(
                message = "Expected a 'expands:' section",
                row = getRow(rawNode),
                column = getColumn(rawNode)
        )
        )
    }

    if (asSection == null) {
        errors.add(
            ParseError(
                message = "Expected a 'as:' section",
                row = getRow(rawNode),
                column = getColumn(rawNode)
        )
        )
    }

    return if (errors.isNotEmpty()) {
        validationFailure(errors)
    } else validationSuccess(tracker, rawNode, ExpandsGroup(
            expandsSection = expandsSection!!,
            asSection = asSection!!
    ))
}

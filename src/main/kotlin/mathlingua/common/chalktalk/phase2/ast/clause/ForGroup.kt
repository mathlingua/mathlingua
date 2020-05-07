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

import mathlingua.common.*
import mathlingua.common.chalktalk.phase1.ast.*
import mathlingua.common.chalktalk.phase2.CodeWriter
import mathlingua.common.chalktalk.phase2.ast.Phase2Node
import mathlingua.common.chalktalk.phase2.ast.section.identifySections
import mathlingua.common.chalktalk.phase2.ast.section.*

data class ForGroup(
    val forSection: ForSection,
    val whereSection: WhereSection?,
    val thenSection: ThenSection
) : Clause {
    override fun forEach(fn: (node: Phase2Node) -> Unit) {
        fn(forSection)
        if (whereSection != null) {
            fn(whereSection)
        }
        fn(thenSection)
    }

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter) =
            toCode(writer, isArg, indent, forSection, whereSection, thenSection)

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) = chalkTransformer(ForGroup(
            forSection = forSection.transform(chalkTransformer) as ForSection,
            whereSection = whereSection?.transform(chalkTransformer) as WhereSection?,
            thenSection = thenSection.transform(chalkTransformer) as ThenSection
    ))
}

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

    val sectionMap: Map<String, List<Section>>
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

    when (val forEvaluation = validateForSection(forNode!![0], tracker)) {
        is ValidationSuccess -> forSection = forEvaluation.value
        is ValidationFailure -> errors.addAll(forEvaluation.errors)
    }

    var whereSection: WhereSection? = null
    if (sectionMap.containsKey("where") && sectionMap["where"]!!.isNotEmpty()) {
        val where = sectionMap["where"]!!
        when (val whereValidation = validateWhereSection(where[0], tracker)) {
            is ValidationSuccess -> whereSection = whereValidation.value
            is ValidationFailure -> errors.addAll(whereValidation.errors)
        }
    }

    var thenSection: ThenSection? = null
    val then = sectionMap["then"]
    when (val thenValidation = validateThenSection(then!![0], tracker)) {
        is ValidationSuccess -> thenSection = thenValidation.value
        is ValidationFailure -> errors.addAll(thenValidation.errors)
    }

    return if (!errors.isEmpty()) {
        validationFailure(errors)
    } else validationSuccess(tracker, rawNode, ForGroup(forSection!!, whereSection, thenSection!!))
}

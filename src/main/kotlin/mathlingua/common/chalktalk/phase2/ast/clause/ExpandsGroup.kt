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

data class ExpandsGroup(
    val expandsSection: ExpandsSection,
    val asSection: AsSection
) : Clause {
    override fun forEach(fn: (node: Phase2Node) -> Unit) {
        fn(expandsSection)
        fn(asSection)
    }

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter) =
            toCode(writer, isArg, indent, expandsSection, asSection)

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) = chalkTransformer(ExpandsGroup(
            expandsSection = expandsSection.transform(chalkTransformer) as ExpandsSection,
            asSection = asSection.transform(chalkTransformer) as AsSection
    ))
}

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
        errors.add(ParseError(
                message = "Expected a 'expands:' section",
                row = getRow(rawNode),
                column = getColumn(rawNode)
        ))
    }

    if (asSection == null) {
        errors.add(ParseError(
                message = "Expected a 'as:' section",
                row = getRow(rawNode),
                column = getColumn(rawNode)
        ))
    }

    return if (errors.isNotEmpty()) {
        validationFailure(errors)
    } else validationSuccess(tracker, rawNode, ExpandsGroup(
            expandsSection = expandsSection!!,
            asSection = asSection!!
    ))
}

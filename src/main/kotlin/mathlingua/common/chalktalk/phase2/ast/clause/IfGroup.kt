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

import mathlingua.common.support.MutableLocationTracker
import mathlingua.common.support.ParseError
import mathlingua.common.support.Validation
import mathlingua.common.support.ValidationFailure
import mathlingua.common.support.ValidationSuccess
import mathlingua.common.chalktalk.phase1.ast.Group
import mathlingua.common.chalktalk.phase1.ast.Phase1Node
import mathlingua.common.chalktalk.phase1.ast.getColumn
import mathlingua.common.chalktalk.phase1.ast.getRow
import mathlingua.common.chalktalk.phase2.CodeWriter
import mathlingua.common.chalktalk.phase2.ast.Phase2Node
import mathlingua.common.chalktalk.phase2.ast.section.ElseIfSection
import mathlingua.common.chalktalk.phase2.ast.section.ElseSection
import mathlingua.common.chalktalk.phase2.ast.section.IfSection
import mathlingua.common.chalktalk.phase2.ast.section.ThenSection
import mathlingua.common.chalktalk.phase2.ast.section.validateElseIfSection
import mathlingua.common.chalktalk.phase2.ast.section.validateElseSection
import mathlingua.common.chalktalk.phase2.ast.section.validateIfSection
import mathlingua.common.chalktalk.phase2.ast.section.validateThenSection
import mathlingua.common.support.validationFailure
import mathlingua.common.support.validationSuccess

data class ElseIfThenPair(
    val elseIfSection: ElseIfSection,
    val thenSection: ThenSection
)

data class IfGroup(
    val ifSection: IfSection,
    val thenSection: ThenSection,
    val elseIfSections: List<ElseIfThenPair>,
    val elseSection: ElseSection?
) : Clause {
    override fun forEach(fn: (node: Phase2Node) -> Unit) {
        fn(ifSection)
        fn(thenSection)
        for (elseIf in elseIfSections) {
            fn(elseIf.elseIfSection)
            fn(elseIf.thenSection)
        }
        if (elseSection != null) {
            fn(elseSection)
        }
    }

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter): CodeWriter {
        val sections = mutableListOf(ifSection, thenSection)
        for (pair in elseIfSections) {
            sections.add(pair.elseIfSection)
            sections.add(pair.thenSection)
        }
        if (elseSection != null) {
            sections.add(elseSection)
        }
        return toCode(writer, isArg, indent, *sections.toTypedArray())
    }

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) = chalkTransformer(IfGroup(
            ifSection = ifSection.transform(chalkTransformer) as IfSection,
            elseIfSections = elseIfSections.map {
                ElseIfThenPair(
                    it.elseIfSection.transform(chalkTransformer) as ElseIfSection,
                    it.thenSection.transform(chalkTransformer) as ThenSection
                )
            },
            thenSection = thenSection.transform(chalkTransformer) as ThenSection,
            elseSection = elseSection?.transform(chalkTransformer) as ElseSection?
    ))
}

fun isIfGroup(node: Phase1Node) = firstSectionMatchesName(node, "if")

fun validateIfGroup(rawNode: Phase1Node, tracker: MutableLocationTracker): Validation<IfGroup> {
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

    var ifSection: IfSection? = null
    var thenSection: ThenSection? = null
    val elseIfSections = mutableListOf<ElseIfThenPair>()
    var elseSection: ElseSection? = null

    if (node.sections.size < 2) {
        errors.add(
            ParseError(
                "Expected at least an if: and then: section",
                getRow(node), getColumn(node)
            )
        )
    } else {
        when (val validation = validateIfSection(node.sections[0], tracker)) {
            is ValidationSuccess -> ifSection = validation.value
            is ValidationFailure -> errors.addAll(validation.errors)
        }

        when (val validation = validateThenSection(node.sections[1], tracker)) {
            is ValidationSuccess -> thenSection = validation.value
            is ValidationFailure -> errors.addAll(validation.errors)
        }

        var endIndex = node.sections.size - 1
        if (node.sections.size >= 3) {
            when (val validation = validateElseSection(node.sections.last(), tracker)) {
                is ValidationSuccess -> {
                    elseSection = validation.value
                    endIndex = node.sections.size - 2
                }
            }
        }

        var i = 2
        while (i <= endIndex) {
            var elseIfSection: ElseIfSection? = null
            var elseIfThenSection: ThenSection? = null
            when (val validation = validateElseIfSection(node.sections[i], tracker)) {
                is ValidationSuccess -> {
                    elseIfSection = validation.value
                    i++
                }
                is ValidationFailure -> errors.addAll(validation.errors)
            }

            if (i < node.sections.size) {
                when (val validation = validateThenSection(node.sections[i], tracker)) {
                    is ValidationSuccess -> {
                        elseIfThenSection = validation.value
                        i++
                    }
                    is ValidationFailure -> errors.addAll(validation.errors)
                }
            }

            if (elseIfSection == null) {
                errors.add(
                    ParseError(
                        "Expected an elseif: section",
                        getRow(node), getColumn(node)
                    )
                )
            }

            if (elseIfThenSection == null) {
                errors.add(
                    ParseError(
                        "Expected a then: section",
                        getRow(node), getColumn(node)
                    )
                )
            }

            if (elseIfSection != null && elseIfThenSection != null) {
                elseIfSections.add(
                    ElseIfThenPair(
                        elseIfSection,
                        elseIfThenSection
                    )
                )
            }
        }
    }

    return if (errors.isNotEmpty()) {
        validationFailure(errors)
    } else validationSuccess(tracker, rawNode, IfGroup(
        ifSection!!,
        thenSection!!,
        elseIfSections,
        elseSection
    ))
}

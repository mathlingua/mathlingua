/*
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *    http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

package mathlingua.chalktalk.phase2.ast.group.clause.piecewise

import mathlingua.chalktalk.phase1.ast.Group
import mathlingua.chalktalk.phase1.ast.Phase1Node
import mathlingua.chalktalk.phase1.ast.getColumn
import mathlingua.chalktalk.phase1.ast.getRow
import mathlingua.chalktalk.phase2.CodeWriter
import mathlingua.chalktalk.phase2.ast.DEFAULT_PIECEWISE_GROUP
import mathlingua.chalktalk.phase2.ast.clause.Clause
import mathlingua.chalktalk.phase2.ast.clause.firstSectionMatchesName
import mathlingua.chalktalk.phase2.ast.common.Phase2Node
import mathlingua.chalktalk.phase2.ast.group.clause.If.ElseSection
import mathlingua.chalktalk.phase2.ast.group.clause.If.isElseSection
import mathlingua.chalktalk.phase2.ast.group.clause.If.validateElseSection
import mathlingua.chalktalk.phase2.ast.group.clause.WhenToPair
import mathlingua.chalktalk.phase2.ast.group.clause.mapping.ToSection
import mathlingua.chalktalk.phase2.ast.group.clause.mapping.isToSection
import mathlingua.chalktalk.phase2.ast.group.clause.mapping.validateToSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.shared.WhenSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.shared.isWhenSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.shared.validateWhenSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.topLevelToCode
import mathlingua.support.MutableLocationTracker
import mathlingua.support.ParseError
import mathlingua.support.Validation
import mathlingua.support.ValidationFailure
import mathlingua.support.ValidationSuccess
import mathlingua.support.validationFailure
import mathlingua.support.validationSuccess

data class PiecewiseGroup(
    val piecewiseSection: PiecewiseSection,
    val whenTo: List<WhenToPair>,
    val elseSection: ElseSection?
) : Clause {

    override fun forEach(fn: (node: Phase2Node) -> Unit) {
        fn(piecewiseSection)
        for (wt in whenTo) {
            fn(wt.whenSection)
            fn(wt.toSection)
        }
        if (elseSection != null) {
            fn(elseSection)
        }
    }

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter): CodeWriter {
        val sections = mutableListOf<Phase2Node?>(piecewiseSection)
        for (wt in whenTo) {
            sections.add(wt.whenSection)
            sections.add(wt.toSection)
        }
        sections.add(elseSection)
        return topLevelToCode(writer, isArg, indent, null, *sections.toTypedArray())
    }

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) =
        chalkTransformer(
            PiecewiseGroup(
                piecewiseSection = piecewiseSection.transform(chalkTransformer) as PiecewiseSection,
                whenTo =
                    whenTo.map {
                        WhenToPair(
                            whenSection = chalkTransformer(it.whenSection) as WhenSection,
                            toSection = chalkTransformer(it.toSection) as ToSection)
                    },
                elseSection = elseSection?.transform(chalkTransformer) as ElseSection?))
}

fun isPiecewiseGroup(node: Phase1Node) = firstSectionMatchesName(node, "piecewise")

fun validatePiecewiseGroup(
    rawNode: Phase1Node, tracker: MutableLocationTracker
): Validation<PiecewiseGroup> {
    val node = rawNode.resolve()

    val errors = ArrayList<ParseError>()
    if (node !is Group) {
        errors.add(ParseError("Expected a Group", getRow(node), getColumn(node)))
        return validationFailure(errors)
    }

    val row = getRow(node)
    val column = getColumn(node)

    val whenToList = mutableListOf<WhenToPair>()
    var elseSection: ElseSection? = null
    if (node.sections.isEmpty()) {
        errors.add(
            ParseError(message = "Expected an piecewise: section", row = row, column = column))
    } else {
        var i = 1
        while (i < node.sections.size) {
            val sec = node.sections[i]
            if (!isWhenSection(sec)) {
                break
            }
            i++

            var whenSection: WhenSection? = null
            when (val validation = validateWhenSection(sec, tracker)
            ) {
                is ValidationSuccess -> whenSection = validation.value
                is ValidationFailure -> errors.addAll(validation.errors)
            }

            if (i >= node.sections.size || !isToSection(node.sections[i])) {
                errors.add(
                    ParseError(
                        message = "A when: section must have an to: section",
                        row = getRow(sec),
                        column = getColumn(sec)))
                break
            }

            var toSection: ToSection? = null
            when (val validation = validateToSection(node.sections[i], tracker)
            ) {
                is ValidationSuccess -> {
                    i++
                    toSection = validation.value
                }
                is ValidationFailure -> errors.addAll(validation.errors)
            }

            if (whenSection != null && toSection != null) {
                whenToList.add(WhenToPair(whenSection, toSection))
            }
        }

        if (i < node.sections.size && isElseSection(node.sections[i])) {
            when (val validation = validateElseSection(node.sections[i], tracker)
            ) {
                is ValidationSuccess -> {
                    i++
                    elseSection = validation.value
                }
                is ValidationFailure -> errors.addAll(validation.errors)
            }
        }

        while (i < node.sections.size) {
            val sec = node.sections[i++]
            errors.add(
                ParseError(
                    message = "Unexpected section ${sec.name.text}",
                    row = getRow(sec),
                    column = getColumn(sec)))
        }
    }

    return if (errors.isNotEmpty()) {
        validationFailure(errors)
    } else {
        validationSuccess(
            PiecewiseGroup(piecewiseSection = PiecewiseSection(), whenTo = whenToList, elseSection))
    }
}

fun neoValidatePiecewiseGroup(
    node: Phase1Node, errors: MutableList<ParseError>, tracker: MutableLocationTracker
) =
    when (val validation = validatePiecewiseGroup(node, tracker)
    ) {
        is ValidationSuccess -> validation.value
        is ValidationFailure -> {
            errors.addAll(validation.errors)
            DEFAULT_PIECEWISE_GROUP
        }
    }

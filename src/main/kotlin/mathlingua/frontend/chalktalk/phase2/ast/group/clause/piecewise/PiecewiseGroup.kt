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

package mathlingua.frontend.chalktalk.phase2.ast.group.clause.piecewise

import mathlingua.frontend.chalktalk.phase1.ast.Phase1Node
import mathlingua.frontend.chalktalk.phase2.CodeWriter
import mathlingua.frontend.chalktalk.phase2.ast.DEFAULT_PIECEWISE_GROUP
import mathlingua.frontend.chalktalk.phase2.ast.clause.Clause
import mathlingua.frontend.chalktalk.phase2.ast.clause.firstSectionMatchesName
import mathlingua.frontend.chalktalk.phase2.ast.common.Phase2Node
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.If.ElseSection
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.If.ThenSection
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.If.isElseSection
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.If.isThenSection
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.If.validateElseSection
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.If.validateThenSection
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.WhenThenPair
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.WhenSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.isWhenSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.validateWhenSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.topLevelToCode
import mathlingua.frontend.chalktalk.phase2.ast.validateGroup
import mathlingua.frontend.support.ParseError

internal data class PiecewiseGroup(
    val piecewiseSection: PiecewiseSection,
    val whenThen: List<WhenThenPair>,
    val elseSection: ElseSection?,
    override val row: Int,
    override val column: Int
) : Clause {

    override fun forEach(fn: (node: Phase2Node) -> Unit) {
        fn(piecewiseSection)
        for (wt in whenThen) {
            fn(wt.whenSection)
            fn(wt.thenSection)
        }
        if (elseSection != null) {
            fn(elseSection)
        }
    }

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter): CodeWriter {
        val sections = mutableListOf<Phase2Node?>(piecewiseSection)
        for (wt in whenThen) {
            sections.add(wt.whenSection)
            sections.add(wt.thenSection)
        }
        sections.add(elseSection)
        return topLevelToCode(null, writer, isArg, indent, null, *sections.toTypedArray())
    }

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) =
        chalkTransformer(
            PiecewiseGroup(
                piecewiseSection = piecewiseSection.transform(chalkTransformer) as PiecewiseSection,
                whenThen =
                    whenThen.map {
                        WhenThenPair(
                            whenSection = chalkTransformer(it.whenSection) as WhenSection,
                            thenSection = chalkTransformer(it.thenSection) as ThenSection)
                    },
                elseSection = elseSection?.transform(chalkTransformer) as ElseSection?,
                row,
                column))
}

internal fun isPiecewiseGroup(node: Phase1Node) = firstSectionMatchesName(node, "piecewise")

internal fun validatePiecewiseGroup(node: Phase1Node, errors: MutableList<ParseError>) =
    validateGroup(node.resolve(), errors, "piecewise", DEFAULT_PIECEWISE_GROUP) { group ->
        if (group.sections.isEmpty() || !isPiecewiseSection(group.sections.first())) {
            errors.add(
                ParseError(
                    message = "Expected an piecewise: section",
                    row = node.row,
                    column = node.column))
            return@validateGroup DEFAULT_PIECEWISE_GROUP
        }

        val piecewiseSection = validatePiecewiseSection(group.sections.first(), errors)
        val whenToList = mutableListOf<WhenThenPair>()
        var elseSection: ElseSection? = null

        var i = 1
        while (i < group.sections.size) {
            val sec = group.sections[i]
            if (!isWhenSection(sec)) {
                break
            }
            i++

            val whenSection = validateWhenSection(sec, errors)
            if (i >= group.sections.size || !isThenSection(group.sections[i])) {
                errors.add(
                    ParseError(
                        message = "A when: section must have an then: section",
                        row = sec.row,
                        column = sec.column))
                break
            }

            val thenSection = validateThenSection(group.sections[i++], errors)
            whenToList.add(WhenThenPair(whenSection, thenSection))
        }

        if (i < group.sections.size && isElseSection(group.sections[i])) {
            elseSection = validateElseSection(group.sections[i++], errors)
        }

        while (i < group.sections.size) {
            val sec = group.sections[i++]
            errors.add(
                ParseError(
                    message = "Unexpected section ${sec.name.text}",
                    row = sec.row,
                    column = sec.column))
        }

        PiecewiseGroup(
            piecewiseSection = piecewiseSection,
            whenThen = whenToList,
            elseSection = elseSection,
            row = node.row,
            column = node.column)
    }

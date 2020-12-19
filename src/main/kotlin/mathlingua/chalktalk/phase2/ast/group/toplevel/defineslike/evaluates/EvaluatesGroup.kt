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

package mathlingua.chalktalk.phase2.ast.group.toplevel.defineslike.evaluates

import mathlingua.chalktalk.phase1.ast.Phase1Node
import mathlingua.chalktalk.phase1.ast.getColumn
import mathlingua.chalktalk.phase1.ast.getRow
import mathlingua.chalktalk.phase2.CodeWriter
import mathlingua.chalktalk.phase2.ast.DEFAULT_EVALUATES_GROUP
import mathlingua.chalktalk.phase2.ast.DEFAULT_EVALUATES_SECTION
import mathlingua.chalktalk.phase2.ast.DEFAULT_ID_STATEMENT
import mathlingua.chalktalk.phase2.ast.clause.IdStatement
import mathlingua.chalktalk.phase2.ast.clause.firstSectionMatchesName
import mathlingua.chalktalk.phase2.ast.common.Phase2Node
import mathlingua.chalktalk.phase2.ast.group.clause.If.ElseSection
import mathlingua.chalktalk.phase2.ast.group.clause.If.isElseSection
import mathlingua.chalktalk.phase2.ast.group.clause.If.neoValidateElseSection
import mathlingua.chalktalk.phase2.ast.group.clause.WhenToPair
import mathlingua.chalktalk.phase2.ast.group.clause.mapping.ToSection
import mathlingua.chalktalk.phase2.ast.group.clause.mapping.isToSection
import mathlingua.chalktalk.phase2.ast.group.clause.mapping.neoValidateToSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.TopLevelGroup
import mathlingua.chalktalk.phase2.ast.group.toplevel.defineslike.WrittenSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.defineslike.foundation.DefinesStatesOrViews
import mathlingua.chalktalk.phase2.ast.group.toplevel.defineslike.isWrittenSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.defineslike.neoValidateWrittenSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.shared.UsingSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.shared.WhenSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.shared.isUsingSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.shared.isWhenSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.shared.metadata.section.MetaDataSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.shared.metadata.section.isMetadataSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.shared.metadata.section.neoValidateMetaDataSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.shared.neoValidateUsingSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.shared.neoValidateWhenSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.topLevelToCode
import mathlingua.chalktalk.phase2.ast.neoGetId
import mathlingua.chalktalk.phase2.ast.neoTrack
import mathlingua.chalktalk.phase2.ast.neoValidateGroup
import mathlingua.chalktalk.phase2.ast.section.neoEnsureNonNull
import mathlingua.support.MutableLocationTracker
import mathlingua.support.ParseError
import mathlingua.transform.signature

data class EvaluatesGroup(
    val signature: String?,
    val id: IdStatement,
    val evaluatesSection: EvaluatesSection,
    val whenTo: List<WhenToPair>,
    val elseSection: ElseSection,
    val usingSection: UsingSection?,
    val writtenSection: WrittenSection?,
    override val metaDataSection: MetaDataSection?
) : TopLevelGroup(metaDataSection), DefinesStatesOrViews {

    override fun forEach(fn: (node: Phase2Node) -> Unit) {
        fn(id)
        fn(evaluatesSection)
        for (wt in whenTo) {
            fn(wt.whenSection)
            fn(wt.toSection)
        }
        fn(elseSection)
        if (usingSection != null) {
            fn(usingSection)
        }
        if (writtenSection != null) {
            fn(writtenSection)
        }
        if (metaDataSection != null) {
            fn(metaDataSection)
        }
    }

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter): CodeWriter {
        val sections = mutableListOf<Phase2Node?>(evaluatesSection)
        for (wt in whenTo) {
            sections.add(wt.whenSection)
            sections.add(wt.toSection)
        }
        sections.add(elseSection)
        sections.add(usingSection)
        sections.add(writtenSection)
        sections.add(metaDataSection)
        return topLevelToCode(writer, isArg, indent, id, *sections.toTypedArray())
    }

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) =
        chalkTransformer(
            EvaluatesGroup(
                signature = signature,
                id = id.transform(chalkTransformer) as IdStatement,
                evaluatesSection = evaluatesSection.transform(chalkTransformer) as EvaluatesSection,
                whenTo =
                    whenTo.map {
                        WhenToPair(
                            whenSection = chalkTransformer(it.whenSection) as WhenSection,
                            toSection = chalkTransformer(it.toSection) as ToSection)
                    },
                elseSection = elseSection.transform(chalkTransformer) as ElseSection,
                usingSection = usingSection?.transform(chalkTransformer) as UsingSection?,
                writtenSection = writtenSection?.transform(chalkTransformer) as WrittenSection?,
                metaDataSection = metaDataSection?.transform(chalkTransformer) as MetaDataSection?))
}

fun isEvaluatesGroup(node: Phase1Node) = firstSectionMatchesName(node, "Evaluates")

fun neoValidateEvaluatesGroup(node: Phase1Node, errors: MutableList<ParseError>, tracker: MutableLocationTracker) =
    neoTrack(node, tracker) {
        neoValidateGroup(node, errors, "Evaluates", DEFAULT_EVALUATES_GROUP) { group ->
            val id = neoGetId(group, errors, DEFAULT_ID_STATEMENT, tracker)
            if (group.sections.isEmpty() || !isEvaluatesSection(group.sections.first())) {
                errors.add(
                    ParseError(
                        message = "Expected an Evaluates section",
                        row = getRow(node),
                        column = getColumn(node)
                    )
                )
                DEFAULT_EVALUATES_GROUP
            } else {
                val startErrorCount = errors.size
                val whenToList = mutableListOf<WhenToPair>()
                var elseSection: ElseSection? = null
                var usingSection: UsingSection? = null
                var metaDataSection: MetaDataSection? = null
                var writtenSection: WrittenSection? = null

                var i = 1
                while (i < group.sections.size) {
                    val sec = group.sections[i]
                    if (!isWhenSection(sec)) {
                        break
                    }
                    i++

                    val whenSection = neoValidateWhenSection(sec, errors, tracker)
                    if (i >= group.sections.size || !isToSection(group.sections[i])) {
                        errors.add(
                            ParseError(
                                message = "A when: section must have an to: section",
                                row = getRow(sec),
                                column = getColumn(sec)))
                        break
                    }

                    val toSection = neoValidateToSection(group.sections[i++], errors, tracker)
                    whenToList.add(WhenToPair(whenSection, toSection))
                }

                if (i < group.sections.size && isElseSection(group.sections[i])) {
                    val errorCount = errors.size
                    elseSection = neoValidateElseSection(group.sections[i], errors, tracker)
                    if (errorCount == errors.size) {
                        i++
                    }
                }

                if (i < group.sections.size && isUsingSection(group.sections[i])) {
                    val errorCount = errors.size
                    usingSection = neoValidateUsingSection(group.sections[i], errors, tracker)
                    if (errorCount == errors.size) {
                        i++
                    }
                }

                if (i < group.sections.size && isWrittenSection(group.sections[i])) {
                    val errorCount = errors.size
                    writtenSection = neoValidateWrittenSection(group.sections[i], errors, tracker)
                    if (errorCount == errors.size) {
                        i++
                    }
                }

                if (i < group.sections.size && isMetadataSection(group.sections[i])) {
                    metaDataSection = neoValidateMetaDataSection(group.sections[i++], errors, tracker)
                }

                while (i < group.sections.size) {
                    val sec = group.sections[i++]
                    errors.add(
                        ParseError(
                            message = "Unexpected section ${sec.name.text}",
                            row = getRow(sec),
                            column = getColumn(sec)))
                }

                if (elseSection == null) {
                    errors.add(
                        ParseError(
                            message = "Expected an else: section",
                            row = getRow(node),
                            column = getColumn(node)
                        ))
                }

                if (startErrorCount != errors.size) {
                    DEFAULT_EVALUATES_GROUP
                } else {
                    EvaluatesGroup(
                        signature = id.signature(),
                        id = id,
                        evaluatesSection = neoEnsureNonNull(group.sections[0], DEFAULT_EVALUATES_SECTION) {
                            neoValidateEvaluatesSection(it, errors, tracker)
                        },
                        whenTo = whenToList,
                        elseSection = elseSection!!,
                        usingSection = usingSection,
                        writtenSection = writtenSection,
                        metaDataSection = metaDataSection
                    )
                }
            }
        }
    }

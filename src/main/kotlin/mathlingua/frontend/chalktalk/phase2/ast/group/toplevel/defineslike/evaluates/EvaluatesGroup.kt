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

package mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.evaluates

import mathlingua.backend.transform.signature
import mathlingua.frontend.chalktalk.phase1.ast.Phase1Node
import mathlingua.frontend.chalktalk.phase1.ast.getColumn
import mathlingua.frontend.chalktalk.phase1.ast.getRow
import mathlingua.frontend.chalktalk.phase2.CodeWriter
import mathlingua.frontend.chalktalk.phase2.ast.DEFAULT_EVALUATES_GROUP
import mathlingua.frontend.chalktalk.phase2.ast.DEFAULT_EVALUATES_SECTION
import mathlingua.frontend.chalktalk.phase2.ast.DEFAULT_ID_STATEMENT
import mathlingua.frontend.chalktalk.phase2.ast.clause.IdStatement
import mathlingua.frontend.chalktalk.phase2.ast.clause.firstSectionMatchesName
import mathlingua.frontend.chalktalk.phase2.ast.common.Phase2Node
import mathlingua.frontend.chalktalk.phase2.ast.getId
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.If.ElseSection
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.If.ThenSection
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.If.isElseSection
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.If.isThenSection
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.If.validateElseSection
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.If.validateThenSection
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.WhenThenPair
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.TopLevelGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.WrittenSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.foundation.DefinesStatesOrViews
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.isWrittenSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.validateWrittenSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.UsingSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.WhenSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.isUsingSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.isWhenSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.metadata.section.MetaDataSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.metadata.section.isMetadataSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.metadata.section.validateMetaDataSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.validateUsingSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.validateWhenSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.topLevelToCode
import mathlingua.frontend.chalktalk.phase2.ast.section.ensureNonNull
import mathlingua.frontend.chalktalk.phase2.ast.track
import mathlingua.frontend.chalktalk.phase2.ast.validateGroup
import mathlingua.frontend.support.MutableLocationTracker
import mathlingua.frontend.support.ParseError

data class EvaluatesGroup(
    val signature: String?,
    val id: IdStatement,
    val evaluatesSection: EvaluatesSection,
    val whenThen: List<WhenThenPair>,
    val elseSection: ElseSection,
    val usingSection: UsingSection?,
    val writtenSection: WrittenSection?,
    override val metaDataSection: MetaDataSection?
) : TopLevelGroup(metaDataSection), DefinesStatesOrViews {

    override fun forEach(fn: (node: Phase2Node) -> Unit) {
        fn(id)
        fn(evaluatesSection)
        for (wt in whenThen) {
            fn(wt.whenSection)
            fn(wt.thenSection)
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
        for (wt in whenThen) {
            sections.add(wt.whenSection)
            sections.add(wt.thenSection)
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
                whenThen =
                    whenThen.map {
                        WhenThenPair(
                            whenSection = chalkTransformer(it.whenSection) as WhenSection,
                            thenSection = chalkTransformer(it.thenSection) as ThenSection)
                    },
                elseSection = elseSection.transform(chalkTransformer) as ElseSection,
                usingSection = usingSection?.transform(chalkTransformer) as UsingSection?,
                writtenSection = writtenSection?.transform(chalkTransformer) as WrittenSection?,
                metaDataSection = metaDataSection?.transform(chalkTransformer) as MetaDataSection?))
}

fun isEvaluatesGroup(node: Phase1Node) = firstSectionMatchesName(node, "Evaluates")

fun validateEvaluatesGroup(
    node: Phase1Node, errors: MutableList<ParseError>, tracker: MutableLocationTracker
) =
    track(node, tracker) {
        validateGroup(node, errors, "Evaluates", DEFAULT_EVALUATES_GROUP) { group ->
            val id = getId(group, errors, DEFAULT_ID_STATEMENT, tracker)
            if (group.sections.isEmpty() || !isEvaluatesSection(group.sections.first())) {
                errors.add(
                    ParseError(
                        message = "Expected an Evaluates section",
                        row = getRow(node),
                        column = getColumn(node)))
                DEFAULT_EVALUATES_GROUP
            } else {
                val startErrorCount = errors.size
                val whenToList = mutableListOf<WhenThenPair>()
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

                    val whenSection = validateWhenSection(sec, errors, tracker)
                    if (i >= group.sections.size || !isThenSection(group.sections[i])) {
                        errors.add(
                            ParseError(
                                message = "A when: section must have an then: section",
                                row = getRow(sec),
                                column = getColumn(sec)))
                        break
                    }

                    val thenSection = validateThenSection(group.sections[i++], errors, tracker)
                    whenToList.add(WhenThenPair(whenSection, thenSection))
                }

                if (i < group.sections.size && isElseSection(group.sections[i])) {
                    val errorCount = errors.size
                    elseSection = validateElseSection(group.sections[i], errors, tracker)
                    if (errorCount == errors.size) {
                        i++
                    }
                }

                if (i < group.sections.size && isUsingSection(group.sections[i])) {
                    val errorCount = errors.size
                    usingSection = validateUsingSection(group.sections[i], errors, tracker)
                    if (errorCount == errors.size) {
                        i++
                    }
                }

                if (i < group.sections.size && isWrittenSection(group.sections[i])) {
                    val errorCount = errors.size
                    writtenSection = validateWrittenSection(group.sections[i], errors, tracker)
                    if (errorCount == errors.size) {
                        i++
                    }
                }

                if (i < group.sections.size && isMetadataSection(group.sections[i])) {
                    metaDataSection = validateMetaDataSection(group.sections[i++], errors, tracker)
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
                            column = getColumn(node)))
                }

                if (startErrorCount != errors.size) {
                    DEFAULT_EVALUATES_GROUP
                } else {
                    EvaluatesGroup(
                        signature = id.signature(),
                        id = id,
                        evaluatesSection =
                            ensureNonNull(group.sections[0], DEFAULT_EVALUATES_SECTION) {
                                validateEvaluatesSection(it, errors, tracker)
                            },
                        whenThen = whenToList,
                        elseSection = elseSection!!,
                        usingSection = usingSection,
                        writtenSection = writtenSection,
                        metaDataSection = metaDataSection)
                }
            }
        }
    }

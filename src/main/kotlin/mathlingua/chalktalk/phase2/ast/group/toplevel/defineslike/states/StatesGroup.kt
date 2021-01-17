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

package mathlingua.chalktalk.phase2.ast.group.toplevel.defineslike.states

import mathlingua.chalktalk.phase1.ast.Phase1Node
import mathlingua.chalktalk.phase2.CodeWriter
import mathlingua.chalktalk.phase2.ast.DEFAULT_ID_STATEMENT
import mathlingua.chalktalk.phase2.ast.DEFAULT_STATES_GROUP
import mathlingua.chalktalk.phase2.ast.DEFAULT_STATES_SECTION
import mathlingua.chalktalk.phase2.ast.DEFAULT_THAT_SECTION
import mathlingua.chalktalk.phase2.ast.clause.IdStatement
import mathlingua.chalktalk.phase2.ast.clause.firstSectionMatchesName
import mathlingua.chalktalk.phase2.ast.common.Phase2Node
import mathlingua.chalktalk.phase2.ast.getId
import mathlingua.chalktalk.phase2.ast.group.toplevel.TopLevelGroup
import mathlingua.chalktalk.phase2.ast.group.toplevel.defineslike.WrittenSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.defineslike.foundation.DefinesStatesOrViews
import mathlingua.chalktalk.phase2.ast.group.toplevel.defineslike.validateWrittenSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.shared.UsingSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.shared.WhenSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.shared.metadata.section.MetaDataSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.shared.metadata.section.validateMetaDataSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.shared.validateUsingSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.shared.validateWhenSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.topLevelToCode
import mathlingua.chalktalk.phase2.ast.section.ensureNonNull
import mathlingua.chalktalk.phase2.ast.section.identifySections
import mathlingua.chalktalk.phase2.ast.section.ifNonNull
import mathlingua.chalktalk.phase2.ast.track
import mathlingua.chalktalk.phase2.ast.validateGroup
import mathlingua.support.MutableLocationTracker
import mathlingua.support.ParseError
import mathlingua.transform.signature

data class StatesGroup(
    val signature: String?,
    val id: IdStatement,
    val statesSection: StatesSection,
    val whenSection: WhenSection?,
    val thatSection: ThatSection,
    val usingSection: UsingSection?,
    val writtenSection: WrittenSection?,
    override val metaDataSection: MetaDataSection?
) : TopLevelGroup(metaDataSection), DefinesStatesOrViews {

    override fun forEach(fn: (node: Phase2Node) -> Unit) {
        fn(id)
        fn(statesSection)
        if (whenSection != null) {
            fn(whenSection)
        }
        fn(thatSection)
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
        val sections = mutableListOf(statesSection, whenSection)
        sections.add(thatSection)
        sections.add(usingSection)
        sections.add(writtenSection)
        sections.add(metaDataSection)
        return topLevelToCode(writer, isArg, indent, id, *sections.toTypedArray())
    }

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) =
        chalkTransformer(
            StatesGroup(
                signature = signature,
                id = id.transform(chalkTransformer) as IdStatement,
                statesSection = statesSection.transform(chalkTransformer) as StatesSection,
                whenSection = whenSection?.transform(chalkTransformer) as WhenSection?,
                thatSection = chalkTransformer(thatSection) as ThatSection,
                usingSection = usingSection?.transform(chalkTransformer) as UsingSection?,
                writtenSection = writtenSection?.transform(chalkTransformer) as WrittenSection?,
                metaDataSection = metaDataSection?.transform(chalkTransformer) as MetaDataSection?))
}

fun isStatesGroup(node: Phase1Node) = firstSectionMatchesName(node, "States")

fun validateStatesGroup(
    node: Phase1Node, errors: MutableList<ParseError>, tracker: MutableLocationTracker
) =
    track(node, tracker) {
        validateGroup(node.resolve(), errors, "States", DEFAULT_STATES_GROUP) { group ->
            identifySections(
                group,
                errors,
                DEFAULT_STATES_GROUP,
                listOf("States", "when?", "that", "using?", "written?", "Metadata?")) { sections ->
                val id = getId(group, errors, DEFAULT_ID_STATEMENT, tracker)
                StatesGroup(
                    signature = id.signature(),
                    id = id,
                    statesSection =
                        ensureNonNull(sections["States"], DEFAULT_STATES_SECTION) {
                            validateStatesSection(it, errors, tracker)
                        },
                    whenSection =
                        ifNonNull(sections["when"]) { validateWhenSection(it, errors, tracker) },
                    thatSection =
                        ensureNonNull(sections["that"], DEFAULT_THAT_SECTION) {
                            validateThatSection(it, errors, tracker)
                        },
                    usingSection =
                        ifNonNull(sections["using"]) { validateUsingSection(it, errors, tracker) },
                    writtenSection =
                        ifNonNull(sections["written"]) {
                            validateWrittenSection(it, errors, tracker)
                        },
                    metaDataSection =
                        ifNonNull(sections["Metadata"]) {
                            validateMetaDataSection(it, errors, tracker)
                        })
            }
        }
    }

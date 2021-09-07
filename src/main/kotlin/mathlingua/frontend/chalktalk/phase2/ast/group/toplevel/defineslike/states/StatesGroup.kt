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

package mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.states

import mathlingua.backend.transform.Signature
import mathlingua.backend.transform.signature
import mathlingua.frontend.chalktalk.phase1.ast.Phase1Node
import mathlingua.frontend.chalktalk.phase2.CodeWriter
import mathlingua.frontend.chalktalk.phase2.ast.DEFAULT_CALLED_SECTION
import mathlingua.frontend.chalktalk.phase2.ast.DEFAULT_STATES_GROUP
import mathlingua.frontend.chalktalk.phase2.ast.DEFAULT_STATES_SECTION
import mathlingua.frontend.chalktalk.phase2.ast.DEFAULT_THAT_SECTION
import mathlingua.frontend.chalktalk.phase2.ast.DEFAULT_WRITTEN_SECTION
import mathlingua.frontend.chalktalk.phase2.ast.clause.DefinesStatesOrViews
import mathlingua.frontend.chalktalk.phase2.ast.clause.IdStatement
import mathlingua.frontend.chalktalk.phase2.ast.clause.firstSectionMatchesName
import mathlingua.frontend.chalktalk.phase2.ast.common.Phase2Node
import mathlingua.frontend.chalktalk.phase2.ast.getId
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.HasSignature
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.HasUsingSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.TopLevelGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.CalledSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.WrittenSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.defines.RequiringSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.defines.validateRequiringSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.validateCalledSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.validateWrittenSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.UsingSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.WhenSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.metadata.section.MetaDataSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.metadata.section.validateMetaDataSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.validateUsingSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.validateWhenSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.topLevelToCode
import mathlingua.frontend.chalktalk.phase2.ast.section.ensureNonNull
import mathlingua.frontend.chalktalk.phase2.ast.section.identifySections
import mathlingua.frontend.chalktalk.phase2.ast.section.ifNonNull
import mathlingua.frontend.chalktalk.phase2.ast.track
import mathlingua.frontend.chalktalk.phase2.ast.validateGroup
import mathlingua.frontend.support.MutableLocationTracker
import mathlingua.frontend.support.ParseError

data class StatesGroup(
    override val signature: Signature?,
    override val id: IdStatement,
    val statesSection: StatesSection,
    val requiringSection: RequiringSection?,
    val whenSection: WhenSection?,
    val thatSection: ThatSection,
    override val usingSection: UsingSection?,
    val writtenSection: WrittenSection,
    val calledSection: CalledSection,
    override val metaDataSection: MetaDataSection?
) : TopLevelGroup(metaDataSection), HasUsingSection, HasSignature, DefinesStatesOrViews {

    override fun forEach(fn: (node: Phase2Node) -> Unit) {
        fn(id)
        fn(statesSection)
        if (requiringSection != null) {
            fn(requiringSection)
        }
        if (whenSection != null) {
            fn(whenSection)
        }
        fn(thatSection)
        if (usingSection != null) {
            fn(usingSection)
        }
        fn(writtenSection)
        fn(calledSection)
        if (metaDataSection != null) {
            fn(metaDataSection)
        }
    }

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter): CodeWriter {
        val sections = mutableListOf(statesSection, requiringSection, whenSection)
        sections.add(thatSection)
        sections.add(usingSection)
        sections.add(writtenSection)
        sections.add(calledSection)
        sections.add(metaDataSection)
        return topLevelToCode(this, writer, isArg, indent, id, *sections.toTypedArray())
    }

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) =
        chalkTransformer(
            StatesGroup(
                signature = signature,
                id = id.transform(chalkTransformer) as IdStatement,
                statesSection = statesSection.transform(chalkTransformer) as StatesSection,
                requiringSection =
                    requiringSection?.transform(chalkTransformer) as RequiringSection?,
                whenSection = whenSection?.transform(chalkTransformer) as WhenSection?,
                thatSection = chalkTransformer(thatSection) as ThatSection,
                usingSection = usingSection?.transform(chalkTransformer) as UsingSection?,
                writtenSection = writtenSection.transform(chalkTransformer) as WrittenSection,
                calledSection = calledSection.transform(chalkTransformer) as CalledSection,
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
                listOf(
                    "States",
                    "requiring?",
                    "when?",
                    "that",
                    "using?",
                    "written",
                    "called",
                    "Metadata?")) { sections ->
                val id = getId(group, errors, tracker)
                StatesGroup(
                    signature = id.signature(tracker),
                    id = id,
                    statesSection =
                        ensureNonNull(sections["States"], DEFAULT_STATES_SECTION) {
                            validateStatesSection(it, errors, tracker)
                        },
                    requiringSection =
                        ifNonNull(sections["requiring"]) {
                            validateRequiringSection(it, errors, tracker)
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
                        ensureNonNull(sections["written"], DEFAULT_WRITTEN_SECTION) {
                            validateWrittenSection(it, errors, tracker)
                        },
                    calledSection =
                        ensureNonNull(sections["called"], DEFAULT_CALLED_SECTION) {
                            validateCalledSection(it, errors, tracker)
                        },
                    metaDataSection =
                        ifNonNull(sections["Metadata"]) {
                            validateMetaDataSection(it, errors, tracker)
                        })
            }
        }
    }

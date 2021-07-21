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

package mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.defines

import mathlingua.backend.transform.Signature
import mathlingua.backend.transform.signature
import mathlingua.frontend.chalktalk.phase1.ast.Phase1Node
import mathlingua.frontend.chalktalk.phase2.CodeWriter
import mathlingua.frontend.chalktalk.phase2.ast.DEFAULT_CALLED_SECTION
import mathlingua.frontend.chalktalk.phase2.ast.DEFAULT_DEFINES_MAPS_GROUP
import mathlingua.frontend.chalktalk.phase2.ast.DEFAULT_DEFINES_SECTION
import mathlingua.frontend.chalktalk.phase2.ast.DEFAULT_MAPS_SECTION
import mathlingua.frontend.chalktalk.phase2.ast.DEFAULT_WRITTEN_SECTION
import mathlingua.frontend.chalktalk.phase2.ast.clause.IdStatement
import mathlingua.frontend.chalktalk.phase2.ast.clause.sectionsMatchNames
import mathlingua.frontend.chalktalk.phase2.ast.common.Phase2Node
import mathlingua.frontend.chalktalk.phase2.ast.getId
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.CalledSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.WrittenSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.validateCalledSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.validateWrittenSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.viewing.ViewingSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.viewing.validateViewingSection
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

data class DefinesMapsGroup(
    override val signature: Signature?,
    override val id: IdStatement,
    override val definesSection: DefinesSection,
    override val requiringSection: RequiringSection?,
    override val whenSection: WhenSection?,
    val meansSection: MeansSection?,
    val mapsSection: MapsSection,
    val satisfyingSection: SatisfyingSection?,
    override val viewingSection: ViewingSection?,
    override val usingSection: UsingSection?,
    override val writtenSection: WrittenSection,
    override val calledSection: CalledSection,
    override val metaDataSection: MetaDataSection?
) : DefinesGroup(metaDataSection) {

    override fun forEach(fn: (node: Phase2Node) -> Unit) {
        fn(id)
        fn(definesSection)
        if (requiringSection != null) {
            fn(requiringSection)
        }
        if (whenSection != null) {
            fn(whenSection)
        }
        if (meansSection != null) {
            fn(meansSection)
        }
        fn(mapsSection)
        if (satisfyingSection != null) {
            fn(satisfyingSection)
        }
        if (viewingSection != null) {
            fn(viewingSection)
        }
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
        val sections =
            mutableListOf(
                definesSection,
                requiringSection,
                whenSection,
                meansSection,
                mapsSection,
                satisfyingSection,
                viewingSection,
                usingSection,
                writtenSection,
                calledSection,
                metaDataSection)
        return topLevelToCode(writer, isArg, indent, id, *sections.toTypedArray())
    }

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) =
        chalkTransformer(
            DefinesMapsGroup(
                signature = signature,
                id = id.transform(chalkTransformer) as IdStatement,
                definesSection = definesSection.transform(chalkTransformer) as DefinesSection,
                requiringSection =
                    requiringSection?.transform(chalkTransformer) as RequiringSection?,
                whenSection = whenSection?.transform(chalkTransformer) as WhenSection?,
                meansSection = meansSection?.transform(chalkTransformer) as MeansSection?,
                mapsSection = mapsSection.transform(chalkTransformer) as MapsSection,
                satisfyingSection =
                    satisfyingSection?.transform(chalkTransformer) as SatisfyingSection?,
                viewingSection = viewingSection?.transform(chalkTransformer) as ViewingSection?,
                usingSection = usingSection?.transform(chalkTransformer) as UsingSection?,
                writtenSection = writtenSection.transform(chalkTransformer) as WrittenSection,
                calledSection = calledSection.transform(chalkTransformer) as CalledSection,
                metaDataSection = metaDataSection?.transform(chalkTransformer) as MetaDataSection?))

    override fun copyWithoutMetadata(): DefinesGroup {
        return copy(metaDataSection = null)
    }
}

fun isDefinesMapsGroup(node: Phase1Node) = sectionsMatchNames(node, "Defines", "maps")

fun validateDefinesMapsGroup(
    node: Phase1Node, errors: MutableList<ParseError>, tracker: MutableLocationTracker
) =
    track(node, tracker) {
        validateGroup(node.resolve(), errors, "Defines", DEFAULT_DEFINES_MAPS_GROUP) { group ->
            identifySections(
                group,
                errors,
                DEFAULT_DEFINES_MAPS_GROUP,
                listOf(
                    "Defines",
                    "requiring?",
                    "when?",
                    "means?",
                    "maps",
                    "satisfying?",
                    "viewing?",
                    "using?",
                    "written",
                    "called",
                    "Metadata?")) { sections ->
                val id = getId(group, errors, tracker)
                val def =
                    DefinesMapsGroup(
                        signature = id.signature(tracker),
                        id = id,
                        definesSection =
                            ensureNonNull(sections["Defines"], DEFAULT_DEFINES_SECTION) {
                                validateDefinesSection(it, errors, tracker)
                            },
                        requiringSection =
                            ifNonNull(sections["requiring"]) {
                                validateRequiringSection(it, errors, tracker)
                            },
                        whenSection =
                            ifNonNull(sections["when"]) {
                                validateWhenSection(it, errors, tracker)
                            },
                        meansSection =
                            ifNonNull(sections["means"]) {
                                validateMeansSection(it, errors, tracker)
                            },
                        mapsSection =
                            ensureNonNull(sections["maps"], DEFAULT_MAPS_SECTION) {
                                validateMapsSection(it, errors, tracker)
                            },
                        satisfyingSection =
                            ifNonNull(sections["satisfying"]) {
                                validateSatisfyingSection(it, errors, tracker)
                            },
                        viewingSection =
                            ifNonNull(sections["viewing"]) {
                                validateViewingSection(it, errors, tracker)
                            },
                        usingSection =
                            ifNonNull(sections["using"]) {
                                validateUsingSection(it, errors, tracker)
                            },
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

                val funcArgsError = checkIfFunctionSignatureMatchDefines(def, tracker)
                if (funcArgsError != null) {
                    errors.add(funcArgsError)
                    DEFAULT_DEFINES_MAPS_GROUP
                } else {
                    def
                }
            }
        }
    }

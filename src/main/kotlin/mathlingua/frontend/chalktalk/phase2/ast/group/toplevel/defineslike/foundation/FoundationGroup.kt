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

package mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.foundation

import mathlingua.frontend.chalktalk.phase1.ast.Phase1Node
import mathlingua.frontend.chalktalk.phase2.CodeWriter
import mathlingua.frontend.chalktalk.phase2.ast.DEFAULT_FOUNDATION_GROUP
import mathlingua.frontend.chalktalk.phase2.ast.DEFAULT_FOUNDATION_SECTION
import mathlingua.frontend.chalktalk.phase2.ast.clause.Clause
import mathlingua.frontend.chalktalk.phase2.ast.clause.firstSectionMatchesName
import mathlingua.frontend.chalktalk.phase2.ast.common.Phase2Node
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.TopLevelGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.metadata.section.MetaDataSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.metadata.section.validateMetaDataSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.topLevelToCode
import mathlingua.frontend.chalktalk.phase2.ast.section.ensureNonNull
import mathlingua.frontend.chalktalk.phase2.ast.section.identifySections
import mathlingua.frontend.chalktalk.phase2.ast.section.ifNonNull
import mathlingua.frontend.chalktalk.phase2.ast.track
import mathlingua.frontend.chalktalk.phase2.ast.validateGroup
import mathlingua.frontend.support.MutableLocationTracker
import mathlingua.frontend.support.ParseError

interface DefinesStatesOrViews : Clause

data class FoundationGroup(
    val foundationSection: FoundationSection, override val metaDataSection: MetaDataSection?
) : TopLevelGroup(metaDataSection) {

    override fun forEach(fn: (node: Phase2Node) -> Unit) {
        fn(foundationSection)
        if (metaDataSection != null) {
            fn(metaDataSection)
        }
    }

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter) =
        topLevelToCode(writer, isArg, indent, null, foundationSection, metaDataSection)

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) =
        chalkTransformer(
            FoundationGroup(
                foundationSection =
                    foundationSection.transform(chalkTransformer) as FoundationSection,
                metaDataSection = metaDataSection?.transform(chalkTransformer) as MetaDataSection?))
}

fun isFoundationGroup(node: Phase1Node) = firstSectionMatchesName(node, "Foundation")

fun validateFoundationGroup(
    node: Phase1Node, errors: MutableList<ParseError>, tracker: MutableLocationTracker
) =
    track(node, tracker) {
        validateGroup(node.resolve(), errors, "Foundation", DEFAULT_FOUNDATION_GROUP) { group ->
            identifySections(
                group, errors, DEFAULT_FOUNDATION_GROUP, listOf("Foundation", "Metadata?")) {
            sections ->
                FoundationGroup(
                    foundationSection =
                        ensureNonNull(sections["Foundation"], DEFAULT_FOUNDATION_SECTION) {
                            validateFoundationSection(it, errors, tracker)
                        },
                    metaDataSection =
                        ifNonNull(sections["Metadata"]) {
                            validateMetaDataSection(it, errors, tracker)
                        })
            }
        }
    }

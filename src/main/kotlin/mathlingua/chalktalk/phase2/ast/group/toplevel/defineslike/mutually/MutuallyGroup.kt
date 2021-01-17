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

package mathlingua.chalktalk.phase2.ast.group.toplevel.defineslike.mutually

import mathlingua.chalktalk.phase1.ast.Phase1Node
import mathlingua.chalktalk.phase2.CodeWriter
import mathlingua.chalktalk.phase2.ast.DEFAULT_MUTUALLY_GROUP
import mathlingua.chalktalk.phase2.ast.DEFAULT_MUTUALLY_SECTION
import mathlingua.chalktalk.phase2.ast.clause.firstSectionMatchesName
import mathlingua.chalktalk.phase2.ast.common.Phase2Node
import mathlingua.chalktalk.phase2.ast.group.toplevel.TopLevelGroup
import mathlingua.chalktalk.phase2.ast.group.toplevel.shared.metadata.section.MetaDataSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.shared.metadata.section.validateMetaDataSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.topLevelToCode
import mathlingua.chalktalk.phase2.ast.section.ensureNonNull
import mathlingua.chalktalk.phase2.ast.section.identifySections
import mathlingua.chalktalk.phase2.ast.section.ifNonNull
import mathlingua.chalktalk.phase2.ast.track
import mathlingua.chalktalk.phase2.ast.validateGroup
import mathlingua.support.MutableLocationTracker
import mathlingua.support.ParseError

data class MutuallyGroup(
    val mutuallySection: MutuallySection, override val metaDataSection: MetaDataSection?
) : TopLevelGroup(metaDataSection) {

    override fun forEach(fn: (node: Phase2Node) -> Unit) {
        fn(mutuallySection)
        if (metaDataSection != null) {
            fn(metaDataSection)
        }
    }

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter) =
        topLevelToCode(writer, isArg, indent, null, mutuallySection, metaDataSection)

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) =
        chalkTransformer(
            MutuallyGroup(
                mutuallySection = mutuallySection.transform(chalkTransformer) as MutuallySection,
                metaDataSection = metaDataSection?.transform(chalkTransformer) as MetaDataSection?))
}

fun isMutuallyGroup(node: Phase1Node) = firstSectionMatchesName(node, "Mutually")

fun validateMutuallyGroup(
    node: Phase1Node, errors: MutableList<ParseError>, tracker: MutableLocationTracker
) =
    track(node, tracker) {
        validateGroup(node.resolve(), errors, "Mutually", DEFAULT_MUTUALLY_GROUP) { group ->
            identifySections(
                group, errors, DEFAULT_MUTUALLY_GROUP, listOf("Mutually", "Metadata?")) {
            sections ->
                MutuallyGroup(
                    mutuallySection =
                        ensureNonNull(sections["Mutually"], DEFAULT_MUTUALLY_SECTION) {
                            validateMutuallySection(it, errors, tracker)
                        },
                    metaDataSection =
                        ifNonNull(sections["Metadata"]) {
                            validateMetaDataSection(it, errors, tracker)
                        })
            }
        }
    }

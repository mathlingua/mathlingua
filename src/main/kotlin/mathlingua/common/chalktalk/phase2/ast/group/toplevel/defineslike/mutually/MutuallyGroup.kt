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

package mathlingua.common.chalktalk.phase2.ast.group.toplevel.defineslike.mutually

import mathlingua.common.chalktalk.phase1.ast.Group
import mathlingua.common.chalktalk.phase1.ast.Phase1Node
import mathlingua.common.chalktalk.phase2.CodeWriter
import mathlingua.common.chalktalk.phase2.ast.clause.firstSectionMatchesName
import mathlingua.common.chalktalk.phase2.ast.common.Phase2Node
import mathlingua.common.chalktalk.phase2.ast.group.toplevel.TopLevelGroup
import mathlingua.common.chalktalk.phase2.ast.group.toplevel.shared.metadata.section.MetaDataSection
import mathlingua.common.chalktalk.phase2.ast.group.toplevel.topLevelToCode
import mathlingua.common.chalktalk.phase2.ast.group.toplevel.validateSingleSectionMetaDataGroup
import mathlingua.common.support.MutableLocationTracker

data class MutuallyGroup(
    val mutuallySection: MutuallySection,
    override val metaDataSection: MetaDataSection?
) : TopLevelGroup(metaDataSection) {

    override fun forEach(fn: (node: Phase2Node) -> Unit) {
        fn(mutuallySection)
        if (metaDataSection != null) {
            fn(metaDataSection)
        }
    }

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter) = topLevelToCode(
        writer,
        isArg,
        indent,
        null,
        mutuallySection,
        metaDataSection
    )

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) = chalkTransformer(
        MutuallyGroup(
            mutuallySection = mutuallySection.transform(chalkTransformer) as MutuallySection,
            metaDataSection = metaDataSection?.transform(chalkTransformer) as MetaDataSection?
        )
    )
}

fun isMutuallyGroup(node: Phase1Node) = firstSectionMatchesName(node, "Mutually")

fun validateMutuallyGroup(groupNode: Group, tracker: MutableLocationTracker) = validateSingleSectionMetaDataGroup(
    tracker,
    groupNode,
    "Mutually",
    ::validateMutuallySection,
    ::MutuallyGroup
)

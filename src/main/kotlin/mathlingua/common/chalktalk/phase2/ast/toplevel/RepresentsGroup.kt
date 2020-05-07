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

package mathlingua.common.chalktalk.phase2.ast.toplevel

import mathlingua.common.MutableLocationTracker
import mathlingua.common.chalktalk.phase1.ast.Group
import mathlingua.common.chalktalk.phase1.ast.Phase1Node
import mathlingua.common.chalktalk.phase2.CodeWriter
import mathlingua.common.chalktalk.phase2.ast.Phase2Node
import mathlingua.common.chalktalk.phase2.ast.clause.IdStatement
import mathlingua.common.chalktalk.phase2.ast.clause.firstSectionMatchesName
import mathlingua.common.chalktalk.phase2.ast.section.*

data class RepresentsGroup(
    val signature: String?,
    val id: IdStatement,
    val representsSection: RepresentsSection,
    val assumingSection: AssumingSection?,
    val thatSections: List<ThatSection>,
    val aliasSection: AliasSection?,
    override val metaDataSection: MetaDataSection?
) : TopLevelGroup(metaDataSection) {

    override fun forEach(fn: (node: Phase2Node) -> Unit) {
        fn(id)
        fn(representsSection)
        if (assumingSection != null) {
            fn(assumingSection)
        }
        thatSections.forEach(fn)
        if (metaDataSection != null) {
            fn(metaDataSection)
        }
    }

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter): CodeWriter {
        val sections = mutableListOf(representsSection, assumingSection)
        sections.addAll(thatSections)
        sections.add(metaDataSection)
        return toCode(
                writer,
                isArg,
                indent,
                id,
                *sections.toTypedArray()
        )
    }

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) = chalkTransformer(RepresentsGroup(
            signature = signature,
            id = id.transform(chalkTransformer) as IdStatement,
            representsSection = representsSection.transform(chalkTransformer) as RepresentsSection,
            assumingSection = assumingSection?.transform(chalkTransformer) as AssumingSection?,
            thatSections = thatSections.map { chalkTransformer(it) as ThatSection },
            aliasSection = aliasSection?.transform(chalkTransformer) as AliasSection?,
            metaDataSection = metaDataSection?.transform(chalkTransformer) as MetaDataSection?
    ))
}

fun isRepresentsGroup(node: Phase1Node) = firstSectionMatchesName(node, "Represents")

fun validateRepresentsGroup(groupNode: Group, tracker: MutableLocationTracker) = validateDefinesLikeGroup(
        tracker,
        groupNode,
        "Represents",
        ::validateRepresentsSection,
        "that",
        ::validateThatSection,
        ::RepresentsGroup
)

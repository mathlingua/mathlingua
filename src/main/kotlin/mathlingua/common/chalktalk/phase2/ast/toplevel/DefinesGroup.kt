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

import mathlingua.common.chalktalk.phase1.ast.Group
import mathlingua.common.chalktalk.phase1.ast.Phase1Node
import mathlingua.common.chalktalk.phase2.CodeWriter
import mathlingua.common.chalktalk.phase2.ast.Phase2Node
import mathlingua.common.chalktalk.phase2.ast.clause.IdStatement
import mathlingua.common.chalktalk.phase2.ast.clause.firstSectionMatchesName
import mathlingua.common.chalktalk.phase2.ast.section.*

data class DefinesGroup(
    val signature: String?,
    val id: IdStatement,
    val definesSection: DefinesSection,
    val assumingSection: AssumingSection?,
    val meansSections: List<MeansSection>,
    val aliasSection: AliasSection?,
    override val metaDataSection: MetaDataSection?,
    override var row: Int,
    override var column: Int
) : TopLevelGroup(metaDataSection) {

    override fun forEach(fn: (node: Phase2Node) -> Unit) {
        fn(id)
        fn(definesSection)
        if (assumingSection != null) {
            fn(assumingSection)
        }
        meansSections.forEach(fn)
        if (metaDataSection != null) {
            fn(metaDataSection)
        }
    }

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter): CodeWriter {
        val sections = mutableListOf(definesSection, assumingSection)
        sections.addAll(meansSections)
        sections.add(metaDataSection)
        return toCode(
                writer,
                isArg,
                indent,
                id,
                *sections.toTypedArray()
        )
    }

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) = chalkTransformer(DefinesGroup(
            signature = signature,
            id = id.transform(chalkTransformer) as IdStatement,
            definesSection = definesSection.transform(chalkTransformer) as DefinesSection,
            assumingSection = assumingSection?.transform(chalkTransformer) as AssumingSection?,
            meansSections = meansSections.map { chalkTransformer(it) as MeansSection },
            aliasSection = aliasSection?.transform(chalkTransformer) as AliasSection?,
            metaDataSection = metaDataSection?.transform(chalkTransformer) as MetaDataSection?,
            row = row,
            column = column
    ))
}

fun isDefinesGroup(node: Phase1Node) = firstSectionMatchesName(node, "Defines")

fun validateDefinesGroup(groupNode: Group) = validateDefinesLikeGroup(
        groupNode,
        "Defines",
        ::validateDefinesSection,
        "means",
        ::validateMeansSection,
        ::DefinesGroup
)

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
import mathlingua.common.chalktalk.phase2.ast.clause.firstSectionMatchesName
import mathlingua.common.chalktalk.phase2.ast.metadata.section.MetaDataSection
import mathlingua.common.chalktalk.phase2.ast.section.*

open class ResultGroup(
    val resultSection: ResultSection,
    open val aliasSection: AliasSection?,
    override val metaDataSection: MetaDataSection?
) : TopLevelGroup(metaDataSection) {

    override fun forEach(fn: (node: Phase2Node) -> Unit) {
        fn(resultSection)
        if (metaDataSection != null) {
            fn(metaDataSection as MetaDataSection)
        }
    }

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter) =
            toCode(writer, isArg, indent, null, resultSection, metaDataSection)

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) = chalkTransformer(ResultGroup(
            resultSection = resultSection.transform(chalkTransformer) as ResultSection,
            metaDataSection = metaDataSection?.transform(chalkTransformer) as MetaDataSection?,
            aliasSection = aliasSection?.transform(chalkTransformer) as AliasSection?
    ))
}

fun isResultGroup(node: Phase1Node) = firstSectionMatchesName(node, "Result")

fun validateResultGroup(groupNode: Group, tracker: MutableLocationTracker) = validateResultLikeGroup(
        tracker,
        groupNode,
        "Result",
        ::validateResultSection,
        ::ResultGroup
)

class TheoremGroup(
    val theoremSection: TheoremSection,
    override val aliasSection: AliasSection?,
    override val metaDataSection: MetaDataSection?
) : ResultGroup(theoremSection, aliasSection, metaDataSection) {

    override fun forEach(fn: (node: Phase2Node) -> Unit) {
        fn(theoremSection)
        if (metaDataSection != null) {
            fn(metaDataSection)
        }
    }

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter) =
            toCode(writer, isArg, indent, null, theoremSection, metaDataSection)

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) = chalkTransformer(TheoremGroup(
            theoremSection = theoremSection.transform(chalkTransformer) as TheoremSection,
            metaDataSection = metaDataSection?.transform(chalkTransformer) as MetaDataSection?,
            aliasSection = aliasSection?.transform(chalkTransformer) as AliasSection?
    ))
}

fun isTheoremGroup(node: Phase1Node) = firstSectionMatchesName(node, "Theorem")

fun validateTheoremGroup(groupNode: Group, tracker: MutableLocationTracker) = validateResultLikeGroup(
        tracker,
        groupNode,
        "Theorem",
        ::validateTheoremSection,
        ::TheoremGroup
)

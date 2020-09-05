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

data class ProblemGroup(
    val problemSection: ProblemSection,
    val usingSection: UsingSection?,
    val whereSection: WhereSection?,
    override val metaDataSection: MetaDataSection?
) : TopLevelGroup(metaDataSection) {

    override fun forEach(fn: (node: Phase2Node) -> Unit) {
        fn(problemSection)
        if (usingSection != null) {
            fn(usingSection)
        }
        if (whereSection != null) {
            fn(whereSection)
        }
        if (metaDataSection != null) {
            fn(metaDataSection)
        }
    }

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter) =
        topLevelToCode(writer, isArg, indent, null, problemSection, usingSection, whereSection, metaDataSection)

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) = chalkTransformer(ProblemGroup(
        problemSection = problemSection.transform(chalkTransformer) as ProblemSection,
        usingSection = usingSection?.transform(chalkTransformer) as UsingSection?,
        whereSection = whereSection?.transform(chalkTransformer) as WhereSection?,
        metaDataSection = metaDataSection?.transform(chalkTransformer) as MetaDataSection?
    ))
}

fun isProblemGroup(node: Phase1Node) = firstSectionMatchesName(node, "Problem")

fun validateProblemGroup(groupNode: Group, tracker: MutableLocationTracker) = validateResultLikeGroup(
    tracker,
    groupNode,
    "Problem",
    ::validateProblemSection,
    ::ProblemGroup
)

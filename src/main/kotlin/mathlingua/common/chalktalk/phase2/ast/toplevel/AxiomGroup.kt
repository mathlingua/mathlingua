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

import mathlingua.common.support.MutableLocationTracker
import mathlingua.common.chalktalk.phase1.ast.Group
import mathlingua.common.chalktalk.phase1.ast.Phase1Node
import mathlingua.common.chalktalk.phase2.CodeWriter
import mathlingua.common.chalktalk.phase2.ast.Phase2Node
import mathlingua.common.chalktalk.phase2.ast.clause.firstSectionMatchesName
import mathlingua.common.chalktalk.phase2.ast.section.UsingSection
import mathlingua.common.chalktalk.phase2.ast.section.AxiomSection
import mathlingua.common.chalktalk.phase2.ast.metadata.section.MetaDataSection
import mathlingua.common.chalktalk.phase2.ast.section.GivenSection
import mathlingua.common.chalktalk.phase2.ast.section.ThenSection
import mathlingua.common.chalktalk.phase2.ast.section.WhereSection
import mathlingua.common.chalktalk.phase2.ast.section.validateAxiomSection

data class AxiomGroup(
    val axiomSection: AxiomSection,
    val givenSection: GivenSection?,
    val givenWhereSection: WhereSection?,
    val thenSection: ThenSection,
    val usingSection: UsingSection?,
    val usingWhereSection: WhereSection?,
    override val metaDataSection: MetaDataSection?
) : TopLevelGroup(metaDataSection) {

    override fun forEach(fn: (node: Phase2Node) -> Unit) {
        fn(axiomSection)
        if (usingSection != null) {
            fn(usingSection)
        }
        if (givenSection != null) {
            fn(givenSection)
        }
        if (givenWhereSection != null) {
            fn(givenWhereSection)
        }
        fn(thenSection)
        if (usingWhereSection != null) {
            fn(usingWhereSection)
        }
        if (metaDataSection != null) {
            fn(metaDataSection)
        }
    }

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter) =
            topLevelToCode(writer, isArg, indent, null, axiomSection,
                givenSection, givenWhereSection, thenSection,
                usingSection, usingWhereSection, metaDataSection)

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) =
            chalkTransformer(AxiomGroup(
                    axiomSection = axiomSection.transform(chalkTransformer) as AxiomSection,
                    givenSection = givenSection?.transform(chalkTransformer) as GivenSection?,
                    givenWhereSection = givenWhereSection?.transform(chalkTransformer) as WhereSection?,
                    thenSection = thenSection.transform(chalkTransformer) as ThenSection,
                    usingSection = usingSection?.transform(chalkTransformer) as UsingSection?,
                    usingWhereSection = usingWhereSection?.transform(chalkTransformer) as WhereSection?,
                    metaDataSection = metaDataSection?.transform(chalkTransformer) as MetaDataSection?
            ))
}

fun isAxiomGroup(node: Phase1Node) = firstSectionMatchesName(node, "Axiom")

fun validateAxiomGroup(groupNode: Group, tracker: MutableLocationTracker) = validateResultLikeGroup(
        tracker,
        groupNode,
        "Axiom",
        ::validateAxiomSection,
        ::AxiomGroup
)

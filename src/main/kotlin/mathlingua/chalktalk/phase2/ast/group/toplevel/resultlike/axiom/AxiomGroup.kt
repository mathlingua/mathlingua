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

package mathlingua.chalktalk.phase2.ast.group.toplevel.resultlike.axiom

import mathlingua.chalktalk.phase1.ast.Group
import mathlingua.chalktalk.phase1.ast.Phase1Node
import mathlingua.chalktalk.phase2.CodeWriter
import mathlingua.chalktalk.phase2.ast.DEFAULT_AXIOM_GROUP
import mathlingua.chalktalk.phase2.ast.DEFAULT_AXIOM_SECTION
import mathlingua.chalktalk.phase2.ast.DEFAULT_THEN_SECTION
import mathlingua.chalktalk.phase2.ast.clause.firstSectionMatchesName
import mathlingua.chalktalk.phase2.ast.common.Phase2Node
import mathlingua.chalktalk.phase2.ast.group.clause.If.ThenSection
import mathlingua.chalktalk.phase2.ast.group.clause.If.neoValidateThenSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.TopLevelGroup
import mathlingua.chalktalk.phase2.ast.group.toplevel.resultlike.theorem.GivenSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.resultlike.theorem.neoValidateGivenSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.shared.UsingSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.shared.WhereSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.shared.metadata.section.MetaDataSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.shared.metadata.section.neoValidateMetaDataSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.shared.neoValidateUsingSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.shared.neoValidateWhereSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.topLevelToCode
import mathlingua.chalktalk.phase2.ast.group.toplevel.validateResultLikeGroup
import mathlingua.chalktalk.phase2.ast.neoTrack
import mathlingua.chalktalk.phase2.ast.neoValidateGroup
import mathlingua.chalktalk.phase2.ast.section.neoEnsureNonNull
import mathlingua.chalktalk.phase2.ast.section.neoIdentifySections
import mathlingua.chalktalk.phase2.ast.section.neoIfNonNull
import mathlingua.support.MutableLocationTracker
import mathlingua.support.ParseError

data class AxiomGroup(
    val axiomSection: AxiomSection,
    val givenSection: GivenSection?,
    val whereSection: WhereSection?,
    val thenSection: ThenSection,
    val usingSection: UsingSection?,
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
        if (whereSection != null) {
            fn(whereSection)
        }
        fn(thenSection)
        if (metaDataSection != null) {
            fn(metaDataSection)
        }
    }

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter) =
        topLevelToCode(
            writer,
            isArg,
            indent,
            null,
            axiomSection,
            givenSection,
            whereSection,
            thenSection,
            usingSection,
            metaDataSection)

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) =
        chalkTransformer(
            AxiomGroup(
                axiomSection = axiomSection.transform(chalkTransformer) as AxiomSection,
                givenSection = givenSection?.transform(chalkTransformer) as GivenSection?,
                whereSection = whereSection?.transform(chalkTransformer) as WhereSection?,
                thenSection = thenSection.transform(chalkTransformer) as ThenSection,
                usingSection = usingSection?.transform(chalkTransformer) as UsingSection?,
                metaDataSection = metaDataSection?.transform(chalkTransformer) as MetaDataSection?))
}

fun isAxiomGroup(node: Phase1Node) = firstSectionMatchesName(node, "Axiom")

fun validateAxiomGroup(groupNode: Group, tracker: MutableLocationTracker) =
    validateResultLikeGroup(tracker, groupNode, "Axiom", ::validateAxiomSection, ::AxiomGroup)

fun neoValidateAxiomGroup(
    node: Phase1Node, errors: MutableList<ParseError>, tracker: MutableLocationTracker
) =
    neoTrack(node, tracker) {
        neoValidateGroup(node.resolve(), errors, "Axiom", DEFAULT_AXIOM_GROUP) { group ->
            neoIdentifySections(
                group,
                errors,
                DEFAULT_AXIOM_GROUP,
                listOf("Axiom", "given?", "where?", "then", "using?", "Metadata?")) { sections ->
                AxiomGroup(
                    axiomSection =
                        neoEnsureNonNull(sections["Axiom"], DEFAULT_AXIOM_SECTION) {
                            neoValidateAxiomSection(it, errors, tracker)
                        },
                    givenSection =
                        neoIfNonNull(sections["given"]) {
                            neoValidateGivenSection(it, errors, tracker)
                        },
                    whereSection =
                        neoIfNonNull(sections["where"]) {
                            neoValidateWhereSection(it, errors, tracker)
                        },
                    thenSection =
                        neoEnsureNonNull(sections["then"], DEFAULT_THEN_SECTION) {
                            neoValidateThenSection(it, errors, tracker)
                        },
                    usingSection =
                        neoIfNonNull(sections["using"]) {
                            neoValidateUsingSection(it, errors, tracker)
                        },
                    metaDataSection =
                        neoIfNonNull(sections["Metadata"]) {
                            neoValidateMetaDataSection(it, errors, tracker)
                        })
            }
        }
    }

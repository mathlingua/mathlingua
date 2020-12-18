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

package mathlingua.chalktalk.phase2.ast.group.toplevel.resultlike.conjecture

import mathlingua.chalktalk.phase1.ast.Group
import mathlingua.chalktalk.phase1.ast.Phase1Node
import mathlingua.chalktalk.phase2.CodeWriter
import mathlingua.chalktalk.phase2.ast.DEFAULT_CONJECTURE_GROUP
import mathlingua.chalktalk.phase2.ast.DEFAULT_CONJECTURE_SECTION
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

data class ConjectureGroup(
    val conjectureSection: ConjectureSection,
    val givenSection: GivenSection?,
    val givenWhereSection: WhereSection?,
    val thenSection: ThenSection,
    val usingSection: UsingSection?,
    override val metaDataSection: MetaDataSection?
) : TopLevelGroup(metaDataSection) {

    override fun forEach(fn: (node: Phase2Node) -> Unit) {
        fn(conjectureSection)
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
            conjectureSection,
            givenSection,
            givenWhereSection,
            thenSection,
            usingSection,
            metaDataSection)

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) =
        chalkTransformer(
            ConjectureGroup(
                conjectureSection =
                    conjectureSection.transform(chalkTransformer) as ConjectureSection,
                givenSection = givenSection?.transform(chalkTransformer) as GivenSection?,
                givenWhereSection = givenWhereSection?.transform(chalkTransformer) as WhereSection?,
                thenSection = thenSection.transform(chalkTransformer) as ThenSection,
                usingSection = usingSection?.transform(chalkTransformer) as UsingSection?,
                metaDataSection = metaDataSection?.transform(chalkTransformer) as MetaDataSection?))
}

fun isConjectureGroup(node: Phase1Node) = firstSectionMatchesName(node, "Conjecture")

fun validateConjectureGroup(groupNode: Group, tracker: MutableLocationTracker) =
    validateResultLikeGroup(
        tracker, groupNode, "Conjecture", ::validateConjectureSection, ::ConjectureGroup)

fun neoValidateConjectureGroup(
    node: Phase1Node, errors: MutableList<ParseError>, tracker: MutableLocationTracker
) =
    neoTrack(node, tracker) {
        neoValidateGroup(node.resolve(), errors, "Conjecture", DEFAULT_CONJECTURE_GROUP) { group ->
            neoIdentifySections(
                group,
                errors,
                DEFAULT_CONJECTURE_GROUP,
                listOf("Conjecture", "given?", "where?", "then", "using?", "Metadata?")) {
            sections ->
                ConjectureGroup(
                    conjectureSection =
                        neoEnsureNonNull(sections["Conjecture"], DEFAULT_CONJECTURE_SECTION) {
                            neoValidateConjectureSection(it, errors, tracker)
                        },
                    givenSection =
                        neoIfNonNull(sections["given"]) {
                            neoValidateGivenSection(it, errors, tracker)
                        },
                    givenWhereSection =
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

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

import mathlingua.chalktalk.phase1.ast.Phase1Node
import mathlingua.chalktalk.phase2.CodeWriter
import mathlingua.chalktalk.phase2.ast.DEFAULT_CONJECTURE_GROUP
import mathlingua.chalktalk.phase2.ast.DEFAULT_CONJECTURE_SECTION
import mathlingua.chalktalk.phase2.ast.DEFAULT_THEN_SECTION
import mathlingua.chalktalk.phase2.ast.clause.IdStatement
import mathlingua.chalktalk.phase2.ast.clause.firstSectionMatchesName
import mathlingua.chalktalk.phase2.ast.common.Phase2Node
import mathlingua.chalktalk.phase2.ast.group.clause.If.ThenSection
import mathlingua.chalktalk.phase2.ast.group.clause.If.validateThenSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.TopLevelGroup
import mathlingua.chalktalk.phase2.ast.group.toplevel.resultlike.theorem.GivenSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.resultlike.theorem.validateGivenSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.shared.UsingSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.shared.WhereSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.shared.metadata.section.MetaDataSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.shared.metadata.section.validateMetaDataSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.shared.validateUsingSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.shared.validateWhereSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.topLevelToCode
import mathlingua.chalktalk.phase2.ast.neoGetOptionalId
import mathlingua.chalktalk.phase2.ast.neoTrack
import mathlingua.chalktalk.phase2.ast.section.neoEnsureNonNull
import mathlingua.chalktalk.phase2.ast.section.neoIdentifySections
import mathlingua.chalktalk.phase2.ast.section.neoIfNonNull
import mathlingua.chalktalk.phase2.ast.validateGroup
import mathlingua.support.MutableLocationTracker
import mathlingua.support.ParseError
import mathlingua.transform.signature

data class ConjectureGroup(
    val signature: String?,
    val id: IdStatement?,
    val conjectureSection: ConjectureSection,
    val givenSection: GivenSection?,
    val whereSection: WhereSection?,
    val thenSection: ThenSection,
    val usingSection: UsingSection?,
    override val metaDataSection: MetaDataSection?
) : TopLevelGroup(metaDataSection) {

    override fun forEach(fn: (node: Phase2Node) -> Unit) {
        if (id != null) {
            fn(id)
        }
        fn(conjectureSection)
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
            id,
            conjectureSection,
            givenSection,
            whereSection,
            thenSection,
            usingSection,
            metaDataSection)

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) =
        chalkTransformer(
            ConjectureGroup(
                signature = signature,
                id = id?.transform(chalkTransformer) as IdStatement?,
                conjectureSection =
                    conjectureSection.transform(chalkTransformer) as ConjectureSection,
                givenSection = givenSection?.transform(chalkTransformer) as GivenSection?,
                whereSection = whereSection?.transform(chalkTransformer) as WhereSection?,
                thenSection = thenSection.transform(chalkTransformer) as ThenSection,
                usingSection = usingSection?.transform(chalkTransformer) as UsingSection?,
                metaDataSection = metaDataSection?.transform(chalkTransformer) as MetaDataSection?))
}

fun isConjectureGroup(node: Phase1Node) = firstSectionMatchesName(node, "Conjecture")

fun validateConjectureGroup(
    node: Phase1Node, errors: MutableList<ParseError>, tracker: MutableLocationTracker
) =
    neoTrack(node, tracker) {
        validateGroup(node.resolve(), errors, "Conjecture", DEFAULT_CONJECTURE_GROUP) { group ->
            val id = neoGetOptionalId(group, errors, tracker)
            neoIdentifySections(
                group,
                errors,
                DEFAULT_CONJECTURE_GROUP,
                listOf("Conjecture", "given?", "where?", "then", "using?", "Metadata?")) {
            sections ->
                ConjectureGroup(
                    signature = id?.signature(),
                    id = id,
                    conjectureSection =
                        neoEnsureNonNull(sections["Conjecture"], DEFAULT_CONJECTURE_SECTION) {
                            validateConjectureSection(it, errors, tracker)
                        },
                    givenSection =
                        neoIfNonNull(sections["given"]) {
                            validateGivenSection(it, errors, tracker)
                        },
                    whereSection =
                        neoIfNonNull(sections["where"]) {
                            validateWhereSection(it, errors, tracker)
                        },
                    thenSection =
                        neoEnsureNonNull(sections["then"], DEFAULT_THEN_SECTION) {
                            validateThenSection(it, errors, tracker)
                        },
                    usingSection =
                        neoIfNonNull(sections["using"]) {
                            validateUsingSection(it, errors, tracker)
                        },
                    metaDataSection =
                        neoIfNonNull(sections["Metadata"]) {
                            validateMetaDataSection(it, errors, tracker)
                        })
            }
        }
    }

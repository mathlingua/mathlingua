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

import mathlingua.chalktalk.phase1.ast.Phase1Node
import mathlingua.chalktalk.phase2.CodeWriter
import mathlingua.chalktalk.phase2.ast.DEFAULT_AXIOM_GROUP
import mathlingua.chalktalk.phase2.ast.DEFAULT_AXIOM_SECTION
import mathlingua.chalktalk.phase2.ast.DEFAULT_THEN_SECTION
import mathlingua.chalktalk.phase2.ast.clause.IdStatement
import mathlingua.chalktalk.phase2.ast.clause.firstSectionMatchesName
import mathlingua.chalktalk.phase2.ast.common.Phase2Node
import mathlingua.chalktalk.phase2.ast.getOptionalId
import mathlingua.chalktalk.phase2.ast.group.clause.If.ThenSection
import mathlingua.chalktalk.phase2.ast.group.clause.If.validateThenSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.TopLevelGroup
import mathlingua.chalktalk.phase2.ast.group.toplevel.resultlike.IfOrIffSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.resultlike.theorem.GivenSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.resultlike.theorem.validateGivenSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.resultlike.validateIfOrIffSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.shared.UsingSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.shared.WhereSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.shared.metadata.section.MetaDataSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.shared.metadata.section.validateMetaDataSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.shared.validateUsingSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.shared.validateWhereSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.topLevelToCode
import mathlingua.chalktalk.phase2.ast.section.ensureNonNull
import mathlingua.chalktalk.phase2.ast.section.identifySections
import mathlingua.chalktalk.phase2.ast.section.ifNonNull
import mathlingua.chalktalk.phase2.ast.track
import mathlingua.chalktalk.phase2.ast.validateGroup
import mathlingua.support.MutableLocationTracker
import mathlingua.support.ParseError
import mathlingua.transform.signature

data class AxiomGroup(
    val signature: String?,
    val id: IdStatement?,
    val axiomSection: AxiomSection,
    val givenSection: GivenSection?,
    val whereSection: WhereSection?,
    val ifOrIffSection: IfOrIffSection?,
    val thenSection: ThenSection,
    val usingSection: UsingSection?,
    override val metaDataSection: MetaDataSection?
) : TopLevelGroup(metaDataSection) {

    override fun forEach(fn: (node: Phase2Node) -> Unit) {
        if (id != null) {
            fn(id)
        }
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
        if (ifOrIffSection != null) {
            fn(ifOrIffSection.resolve())
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
            axiomSection,
            givenSection,
            whereSection,
            ifOrIffSection?.resolve(),
            thenSection,
            usingSection,
            metaDataSection)

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) =
        chalkTransformer(
            AxiomGroup(
                signature = signature,
                id = id?.transform(chalkTransformer) as IdStatement?,
                axiomSection = axiomSection.transform(chalkTransformer) as AxiomSection,
                givenSection = givenSection?.transform(chalkTransformer) as GivenSection?,
                whereSection = whereSection?.transform(chalkTransformer) as WhereSection?,
                ifOrIffSection = ifOrIffSection?.transform(chalkTransformer),
                thenSection = thenSection.transform(chalkTransformer) as ThenSection,
                usingSection = usingSection?.transform(chalkTransformer) as UsingSection?,
                metaDataSection = metaDataSection?.transform(chalkTransformer) as MetaDataSection?))
}

fun isAxiomGroup(node: Phase1Node) = firstSectionMatchesName(node, "Axiom")

fun validateAxiomGroup(
    node: Phase1Node, errors: MutableList<ParseError>, tracker: MutableLocationTracker
) =
    track(node, tracker) {
        validateGroup(node.resolve(), errors, "Axiom", DEFAULT_AXIOM_GROUP) { group ->
            val id = getOptionalId(group, errors, tracker)
            identifySections(
                group,
                errors,
                DEFAULT_AXIOM_GROUP,
                listOf(
                    "Axiom", "given?", "where?", "if?", "iff?", "then", "using?", "Metadata?")) {
            sections ->
                AxiomGroup(
                    signature = id?.signature(),
                    id = id,
                    axiomSection =
                        ensureNonNull(sections["Axiom"], DEFAULT_AXIOM_SECTION) {
                            validateAxiomSection(it, errors, tracker)
                        },
                    givenSection =
                        ifNonNull(sections["given"]) { validateGivenSection(it, errors, tracker) },
                    whereSection =
                        ifNonNull(sections["where"]) { validateWhereSection(it, errors, tracker) },
                    ifOrIffSection = validateIfOrIffSection(node, sections, errors, tracker),
                    thenSection =
                        ensureNonNull(sections["then"], DEFAULT_THEN_SECTION) {
                            validateThenSection(it, errors, tracker)
                        },
                    usingSection =
                        ifNonNull(sections["using"]) { validateUsingSection(it, errors, tracker) },
                    metaDataSection =
                        ifNonNull(sections["Metadata"]) {
                            validateMetaDataSection(it, errors, tracker)
                        })
            }
        }
    }

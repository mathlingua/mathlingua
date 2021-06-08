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

package mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.resultlike.theorem

import mathlingua.backend.transform.Signature
import mathlingua.backend.transform.signature
import mathlingua.frontend.chalktalk.phase1.ast.Phase1Node
import mathlingua.frontend.chalktalk.phase2.CodeWriter
import mathlingua.frontend.chalktalk.phase2.ast.DEFAULT_THEN_SECTION
import mathlingua.frontend.chalktalk.phase2.ast.DEFAULT_THEOREM_GROUP
import mathlingua.frontend.chalktalk.phase2.ast.DEFAULT_THEOREM_SECTION
import mathlingua.frontend.chalktalk.phase2.ast.clause.IdStatement
import mathlingua.frontend.chalktalk.phase2.ast.clause.firstSectionMatchesName
import mathlingua.frontend.chalktalk.phase2.ast.common.Phase2Node
import mathlingua.frontend.chalktalk.phase2.ast.getOptionalId
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.If.ThenSection
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.If.validateThenSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.HasUsingSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.TopLevelGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.resultlike.IfOrIffSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.resultlike.validateIfOrIffSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.UsingSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.WhereSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.metadata.section.MetaDataSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.metadata.section.validateMetaDataSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.validateUsingSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.validateWhereSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.topLevelToCode
import mathlingua.frontend.chalktalk.phase2.ast.section.ensureNonNull
import mathlingua.frontend.chalktalk.phase2.ast.section.identifySections
import mathlingua.frontend.chalktalk.phase2.ast.section.ifNonNull
import mathlingua.frontend.chalktalk.phase2.ast.track
import mathlingua.frontend.chalktalk.phase2.ast.validateGroup
import mathlingua.frontend.support.MutableLocationTracker
import mathlingua.frontend.support.ParseError

data class TheoremGroup(
    val signature: Signature?,
    val id: IdStatement?,
    val theoremSection: TheoremSection,
    val givenSection: GivenSection?,
    val givenWhereSection: WhereSection?,
    val ifOrIffSection: IfOrIffSection?,
    val thenSection: ThenSection,
    override val usingSection: UsingSection?,
    override val metaDataSection: MetaDataSection?,
    val proofSection: ProofSection?
) : TopLevelGroup(metaDataSection), HasUsingSection {

    override fun forEach(fn: (node: Phase2Node) -> Unit) {
        if (id != null) {
            fn(id)
        }
        fn(theoremSection)
        if (usingSection != null) {
            fn(usingSection)
        }
        if (givenSection != null) {
            fn(givenSection)
        }
        if (givenWhereSection != null) {
            fn(givenWhereSection)
        }
        if (ifOrIffSection != null) {
            fn(ifOrIffSection.resolve())
        }
        fn(thenSection)
        if (metaDataSection != null) {
            fn(metaDataSection)
        }
        if (proofSection != null) {
            fn(proofSection)
        }
    }

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter) =
        topLevelToCode(
            writer,
            isArg,
            indent,
            id,
            theoremSection,
            givenSection,
            givenWhereSection,
            ifOrIffSection?.resolve(),
            thenSection,
            usingSection,
            metaDataSection,
            proofSection)

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) =
        chalkTransformer(
            TheoremGroup(
                signature = signature,
                id = id?.transform(chalkTransformer) as IdStatement?,
                theoremSection = theoremSection.transform(chalkTransformer) as TheoremSection,
                givenSection = givenSection?.transform(chalkTransformer) as GivenSection?,
                givenWhereSection = givenWhereSection?.transform(chalkTransformer) as WhereSection?,
                ifOrIffSection = ifOrIffSection?.transform(chalkTransformer),
                thenSection = thenSection.transform(chalkTransformer) as ThenSection,
                usingSection = usingSection?.transform(chalkTransformer) as UsingSection?,
                metaDataSection = metaDataSection?.transform(chalkTransformer) as MetaDataSection?,
                proofSection = proofSection?.transform(chalkTransformer) as ProofSection?))
}

fun isTheoremGroup(node: Phase1Node) = firstSectionMatchesName(node, "Theorem")

fun validateTheoremGroup(
    node: Phase1Node, errors: MutableList<ParseError>, tracker: MutableLocationTracker
) =
    track(node, tracker) {
        validateGroup(node.resolve(), errors, "Theorem", DEFAULT_THEOREM_GROUP) { group ->
            val id = getOptionalId(group, errors, tracker)
            identifySections(
                group,
                errors,
                DEFAULT_THEOREM_GROUP,
                listOf(
                    "Theorem",
                    "given?",
                    "where?",
                    "if?",
                    "iff?",
                    "then",
                    "using?",
                    "Metadata?",
                    "Proof?")) { sections ->
                TheoremGroup(
                    signature = id?.signature(tracker),
                    id = id,
                    theoremSection =
                        ensureNonNull(sections["Theorem"], DEFAULT_THEOREM_SECTION) {
                            validateTheoremSection(it, errors, tracker)
                        },
                    givenSection =
                        ifNonNull(sections["given"]) { validateGivenSection(it, errors, tracker) },
                    givenWhereSection =
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
                        },
                    proofSection =
                        ifNonNull(sections["Proof"]) { validateProofSection(it, errors, tracker) })
            }
        }
    }

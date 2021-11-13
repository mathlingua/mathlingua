/*
 * Copyright 2020 The MathLingua Authors
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

package mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.resultlike.conjecture

import mathlingua.backend.transform.Signature
import mathlingua.backend.transform.signature
import mathlingua.frontend.chalktalk.phase1.ast.Phase1Node
import mathlingua.frontend.chalktalk.phase2.CodeWriter
import mathlingua.frontend.chalktalk.phase2.ast.DEFAULT_CONJECTURE_GROUP
import mathlingua.frontend.chalktalk.phase2.ast.DEFAULT_CONJECTURE_SECTION
import mathlingua.frontend.chalktalk.phase2.ast.DEFAULT_THEN_SECTION
import mathlingua.frontend.chalktalk.phase2.ast.clause.IdStatement
import mathlingua.frontend.chalktalk.phase2.ast.clause.firstSectionMatchesName
import mathlingua.frontend.chalktalk.phase2.ast.common.Phase2Node
import mathlingua.frontend.chalktalk.phase2.ast.getOptionalId
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.If.ThenSection
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.If.validateThenSection
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.iff.IffSection
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.iff.validateIffSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.HasSignature
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.HasUsingSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.TopLevelGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.resultlike.theorem.GivenSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.resultlike.theorem.validateGivenSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.UsingSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.WhenSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.metadata.section.MetaDataSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.metadata.section.validateMetaDataSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.validateUsingSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.validateWhenSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.topLevelToCode
import mathlingua.frontend.chalktalk.phase2.ast.section.ensureNonNull
import mathlingua.frontend.chalktalk.phase2.ast.section.identifySections
import mathlingua.frontend.chalktalk.phase2.ast.section.ifNonNull
import mathlingua.frontend.chalktalk.phase2.ast.track
import mathlingua.frontend.chalktalk.phase2.ast.validateGroup
import mathlingua.frontend.support.MutableLocationTracker
import mathlingua.frontend.support.ParseError

data class ConjectureGroup(
    override val signature: Signature?,
    override val id: IdStatement?,
    val conjectureSection: ConjectureSection,
    val givenSection: GivenSection?,
    val whenSection: WhenSection?,
    val thenSection: ThenSection,
    val iffSection: IffSection?,
    override val usingSection: UsingSection?,
    override val metaDataSection: MetaDataSection?
) : TopLevelGroup(metaDataSection), HasUsingSection, HasSignature {

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
        if (whenSection != null) {
            fn(whenSection)
        }
        fn(thenSection)
        if (iffSection != null) {
            fn(iffSection)
        }
        if (metaDataSection != null) {
            fn(metaDataSection)
        }
    }

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter) =
        topLevelToCode(
            this,
            writer,
            isArg,
            indent,
            id,
            conjectureSection,
            givenSection,
            whenSection,
            thenSection,
            iffSection,
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
                whenSection = whenSection?.transform(chalkTransformer) as WhenSection?,
                thenSection = thenSection.transform(chalkTransformer) as ThenSection,
                iffSection = iffSection?.transform(chalkTransformer) as IffSection?,
                usingSection = usingSection?.transform(chalkTransformer) as UsingSection?,
                metaDataSection = metaDataSection?.transform(chalkTransformer) as MetaDataSection?))
}

fun isConjectureGroup(node: Phase1Node) = firstSectionMatchesName(node, "Conjecture")

fun validateConjectureGroup(
    node: Phase1Node, errors: MutableList<ParseError>, tracker: MutableLocationTracker
) =
    track(node, tracker) {
        validateGroup(node.resolve(), errors, "Conjecture", DEFAULT_CONJECTURE_GROUP) { group ->
            val id = getOptionalId(group, errors, tracker)
            identifySections(
                group,
                errors,
                DEFAULT_CONJECTURE_GROUP,
                listOf("Conjecture", "given?", "when?", "then", "iff?", "using?", "Metadata?")) {
            sections ->
                ConjectureGroup(
                    signature = id?.signature(tracker),
                    id = id,
                    conjectureSection =
                        ensureNonNull(sections["Conjecture"], DEFAULT_CONJECTURE_SECTION) {
                            validateConjectureSection(it, errors, tracker)
                        },
                    givenSection =
                        ifNonNull(sections["given"]) { validateGivenSection(it, errors, tracker) },
                    whenSection =
                        ifNonNull(sections["when"]) { validateWhenSection(it, errors, tracker) },
                    thenSection =
                        ensureNonNull(sections["then"], DEFAULT_THEN_SECTION) {
                            validateThenSection(it, errors, tracker)
                        },
                    iffSection =
                        ifNonNull(sections["iff"]) { validateIffSection(it, errors, tracker) },
                    usingSection =
                        ifNonNull(sections["using"]) { validateUsingSection(it, errors, tracker) },
                    metaDataSection =
                        ifNonNull(sections["Metadata"]) {
                            validateMetaDataSection(it, errors, tracker)
                        })
            }
        }
    }

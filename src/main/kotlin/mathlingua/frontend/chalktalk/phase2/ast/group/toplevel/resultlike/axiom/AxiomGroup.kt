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

package mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.resultlike.axiom

import mathlingua.backend.transform.Signature
import mathlingua.backend.transform.signature
import mathlingua.frontend.chalktalk.phase1.ast.Phase1Node
import mathlingua.frontend.chalktalk.phase2.CodeWriter
import mathlingua.frontend.chalktalk.phase2.ast.DEFAULT_AXIOM_GROUP
import mathlingua.frontend.chalktalk.phase2.ast.DEFAULT_AXIOM_SECTION
import mathlingua.frontend.chalktalk.phase2.ast.DEFAULT_THEN_SECTION
import mathlingua.frontend.chalktalk.phase2.ast.clause.IdStatement
import mathlingua.frontend.chalktalk.phase2.ast.clause.firstSectionMatchesName
import mathlingua.frontend.chalktalk.phase2.ast.common.Phase2Node
import mathlingua.frontend.chalktalk.phase2.ast.getOptionalId
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.If.ThenSection
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.If.validateThenSection
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.exists.SuchThatSection
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.exists.validateSuchThatSection
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.iff.IffSection
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.iff.validateIffSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.HasSignature
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.HasUsingSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.TopLevelGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.providing.symbols.WhereSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.providing.symbols.validateWhereSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.resultlike.theorem.GivenSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.resultlike.theorem.validateGivenSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.UsingSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.metadata.section.MetaDataSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.metadata.section.validateMetaDataSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.validateUsingSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.topLevelToCode
import mathlingua.frontend.chalktalk.phase2.ast.section.ensureNonNull
import mathlingua.frontend.chalktalk.phase2.ast.section.identifySections
import mathlingua.frontend.chalktalk.phase2.ast.section.ifNonNull
import mathlingua.frontend.chalktalk.phase2.ast.validateGroup
import mathlingua.frontend.support.ParseError

internal data class AxiomGroup(
    override val signature: Signature?,
    override val id: IdStatement?,
    val axiomSection: AxiomSection,
    val givenSection: GivenSection?,
    val whereSection: WhereSection?,
    val suchThatSection: SuchThatSection?,
    val thenSection: ThenSection,
    val iffSection: IffSection?,
    override val usingSection: UsingSection?,
    override val metaDataSection: MetaDataSection?,
    override val row: Int,
    override val column: Int
) : TopLevelGroup(metaDataSection), HasUsingSection, HasSignature {

    override fun forEach(fn: (node: Phase2Node) -> Unit) {
        if (id != null) {
            fn(id)
        }
        fn(axiomSection)
        if (givenSection != null) {
            fn(givenSection)
        }
        if (whereSection != null) {
            fn(whereSection)
        }
        if (suchThatSection != null) {
            fn(suchThatSection)
        }
        fn(thenSection)
        if (iffSection != null) {
            fn(iffSection)
        }
        if (usingSection != null) {
            fn(usingSection)
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
            axiomSection,
            givenSection,
            whereSection,
            suchThatSection,
            thenSection,
            iffSection,
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
                suchThatSection = suchThatSection?.transform(chalkTransformer) as SuchThatSection?,
                thenSection = thenSection.transform(chalkTransformer) as ThenSection,
                iffSection = iffSection?.transform(chalkTransformer) as IffSection?,
                usingSection = usingSection?.transform(chalkTransformer) as UsingSection?,
                metaDataSection = metaDataSection?.transform(chalkTransformer) as MetaDataSection?,
                row,
                column))
}

internal fun isAxiomGroup(node: Phase1Node) = firstSectionMatchesName(node, "Axiom")

internal fun validateAxiomGroup(node: Phase1Node, errors: MutableList<ParseError>) =
    validateGroup(node.resolve(), errors, "Axiom", DEFAULT_AXIOM_GROUP) { group ->
        val id = getOptionalId(group, errors)
        identifySections(
            group,
            errors,
            DEFAULT_AXIOM_GROUP,
            listOf(
                "Axiom", "given?", "where?", "suchThat?", "then", "iff?", "using?", "Metadata?")) {
        sections ->
            AxiomGroup(
                signature = id?.signature(),
                id = id,
                axiomSection =
                    ensureNonNull(sections["Axiom"], DEFAULT_AXIOM_SECTION) {
                        validateAxiomSection(it, errors)
                    },
                givenSection = ifNonNull(sections["given"]) { validateGivenSection(it, errors) },
                whereSection = ifNonNull(sections["where"]) { validateWhereSection(it, errors) },
                suchThatSection =
                    ifNonNull(sections["suchThat"]) { validateSuchThatSection(it, errors) },
                thenSection =
                    ensureNonNull(sections["then"], DEFAULT_THEN_SECTION) {
                        validateThenSection(it, errors)
                    },
                iffSection = ifNonNull(sections["iff"]) { validateIffSection(it, errors) },
                usingSection = ifNonNull(sections["using"]) { validateUsingSection(it, errors) },
                metaDataSection =
                    ifNonNull(sections["Metadata"]) { validateMetaDataSection(it, errors) },
                row = node.row,
                column = node.column)
        }
    }

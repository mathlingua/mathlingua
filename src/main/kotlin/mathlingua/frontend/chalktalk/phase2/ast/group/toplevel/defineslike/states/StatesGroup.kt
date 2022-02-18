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

package mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.states

import mathlingua.backend.transform.Signature
import mathlingua.backend.transform.signature
import mathlingua.frontend.chalktalk.phase1.ast.Phase1Node
import mathlingua.frontend.chalktalk.phase2.CodeWriter
import mathlingua.frontend.chalktalk.phase2.ast.DEFAULT_STATES_GROUP
import mathlingua.frontend.chalktalk.phase2.ast.DEFAULT_STATES_SECTION
import mathlingua.frontend.chalktalk.phase2.ast.DEFAULT_THAT_SECTION
import mathlingua.frontend.chalktalk.phase2.ast.DEFAULT_WRITTEN_SECTION
import mathlingua.frontend.chalktalk.phase2.ast.clause.DefinesStatesOrViews
import mathlingua.frontend.chalktalk.phase2.ast.clause.IdStatement
import mathlingua.frontend.chalktalk.phase2.ast.clause.firstSectionMatchesName
import mathlingua.frontend.chalktalk.phase2.ast.common.Phase2Node
import mathlingua.frontend.chalktalk.phase2.ast.getId
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.HasSignature
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.HasUsingSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.TopLevelGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.CalledSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.WrittenSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.validateCalledSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.validateWrittenSection
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
import mathlingua.frontend.chalktalk.phase2.ast.validateGroup
import mathlingua.frontend.support.ParseError

internal data class StatesGroup(
    override val signature: Signature?,
    override val id: IdStatement,
    val statesSection: StatesSection,
    val givenSection: GivenSection?,
    val whenSection: WhenSection?,
    val thatSection: ThatSection,
    override val usingSection: UsingSection?,
    val writtenSection: WrittenSection,
    val calledSection: CalledSection?,
    override val metaDataSection: MetaDataSection?,
    override val row: Int,
    override val column: Int
) : TopLevelGroup(metaDataSection), HasUsingSection, HasSignature, DefinesStatesOrViews {

    override fun forEach(fn: (node: Phase2Node) -> Unit) {
        fn(id)
        fn(statesSection)
        if (givenSection != null) {
            fn(givenSection)
        }
        if (whenSection != null) {
            fn(whenSection)
        }
        fn(thatSection)
        if (usingSection != null) {
            fn(usingSection)
        }
        fn(writtenSection)
        if (calledSection != null) {
            fn(calledSection)
        }
        if (metaDataSection != null) {
            fn(metaDataSection)
        }
    }

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter): CodeWriter {
        val sections = mutableListOf(statesSection, givenSection, whenSection)
        sections.add(thatSection)
        sections.add(usingSection)
        sections.add(writtenSection)
        sections.add(calledSection)
        sections.add(metaDataSection)
        return topLevelToCode(this, writer, isArg, indent, id, *sections.toTypedArray())
    }

    fun getCalled() =
        calledSection?.forms
            ?: writtenSection.forms.map {
                "$${it.removeSurrounding("\"", "\"").replace("textrm", "textbf")}$"
            }

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) =
        chalkTransformer(
            StatesGroup(
                signature = signature,
                id = id.transform(chalkTransformer) as IdStatement,
                statesSection = statesSection.transform(chalkTransformer) as StatesSection,
                givenSection = givenSection?.transform(chalkTransformer) as GivenSection?,
                whenSection = whenSection?.transform(chalkTransformer) as WhenSection?,
                thatSection = chalkTransformer(thatSection) as ThatSection,
                usingSection = usingSection?.transform(chalkTransformer) as UsingSection?,
                writtenSection = writtenSection.transform(chalkTransformer) as WrittenSection,
                calledSection = calledSection?.transform(chalkTransformer) as CalledSection?,
                metaDataSection = metaDataSection?.transform(chalkTransformer) as MetaDataSection?,
                row = row,
                column = column))
}

internal fun isStatesGroup(node: Phase1Node) = firstSectionMatchesName(node, "States")

internal fun validateStatesGroup(node: Phase1Node, errors: MutableList<ParseError>) =
    validateGroup(node.resolve(), errors, "States", DEFAULT_STATES_GROUP) { group ->
        identifySections(
            group,
            errors,
            DEFAULT_STATES_GROUP,
            listOf(
                "States",
                "given?",
                "when?",
                "that",
                "using?",
                "written",
                "called?",
                "Metadata?")) { sections ->
            val id = getId(group, errors)
            StatesGroup(
                signature = id.signature(),
                id = id,
                statesSection =
                    ensureNonNull(sections["States"], DEFAULT_STATES_SECTION) {
                        validateStatesSection(it, errors)
                    },
                givenSection = ifNonNull(sections["given"]) { validateGivenSection(it, errors) },
                whenSection = ifNonNull(sections["when"]) { validateWhenSection(it, errors) },
                thatSection =
                    ensureNonNull(sections["that"], DEFAULT_THAT_SECTION) {
                        validateThatSection(it, errors)
                    },
                usingSection = ifNonNull(sections["using"]) { validateUsingSection(it, errors) },
                writtenSection =
                    ensureNonNull(sections["written"], DEFAULT_WRITTEN_SECTION) {
                        validateWrittenSection(it, errors)
                    },
                calledSection = ifNonNull(sections["called"]) { validateCalledSection(it, errors) },
                metaDataSection =
                    ifNonNull(sections["Metadata"]) { validateMetaDataSection(it, errors) },
                row = node.row,
                column = node.column)
        }
    }

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

package mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.views

import mathlingua.backend.transform.signature
import mathlingua.frontend.chalktalk.phase1.ast.Phase1Node
import mathlingua.frontend.chalktalk.phase2.CodeWriter
import mathlingua.frontend.chalktalk.phase2.ast.DEFAULT_ID_STATEMENT
import mathlingua.frontend.chalktalk.phase2.ast.DEFAULT_SINGLE_AS_SECTION
import mathlingua.frontend.chalktalk.phase2.ast.DEFAULT_SINGLE_FROM_SECTION
import mathlingua.frontend.chalktalk.phase2.ast.DEFAULT_SINGLE_TO_SECTION
import mathlingua.frontend.chalktalk.phase2.ast.DEFAULT_VIEWS_GROUP
import mathlingua.frontend.chalktalk.phase2.ast.DEFAULT_VIEWS_SECTION
import mathlingua.frontend.chalktalk.phase2.ast.clause.IdStatement
import mathlingua.frontend.chalktalk.phase2.ast.clause.firstSectionMatchesName
import mathlingua.frontend.chalktalk.phase2.ast.common.Phase2Node
import mathlingua.frontend.chalktalk.phase2.ast.getId
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.HasUsingSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.TopLevelGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.foundation.DefinesStatesOrViews
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.UsingSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.metadata.section.MetaDataSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.metadata.section.validateMetaDataSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.validateUsingSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.topLevelToCode
import mathlingua.frontend.chalktalk.phase2.ast.section.ensureNonNull
import mathlingua.frontend.chalktalk.phase2.ast.section.identifySections
import mathlingua.frontend.chalktalk.phase2.ast.section.ifNonNull
import mathlingua.frontend.chalktalk.phase2.ast.track
import mathlingua.frontend.chalktalk.phase2.ast.validateGroup
import mathlingua.frontend.support.MutableLocationTracker
import mathlingua.frontend.support.ParseError

data class ViewsGroup(
    val signature: String?,
    val id: IdStatement,
    val viewsSection: ViewsSection,
    val singleFromSection: SingleFromSection,
    val singleToSection: SingleToSection,
    val asSection: SingleAsSection,
    override val usingSection: UsingSection?,
    override val metaDataSection: MetaDataSection?
) : TopLevelGroup(metaDataSection), HasUsingSection, DefinesStatesOrViews {

    override fun forEach(fn: (node: Phase2Node) -> Unit) {
        fn(id)
        fn(viewsSection)
        fn(singleFromSection)
        fn(singleToSection)
        fn(asSection)
        if (usingSection != null) {
            fn(usingSection)
        }
        if (metaDataSection != null) {
            fn(metaDataSection)
        }
    }

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter): CodeWriter {
        return topLevelToCode(
            writer,
            isArg,
            indent,
            id,
            viewsSection,
            singleFromSection,
            singleToSection,
            asSection,
            usingSection,
            metaDataSection)
    }

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) =
        chalkTransformer(
            ViewsGroup(
                signature = signature,
                id = id.transform(chalkTransformer) as IdStatement,
                viewsSection = viewsSection.transform(chalkTransformer) as ViewsSection,
                singleFromSection =
                    singleFromSection.transform(chalkTransformer) as SingleFromSection,
                singleToSection = singleToSection.transform(chalkTransformer) as SingleToSection,
                asSection = asSection.transform(chalkTransformer) as SingleAsSection,
                usingSection = usingSection?.transform(chalkTransformer) as UsingSection?,
                metaDataSection = metaDataSection?.transform(chalkTransformer) as MetaDataSection?))
}

fun isViewsGroup(node: Phase1Node) = firstSectionMatchesName(node, "Views")

fun validateViewsGroup(
    node: Phase1Node, errors: MutableList<ParseError>, tracker: MutableLocationTracker
) =
    track(node, tracker) {
        validateGroup(node.resolve(), errors, "Views", DEFAULT_VIEWS_GROUP) { group ->
            identifySections(
                group,
                errors,
                DEFAULT_VIEWS_GROUP,
                listOf("Views", "from", "to", "as", "using?", "Metadata?")) { sections ->
                val id = getId(group, errors, DEFAULT_ID_STATEMENT, tracker)
                ViewsGroup(
                    signature = id.signature(),
                    id = id,
                    viewsSection =
                        ensureNonNull(sections["Views"], DEFAULT_VIEWS_SECTION) {
                            validateViewsSection(it, errors, tracker)
                        },
                    singleFromSection =
                        ensureNonNull(sections["from"], DEFAULT_SINGLE_FROM_SECTION) {
                            validateSingleFromSection(it, errors, tracker)
                        },
                    singleToSection =
                        ensureNonNull(sections["to"], DEFAULT_SINGLE_TO_SECTION) {
                            validateSingleToSection(it, errors, tracker)
                        },
                    asSection =
                        ensureNonNull(sections["as"], DEFAULT_SINGLE_AS_SECTION) {
                            validateSingleAsSection(it, errors, tracker)
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

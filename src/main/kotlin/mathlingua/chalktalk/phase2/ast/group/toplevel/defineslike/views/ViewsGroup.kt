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

package mathlingua.chalktalk.phase2.ast.group.toplevel.defineslike.views

import mathlingua.chalktalk.phase1.ast.Group
import mathlingua.chalktalk.phase1.ast.Phase1Node
import mathlingua.chalktalk.phase2.CodeWriter
import mathlingua.chalktalk.phase2.ast.DEFAULT_ID_STATEMENT
import mathlingua.chalktalk.phase2.ast.DEFAULT_SINGLE_AS_SECTION
import mathlingua.chalktalk.phase2.ast.DEFAULT_SINGLE_FROM_SECTION
import mathlingua.chalktalk.phase2.ast.DEFAULT_SINGLE_TO_SECTION
import mathlingua.chalktalk.phase2.ast.DEFAULT_VIEWS_GROUP
import mathlingua.chalktalk.phase2.ast.DEFAULT_VIEWS_SECTION
import mathlingua.chalktalk.phase2.ast.clause.IdStatement
import mathlingua.chalktalk.phase2.ast.clause.Validator
import mathlingua.chalktalk.phase2.ast.clause.firstSectionMatchesName
import mathlingua.chalktalk.phase2.ast.clause.validateIdMetadataGroup
import mathlingua.chalktalk.phase2.ast.common.Phase2Node
import mathlingua.chalktalk.phase2.ast.group.toplevel.TopLevelGroup
import mathlingua.chalktalk.phase2.ast.group.toplevel.defineslike.foundation.DefinesStatesOrViews
import mathlingua.chalktalk.phase2.ast.group.toplevel.shared.UsingSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.shared.metadata.section.MetaDataSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.shared.metadata.section.neoValidateMetaDataSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.shared.neoValidateUsingSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.shared.validateUsingSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.topLevelToCode
import mathlingua.chalktalk.phase2.ast.neoGetId
import mathlingua.chalktalk.phase2.ast.neoTrack
import mathlingua.chalktalk.phase2.ast.neoValidateGroup
import mathlingua.chalktalk.phase2.ast.section.neoEnsureNonNull
import mathlingua.chalktalk.phase2.ast.section.neoIdentifySections
import mathlingua.chalktalk.phase2.ast.section.neoIfNonNull
import mathlingua.support.MutableLocationTracker
import mathlingua.support.ParseError
import mathlingua.support.validationSuccess
import mathlingua.transform.signature

data class ViewsGroup(
    val signature: String?,
    val id: IdStatement,
    val viewsSection: ViewsSection,
    val singleFromSection: SingleFromSection,
    val singleToSection: SingleToSection,
    val asSection: SingleAsSection,
    val usingSection: UsingSection?,
    override val metaDataSection: MetaDataSection?
) : TopLevelGroup(metaDataSection), DefinesStatesOrViews {

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

fun validateViewsGroup(groupNode: Group, tracker: MutableLocationTracker) =
    validateIdMetadataGroup(
        tracker,
        groupNode,
        listOf(
            Validator(name = "Views", optional = false, ::validateViewsSection),
            Validator(name = "from", optional = false, ::validateSingleFromSection),
            Validator(name = "to", optional = false, ::validateSingleToSection),
            Validator(name = "as", optional = false, ::validateSingleAsSection),
            Validator(name = "using", optional = true, ::validateUsingSection))) {
    id,
    sections,
    metaDataSection ->
        validationSuccess(
            tracker,
            groupNode,
            ViewsGroup(
                id.signature(),
                id,
                sections["Views"] as ViewsSection,
                sections["from"] as SingleFromSection,
                sections["to"] as SingleToSection,
                sections["as"] as SingleAsSection,
                sections["using"] as UsingSection?,
                metaDataSection))
    }

fun neoValidateViewsGroup(
    node: Phase1Node, errors: MutableList<ParseError>, tracker: MutableLocationTracker
) =
    neoTrack(node, tracker) {
        neoValidateGroup(node.resolve(), errors, "Views", DEFAULT_VIEWS_GROUP) { group ->
            neoIdentifySections(
                group,
                errors,
                DEFAULT_VIEWS_GROUP,
                listOf("Views", "from", "to", "as", "using?", "Metadata?")) { sections ->
                val id = neoGetId(group, errors, DEFAULT_ID_STATEMENT, tracker)
                ViewsGroup(
                    signature = id.signature(),
                    id = id,
                    viewsSection =
                        neoEnsureNonNull(sections["Views"], DEFAULT_VIEWS_SECTION) {
                            neoValidateViewsSection(it, errors, tracker)
                        },
                    singleFromSection =
                        neoEnsureNonNull(sections["from"], DEFAULT_SINGLE_FROM_SECTION) {
                            neoValidateSingleFromSection(it, errors, tracker)
                        },
                    singleToSection =
                        neoEnsureNonNull(sections["to"], DEFAULT_SINGLE_TO_SECTION) {
                            neoValidateSingleToSection(it, errors, tracker)
                        },
                    asSection =
                        neoEnsureNonNull(sections["as"], DEFAULT_SINGLE_AS_SECTION) {
                            neoValidateSingleAsSection(it, errors, tracker)
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

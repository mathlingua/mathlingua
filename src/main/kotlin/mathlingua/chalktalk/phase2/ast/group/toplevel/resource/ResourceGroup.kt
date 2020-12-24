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

package mathlingua.chalktalk.phase2.ast.group.toplevel.resource

import mathlingua.chalktalk.phase1.ast.Phase1Node
import mathlingua.chalktalk.phase2.CodeWriter
import mathlingua.chalktalk.phase2.ast.DEFAULT_RESOURCE_GROUP
import mathlingua.chalktalk.phase2.ast.DEFAULT_RESOURCE_SECTION
import mathlingua.chalktalk.phase2.ast.clause.IdStatement
import mathlingua.chalktalk.phase2.ast.clause.firstSectionMatchesName
import mathlingua.chalktalk.phase2.ast.common.Phase2Node
import mathlingua.chalktalk.phase2.ast.group.toplevel.TopLevelGroup
import mathlingua.chalktalk.phase2.ast.group.toplevel.shared.metadata.section.MetaDataSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.shared.metadata.section.neoValidateMetaDataSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.topLevelToCode
import mathlingua.chalktalk.phase2.ast.neoTrack
import mathlingua.chalktalk.phase2.ast.neoValidateGroup
import mathlingua.chalktalk.phase2.ast.section.neoEnsureNonNull
import mathlingua.chalktalk.phase2.ast.section.neoIdentifySections
import mathlingua.chalktalk.phase2.ast.section.neoIfNonNull
import mathlingua.support.MutableLocationTracker
import mathlingua.support.ParseError
import mathlingua.support.validationFailure

data class ResourceGroup(
    val id: String,
    val sourceSection: ResourceSection,
    override val metaDataSection: MetaDataSection?
) : TopLevelGroup(metaDataSection) {

    override fun forEach(fn: (node: Phase2Node) -> Unit) {
        fn(sourceSection)
        if (metaDataSection != null) {
            fn(metaDataSection)
        }
    }

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter) =
        topLevelToCode(
            writer,
            isArg,
            indent,
            IdStatement(id, validationFailure(emptyList())),
            sourceSection,
            metaDataSection)

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) =
        chalkTransformer(
            ResourceGroup(
                id = id,
                sourceSection = sourceSection.transform(chalkTransformer) as ResourceSection,
                metaDataSection = metaDataSection?.transform(chalkTransformer) as? MetaDataSection))
}

fun isResourceGroup(node: Phase1Node) = firstSectionMatchesName(node, "Resource")

fun neoValidateResourceGroup(
    node: Phase1Node, errors: MutableList<ParseError>, tracker: MutableLocationTracker
) =
    neoTrack(node, tracker) {
        neoValidateGroup(node.resolve(), errors, "Resource", DEFAULT_RESOURCE_GROUP) { group ->
            neoIdentifySections(
                group, errors, DEFAULT_RESOURCE_GROUP, listOf("Resource", "Metadata?")) {
            sections ->
                if (group.id == null) {
                    DEFAULT_RESOURCE_GROUP
                } else {
                    ResourceGroup(
                        id = group.id.text.removeSurrounding("[", "]"),
                        sourceSection =
                            neoEnsureNonNull(sections["Resource"], DEFAULT_RESOURCE_SECTION) {
                                neoValidateResourceSection(it, errors, tracker)
                            },
                        metaDataSection =
                            neoIfNonNull(sections["Metadata"]) {
                                neoValidateMetaDataSection(it, errors, tracker)
                            })
                }
            }
        }
    }
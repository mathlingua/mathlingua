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

package mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.resource

import mathlingua.frontend.chalktalk.phase1.ast.Phase1Node
import mathlingua.frontend.chalktalk.phase2.CodeWriter
import mathlingua.frontend.chalktalk.phase2.ast.DEFAULT_RESOURCE_GROUP
import mathlingua.frontend.chalktalk.phase2.ast.DEFAULT_RESOURCE_SECTION
import mathlingua.frontend.chalktalk.phase2.ast.clause.IdStatement
import mathlingua.frontend.chalktalk.phase2.ast.clause.firstSectionMatchesName
import mathlingua.frontend.chalktalk.phase2.ast.common.Phase2Node
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.TopLevelGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.metadata.section.MetaDataSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.metadata.section.validateMetaDataSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.topLevelToCode
import mathlingua.frontend.chalktalk.phase2.ast.section.ensureNonNull
import mathlingua.frontend.chalktalk.phase2.ast.section.identifySections
import mathlingua.frontend.chalktalk.phase2.ast.section.ifNonNull
import mathlingua.frontend.chalktalk.phase2.ast.validateGroup
import mathlingua.frontend.support.ParseError
import mathlingua.frontend.support.validationFailure

internal data class ResourceGroup(
    val id: String,
    val resourceSection: ResourceSection,
    override val metaDataSection: MetaDataSection?,
    override val row: Int,
    override val column: Int
) : TopLevelGroup(metaDataSection) {

    override fun forEach(fn: (node: Phase2Node) -> Unit) {
        fn(resourceSection)
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
            IdStatement(id, validationFailure(emptyList()), row, column),
            resourceSection,
            metaDataSection)

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) =
        chalkTransformer(
            ResourceGroup(
                id = id,
                resourceSection = resourceSection.transform(chalkTransformer) as ResourceSection,
                metaDataSection = metaDataSection?.transform(chalkTransformer) as? MetaDataSection,
                row = row,
                column = column))
}

internal fun isResourceGroup(node: Phase1Node) = firstSectionMatchesName(node, "Resource")

internal fun validateResourceGroup(node: Phase1Node, errors: MutableList<ParseError>) =
    validateGroup(node.resolve(), errors, "Resource", DEFAULT_RESOURCE_GROUP) { group ->
        identifySections(group, errors, DEFAULT_RESOURCE_GROUP, listOf("Resource", "Metadata?")) {
        sections ->
            if (group.id == null) {
                DEFAULT_RESOURCE_GROUP
            } else {
                ResourceGroup(
                    id = group.id.text.removeSurrounding("[", "]"),
                    resourceSection =
                        ensureNonNull(sections["Resource"], DEFAULT_RESOURCE_SECTION) {
                            validateResourceSection(it, errors)
                        },
                    metaDataSection =
                        ifNonNull(sections["Metadata"]) { validateMetaDataSection(it, errors) },
                    row = node.row,
                    column = node.column)
            }
        }
    }

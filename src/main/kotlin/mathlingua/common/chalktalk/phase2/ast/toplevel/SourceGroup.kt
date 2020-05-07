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

package mathlingua.common.chalktalk.phase2.ast.toplevel

import mathlingua.common.*
import mathlingua.common.chalktalk.phase1.ast.Group
import mathlingua.common.chalktalk.phase1.ast.Phase1Node
import mathlingua.common.chalktalk.phase1.ast.getColumn
import mathlingua.common.chalktalk.phase1.ast.getRow
import mathlingua.common.chalktalk.phase2.CodeWriter
import mathlingua.common.chalktalk.phase2.ast.Phase2Node
import mathlingua.common.chalktalk.phase2.ast.clause.IdStatement
import mathlingua.common.chalktalk.phase2.ast.clause.firstSectionMatchesName
import mathlingua.common.chalktalk.phase2.ast.metadata.section.MetaDataSection
import mathlingua.common.chalktalk.phase2.ast.section.SourceSection
import mathlingua.common.chalktalk.phase2.ast.metadata.section.validateMetaDataSection
import mathlingua.common.chalktalk.phase2.ast.section.validateSourceSection

data class SourceGroup(
    val id: String,
    val sourceSection: SourceSection,
    override val metaDataSection: MetaDataSection?
) : TopLevelGroup(metaDataSection) {
    override fun forEach(fn: (node: Phase2Node) -> Unit) {
        fn(sourceSection)
        if (metaDataSection != null) {
            fn(metaDataSection)
        }
    }

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter) = toCode(writer, isArg, indent,
            IdStatement(
                    id,
                    validationFailure(emptyList())
            ), sourceSection, metaDataSection)

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) = chalkTransformer(SourceGroup(
            id = id,
            sourceSection = sourceSection.transform(chalkTransformer) as SourceSection,
            metaDataSection = metaDataSection?.transform(chalkTransformer) as? MetaDataSection
    ))
}

fun isSourceGroup(node: Phase1Node) = firstSectionMatchesName(node, "Source")

fun validateSourceGroup(groupNode: Group, tracker: MutableLocationTracker): Validation<SourceGroup> {
    val id = groupNode.id
    if (id == null) {
        return validationFailure(listOf(
                ParseError("A Source group must have an id",
                        getRow(groupNode), getColumn(groupNode))
        ))
    }

    // id.text is of the form [...]
    // The [ and ] need to be removed.
    val idText = id.text.substring(1, id.text.length - 1)

    val errors = mutableListOf<ParseError>()
    if (!Regex("[a-zA-Z0-9]+").matches(idText)) {
        errors.add(
                ParseError("A source id can only contain numbers and letters",
                        getRow(groupNode), getColumn(groupNode)
                )
        )
    }

    val sections = groupNode.sections
    if (sections.isEmpty()) {
        errors.add(
                ParseError("Expected a Source section",
                        getRow(groupNode), getColumn(groupNode))
        )
    }

    val sourceSection = sections[0]
    val sourceValidation = validateSourceSection(sourceSection, tracker)
    if (sourceValidation is ValidationFailure) {
        errors.addAll(sourceValidation.errors)
    }

    var metaDataSection: MetaDataSection? = null
    if (sections.size >= 2) {
        val metadataValidation = validateMetaDataSection(sections[1], tracker)
        metaDataSection = when (metadataValidation) {
            is ValidationFailure -> {
                errors.addAll(metadataValidation.errors)
                null
            }
            is ValidationSuccess -> {
                metadataValidation.value
            }
        }
    }

    if (sections.size > 2) {
        errors.add(
                ParseError("A Source group can only have a Source section and optionally a Metadata section",
                        getRow(groupNode), getColumn(groupNode))
        )
    }

    if (errors.isNotEmpty()) {
        return validationFailure(errors)
    }

    return validationSuccess(
            tracker,
            groupNode,
            SourceGroup(
                    id = idText,
                    sourceSection = (sourceValidation as ValidationSuccess).value,
                    metaDataSection = metaDataSection
            )
    )
}

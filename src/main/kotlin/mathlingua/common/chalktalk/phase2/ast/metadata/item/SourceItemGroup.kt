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

package mathlingua.common.chalktalk.phase2.ast.metadata.item

import mathlingua.common.chalktalk.phase1.ast.*
import mathlingua.common.chalktalk.phase1.ast.getColumn
import mathlingua.common.chalktalk.phase1.ast.getRow
import mathlingua.common.chalktalk.phase2.CodeWriter
import mathlingua.common.chalktalk.phase2.ast.Phase2Node
import mathlingua.common.chalktalk.phase2.ast.clause.firstSectionMatchesName
import mathlingua.common.chalktalk.phase2.ast.metadata.section.ContentItemSection
import mathlingua.common.chalktalk.phase2.ast.metadata.section.OffsetItemSection
import mathlingua.common.chalktalk.phase2.ast.metadata.section.PageItemSection
import mathlingua.common.chalktalk.phase2.ast.metadata.section.SourceItemSection
import mathlingua.common.chalktalk.phase2.ast.metadata.validateStringSection
import mathlingua.common.chalktalk.phase2.ast.section.identifySections
import mathlingua.common.support.MutableLocationTracker
import mathlingua.common.support.ParseError
import mathlingua.common.support.Validation
import mathlingua.common.support.ValidationFailure
import mathlingua.common.support.ValidationSuccess
import mathlingua.common.support.validationFailure
import mathlingua.common.support.validationSuccess

data class SourceItemGroup(
    val sourceSection: SourceItemSection,
    val pageSection: PageItemSection?,
    val offsetSection: OffsetItemSection?,
    val contentSection: ContentItemSection?
) : MetaDataItem {
    override fun forEach(fn: (node: Phase2Node) -> Unit) {
        fn(sourceSection)

        if (pageSection != null) {
            fn(pageSection)
        }

        if (offsetSection != null) {
            fn(offsetSection)
        }

        if (contentSection != null) {
            fn(contentSection)
        }
    }

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter) = mathlingua.common.chalktalk.phase2.ast.toplevel.topLevelToCode(
            writer,
            isArg,
            indent,
            null,
            sourceSection,
            pageSection,
            offsetSection,
            contentSection
    )

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) =
            chalkTransformer(SourceItemGroup(
                    sourceSection = sourceSection.transform(chalkTransformer) as SourceItemSection,
                    pageSection = pageSection?.transform(chalkTransformer) as PageItemSection,
                    offsetSection = offsetSection?.transform(chalkTransformer) as OffsetItemSection,
                    contentSection = contentSection?.transform(chalkTransformer) as ContentItemSection
            ))
}

fun isSourceItemGroup(node: Phase1Node) = firstSectionMatchesName(node, "source")

fun validateSourceItemGroup(groupNode: Group, tracker: MutableLocationTracker): Validation<SourceItemGroup> {
    val errors = ArrayList<ParseError>()
    val group = groupNode.resolve()
    if (group.id != null) {
        errors.add(
                ParseError(
                        "A reference source cannot have an Id",
                        getRow(group), getColumn(group)
                )
        )
    }

    val sections = group.sections

    val sectionMap: Map<String, List<Section>>
    try {
        sectionMap = identifySections(
                sections, "source", "page?", "offset?", "content?"
        )
    } catch (e: ParseError) {
        errors.add(ParseError(e.message, e.row, e.column))
        return validationFailure(errors)
    }

    val rawSource = sectionMap["source"]!![0]
    var sourceSection: SourceItemSection? = null
    when (val validation = validateStringSection(tracker, rawSource, "source", ::SourceItemSection)) {
        is ValidationSuccess -> sourceSection = validation.value
        is ValidationFailure -> errors.addAll(validation.errors)
    }

    val rawPage = sectionMap["page"]
    var pageSection: PageItemSection? = null
    if (rawPage != null && rawPage.isNotEmpty()) {
        when (val validation = validateStringSection(tracker, rawPage[0], "page", ::PageItemSection)) {
            is ValidationSuccess -> pageSection = validation.value
            is ValidationFailure -> errors.addAll(validation.errors)
        }
    }

    val rawOffset = sectionMap["offset"]
    var offsetSection: OffsetItemSection? = null
    if (rawOffset != null && rawOffset.isNotEmpty()) {
        when (val validation = validateStringSection(tracker, rawOffset[0], "offset", ::OffsetItemSection)) {
            is ValidationSuccess -> offsetSection = validation.value
            is ValidationFailure -> errors.addAll(validation.errors)
        }
    }

    val rawContent = sectionMap["content"]
    var contentSection: ContentItemSection? = null
    if (rawContent != null && rawContent.isNotEmpty()) {
        when (val validation = validateStringSection(tracker, rawContent[0], "content", ::ContentItemSection)) {
            is ValidationSuccess -> contentSection = validation.value
            is ValidationFailure -> errors.addAll(validation.errors)
        }
    }

    return if (errors.isNotEmpty()) {
        validationFailure(errors)
    } else {
        validationSuccess(
                tracker,
                groupNode,
                SourceItemGroup(
                        sourceSection = sourceSection!!,
                        pageSection = pageSection,
                        offsetSection = offsetSection,
                        contentSection = contentSection)
        )
    }
}

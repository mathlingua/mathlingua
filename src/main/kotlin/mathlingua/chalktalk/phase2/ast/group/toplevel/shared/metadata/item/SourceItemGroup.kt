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

package mathlingua.chalktalk.phase2.ast.group.toplevel.shared.metadata.item

import mathlingua.chalktalk.phase1.ast.Group
import mathlingua.chalktalk.phase1.ast.Phase1Node
import mathlingua.common.chalktalk.phase1.ast.*
import mathlingua.chalktalk.phase2.CodeWriter
import mathlingua.chalktalk.phase2.ast.clause.Validator
import mathlingua.chalktalk.phase2.ast.common.Phase2Node
import mathlingua.chalktalk.phase2.ast.clause.firstSectionMatchesName
import mathlingua.chalktalk.phase2.ast.clause.validateGroup
import mathlingua.chalktalk.phase2.ast.group.toplevel.topLevelToCode
import mathlingua.chalktalk.phase2.ast.group.toplevel.shared.metadata.section.ContentItemSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.shared.metadata.section.OffsetItemSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.shared.metadata.section.PageItemSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.shared.metadata.section.SourceItemSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.shared.metadata.validateStringSection
import mathlingua.support.MutableLocationTracker
import mathlingua.support.validationSuccess

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

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter) = topLevelToCode(
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
            chalkTransformer(
                SourceItemGroup(
                    sourceSection = sourceSection.transform(chalkTransformer) as SourceItemSection,
                    pageSection = pageSection?.transform(chalkTransformer) as PageItemSection,
                    offsetSection = offsetSection?.transform(chalkTransformer) as OffsetItemSection,
                    contentSection = contentSection?.transform(chalkTransformer) as ContentItemSection
            )
            )
}

fun isSourceItemGroup(node: Phase1Node) = firstSectionMatchesName(node, "source")

fun validateSourceItemGroup(groupNode: Group, tracker: MutableLocationTracker) = validateGroup(
    tracker,
    groupNode,
    listOf(
        Validator(
            name = "source",
            optional = false,
            validate = { rawPage, track ->
                validateStringSection(track, rawPage, "source", ::SourceItemSection)
            }
        ),
        Validator(
            name = "page",
            optional = true,
            validate = { rawPage, track ->
                validateStringSection(track, rawPage, "page", ::PageItemSection)
            }
        ),
        Validator(
            name = "offset",
            optional = true,
            validate = { rawPage, track ->
                validateStringSection(track, rawPage, "offset", ::OffsetItemSection)
            }
        ),
        Validator(
            name = "content",
            optional = true,
            validate = { rawPage, track ->
                validateStringSection(track, rawPage, "content", ::ContentItemSection)
            }
        )
        )
) {
    validationSuccess(
        tracker,
        groupNode,
        SourceItemGroup(
            sourceSection = it["source"] as SourceItemSection,
            pageSection = it["page"] as PageItemSection?,
            offsetSection = it["offset"] as OffsetItemSection?,
            contentSection = it["content"] as ContentItemSection?)
    )
}

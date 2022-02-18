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

package mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.metadata.item

import mathlingua.frontend.chalktalk.phase1.ast.Phase1Node
import mathlingua.frontend.chalktalk.phase2.CodeWriter
import mathlingua.frontend.chalktalk.phase2.ast.DEFAULT_SOURCE_ITEM_GROUP
import mathlingua.frontend.chalktalk.phase2.ast.DEFAULT_SOURCE_ITEM_SECTION
import mathlingua.frontend.chalktalk.phase2.ast.clause.firstSectionMatchesName
import mathlingua.frontend.chalktalk.phase2.ast.common.Phase2Node
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.metadata.section.ContentItemSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.metadata.section.OffsetItemSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.metadata.section.PageItemSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.metadata.section.SourceItemSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.metadata.section.validateContentItemSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.metadata.section.validateOffsetItemSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.metadata.section.validatePageItemSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.metadata.section.validateSourceItemSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.topLevelToCode
import mathlingua.frontend.chalktalk.phase2.ast.section.ensureNonNull
import mathlingua.frontend.chalktalk.phase2.ast.section.identifySections
import mathlingua.frontend.chalktalk.phase2.ast.section.ifNonNull
import mathlingua.frontend.chalktalk.phase2.ast.validateGroup
import mathlingua.frontend.support.ParseError

internal data class SourceItemGroup(
    val sourceSection: SourceItemSection,
    val pageSection: PageItemSection?,
    val offsetSection: OffsetItemSection?,
    val contentSection: ContentItemSection?,
    override val row: Int,
    override val column: Int
) : MetaDataItem, ResourceItem {
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

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter) =
        topLevelToCode(
            null,
            writer,
            isArg,
            indent,
            null,
            sourceSection,
            pageSection,
            offsetSection,
            contentSection)

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) =
        chalkTransformer(
            SourceItemGroup(
                sourceSection = sourceSection.transform(chalkTransformer) as SourceItemSection,
                pageSection = pageSection?.transform(chalkTransformer) as PageItemSection,
                offsetSection = offsetSection?.transform(chalkTransformer) as OffsetItemSection,
                contentSection = contentSection?.transform(chalkTransformer) as ContentItemSection,
                row = row,
                column = column))
}

internal fun isSourceItemGroup(node: Phase1Node) = firstSectionMatchesName(node, "source")

internal fun validateSourceItemGroup(node: Phase1Node, errors: MutableList<ParseError>) =
    validateGroup(node.resolve(), errors, "source", DEFAULT_SOURCE_ITEM_GROUP) { group ->
        identifySections(
            group,
            errors,
            DEFAULT_SOURCE_ITEM_GROUP,
            listOf("source", "page?", "offset?", "content?")) { sections ->
            SourceItemGroup(
                sourceSection =
                    ensureNonNull(sections["source"], DEFAULT_SOURCE_ITEM_SECTION) {
                        validateSourceItemSection(it, errors)
                    },
                pageSection = ifNonNull(sections["page"]) { validatePageItemSection(it, errors) },
                offsetSection =
                    ifNonNull(sections["offset"]) { validateOffsetItemSection(it, errors) },
                contentSection =
                    ifNonNull(sections["content"]) { validateContentItemSection(it, errors) },
                row = node.row,
                column = node.column)
        }
    }

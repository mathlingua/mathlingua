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

import mathlingua.chalktalk.phase1.ast.Phase1Node
import mathlingua.chalktalk.phase2.CodeWriter
import mathlingua.chalktalk.phase2.ast.DEFAULT_SOURCE_ITEM_GROUP
import mathlingua.chalktalk.phase2.ast.DEFAULT_SOURCE_ITEM_SECTION
import mathlingua.chalktalk.phase2.ast.clause.firstSectionMatchesName
import mathlingua.chalktalk.phase2.ast.common.Phase2Node
import mathlingua.chalktalk.phase2.ast.group.toplevel.shared.metadata.section.ContentItemSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.shared.metadata.section.OffsetItemSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.shared.metadata.section.PageItemSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.shared.metadata.section.SourceItemSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.shared.metadata.section.neoValidateContentItemSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.shared.metadata.section.neoValidateOffsetItemSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.shared.metadata.section.neoValidatePageItemSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.shared.metadata.section.neoValidateSourceItemSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.topLevelToCode
import mathlingua.chalktalk.phase2.ast.neoValidateGroup
import mathlingua.chalktalk.phase2.ast.section.neoEnsureNonNull
import mathlingua.chalktalk.phase2.ast.section.neoIdentifySections
import mathlingua.chalktalk.phase2.ast.section.neoIfNonNull
import mathlingua.support.MutableLocationTracker
import mathlingua.support.ParseError

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

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter) =
        topLevelToCode(
            writer, isArg, indent, null, sourceSection, pageSection, offsetSection, contentSection)

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) =
        chalkTransformer(
            SourceItemGroup(
                sourceSection = sourceSection.transform(chalkTransformer) as SourceItemSection,
                pageSection = pageSection?.transform(chalkTransformer) as PageItemSection,
                offsetSection = offsetSection?.transform(chalkTransformer) as OffsetItemSection,
                contentSection = contentSection?.transform(chalkTransformer) as ContentItemSection))
}

fun isSourceItemGroup(node: Phase1Node) = firstSectionMatchesName(node, "source")

fun neoValidateSourceItemGroup(
    node: Phase1Node, errors: MutableList<ParseError>, tracker: MutableLocationTracker
) =
    neoValidateGroup(node, errors, "source", DEFAULT_SOURCE_ITEM_GROUP) { group ->
        neoIdentifySections(
            group,
            errors,
            DEFAULT_SOURCE_ITEM_GROUP,
            listOf("source", "page?", "offset?", "content?")) { sections ->
            SourceItemGroup(
                sourceSection =
                    neoEnsureNonNull(sections["source"], DEFAULT_SOURCE_ITEM_SECTION) {
                        neoValidateSourceItemSection(it, errors, tracker)
                    },
                pageSection =
                    neoIfNonNull(sections["page"]) {
                        neoValidatePageItemSection(it, errors, tracker)
                    },
                offsetSection =
                    neoIfNonNull(sections["offset"]) {
                        neoValidateOffsetItemSection(it, errors, tracker)
                    },
                contentSection =
                    neoIfNonNull(sections["content"]) {
                        neoValidateContentItemSection(it, errors, tracker)
                    })
        }
    }

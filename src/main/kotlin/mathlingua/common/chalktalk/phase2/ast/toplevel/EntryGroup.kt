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

import mathlingua.common.support.MutableLocationTracker
import mathlingua.common.chalktalk.phase1.ast.Phase1Node
import mathlingua.common.chalktalk.phase2.CodeWriter
import mathlingua.common.chalktalk.phase2.ast.Phase2Node
import mathlingua.common.chalktalk.phase2.ast.clause.firstSectionMatchesName
import mathlingua.common.chalktalk.phase2.ast.metadata.section.MetaDataSection
import mathlingua.common.chalktalk.phase2.ast.section.ContentSection
import mathlingua.common.chalktalk.phase2.ast.section.EntrySection
import mathlingua.common.chalktalk.phase2.ast.section.TypeSection
import mathlingua.common.chalktalk.phase2.ast.section.validateContentSection
import mathlingua.common.chalktalk.phase2.ast.section.validateEntrySection
import mathlingua.common.chalktalk.phase2.ast.section.validateTypeSection

data class EntryGroup(
    val entrySection: EntrySection,
    val typeSection: TypeSection,
    val contentSection: ContentSection,
    override val metaDataSection: MetaDataSection?
) : TopLevelGroup(metaDataSection) {
    override fun forEach(fn: (node: Phase2Node) -> Unit) {
        fn(entrySection)
        fn(typeSection)
        fn(contentSection)
        if (metaDataSection != null) {
            fn(metaDataSection)
        }
    }

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter) =
        topLevelToCode(writer, isArg, indent, null,
            entrySection, typeSection, contentSection, metaDataSection)

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) =
        chalkTransformer(EntryGroup(
            entrySection = entrySection.transform(chalkTransformer) as EntrySection,
            typeSection = typeSection.transform(chalkTransformer) as TypeSection,
            contentSection = contentSection.transform(chalkTransformer) as ContentSection,
            metaDataSection = metaDataSection?.transform(chalkTransformer) as MetaDataSection?
        ))
}

fun isEntryGroup(node: Phase1Node) = firstSectionMatchesName(node, "Entry")

fun validateEntryGroup(node: Phase1Node, tracker: MutableLocationTracker) = validateTripleSectionMetaDataGroup(
    tracker,
    node,
    "Entry",
    ::validateEntrySection,
    "type",
    ::validateTypeSection,
    "content",
    ::validateContentSection,
    ::EntryGroup
)

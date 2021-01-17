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

package mathlingua.chalktalk.phase2.ast.group.toplevel.entry

import mathlingua.chalktalk.phase1.ast.Phase1Node
import mathlingua.chalktalk.phase2.CodeWriter
import mathlingua.chalktalk.phase2.ast.DEFAULT_CONTENT_SECTION
import mathlingua.chalktalk.phase2.ast.DEFAULT_ENTRY_GROUP
import mathlingua.chalktalk.phase2.ast.DEFAULT_ENTRY_SECTION
import mathlingua.chalktalk.phase2.ast.DEFAULT_TYPE_SECTION
import mathlingua.chalktalk.phase2.ast.clause.firstSectionMatchesName
import mathlingua.chalktalk.phase2.ast.common.Phase2Node
import mathlingua.chalktalk.phase2.ast.group.toplevel.TopLevelGroup
import mathlingua.chalktalk.phase2.ast.group.toplevel.shared.metadata.section.MetaDataSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.shared.metadata.section.validateMetaDataSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.topLevelToCode
import mathlingua.chalktalk.phase2.ast.section.ensureNonNull
import mathlingua.chalktalk.phase2.ast.section.identifySections
import mathlingua.chalktalk.phase2.ast.section.ifNonNull
import mathlingua.chalktalk.phase2.ast.track
import mathlingua.chalktalk.phase2.ast.validateGroup
import mathlingua.support.MutableLocationTracker
import mathlingua.support.ParseError

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
        topLevelToCode(
            writer, isArg, indent, null, entrySection, typeSection, contentSection, metaDataSection)

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) =
        chalkTransformer(
            EntryGroup(
                entrySection = entrySection.transform(chalkTransformer) as EntrySection,
                typeSection = typeSection.transform(chalkTransformer) as TypeSection,
                contentSection = contentSection.transform(chalkTransformer) as ContentSection,
                metaDataSection = metaDataSection?.transform(chalkTransformer) as MetaDataSection?))
}

fun isEntryGroup(node: Phase1Node) = firstSectionMatchesName(node, "Entry")

fun validateEntryGroup(
    node: Phase1Node, errors: MutableList<ParseError>, tracker: MutableLocationTracker
) =
    track(node, tracker) {
        validateGroup(node.resolve(), errors, "Entry", DEFAULT_ENTRY_GROUP) { group ->
            identifySections(
                group,
                errors,
                DEFAULT_ENTRY_GROUP,
                listOf("Entry", "type", "content", "Metadata?")) { sections ->
                EntryGroup(
                    entrySection =
                        ensureNonNull(sections["Entry"], DEFAULT_ENTRY_SECTION) {
                            validateEntrySection(it, errors, tracker)
                        },
                    typeSection =
                        ensureNonNull(sections["type"], DEFAULT_TYPE_SECTION) {
                            validateTypeSection(it, errors, tracker)
                        },
                    contentSection =
                        ensureNonNull(sections["content"], DEFAULT_CONTENT_SECTION) {
                            validateContentSection(it, errors, tracker)
                        },
                    metaDataSection =
                        ifNonNull(sections["Metadata"]) {
                            validateMetaDataSection(it, errors, tracker)
                        })
            }
        }
    }

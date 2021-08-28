/*
 * Copyright 2021
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

package mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.note

import mathlingua.frontend.chalktalk.phase1.ast.Phase1Node
import mathlingua.frontend.chalktalk.phase2.CodeWriter
import mathlingua.frontend.chalktalk.phase2.ast.DEFAULT_CONTENT_SECTION
import mathlingua.frontend.chalktalk.phase2.ast.DEFAULT_NOTE_GROUP
import mathlingua.frontend.chalktalk.phase2.ast.DEFAULT_NOTE_SECTION
import mathlingua.frontend.chalktalk.phase2.ast.clause.IdStatement
import mathlingua.frontend.chalktalk.phase2.ast.clause.firstSectionMatchesName
import mathlingua.frontend.chalktalk.phase2.ast.common.Phase2Node
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.TopLevelGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.metadata.section.MetaDataSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.metadata.section.validateMetaDataSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.topLevelToCode
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.topic.ContentSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.topic.validateContentSection
import mathlingua.frontend.chalktalk.phase2.ast.section.ensureNonNull
import mathlingua.frontend.chalktalk.phase2.ast.section.identifySections
import mathlingua.frontend.chalktalk.phase2.ast.section.ifNonNull
import mathlingua.frontend.chalktalk.phase2.ast.track
import mathlingua.frontend.chalktalk.phase2.ast.validateGroup
import mathlingua.frontend.support.MutableLocationTracker
import mathlingua.frontend.support.ParseError
import mathlingua.frontend.support.validationFailure

data class NoteGroup(
    val id: String?,
    val noteSection: NoteSection,
    val contentSection: ContentSection,
    override val metaDataSection: MetaDataSection?
) : TopLevelGroup(metaDataSection) {
    override fun forEach(fn: (node: Phase2Node) -> Unit) {
        fn(noteSection)
        fn(contentSection)
        if (metaDataSection != null) {
            fn(metaDataSection)
        }
    }

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter) =
        topLevelToCode(
            writer,
            isArg,
            indent,
            if (id == null) {
                null
            } else {
                IdStatement(text = id, texTalkRoot = validationFailure(emptyList()))
            },
            noteSection,
            contentSection,
            metaDataSection)

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) =
        chalkTransformer(
            NoteGroup(
                id = id,
                noteSection = noteSection.transform(chalkTransformer) as NoteSection,
                contentSection = contentSection.transform(chalkTransformer) as ContentSection,
                metaDataSection = metaDataSection?.transform(chalkTransformer) as MetaDataSection?))
}

fun isNoteGroup(node: Phase1Node) = firstSectionMatchesName(node, "Note")

fun validateNoteGroup(
    node: Phase1Node, errors: MutableList<ParseError>, tracker: MutableLocationTracker
) =
    track(node, tracker) {
        validateGroup(node.resolve(), errors, "Note", DEFAULT_NOTE_GROUP) { group ->
            identifySections(
                group, errors, DEFAULT_NOTE_GROUP, listOf("Note", "content", "Metadata?")) {
            sections ->
                NoteGroup(
                    id = group.id?.text?.removeSurrounding("[", "]"),
                    noteSection =
                        ensureNonNull(sections["Note"], DEFAULT_NOTE_SECTION) {
                            validateNoteSection(it, errors, tracker)
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

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

import mathlingua.common.MutableLocationTracker
import mathlingua.common.chalktalk.phase1.ast.Group
import mathlingua.common.chalktalk.phase1.ast.Phase1Node
import mathlingua.common.chalktalk.phase2.CodeWriter
import mathlingua.common.chalktalk.phase2.ast.Phase2Node
import mathlingua.common.chalktalk.phase2.ast.clause.firstSectionMatchesName
import mathlingua.common.chalktalk.phase2.ast.metadata.section.MetaDataSection
import mathlingua.common.chalktalk.phase2.ast.section.*

data class NoteGroup(
    val noteSection: NoteSection,
    override val metaDataSection: MetaDataSection?
) : TopLevelGroup(metaDataSection) {

    override fun forEach(fn: (node: Phase2Node) -> Unit) {
        fn(noteSection)
        if (metaDataSection != null) {
            fn(metaDataSection)
        }
    }

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter) =
        topLevelToCode(writer, isArg, indent, null, noteSection, metaDataSection)

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) = chalkTransformer(NoteGroup(
        noteSection = noteSection.transform(chalkTransformer) as NoteSection,
        metaDataSection = metaDataSection?.transform(chalkTransformer) as MetaDataSection?
    ))
}

fun isNoteGroup(node: Phase1Node) = firstSectionMatchesName(node, "Note")

fun validateNoteGroup(groupNode: Group, tracker: MutableLocationTracker) = validateProtoResultLikeGroup(
    tracker,
    groupNode,
    "Note",
    ::validateNoteSection,
    ::NoteGroup
)

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

package mathlingua.chalktalk.phase2.ast.group.toplevel.defineslike

import mathlingua.chalktalk.phase1.ast.ChalkTalkTokenType
import mathlingua.chalktalk.phase1.ast.Phase1Node
import mathlingua.chalktalk.phase1.ast.Phase1Token
import mathlingua.chalktalk.phase1.ast.Section
import mathlingua.chalktalk.phase1.ast.getColumn
import mathlingua.chalktalk.phase1.ast.getRow
import mathlingua.chalktalk.phase2.CodeWriter
import mathlingua.chalktalk.phase2.ast.DEFAULT_WRITTEN_SECTION
import mathlingua.chalktalk.phase2.ast.common.Phase2Node
import mathlingua.chalktalk.phase2.ast.group.toplevel.validateTextListSection
import mathlingua.chalktalk.phase2.ast.neoTrack
import mathlingua.chalktalk.phase2.ast.neoValidateSection
import mathlingua.support.MutableLocationTracker
import mathlingua.support.ParseError
import mathlingua.support.Validation

data class WrittenSection(val forms: List<String>) : Phase2Node {
    override fun forEach(fn: (node: Phase2Node) -> Unit) {}

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter): CodeWriter {
        writer.writeIndent(isArg, indent)
        writer.writeHeader("written")
        writer.writeNewline()
        for (imp in forms) {
            writer.writeIndent(true, indent + 2)
            writer.writeDirect(imp)
        }
        return writer
    }

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) =
        chalkTransformer(this)
}

fun isWrittenSection(sec: Section) = sec.name.text == "written"

fun validateWrittenSection(
    rawNode: Phase1Node, tracker: MutableLocationTracker
): Validation<WrittenSection> =
    validateTextListSection(rawNode, tracker, "written", ::WrittenSection)

fun neoValidateWrittenSection(
    node: Phase1Node, errors: MutableList<ParseError>, tracker: MutableLocationTracker
) =
    neoTrack(node, tracker) {
        neoValidateSection(node.resolve(), errors, "written", DEFAULT_WRITTEN_SECTION) { section ->
            if (section.args.isEmpty() ||
                !section.args.all {
                    it.chalkTalkTarget is Phase1Token &&
                        it.chalkTalkTarget.type == ChalkTalkTokenType.String
                }) {
                errors.add(
                    ParseError(
                        message = "Expected a list of strings",
                        row = getRow(section),
                        column = getColumn(section)))
                DEFAULT_WRITTEN_SECTION
            } else {
                WrittenSection(
                    forms = section.args.map { (it.chalkTalkTarget as Phase1Token).text })
            }
        }
    }

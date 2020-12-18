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

import mathlingua.chalktalk.phase1.ast.ChalkTalkTokenType
import mathlingua.chalktalk.phase1.ast.Phase1Node
import mathlingua.chalktalk.phase1.ast.Phase1Token
import mathlingua.chalktalk.phase1.ast.getColumn
import mathlingua.chalktalk.phase1.ast.getRow
import mathlingua.chalktalk.phase2.CodeWriter
import mathlingua.chalktalk.phase2.ast.DEFAULT_TYPE_SECTION
import mathlingua.chalktalk.phase2.ast.common.Phase2Node
import mathlingua.chalktalk.phase2.ast.neoTrack
import mathlingua.chalktalk.phase2.ast.neoValidateSection
import mathlingua.chalktalk.phase2.ast.section.validateTextSection
import mathlingua.support.MutableLocationTracker
import mathlingua.support.ParseError
import mathlingua.support.Validation
import mathlingua.support.ValidationFailure
import mathlingua.support.ValidationSuccess
import mathlingua.support.validationFailure
import mathlingua.support.validationSuccess

data class TypeSection(val text: String) : Phase2Node {
    override fun forEach(fn: (node: Phase2Node) -> Unit) {}

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter): CodeWriter {
        writer.writeIndent(isArg, indent)
        writer.writeHeader("type")
        writer.writeIndent(false, 1)
        writer.writeDirect(text)
        return writer
    }

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) =
        chalkTransformer(this)
}

fun validateTypeSection(
    rawNode: Phase1Node, tracker: MutableLocationTracker
): Validation<TypeSection> =
    when (val validation = validateTextSection(rawNode, "type", tracker)
    ) {
        is ValidationFailure -> validationFailure(validation.errors)
        is ValidationSuccess ->
            validationSuccess(tracker, rawNode, TypeSection(validation.value.text))
    }

fun neoValidateTypeSection(
    node: Phase1Node, errors: MutableList<ParseError>, tracker: MutableLocationTracker
) =
    neoTrack(node, tracker) {
        neoValidateSection(node.resolve(), errors, "type", DEFAULT_TYPE_SECTION) { section ->
            if (section.args.isEmpty() ||
                section.args[0].chalkTalkTarget !is Phase1Token ||
                (section.args[0].chalkTalkTarget as Phase1Token).type !=
                    ChalkTalkTokenType.String) {
                errors.add(
                    ParseError(
                        message = "Expected a string",
                        row = getRow(section),
                        column = getColumn(section)))
                DEFAULT_TYPE_SECTION
            } else {
                TypeSection(text = (section.args[0].chalkTalkTarget as Phase1Token).text)
            }
        }
    }

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

package mathlingua.chalktalk.phase2.ast.group.toplevel.shared.metadata.section

import mathlingua.chalktalk.phase1.ast.ChalkTalkTokenType
import mathlingua.chalktalk.phase1.ast.Phase1Node
import mathlingua.chalktalk.phase1.ast.Phase1Token
import mathlingua.chalktalk.phase1.ast.getColumn
import mathlingua.chalktalk.phase1.ast.getRow
import mathlingua.chalktalk.phase2.CodeWriter
import mathlingua.chalktalk.phase2.ast.DEFAULT_SOURCE_ITEM_SECTION
import mathlingua.chalktalk.phase2.ast.common.Phase2Node
import mathlingua.chalktalk.phase2.ast.group.toplevel.shared.metadata.indentedStringSection
import mathlingua.chalktalk.phase2.ast.track
import mathlingua.chalktalk.phase2.ast.validateSection
import mathlingua.chalktalk.phase2.ast.validateSingleArg
import mathlingua.support.MutableLocationTracker
import mathlingua.support.ParseError

data class SourceItemSection(val sourceReference: String) : Phase2Node {
    override fun forEach(fn: (node: Phase2Node) -> Unit) {}

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter) =
        indentedStringSection(writer, isArg, indent, "source", sourceReference)

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node): Phase2Node {
        return chalkTransformer(this)
    }
}

fun validateSourceItemSection(
    node: Phase1Node, errors: MutableList<ParseError>, tracker: MutableLocationTracker
) =
    track(node, tracker) {
        validateSection(node, errors, "source", DEFAULT_SOURCE_ITEM_SECTION) { section ->
            validateSingleArg(section, errors, DEFAULT_SOURCE_ITEM_SECTION, "string") { arg ->
                if (arg !is Phase1Token || arg.type != ChalkTalkTokenType.String) {
                    errors.add(
                        ParseError(
                            message = "Expected a string",
                            row = getRow(node),
                            column = getColumn(node)))
                    DEFAULT_SOURCE_ITEM_SECTION
                } else {
                    SourceItemSection(sourceReference = arg.text)
                }
            }
        }
    }

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

package mathlingua.common.chalktalk.phase2.ast.metadata

import mathlingua.common.*
import mathlingua.common.chalktalk.phase1.ast.*
import mathlingua.common.chalktalk.phase1.ast.getColumn
import mathlingua.common.chalktalk.phase1.ast.getRow
import mathlingua.common.chalktalk.phase2.CodeWriter
import mathlingua.common.chalktalk.phase2.ast.Phase2Node

internal fun indentedStringSection(writer: CodeWriter, isArg: Boolean, indent: Int, sectionName: String, value: String): CodeWriter {
    writer.writeIndent(isArg, indent)
    writer.writeHeader(sectionName)
    writer.writeSpace()
    writer.writeText(value.removeSurrounding("\"", "\""))
    return writer
}

internal fun <T : Phase2Node> validateStringSection(
    tracker: MutableLocationTracker,
    rawNode: Phase1Node,
    expectedName: String,
    fn: (text: String) -> T
): Validation<T> {
    val node = rawNode.resolve()
    val errors = ArrayList<ParseError>()
    if (node !is Section) {
        errors.add(
                ParseError(
                        "Expected a Section",
                        getRow(node), getColumn(node)
                )
        )
    }

    val (name, args) = node as Section
    if (name.text != expectedName) {
        errors.add(
                ParseError(
                        "Expected a Section with name " +
                                expectedName + " but found " + name.text,
                        getRow(node), getColumn(node)
                )
        )
    }

    if (args.size != 1 ||
            args[0].chalkTalkTarget !is Phase1Token ||
            (args[0].chalkTalkTarget as Phase1Token).type != ChalkTalkTokenType.String) {
        errors.add(
                ParseError(
                        "Section '" + name.text + "' requires a single string argument.",
                        getRow(node), getColumn(node)
                )
        )
    }

    val token = args[0].chalkTalkTarget as Phase1Token
    return if (errors.isNotEmpty()) {
        validationFailure(errors)
    } else validationSuccess(tracker, rawNode, fn(token.text))
}

internal fun isSingleSectionGroup(node: Phase1Node): Boolean {
    if (node !is Group) {
        return false
    }
    return node.sections.size == 1
}

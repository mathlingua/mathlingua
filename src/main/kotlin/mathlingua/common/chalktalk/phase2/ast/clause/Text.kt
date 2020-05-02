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

package mathlingua.common.chalktalk.phase2.ast.clause

import mathlingua.common.ParseError
import mathlingua.common.Validation
import mathlingua.common.ValidationFailure
import mathlingua.common.ValidationSuccess
import mathlingua.common.chalktalk.phase1.ast.*
import mathlingua.common.chalktalk.phase2.CodeWriter
import mathlingua.common.chalktalk.phase2.ast.Phase2Node

data class Text(
    val text: String,
    override var row: Int,
    override var column: Int
) : Clause {
    override fun forEach(fn: (node: Phase2Node) -> Unit) {}

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter): CodeWriter {
        writer.writeIndent(isArg, indent)
        writer.writeText(text.removeSurrounding("\"", "\""))
        return writer
    }

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) = chalkTransformer(this)
}

fun isText(node: Phase1Node) = node is Phase1Token && node.type === ChalkTalkTokenType.String

fun validateText(rawNode: Phase1Node): Validation<Text> {
    val node = rawNode.resolve()

    val errors = ArrayList<ParseError>()
    if (node !is Phase1Token) {
        errors.add(
                ParseError(
                        "Cannot convert a to a ChalkTalkToken",
                        getRow(node), getColumn(node)
                )
        )
    }

    val (text, type, row, column) = node as Phase1Token
    if (type !== ChalkTalkTokenType.String) {
        errors.add(
                ParseError(
                        "Cannot convert a " + node.toCode() + " to Text",
                        row, column
                )
        )
        return ValidationFailure(errors)
    }

    return ValidationSuccess(Text(text, getRow(node), getColumn(node)))
}

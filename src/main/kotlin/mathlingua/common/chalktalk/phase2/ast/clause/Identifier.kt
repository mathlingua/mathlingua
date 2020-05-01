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

fun isIdentifier(node: Phase1Node) = node is Phase1Token && node.type === ChalkTalkTokenType.Name

fun validateIdentifier(rawNode: Phase1Node): Validation<Identifier> {
    val node = rawNode.resolve()

    val errors = ArrayList<ParseError>()
    if (node !is Phase1Token) {
        errors.add(
                ParseError(
                        "Cannot convert to a ChalkTalkToken",
                        getRow(node), getColumn(node)
                )
        )
        return ValidationFailure(errors)
    }

    val (text, type, row, column) = node
    if (type !== ChalkTalkTokenType.Name) {
        errors.add(
                ParseError(
                        "A token of type $type is not an identifier",
                        row, column
                )
        )
        return ValidationFailure(errors)
    }

    var realText = text
    var isVarArgs = false
    if (text.endsWith("...")) {
        realText = text.substring(0, text.length - 3)
        isVarArgs = true
    }

    return ValidationSuccess(
            Identifier(
                    name = realText,
                    isVarArgs = isVarArgs,
                    row = getRow(node),
                    column = getColumn(node)
            )
    )
}

data class Identifier(
    val name: String,
    val isVarArgs: Boolean,
    override var row: Int,
    override var column: Int
) : Target {
    override fun forEach(fn: (node: Phase2Node) -> Unit) {}

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter): CodeWriter {
        writer.writeIndent(isArg, indent)
        writer.writeIdentifier(name, isVarArgs)
        return writer
    }

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) = chalkTransformer(this)
}

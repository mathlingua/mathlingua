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

import mathlingua.common.Validation
import mathlingua.common.ValidationFailure
import mathlingua.common.ValidationSuccess
import mathlingua.common.chalktalk.phase1.ast.Phase1Node
import mathlingua.common.chalktalk.phase2.CodeWriter
import mathlingua.common.chalktalk.phase2.ast.Phase2Node
import mathlingua.common.textalk.ExpressionTexTalkNode

data class IdStatement(
    val text: String,
    val texTalkRoot: Validation<ExpressionTexTalkNode>,
    override var row: Int,
    override var column: Int
) : Clause {
    override fun forEach(fn: (node: Phase2Node) -> Unit) {}

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter): CodeWriter {
        writer.writeIndent(isArg, indent)
        writer.writeStatement(text, texTalkRoot)
        return writer
    }

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) = chalkTransformer(this)

    fun toStatement() = Statement(
            text = text,
            texTalkRoot = texTalkRoot,
            row = row,
            column = column
    )
}

fun validateIdStatement(rawNode: Phase1Node): Validation<IdStatement> =
        when (val validation = validateStatement(rawNode)) {
            is ValidationSuccess -> ValidationSuccess(IdStatement(
                    text = validation.value.text,
                    texTalkRoot = validation.value.texTalkRoot,
                    row = validation.value.row,
                    column = validation.value.column
            ))
            is ValidationFailure -> ValidationFailure(validation.errors)
        }

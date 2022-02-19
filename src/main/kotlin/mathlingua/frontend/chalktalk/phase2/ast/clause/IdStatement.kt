/*
 * Copyright 2020 The MathLingua Authors
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

package mathlingua.frontend.chalktalk.phase2.ast.clause

import mathlingua.frontend.chalktalk.phase1.ast.Phase1Node
import mathlingua.frontend.chalktalk.phase2.CodeWriter
import mathlingua.frontend.chalktalk.phase2.ast.common.Phase2Node
import mathlingua.frontend.support.ParseError
import mathlingua.frontend.support.Validation
import mathlingua.frontend.textalk.ExpressionTexTalkNode

internal data class IdStatement(
    val text: String,
    val texTalkRoot: Validation<ExpressionTexTalkNode>,
    override val row: Int,
    override val column: Int
) : Clause {
    override fun forEach(fn: (node: Phase2Node) -> Unit) {}

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter): CodeWriter {
        writer.writeIndent(isArg, indent)
        writer.writeId(this)
        return writer
    }

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) =
        chalkTransformer(this)

    fun toStatement() =
        Statement(text = text, texTalkRoot = texTalkRoot, row, column, isInline = true)
}

internal fun validateIdStatement(node: Phase1Node, errors: MutableList<ParseError>): IdStatement {
    val statement = validateStatement(node.resolve(), errors, isInline = true)
    return IdStatement(
        text = statement.text, texTalkRoot = statement.texTalkRoot, node.row, node.column)
}

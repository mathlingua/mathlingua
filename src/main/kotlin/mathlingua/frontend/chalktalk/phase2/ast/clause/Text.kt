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

import mathlingua.frontend.chalktalk.phase1.ast.ChalkTalkTokenType
import mathlingua.frontend.chalktalk.phase1.ast.Inlineable
import mathlingua.frontend.chalktalk.phase1.ast.Phase1Node
import mathlingua.frontend.chalktalk.phase1.ast.Phase1Token
import mathlingua.frontend.chalktalk.phase2.CodeWriter
import mathlingua.frontend.chalktalk.phase2.ast.DEFAULT_TEXT
import mathlingua.frontend.chalktalk.phase2.ast.common.Phase2Node
import mathlingua.frontend.chalktalk.phase2.ast.validateByInlineableTransform
import mathlingua.frontend.support.ParseError

internal data class Text(
    val text: String,
    override val row: Int,
    override val column: Int,
    override val isInline: Boolean
) : Clause, Inlineable {
    override fun forEach(fn: (node: Phase2Node) -> Unit) {}

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter): CodeWriter {
        writer.writeIndent(isArg, indent)
        writer.writeText(text)
        return writer
    }

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) =
        chalkTransformer(this)
}

internal fun isText(node: Phase1Node) =
    node is Phase1Token && node.type === ChalkTalkTokenType.String

internal fun validateText(
    node: Phase1Node, errors: MutableList<ParseError>, isInline: Boolean
): Text =
    validateByInlineableTransform(
        node = node.resolve(),
        errors = errors,
        default = DEFAULT_TEXT,
        message = "Expected text",
        isInline = isInline,
        transform = {
            if (it is Phase1Token && it.type == ChalkTalkTokenType.String) {
                it
            } else {
                null
            }
        }) { n, row, column, inline -> Text(text = n.text, row, column, inline) }

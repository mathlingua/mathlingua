/*
 * Copyright 2021 The MathLingua Authors
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

package mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.metadata.item

import mathlingua.frontend.chalktalk.phase1.ast.ChalkTalkTokenType
import mathlingua.frontend.chalktalk.phase1.ast.Phase1Node
import mathlingua.frontend.chalktalk.phase1.ast.Phase1Token
import mathlingua.frontend.chalktalk.phase2.CodeWriter
import mathlingua.frontend.chalktalk.phase2.ast.common.Phase2Node
import mathlingua.frontend.support.ParseError

internal data class StringItem(val text: String, override val row: Int, override val column: Int) :
    ResourceItem {
    override fun forEach(fn: (node: Phase2Node) -> Unit) {}

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter): CodeWriter {
        writer.writeSpace()
        writer.writeText(text)
        return writer
    }

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) =
        chalkTransformer(this)
}

internal fun isStringItem(node: Phase1Node) =
    node is Phase1Token && node.type == ChalkTalkTokenType.String

internal fun validateStringItem(
    node: Phase1Node,
    @Suppress("UNUSED_PARAMETER")
    errors: MutableList<ParseError>,
) = StringItem(text = (node.resolve() as Phase1Token).text, node.row, node.column)

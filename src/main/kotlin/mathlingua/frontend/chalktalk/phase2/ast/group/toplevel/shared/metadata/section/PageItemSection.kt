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

package mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.metadata.section

import mathlingua.frontend.chalktalk.phase1.ast.ChalkTalkTokenType
import mathlingua.frontend.chalktalk.phase1.ast.Phase1Node
import mathlingua.frontend.chalktalk.phase1.ast.Phase1Token
import mathlingua.frontend.chalktalk.phase2.CodeWriter
import mathlingua.frontend.chalktalk.phase2.ast.DEFAULT_PAGE_ITEM_SECTION
import mathlingua.frontend.chalktalk.phase2.ast.common.Phase2Node
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.metadata.indentedStringSection
import mathlingua.frontend.chalktalk.phase2.ast.validateSection
import mathlingua.frontend.chalktalk.phase2.ast.validateSingleArg
import mathlingua.frontend.support.ParseError

internal data class PageItemSection(
    val page: String, override val row: Int, override val column: Int
) : Phase2Node {
    override fun forEach(fn: (node: Phase2Node) -> Unit) {}

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter) =
        indentedStringSection(writer, isArg, indent, "page", page)

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node): Phase2Node {
        return chalkTransformer(this)
    }
}

internal fun validatePageItemSection(node: Phase1Node, errors: MutableList<ParseError>) =
    validateSection(node, errors, "page", DEFAULT_PAGE_ITEM_SECTION) { section ->
        validateSingleArg(section, errors, DEFAULT_PAGE_ITEM_SECTION, "string") { arg ->
            val resolved = arg.resolve()
            if (resolved !is Phase1Token || resolved.type != ChalkTalkTokenType.String) {
                errors.add(
                    ParseError(message = "Expected a string", row = node.row, column = node.column))
                DEFAULT_PAGE_ITEM_SECTION
            } else {
                PageItemSection(page = resolved.text, node.row, node.column)
            }
        }
    }

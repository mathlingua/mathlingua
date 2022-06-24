/*
 * Copyright 2022 Dominic Kramer
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

package mathlingua.lib.frontend.textalk

import mathlingua.lib.frontend.Diagnostic
import mathlingua.lib.frontend.DiagnosticOrigin
import mathlingua.lib.frontend.DiagnosticType
import mathlingua.lib.frontend.ast.TexTalkNode
import mathlingua.lib.frontend.ast.TexTalkTokenType

internal inline fun <reified T> List<TexTalkNode>.filterAndError(
    diagnostics: MutableList<Diagnostic>
): MutableList<T> {
    val result = mutableListOf<T>()
    for (item in this) {
        if (item is T) {
            result.add(item)
        } else {
            diagnostics.add(
                error(
                    message = "Expected a ${T::class.java.simpleName}",
                    row = item.metadata.row,
                    column = item.metadata.column))
        }
    }
    return result
}

internal fun error(message: String, row: Int, column: Int) =
    Diagnostic(
        type = DiagnosticType.Error,
        origin = DiagnosticOrigin.ChalkTalkParser,
        message = message,
        row = row,
        column = column)

internal fun MutableList<TreeNode>.has(type: TexTalkTokenType) =
    this.isNotEmpty() && this.first().isAtomOfType(type)

internal fun TreeNode.isAtomOfType(type: TexTalkTokenType) =
    this is AtomTreeNode && this.token.type == type

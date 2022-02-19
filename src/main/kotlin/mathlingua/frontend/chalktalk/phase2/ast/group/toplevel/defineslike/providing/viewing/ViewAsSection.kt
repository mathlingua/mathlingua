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

package mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.providing.viewing

import mathlingua.frontend.chalktalk.phase1.ast.Phase1Node
import mathlingua.frontend.chalktalk.phase2.CodeWriter
import mathlingua.frontend.chalktalk.phase2.ast.DEFAULT_VIEW_AS_SECTION
import mathlingua.frontend.chalktalk.phase2.ast.clause.Statement
import mathlingua.frontend.chalktalk.phase2.ast.clause.validateStatement
import mathlingua.frontend.chalktalk.phase2.ast.common.Phase2Node
import mathlingua.frontend.chalktalk.phase2.ast.validateSection
import mathlingua.frontend.chalktalk.phase2.ast.validateSingleArg
import mathlingua.frontend.support.ParseError

internal data class ViewAsSection(
    val statement: Statement, override val row: Int, override val column: Int
) : Phase2Node {
    override fun forEach(fn: (node: Phase2Node) -> Unit) {
        fn(statement)
    }

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter): CodeWriter {
        writer.writeIndent(isArg, indent)
        writer.writeHeader("as")
        writer.append(statement, false, 1)
        return writer
    }

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) =
        chalkTransformer(
            ViewAsSection(statement = chalkTransformer(statement) as Statement, row, column))
}

internal fun validateViewingAsSection(node: Phase1Node, errors: MutableList<ParseError>) =
    validateSection(node.resolve(), errors, "as", DEFAULT_VIEW_AS_SECTION) { sec ->
        validateSingleArg(sec, errors, DEFAULT_VIEW_AS_SECTION, "statement") { arg ->
            ViewAsSection(
                statement = validateStatement(arg, errors, arg.isInline), node.row, node.column)
        }
    }

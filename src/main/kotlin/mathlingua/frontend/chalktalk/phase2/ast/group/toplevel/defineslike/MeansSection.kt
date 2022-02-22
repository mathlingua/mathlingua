/*
 * Copyright 2022 The MathLingua Authors
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

package mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike

import mathlingua.frontend.chalktalk.phase1.ast.Phase1Node
import mathlingua.frontend.chalktalk.phase2.CodeWriter
import mathlingua.frontend.chalktalk.phase2.ast.DEFAULT_EXTENDING_SECTION
import mathlingua.frontend.chalktalk.phase2.ast.clause.Statement
import mathlingua.frontend.chalktalk.phase2.ast.clause.validateStatement
import mathlingua.frontend.chalktalk.phase2.ast.common.Phase2Node
import mathlingua.frontend.chalktalk.phase2.ast.validateSection
import mathlingua.frontend.support.ParseError

internal data class MeansSection(
    val statements: List<Statement>, override val row: Int, override val column: Int
) : Phase2Node {
    override fun forEach(fn: (node: Phase2Node) -> Unit) = statements.forEach(fn)

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter): CodeWriter {
        writer.writeIndent(isArg, indent)
        writer.writeHeader("means")
        for (i in statements.indices) {
            val stmt = statements[i]
            if (stmt.isInline) {
                if (i != 0) {
                    writer.writeComma()
                }
                writer.writeSpace()
                writer.writeStatement(stmt.text, stmt.texTalkRoot)
            } else {
                writer.writeNewline()
                writer.writeIndent(true, 2)
                writer.writeStatement(stmt.text, stmt.texTalkRoot)
            }
        }
        return writer
    }

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) =
        chalkTransformer(
            MeansSection(
                statements = statements.map { it.transform(chalkTransformer) as Statement },
                row,
                column))
}

internal fun validateExtendingSection(node: Phase1Node, errors: MutableList<ParseError>) =
    validateSection(node.resolve(), errors, DEFAULT_EXTENDING_SECTION) {
        // more robust validation is done in the
        //   SourceCollection.findInvalidMeansSection(DefinesGroup)
        // because it needs to know what vars are defined in the `Defines:` section
        val statements = it.args.map { arg -> validateStatement(arg, errors, arg.isInline) }
        MeansSection(statements = statements, node.row, node.column)
    }

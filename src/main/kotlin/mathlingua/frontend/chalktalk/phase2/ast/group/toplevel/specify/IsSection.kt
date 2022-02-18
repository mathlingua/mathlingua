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

package mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.specify

import mathlingua.frontend.chalktalk.phase1.ast.ChalkTalkTokenType
import mathlingua.frontend.chalktalk.phase1.ast.Phase1Node
import mathlingua.frontend.chalktalk.phase1.ast.Phase1Token
import mathlingua.frontend.chalktalk.phase2.CodeWriter
import mathlingua.frontend.chalktalk.phase2.ast.DEFAULT_IS_SECTION
import mathlingua.frontend.chalktalk.phase2.ast.clause.Statement
import mathlingua.frontend.chalktalk.phase2.ast.clause.validateStatement
import mathlingua.frontend.chalktalk.phase2.ast.common.Phase2Node
import mathlingua.frontend.chalktalk.phase2.ast.validateSection
import mathlingua.frontend.support.ParseError
import mathlingua.frontend.support.ValidationSuccess
import mathlingua.frontend.textalk.Command

internal data class IsSection(
    val statement: Statement, override val row: Int, override val column: Int
) : Phase2Node {
    override fun forEach(fn: (node: Phase2Node) -> Unit) = fn(statement)

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter): CodeWriter {
        writer.writeIndent(isArg, indent)
        writer.writeHeader("is")
        writer.writeIndent(false, 1)
        writer.writeStatement(statement.text, statement.texTalkRoot)
        return writer
    }

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) =
        chalkTransformer(
            IsSection(statement = statement.transform(chalkTransformer) as Statement, row, column))
}

internal fun validateIsSection(node: Phase1Node, errors: MutableList<ParseError>) =
    validateSection(node.resolve(), errors, "is", DEFAULT_IS_SECTION) { section ->
        if (section.args.isEmpty() ||
            section.args[0].chalkTalkTarget !is Phase1Token ||
            (section.args[0].chalkTalkTarget as Phase1Token).type != ChalkTalkTokenType.Statement) {
            errors.add(
                ParseError(
                    message = "Expected a statement", row = section.row, column = section.column))
            DEFAULT_IS_SECTION
        } else {
            val errBefore = errors.size
            val statement = validateStatement(section.args[0], errors)
            if (errBefore != errors.size || statement.texTalkRoot !is ValidationSuccess) {
                DEFAULT_IS_SECTION
            } else if (statement.texTalkRoot.value.children.size != 1 ||
                (statement.texTalkRoot.value.children[0] !is Command)) {
                errors.add(
                    ParseError(
                        message = "Expected a single command of the form \\...",
                        row = section.row,
                        column = section.column))
                DEFAULT_IS_SECTION
            } else {
                IsSection(statement = statement, node.row, node.column)
            }
        }
    }

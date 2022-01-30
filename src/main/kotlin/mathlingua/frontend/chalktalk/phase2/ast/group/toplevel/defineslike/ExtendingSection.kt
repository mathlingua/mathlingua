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

import mathlingua.frontend.chalktalk.phase1.ast.ChalkTalkTokenType
import mathlingua.frontend.chalktalk.phase1.ast.Phase1Node
import mathlingua.frontend.chalktalk.phase1.ast.Phase1Token
import mathlingua.frontend.chalktalk.phase1.ast.getColumn
import mathlingua.frontend.chalktalk.phase1.ast.getRow
import mathlingua.frontend.chalktalk.phase2.CodeWriter
import mathlingua.frontend.chalktalk.phase2.ast.DEFAULT_EXTENDING_SECTION
import mathlingua.frontend.chalktalk.phase2.ast.clause.Statement
import mathlingua.frontend.chalktalk.phase2.ast.clause.validateStatement
import mathlingua.frontend.chalktalk.phase2.ast.common.Phase2Node
import mathlingua.frontend.chalktalk.phase2.ast.track
import mathlingua.frontend.chalktalk.phase2.ast.validateSection
import mathlingua.frontend.support.MutableLocationTracker
import mathlingua.frontend.support.ParseError
import mathlingua.frontend.support.ValidationSuccess
import mathlingua.frontend.textalk.ColonEqualsTexTalkNode
import mathlingua.frontend.textalk.InTexTalkNode
import mathlingua.frontend.textalk.IsTexTalkNode

internal data class ExtendingSection(val isStatement: Statement) : Phase2Node {
    override fun forEach(fn: (node: Phase2Node) -> Unit) = fn(isStatement)

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter): CodeWriter {
        writer.writeIndent(isArg, indent)
        writer.writeHeader("extending")
        writer.writeIndent(false, 1)
        writer.writeStatement(isStatement.text, isStatement.texTalkRoot)
        return writer
    }

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) =
        chalkTransformer(
            ExtendingSection(isStatement = isStatement.transform(chalkTransformer) as Statement))
}

internal fun validateExtendingSection(
    node: Phase1Node, errors: MutableList<ParseError>, tracker: MutableLocationTracker
) =
    track(node, tracker) {
        validateSection(node.resolve(), errors, "extending", DEFAULT_EXTENDING_SECTION) { section ->
            if (section.args.isEmpty() ||
                section.args[0].chalkTalkTarget !is Phase1Token ||
                (section.args[0].chalkTalkTarget as Phase1Token).type !=
                    ChalkTalkTokenType.Statement) {
                errors.add(
                    ParseError(
                        message = "Expected an 'is', 'in', or ':=' statement",
                        row = getRow(section),
                        column = getColumn(section)))
                DEFAULT_EXTENDING_SECTION
            } else {
                val errBefore = errors.size
                val statement = validateStatement(section.args[0], errors, tracker)
                if (errBefore != errors.size || statement.texTalkRoot !is ValidationSuccess) {
                    DEFAULT_EXTENDING_SECTION
                } else if (statement.texTalkRoot.value.children.size != 1 ||
                    (statement.texTalkRoot.value.children[0] !is IsTexTalkNode &&
                        statement.texTalkRoot.value.children[0] !is InTexTalkNode &&
                        statement.texTalkRoot.value.children[0] !is ColonEqualsTexTalkNode)) {
                    errors.add(
                        ParseError(
                            message = "Expected an 'is', 'in', or ':=' statement",
                            row = getRow(section),
                            column = getColumn(section)))
                    DEFAULT_EXTENDING_SECTION
                } else {
                    ExtendingSection(isStatement = statement)
                }
            }
        }
    }

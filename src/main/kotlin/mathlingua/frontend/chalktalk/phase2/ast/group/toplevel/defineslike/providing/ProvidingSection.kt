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

package mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.providing

import mathlingua.frontend.chalktalk.phase1.ast.Phase1Node
import mathlingua.frontend.chalktalk.phase2.CodeWriter
import mathlingua.frontend.chalktalk.phase2.ast.DEFAULT_PROVIDING_SECTION
import mathlingua.frontend.chalktalk.phase2.ast.clause.ClauseListNode
import mathlingua.frontend.chalktalk.phase2.ast.clause.validateClauseListNode
import mathlingua.frontend.chalktalk.phase2.ast.common.Phase2Node
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.providing.equality.EqualityGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.providing.memberSymbols.MemberSymbolsGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.providing.membership.MembershipGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.providing.symbols.SymbolsGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.providing.viewing.ViewGroup
import mathlingua.frontend.chalktalk.phase2.ast.validateSection
import mathlingua.frontend.support.ParseError

internal data class ProvidingSection(
    val clauses: ClauseListNode, override val row: Int, override val column: Int
) : Phase2Node {
    override fun forEach(fn: (node: Phase2Node) -> Unit) = clauses.forEach(fn)

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter): CodeWriter {
        writer.writeIndent(isArg, indent)
        writer.writeHeader("Providing")
        writer.append(clauses, true, indent + 2)
        return writer
    }

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) =
        chalkTransformer(
            ProvidingSection(
                clauses = clauses.transform(chalkTransformer) as ClauseListNode, row, column))
}

internal fun validateProvidingSection(node: Phase1Node, errors: MutableList<ParseError>) =
    validateSection(node.resolve(), errors, "Providing", DEFAULT_PROVIDING_SECTION) {
        val clauses = validateClauseListNode(it, errors)
        val errorCountBefore = errors.size
        for (clause in clauses.clauses) {
            if (clause !is MembershipGroup &&
                clause !is ViewGroup &&
                clause !is EqualityGroup &&
                clause !is SymbolsGroup &&
                clause !is MemberSymbolsGroup) {
                errors.add(
                    ParseError(
                        message =
                            "Expected either a symbols:, memberSymbols:, membership:, view:, or equality: group",
                        row = clause.row,
                        column = clause.column))
            }
        }
        if (errors.size != errorCountBefore) {
            DEFAULT_PROVIDING_SECTION
        } else {
            ProvidingSection(clauses = clauses, node.row, node.column)
        }
    }

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

package mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.defines

import mathlingua.frontend.chalktalk.phase1.ast.Phase1Node
import mathlingua.frontend.chalktalk.phase2.CodeWriter
import mathlingua.frontend.chalktalk.phase2.ast.DEFAULT_SATISFYING_SECTION
import mathlingua.frontend.chalktalk.phase2.ast.clause.ClauseListNode
import mathlingua.frontend.chalktalk.phase2.ast.clause.validateClauseListNode
import mathlingua.frontend.chalktalk.phase2.ast.common.Phase2Node
import mathlingua.frontend.chalktalk.phase2.ast.validateSection
import mathlingua.frontend.support.ParseError

internal data class SatisfyingSection(
    val clauses: ClauseListNode, override val row: Int, override val column: Int
) : Phase2Node {
    override fun forEach(fn: (node: Phase2Node) -> Unit) = clauses.forEach(fn)

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter): CodeWriter {
        writer.writeIndent(isArg, indent)
        writer.writeHeader("satisfying")
        /*
        // A single means entry should be printed on the same line. That is,
        //   means: '...'
        // instead of
        //   means:
        //   . '...'
        //
        if (clauses.clauses.size == 1 &&
            clauses.clauses[0] !is ForAllGroup &&
            clauses.clauses[0] !is ExistsGroup &&
            clauses.clauses[0] !is ExistsUniqueGroup &&
            clauses.clauses[0] !is IfGroup &&
            clauses.clauses[0] !is IffGroup &&
            clauses.clauses[0] !is AndGroup &&
            clauses.clauses[0] !is OrGroup &&
            clauses.clauses[0] !is PiecewiseGroup &&
            clauses.clauses[0] !is NotGroup &&
            clauses.clauses[0] !is GeneratedGroup) {
            writer.append(clauses.clauses[0], false, 1)
        } else {
            if (clauses.clauses.isNotEmpty()) {
                writer.writeNewline()
            }
            writer.append(clauses, true, indent + 2)
        }
         */
        writer.append(clauses, true, indent + 2)
        return writer
    }

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) =
        chalkTransformer(
            SatisfyingSection(
                clauses = clauses.transform(chalkTransformer) as ClauseListNode, row, column))
}

internal fun validateSatisfiesSection(node: Phase1Node, errors: MutableList<ParseError>) =
    validateSection(node.resolve(), errors, "satisfying", DEFAULT_SATISFYING_SECTION) {
        SatisfyingSection(clauses = validateClauseListNode(it, errors), node.row, node.column)
    }
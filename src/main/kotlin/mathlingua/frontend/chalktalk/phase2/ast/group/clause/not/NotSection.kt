/*
 * Copyright 2020 Google LLC
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

package mathlingua.frontend.chalktalk.phase2.ast.group.clause.not

import mathlingua.frontend.chalktalk.phase1.ast.Phase1Node
import mathlingua.frontend.chalktalk.phase2.CodeWriter
import mathlingua.frontend.chalktalk.phase2.ast.DEFAULT_NOT_SECTION
import mathlingua.frontend.chalktalk.phase2.ast.clause.ClauseListNode
import mathlingua.frontend.chalktalk.phase2.ast.clause.Text
import mathlingua.frontend.chalktalk.phase2.ast.clause.validateClauseListNode
import mathlingua.frontend.chalktalk.phase2.ast.common.Phase2Node
import mathlingua.frontend.chalktalk.phase2.ast.track
import mathlingua.frontend.chalktalk.phase2.ast.validateSection
import mathlingua.frontend.support.MutableLocationTracker
import mathlingua.frontend.support.ParseError

data class NotSection(val clauses: ClauseListNode) : Phase2Node {
    override fun forEach(fn: (node: Phase2Node) -> Unit) = clauses.forEach(fn)

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter): CodeWriter {
        writer.writeIndent(isArg, indent)
        writer.writeHeader("not")
        if (clauses.clauses.size == 1 && clauses.clauses[0] is Text) {
            writer.append(clauses.clauses.first(), false, 1)
        } else {
            if (clauses.clauses.isNotEmpty()) {
                writer.writeNewline()
            }
            writer.append(clauses, true, indent + 2)
        }
        return writer
    }

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) =
        chalkTransformer(
            NotSection(clauses = clauses.transform(chalkTransformer) as ClauseListNode))
}

fun validateNotSection(
    node: Phase1Node, errors: MutableList<ParseError>, tracker: MutableLocationTracker
) =
    track(node, tracker) {
        validateSection(node.resolve(), errors, "not", DEFAULT_NOT_SECTION) {
            NotSection(clauses = validateClauseListNode(it, errors, tracker))
        }
    }

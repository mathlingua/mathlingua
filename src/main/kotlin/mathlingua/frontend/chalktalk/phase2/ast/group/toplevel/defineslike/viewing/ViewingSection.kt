/*
 * Copyright 2021
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

package mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.viewing

import mathlingua.frontend.chalktalk.phase1.ast.Phase1Node
import mathlingua.frontend.chalktalk.phase2.CodeWriter
import mathlingua.frontend.chalktalk.phase2.ast.DEFAULT_VIEWING_SECTION
import mathlingua.frontend.chalktalk.phase2.ast.clause.ClauseListNode
import mathlingua.frontend.chalktalk.phase2.ast.clause.validateClauseListNode
import mathlingua.frontend.chalktalk.phase2.ast.common.Phase2Node
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.viewing.equality.EqualityGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.viewing.membership.MembershipGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.viewing.viewingas.ViewingAsGroup
import mathlingua.frontend.chalktalk.phase2.ast.track
import mathlingua.frontend.chalktalk.phase2.ast.validateSection
import mathlingua.frontend.support.Location
import mathlingua.frontend.support.MutableLocationTracker
import mathlingua.frontend.support.ParseError

data class ViewingSection(val clauses: ClauseListNode) : Phase2Node {
    override fun forEach(fn: (node: Phase2Node) -> Unit) = clauses.forEach(fn)

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter): CodeWriter {
        writer.writeIndent(isArg, indent)
        writer.writeHeader("viewing")
        if (clauses.clauses.isNotEmpty()) {
            writer.writeNewline()
        }
        writer.append(clauses, true, indent + 2)
        return writer
    }

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) =
        chalkTransformer(
            ViewingSection(clauses = clauses.transform(chalkTransformer) as ClauseListNode))
}

fun validateViewingSection(
    node: Phase1Node, errors: MutableList<ParseError>, tracker: MutableLocationTracker
) =
    track(node, tracker) {
        validateSection(node.resolve(), errors, "viewing", DEFAULT_VIEWING_SECTION) {
            val clauses = validateClauseListNode(it, errors, tracker)
            val errorCountBefore = errors.size
            for (clause in clauses.clauses) {
                if (clause !is MembershipGroup &&
                    clause !is ViewingAsGroup &&
                    clause !is EqualityGroup) {
                    val location = tracker.getLocationOf(clause) ?: Location(row = -1, column = -1)
                    errors.add(
                        ParseError(
                            message = "Expected either a membership:, as:, or equality: group",
                            row = location.row,
                            column = location.column))
                }
            }
            if (errors.size != errorCountBefore) {
                DEFAULT_VIEWING_SECTION
            } else {
                ViewingSection(clauses = clauses)
            }
        }
    }

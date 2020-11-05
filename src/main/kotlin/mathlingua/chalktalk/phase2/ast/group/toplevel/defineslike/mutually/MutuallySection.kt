/*
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *    http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

package mathlingua.chalktalk.phase2.ast.group.toplevel.defineslike.mutually

import mathlingua.chalktalk.phase1.ast.Phase1Node
import mathlingua.chalktalk.phase2.CodeWriter
import mathlingua.chalktalk.phase2.ast.common.Phase2Node
import mathlingua.chalktalk.phase2.ast.group.toplevel.defineslike.foundation.DefinesStatesOrViews
import mathlingua.chalktalk.phase2.ast.validator.AtLeast
import mathlingua.chalktalk.phase2.ast.validator.validateClauseList
import mathlingua.support.MutableLocationTracker

data class MutuallySection(val items: List<DefinesStatesOrViews>) : Phase2Node {
    override fun forEach(fn: (node: Phase2Node) -> Unit) = items.forEach(fn)

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter): CodeWriter {
        writer.writeIndent(isArg, indent)
        writer.writeHeader("Mutually")
        for (item in items) {
            writer.writeNewline()
            writer.append(item, true, indent + 2)
        }
        return writer
    }

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) =
        chalkTransformer(
            MutuallySection(items = items.map { chalkTransformer(it) as DefinesStatesOrViews }))
}

fun validateMutuallySection(node: Phase1Node, tracker: MutableLocationTracker) =
    validateClauseList(AtLeast(1), tracker, node, "Mutually") {
        val items = mutableListOf<DefinesStatesOrViews>()
        for (clause in it.clauses) {
            if (clause is DefinesStatesOrViews) {
                items.add(clause)
            } else {
                throw Exception("Expected a Defines:, States:, or Views:")
            }
        }

        MutuallySection(items = items)
    }

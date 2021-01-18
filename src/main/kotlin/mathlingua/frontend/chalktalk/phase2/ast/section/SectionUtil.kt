/*
 * Copyright 2019 Google LLC
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

package mathlingua.frontend.chalktalk.phase2.ast.section

import mathlingua.frontend.chalktalk.phase2.CodeWriter
import mathlingua.frontend.chalktalk.phase2.ast.clause.AbstractionNode
import mathlingua.frontend.chalktalk.phase2.ast.clause.AssignmentNode
import mathlingua.frontend.chalktalk.phase2.ast.clause.Identifier
import mathlingua.frontend.chalktalk.phase2.ast.clause.Target
import mathlingua.frontend.chalktalk.phase2.ast.clause.TupleNode

private fun canBeOnOneLine(target: Target) =
    target is Identifier ||
        target is TupleNode ||
        target is AbstractionNode ||
        target is AssignmentNode

fun appendTargetArgs(writer: CodeWriter, targets: List<Target>, indent: Int) {
    var i = 0
    while (i < targets.size) {
        val lineItems = mutableListOf<Target>()
        while (i < targets.size && canBeOnOneLine(targets[i])) {
            lineItems.add(targets[i++])
        }
        if (lineItems.isEmpty()) {
            writer.writeNewline()
            writer.append(targets[i++], true, indent)
        } else {
            writer.writeSpace()
            for (j in lineItems.indices) {
                writer.append(lineItems[j], false, 0)
                if (j != lineItems.size - 1) {
                    writer.writeComma()
                    writer.writeSpace()
                }
            }
        }
    }
}

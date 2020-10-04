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

package mathlingua.common.chalktalk.phase2.ast.section

import mathlingua.common.support.MutableLocationTracker
import mathlingua.common.chalktalk.phase1.ast.Phase1Node
import mathlingua.common.chalktalk.phase2.*
import mathlingua.common.chalktalk.phase2.ast.Phase2Node
import mathlingua.common.chalktalk.phase2.ast.clause.Target
import mathlingua.common.chalktalk.phase2.ast.clause.validateTargetList

data class GivenSection(val targets: List<Target>) : Phase2Node {
    override fun forEach(fn: (node: Phase2Node) -> Unit) = targets.forEach(fn)

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter): CodeWriter {
        writer.writeIndent(isArg, indent)
        writer.writeHeader("given")
        appendTargetArgs(writer, targets, indent + 2)
        return writer
    }

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) =
        chalkTransformer(GivenSection(
            targets = targets.map { it.transform(chalkTransformer) as Target }
        ))
}

fun validateGivenSection(node: Phase1Node, tracker: MutableLocationTracker) = validateTargetList(
    tracker,
    node,
    "given",
    ::GivenSection
)

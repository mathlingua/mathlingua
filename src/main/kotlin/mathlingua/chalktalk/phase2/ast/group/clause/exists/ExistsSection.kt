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

package mathlingua.chalktalk.phase2.ast.group.clause.exists

import mathlingua.support.MutableLocationTracker
import mathlingua.chalktalk.phase1.ast.Phase1Node
import mathlingua.chalktalk.phase2.CodeWriter
import mathlingua.common.chalktalk.phase2.*
import mathlingua.chalktalk.phase2.ast.common.Phase2Node
import mathlingua.chalktalk.phase2.ast.clause.Target
import mathlingua.chalktalk.phase2.ast.validator.validateTargetList
import mathlingua.chalktalk.phase2.ast.section.appendTargetArgs

data class ExistsSection(val identifiers: List<Target>) : Phase2Node {
    override fun forEach(fn: (node: Phase2Node) -> Unit) = identifiers.forEach(fn)

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter): CodeWriter {
        writer.writeIndent(isArg, indent)
        writer.writeHeader("exists")
        appendTargetArgs(writer, identifiers, indent + 2)
        return writer
    }

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) =
            chalkTransformer(ExistsSection(
                    identifiers = identifiers.map { it.transform(chalkTransformer) as Target }
            ))
}

fun validateExistsSection(node: Phase1Node, tracker: MutableLocationTracker) = validateTargetList(
        tracker,
        node,
        "exists",
        ::ExistsSection
)

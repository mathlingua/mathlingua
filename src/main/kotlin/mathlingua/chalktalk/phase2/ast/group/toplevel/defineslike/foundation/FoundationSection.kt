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

package mathlingua.chalktalk.phase2.ast.group.toplevel.defineslike.foundation

import mathlingua.support.MutableLocationTracker
import mathlingua.chalktalk.phase1.ast.Phase1Node
import mathlingua.chalktalk.phase2.CodeWriter
import mathlingua.chalktalk.phase2.ast.common.Phase2Node
import mathlingua.chalktalk.phase2.ast.validator.Exactly
import mathlingua.chalktalk.phase2.ast.validator.validateClauseList
import java.lang.ClassCastException
import java.lang.Exception

data class FoundationSection(val content: DefinesStatesOrViews) : Phase2Node {
    override fun forEach(fn: (node: Phase2Node) -> Unit) {
        fn(content)
    }

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter): CodeWriter {
        writer.writeIndent(isArg, indent)
        writer.writeHeader("Foundation")
        writer.writeNewline()
        writer.append(content, true, indent + 2)
        return writer
    }

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) =
        chalkTransformer(
            FoundationSection(
            content = chalkTransformer(content) as DefinesStatesOrViews
        )
        )
}

fun validateFoundationSection(node: Phase1Node, tracker: MutableLocationTracker) = validateClauseList(
    Exactly(1),
    tracker,
    node,
    "Foundation"
) {
    try {
        FoundationSection(content = it.clauses[0] as DefinesStatesOrViews)
    } catch (e: ClassCastException) {
        throw Exception("Expected a Defines, Represents, or a Views group")
    }
}

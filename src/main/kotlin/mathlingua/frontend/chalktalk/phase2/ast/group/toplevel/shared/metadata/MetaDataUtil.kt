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

package mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.metadata

import mathlingua.frontend.chalktalk.phase1.ast.Group
import mathlingua.frontend.chalktalk.phase1.ast.Phase1Node
import mathlingua.frontend.chalktalk.phase2.CodeWriter

internal fun indentedStringSection(
    writer: CodeWriter, isArg: Boolean, indent: Int, sectionName: String, value: String
): CodeWriter {
    writer.writeIndent(isArg, indent)
    writer.writeHeader(sectionName)
    writer.writeSpace()
    writer.writeText(value)
    return writer
}

internal fun isSingleSectionGroup(node: Phase1Node): Boolean {
    if (node !is Group) {
        return false
    }
    return node.sections.size == 1
}

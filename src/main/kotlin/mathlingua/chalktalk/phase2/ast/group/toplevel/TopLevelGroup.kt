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

package mathlingua.chalktalk.phase2.ast.group.toplevel

import mathlingua.chalktalk.phase2.CodeWriter
import mathlingua.chalktalk.phase2.ast.clause.IdStatement
import mathlingua.chalktalk.phase2.ast.common.Phase2Node
import mathlingua.chalktalk.phase2.ast.group.toplevel.shared.metadata.section.MetaDataSection

abstract class TopLevelGroup(open val metaDataSection: MetaDataSection?) : Phase2Node

fun topLevelToCode(
    writer: CodeWriter, isArg: Boolean, indent: Int, id: IdStatement?, vararg sections: Phase2Node?
): CodeWriter {
    writer.beginTopLevel()
    var useAsArg = isArg
    if (id != null) {
        writer.writeIndent(isArg, indent)
        writer.writeId(id)
        writer.writeNewline()
        useAsArg = false
    }

    val nonNullSections = sections.filterNotNull()
    for (i in nonNullSections.indices) {
        val sect = nonNullSections[i]
        writer.append(sect, useAsArg, indent)
        useAsArg = false
        if (i != nonNullSections.size - 1) {
            writer.writeNewline()
        }
    }
    writer.endTopLevel()

    return writer
}

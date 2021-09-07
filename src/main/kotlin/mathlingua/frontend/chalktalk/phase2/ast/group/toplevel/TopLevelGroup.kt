/*
 * Copyright 2019 The MathLingua Authors
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

package mathlingua.frontend.chalktalk.phase2.ast.group.toplevel

import mathlingua.backend.transform.Signature
import mathlingua.frontend.chalktalk.phase1.ast.BlockComment
import mathlingua.frontend.chalktalk.phase1.ast.Phase1Node
import mathlingua.frontend.chalktalk.phase2.CodeWriter
import mathlingua.frontend.chalktalk.phase2.ast.clause.IdStatement
import mathlingua.frontend.chalktalk.phase2.ast.common.Phase2Node
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.UsingSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.metadata.section.MetaDataSection
import mathlingua.getRandomUuid

abstract class TopLevelGroup(open val metaDataSection: MetaDataSection?) : Phase2Node

class TopLevelBlockComment(val blockComment: BlockComment) : TopLevelGroup(null) {
    override fun forEach(fn: (node: Phase2Node) -> Unit) {}

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter): CodeWriter {
        writer.writeBlockComment(blockComment.text)
        return writer
    }

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) =
        chalkTransformer(this)
}

fun isBlockComment(node: Phase1Node) = node is BlockComment

fun topLevelToCode(
    topLevelGroup: TopLevelGroup?,
    writer: CodeWriter,
    isArg: Boolean,
    indent: Int,
    id: IdStatement?,
    vararg sections: Phase2Node?
): CodeWriter {
    if (topLevelGroup != null) {
        writer.beginTopLevel(topLevelGroup, getRandomUuid())
    }
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
    if (topLevelGroup != null) {
        writer.endTopLevel(0)
    }
    return writer
}

internal interface HasUsingSection {
    val usingSection: UsingSection?
}

internal interface HasSignature {
    val signature: Signature?
    val id: IdStatement?
}

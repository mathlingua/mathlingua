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

package mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.metadata.section

import mathlingua.frontend.chalktalk.phase1.ast.Phase1Node
import mathlingua.frontend.chalktalk.phase2.CodeWriter
import mathlingua.frontend.chalktalk.phase2.ast.DEFAULT_RELATED_SECTION
import mathlingua.frontend.chalktalk.phase2.ast.common.Phase2Node
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.metadata.item.StringItem
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.metadata.item.validateStringItem
import mathlingua.frontend.chalktalk.phase2.ast.track
import mathlingua.frontend.chalktalk.phase2.ast.validateSection
import mathlingua.frontend.support.MutableLocationTracker
import mathlingua.frontend.support.ParseError

data class RelatedSection(val items: List<StringItem>) : Phase2Node {
    override fun forEach(fn: (node: Phase2Node) -> Unit) = items.forEach(fn)

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter): CodeWriter {
        writer.writeIndent(isArg, indent)
        writer.writeHeader("related")
        if (items.size == 1 && items[0] is StringItem) {
            writer.append(items[0], false, 1)
        } else {
            for (item in items) {
                writer.writeNewline()
                writer.append(item, true, indent + 2)
            }
        }
        return writer
    }

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) =
        chalkTransformer(RelatedSection(items = items.map { chalkTransformer(it) as StringItem }))
}

fun validateRelatedSection(
    node: Phase1Node, errors: MutableList<ParseError>, tracker: MutableLocationTracker
) =
    track(node, tracker) {
        validateSection(node, errors, "related", DEFAULT_RELATED_SECTION) { section ->
            RelatedSection(items = section.args.map { validateStringItem(it, errors, tracker) })
        }
    }

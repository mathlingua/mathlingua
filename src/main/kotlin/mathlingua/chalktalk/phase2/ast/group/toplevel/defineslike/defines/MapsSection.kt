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

package mathlingua.chalktalk.phase2.ast.group.toplevel.defineslike.defines

import mathlingua.chalktalk.phase1.ast.Phase1Node
import mathlingua.chalktalk.phase2.CodeWriter
import mathlingua.chalktalk.phase2.ast.DEFAULT_MAPS_SECTION
import mathlingua.chalktalk.phase2.ast.common.Phase2Node
import mathlingua.chalktalk.phase2.ast.group.clause.from.FromGroup
import mathlingua.chalktalk.phase2.ast.group.clause.from.validateFromGroup
import mathlingua.chalktalk.phase2.ast.track
import mathlingua.chalktalk.phase2.ast.validateSection
import mathlingua.chalktalk.phase2.ast.validateSingleArg
import mathlingua.support.MutableLocationTracker
import mathlingua.support.ParseError

data class MapsSection(val fromGroup: FromGroup) : Phase2Node {
    override fun forEach(fn: (node: Phase2Node) -> Unit) {
        fn(fromGroup)
    }

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter): CodeWriter {
        writer.writeIndent(isArg, indent)
        writer.writeHeader("maps")
        writer.writeNewline()
        writer.append(fromGroup, true, indent + 2)
        return writer
    }

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) =
        chalkTransformer(
            MapsSection(fromGroup = fromGroup.transform(chalkTransformer) as FromGroup))
}

fun validateMapsSection(
    node: Phase1Node, errors: MutableList<ParseError>, tracker: MutableLocationTracker
) =
    track(node, tracker) {
        validateSection(node.resolve(), errors, "maps", DEFAULT_MAPS_SECTION) { section ->
            validateSingleArg(section, errors, DEFAULT_MAPS_SECTION, "from group") {
                MapsSection(fromGroup = validateFromGroup(it, errors, tracker))
            }
        }
    }

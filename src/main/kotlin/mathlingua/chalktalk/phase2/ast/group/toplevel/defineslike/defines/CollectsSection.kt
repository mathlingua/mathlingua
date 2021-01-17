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
import mathlingua.chalktalk.phase2.ast.DEFAULT_COLLECTS_SECTION
import mathlingua.chalktalk.phase2.ast.common.Phase2Node
import mathlingua.chalktalk.phase2.ast.group.clause.given.GivenGroup
import mathlingua.chalktalk.phase2.ast.group.clause.given.validateGivenGroup
import mathlingua.chalktalk.phase2.ast.track
import mathlingua.chalktalk.phase2.ast.validateSection
import mathlingua.chalktalk.phase2.ast.validateSingleArg
import mathlingua.support.MutableLocationTracker
import mathlingua.support.ParseError

data class CollectsSection(val givenGroup: GivenGroup) : Phase2Node {
    override fun forEach(fn: (node: Phase2Node) -> Unit) {
        fn(givenGroup)
    }

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter): CodeWriter {
        writer.writeIndent(isArg, indent)
        writer.writeHeader("collects")
        writer.writeNewline()
        writer.append(givenGroup, true, indent + 2)
        return writer
    }

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) =
        chalkTransformer(
            CollectsSection(givenGroup = givenGroup.transform(chalkTransformer) as GivenGroup))
}

fun validateCollectsSection(
    node: Phase1Node, errors: MutableList<ParseError>, tracker: MutableLocationTracker
) =
    track(node, tracker) {
        validateSection(node.resolve(), errors, "collects", DEFAULT_COLLECTS_SECTION) { section ->
            validateSingleArg(section, errors, DEFAULT_COLLECTS_SECTION, "given group") { arg ->
                CollectsSection(givenGroup = validateGivenGroup(arg.resolve(), errors, tracker))
            }
        }
    }

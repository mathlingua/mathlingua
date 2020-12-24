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
import mathlingua.chalktalk.phase2.ast.DEFAULT_GENERATED_SECTION
import mathlingua.chalktalk.phase2.ast.common.Phase2Node
import mathlingua.chalktalk.phase2.ast.group.clause.inductively.InductivelyGroup
import mathlingua.chalktalk.phase2.ast.group.clause.inductively.validateInductivelyGroup
import mathlingua.chalktalk.phase2.ast.neoTrack
import mathlingua.chalktalk.phase2.ast.validateSection
import mathlingua.chalktalk.phase2.ast.validateSingleArg
import mathlingua.support.MutableLocationTracker
import mathlingua.support.ParseError

data class GeneratedSection(val inductivelyGroup: InductivelyGroup) : Phase2Node {
    override fun forEach(fn: (node: Phase2Node) -> Unit) {
        fn(inductivelyGroup)
    }

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter): CodeWriter {
        writer.writeIndent(isArg, indent)
        writer.writeHeader("generated")
        writer.append(inductivelyGroup, false, 1)
        return writer
    }

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) =
        chalkTransformer(
            GeneratedSection(
                inductivelyGroup =
                    inductivelyGroup.transform(chalkTransformer) as InductivelyGroup))
}

fun validateGeneratedSection(
    node: Phase1Node, errors: MutableList<ParseError>, tracker: MutableLocationTracker
) =
    neoTrack(node, tracker) {
        validateSection(node.resolve(), errors, "generated", DEFAULT_GENERATED_SECTION) { section ->
            validateSingleArg(section, errors, DEFAULT_GENERATED_SECTION, "generated group") {
                GeneratedSection(inductivelyGroup = validateInductivelyGroup(it, errors, tracker))
            }
        }
    }

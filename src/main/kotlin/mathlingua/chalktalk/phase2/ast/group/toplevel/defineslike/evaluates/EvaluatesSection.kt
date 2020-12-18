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

package mathlingua.chalktalk.phase2.ast.group.toplevel.defineslike.evaluates

import mathlingua.chalktalk.phase1.ast.Phase1Node
import mathlingua.chalktalk.phase1.ast.Section
import mathlingua.chalktalk.phase1.ast.getColumn
import mathlingua.chalktalk.phase1.ast.getRow
import mathlingua.chalktalk.phase2.CodeWriter
import mathlingua.chalktalk.phase2.ast.DEFAULT_EVALUATES_SECTION
import mathlingua.chalktalk.phase2.ast.common.Phase2Node
import mathlingua.chalktalk.phase2.ast.neoTrack
import mathlingua.chalktalk.phase2.ast.neoValidateSection
import mathlingua.support.MutableLocationTracker
import mathlingua.support.ParseError
import mathlingua.support.Validation
import mathlingua.support.validationFailure
import mathlingua.support.validationSuccess

class EvaluatesSection : Phase2Node {
    override fun forEach(fn: (node: Phase2Node) -> Unit) {}

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter): CodeWriter {
        writer.writeIndent(isArg, indent)
        writer.writeHeader("Evaluates")
        return writer
    }

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) =
        chalkTransformer(this)
}

fun validateEvaluatesSection(
    node: Phase1Node, tracker: MutableLocationTracker
): Validation<EvaluatesSection> {
    val errors = ArrayList<ParseError>()
    if (node !is Section) {
        errors.add(ParseError("Expected a EvaluatesSection", getRow(node), getColumn(node)))
    }

    val sect = node as Section
    if (sect.args.isNotEmpty()) {
        errors.add(
            ParseError("An Evaluates cannot have any arguments", getRow(node), getColumn(node)))
    }

    if (sect.name.text != "Evaluates") {
        errors.add(ParseError("Expected a section named Evaluates", getRow(node), getColumn(node)))
    }

    return if (errors.isNotEmpty()) {
        validationFailure(errors)
    } else {
        validationSuccess(tracker, node, EvaluatesSection())
    }
}

fun neoValidateEvaluatesSection(
    node: Phase1Node, errors: MutableList<ParseError>, tracker: MutableLocationTracker
) =
    neoTrack(node, tracker) {
        neoValidateSection(node.resolve(), errors, "Evaluates", DEFAULT_EVALUATES_SECTION) {
            EvaluatesSection()
        }
    }

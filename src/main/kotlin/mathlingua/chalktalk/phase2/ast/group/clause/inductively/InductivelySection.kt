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

package mathlingua.chalktalk.phase2.ast.group.clause.inductively

import mathlingua.chalktalk.phase1.ast.Phase1Node
import mathlingua.chalktalk.phase1.ast.Section
import mathlingua.chalktalk.phase1.ast.getColumn
import mathlingua.chalktalk.phase1.ast.getRow
import mathlingua.chalktalk.phase2.CodeWriter
import mathlingua.chalktalk.phase2.ast.DEFAULT_INDUCTIVELY_SECTION
import mathlingua.chalktalk.phase2.ast.common.Phase2Node
import mathlingua.chalktalk.phase2.ast.neoTrack
import mathlingua.chalktalk.phase2.ast.neoValidateSection
import mathlingua.support.MutableLocationTracker
import mathlingua.support.ParseError
import mathlingua.support.Validation
import mathlingua.support.validationFailure
import mathlingua.support.validationSuccess

class InductivelySection : Phase2Node {
    override fun forEach(fn: (node: Phase2Node) -> Unit) {}

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter): CodeWriter {
        writer.writeIndent(isArg, indent)
        writer.writeHeader("inductively")
        return writer
    }

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) =
        chalkTransformer(this)
}

fun validateInductivelySection(
    node: Phase1Node, tracker: MutableLocationTracker
): Validation<InductivelySection> {
    val errors = ArrayList<ParseError>()
    if (node !is Section) {
        errors.add(ParseError("Expected an inductively:", getRow(node), getColumn(node)))
    }

    val sect = node as Section
    if (sect.args.isNotEmpty()) {
        errors.add(
            ParseError("An inductively: cannot have any arguments", getRow(node), getColumn(node)))
    }

    if (sect.name.text != "inductively") {
        errors.add(
            ParseError("Expected a section named inductively:", getRow(node), getColumn(node)))
    }

    return if (errors.isNotEmpty()) {
        validationFailure(errors)
    } else {
        validationSuccess(tracker, node, InductivelySection())
    }
}

fun neoValidateInductivelySection(
    node: Phase1Node, errors: MutableList<ParseError>, tracker: MutableLocationTracker
) =
    neoTrack(node, tracker) {
        neoValidateSection(node.resolve(), errors, "inductively", DEFAULT_INDUCTIVELY_SECTION) {
            InductivelySection()
        }
    }

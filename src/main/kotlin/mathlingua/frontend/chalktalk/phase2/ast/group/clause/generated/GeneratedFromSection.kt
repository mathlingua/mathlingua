/*
 * Copyright 2022 The MathLingua Authors
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

package mathlingua.frontend.chalktalk.phase2.ast.group.clause.generated

import mathlingua.frontend.chalktalk.phase1.ast.Phase1Node
import mathlingua.frontend.chalktalk.phase1.ast.getColumn
import mathlingua.frontend.chalktalk.phase1.ast.getRow
import mathlingua.frontend.chalktalk.phase2.CodeWriter
import mathlingua.frontend.chalktalk.phase2.ast.DEFAULT_GENERATED_FROM_SECTION
import mathlingua.frontend.chalktalk.phase2.ast.clause.AbstractionNode
import mathlingua.frontend.chalktalk.phase2.ast.common.Phase2Node
import mathlingua.frontend.chalktalk.phase2.ast.track
import mathlingua.frontend.chalktalk.phase2.ast.validateTargetSection
import mathlingua.frontend.support.Location
import mathlingua.frontend.support.MutableLocationTracker
import mathlingua.frontend.support.ParseError

internal data class GeneratedFromSection(val forms: List<AbstractionNode>) : Phase2Node {
    override fun forEach(fn: (node: Phase2Node) -> Unit) = forms.forEach(fn)

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter): CodeWriter {
        writer.writeIndent(isArg, indent)
        writer.writeHeader("from")
        for (form in forms) {
            writer.writeNewline()
            writer.append(form, true, indent + 2)
        }
        return writer
    }

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) =
        chalkTransformer(
            GeneratedFromSection(
                forms = forms.map { it.transform(chalkTransformer) as AbstractionNode }))
}

internal fun validateGeneratedFromSection(
    node: Phase1Node, errors: MutableList<ParseError>, tracker: MutableLocationTracker
) =
    track(node, tracker) {
        validateTargetSection(
            node.resolve(), errors, "from", DEFAULT_GENERATED_FROM_SECTION, tracker) { targets ->
            var errorFound = false
            for (target in targets) {
                if (target !is AbstractionNode ||
                    target.abstraction.isVarArgs ||
                    target.abstraction.isEnclosed ||
                    target.abstraction.parts.size > 1 ||
                    target.abstraction.subParams != null) {
                    errorFound = true
                    val location =
                        tracker.getLocationOf(target)
                            ?: Location(row = getRow(node), column = getColumn(node))
                    errors.add(
                        ParseError(
                            message = "Expected a target of the form `X` or `f(x)`",
                            row = location.row,
                            column = location.column))
                }
            }

            if (errorFound) {
                DEFAULT_GENERATED_FROM_SECTION
            } else {
                GeneratedFromSection(forms = targets.map { it as AbstractionNode })
            }
        }
    }

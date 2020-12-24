/*
 * Copyright 2020 Google LLC
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

package mathlingua.chalktalk.phase2.ast.group.clause.expands

import mathlingua.chalktalk.phase1.ast.Abstraction
import mathlingua.chalktalk.phase1.ast.Phase1Node
import mathlingua.chalktalk.phase1.ast.getColumn
import mathlingua.chalktalk.phase1.ast.getRow
import mathlingua.chalktalk.phase2.CodeWriter
import mathlingua.chalktalk.phase2.ast.DEFAULT_EXPANDS_SECTION
import mathlingua.chalktalk.phase2.ast.clause.AbstractionNode
import mathlingua.chalktalk.phase2.ast.common.Phase2Node
import mathlingua.chalktalk.phase2.ast.neoTrack
import mathlingua.chalktalk.phase2.ast.section.appendTargetArgs
import mathlingua.chalktalk.phase2.ast.validateSection
import mathlingua.support.MutableLocationTracker
import mathlingua.support.ParseError

data class ExpandsSection(val targets: List<AbstractionNode>) : Phase2Node {
    override fun forEach(fn: (node: Phase2Node) -> Unit) = targets.forEach(fn)

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter): CodeWriter {
        writer.writeIndent(isArg, indent)
        writer.writeHeader("expands")
        appendTargetArgs(writer, targets, indent + 2)
        return writer
    }

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) =
        chalkTransformer(
            ExpandsSection(
                targets = targets.map { it.transform(chalkTransformer) as AbstractionNode }))
}

// only x... or {x}... is valid
private fun isValidAbstraction(abstraction: Abstraction) =
    abstraction.subParams == null &&
        abstraction.parts.size == 1 &&
        abstraction.parts[0].subParams == null &&
        abstraction.parts[0].params == null &&
        ((!abstraction.isEnclosed && abstraction.parts[0].name.text.endsWith("...")) ||
            (abstraction.isEnclosed &&
                abstraction.isVarArgs &&
                !abstraction.parts[0].name.text.endsWith("...")))

fun validateExpandsSection(
    node: Phase1Node, errors: MutableList<ParseError>, tracker: MutableLocationTracker
) =
    neoTrack(node, tracker) {
        validateSection(node, errors, "expands", DEFAULT_EXPANDS_SECTION) { section ->
            if (section.args.isEmpty() ||
                section.args.any {
                    it.chalkTalkTarget !is Abstraction || !isValidAbstraction(it.chalkTalkTarget)
                }) {
                errors.add(
                    ParseError(
                        message = "Expected an abstraction",
                        row = getRow(node),
                        column = getColumn(node)))
                DEFAULT_EXPANDS_SECTION
            } else {
                ExpandsSection(
                    targets =
                        section.args.map {
                            AbstractionNode(abstraction = it.chalkTalkTarget as Abstraction)
                        })
            }
        }
    }

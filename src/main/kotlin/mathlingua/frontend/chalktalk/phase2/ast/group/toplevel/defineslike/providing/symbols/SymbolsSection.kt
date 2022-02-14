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

package mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.providing.symbols

import mathlingua.frontend.chalktalk.phase1.ast.Phase1Node
import mathlingua.frontend.chalktalk.phase2.CodeWriter
import mathlingua.frontend.chalktalk.phase2.ast.DEFAULT_SYMBOLS_SECTION
import mathlingua.frontend.chalktalk.phase2.ast.clause.Target
import mathlingua.frontend.chalktalk.phase2.ast.common.Phase2Node
import mathlingua.frontend.chalktalk.phase2.ast.section.appendTargetArgs
import mathlingua.frontend.chalktalk.phase2.ast.track
import mathlingua.frontend.chalktalk.phase2.ast.validateTargetSection
import mathlingua.frontend.support.MutableLocationTracker
import mathlingua.frontend.support.ParseError

internal data class SymbolsSection(val targets: List<Target>) : Phase2Node {
    override fun forEach(fn: (node: Phase2Node) -> Unit) = targets.forEach(fn)

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter): CodeWriter {
        writer.writeIndent(isArg, indent)
        writer.writeHeader("symbols")
        appendTargetArgs(writer, targets, indent + 2)
        return writer
    }

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) =
        chalkTransformer(
            SymbolsSection(targets = targets.map { it.transform(chalkTransformer) as Target }))
}

internal fun validateSymbolsSection(
    node: Phase1Node, errors: MutableList<ParseError>, tracker: MutableLocationTracker
) =
    track(node, tracker) {
        validateTargetSection(
            node.resolve(), errors, "symbols", DEFAULT_SYMBOLS_SECTION, tracker) { targets ->
            SymbolsSection(targets = targets)
        }
    }

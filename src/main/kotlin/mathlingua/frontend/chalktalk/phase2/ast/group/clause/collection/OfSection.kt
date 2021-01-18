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

package mathlingua.frontend.chalktalk.phase2.ast.group.clause.collection

import mathlingua.frontend.chalktalk.phase1.ast.Phase1Node
import mathlingua.frontend.chalktalk.phase2.CodeWriter
import mathlingua.frontend.chalktalk.phase2.ast.DEFAULT_OF_SECTION
import mathlingua.frontend.chalktalk.phase2.ast.clause.Statement
import mathlingua.frontend.chalktalk.phase2.ast.clause.validateStatement
import mathlingua.frontend.chalktalk.phase2.ast.common.Phase2Node
import mathlingua.frontend.chalktalk.phase2.ast.track
import mathlingua.frontend.chalktalk.phase2.ast.validateSection
import mathlingua.frontend.chalktalk.phase2.ast.validateSingleArg
import mathlingua.frontend.support.MutableLocationTracker
import mathlingua.frontend.support.ParseError

data class OfSection(val statement: Statement) : Phase2Node {
    override fun forEach(fn: (node: Phase2Node) -> Unit) {
        fn(statement)
    }

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter): CodeWriter {
        writer.writeIndent(isArg, indent)
        writer.writeHeader("of")
        writer.append(statement, false, 1)
        return writer
    }

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) =
        chalkTransformer(OfSection(statement = statement.transform(chalkTransformer) as Statement))
}

fun validateOfSection(
    node: Phase1Node, errors: MutableList<ParseError>, tracker: MutableLocationTracker
) =
    track(node, tracker) {
        validateSection(node.resolve(), errors, "of", DEFAULT_OF_SECTION) { section ->
            validateSingleArg(section, errors, DEFAULT_OF_SECTION, "statement") {
                OfSection(statement = validateStatement(it, errors, tracker))
            }
        }
    }

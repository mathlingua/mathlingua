/*
 * Copyright 2020 The MathLingua Authors
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

import mathlingua.frontend.chalktalk.phase1.ast.ChalkTalkTokenType
import mathlingua.frontend.chalktalk.phase1.ast.Phase1Node
import mathlingua.frontend.chalktalk.phase1.ast.Phase1Token
import mathlingua.frontend.chalktalk.phase2.CodeWriter
import mathlingua.frontend.chalktalk.phase2.ast.DEFAULT_STRING_SECTION
import mathlingua.frontend.chalktalk.phase2.ast.common.Phase2Node
import mathlingua.frontend.chalktalk.phase2.ast.validateSection
import mathlingua.frontend.support.ParseError

internal data class StringSection(
    val name: String, val values: List<String>, override val row: Int, override val column: Int
) : Phase2Node {
    override fun forEach(fn: (node: Phase2Node) -> Unit) {}

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter): CodeWriter {
        writer.writeIndent(isArg, indent)
        writer.writeHeader(name)
        if (values.size == 1) {
            writer.writeSpace()
            writer.writeText(values[0])
        } else {
            for (value in values) {
                writer.writeNewline()
                writer.writeIndent(true, indent + 2)
                writer.writeText(value)
            }
        }
        return writer
    }

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) =
        chalkTransformer(this)
}

internal fun validateStringSection(
    name: String, node: Phase1Node, errors: MutableList<ParseError>
) =
    validateSection(node, errors, name, DEFAULT_STRING_SECTION) { section ->
        if (section.args.isNotEmpty() &&
            !section.args.all {
                it.chalkTalkTarget is Phase1Token &&
                    it.chalkTalkTarget.type == ChalkTalkTokenType.String
            }) {
            errors.add(
                ParseError(
                    message = "Expected a list of strings",
                    row = section.row,
                    column = section.column))
            DEFAULT_STRING_SECTION
        } else {
            StringSection(
                name = name,
                values = section.args.map { (it.chalkTalkTarget as Phase1Token).text },
                row = node.row,
                column = node.column)
        }
    }

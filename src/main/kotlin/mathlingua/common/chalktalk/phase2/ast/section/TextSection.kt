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

package mathlingua.common.chalktalk.phase2.ast.section

import mathlingua.common.*
import mathlingua.common.chalktalk.phase1.ast.*
import mathlingua.common.chalktalk.phase2.CodeWriter
import mathlingua.common.chalktalk.phase2.ast.Phase2Node

data class TextSection(
    val name: String,
    val text: String
) : Phase2Node {
    override fun forEach(fn: (node: Phase2Node) -> Unit) {
    }

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter): CodeWriter {
        writer.writeIndent(isArg, indent)
        writer.writeHeader(name)
        writer.writeNewline()
        writer.writeIndent(true, indent + 2)
        writer.writeDirect(text)
        return writer
    }

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) =
            chalkTransformer(this)
}

fun validateTextSection(
    rawNode: Phase1Node,
    name: String,
    tracker: MutableLocationTracker
): Validation<TextSection> {
    val node = rawNode.resolve()
    val row = getRow(node)
    val column = getColumn(node)

    val errors = ArrayList<ParseError>()
    if (node !is Section) {
        errors.add(
                ParseError(
                        "Expected a Section",
                        getRow(node), getColumn(node)
                )
        )
    }

    val sect = node as Section
    if (sect.name.text != name) {
        errors.add(
                ParseError(
                        "Expected a Section with name " +
                                name + " but found " + sect.name.text,
                        row, column
                )
        )
    }

    if (sect.args.size != 1) {
        errors.add(
                ParseError(
                        "Section '" + sect.name.text + "' requires exactly one text argument.",
                        row, column
                )
        )
        return validationFailure(errors)
    }

    val arg = sect.args[0].chalkTalkTarget
    if (arg !is Phase1Token) {
        errors.add(ParseError(
                "Expected a string but found ${arg.toCode()}",
                row, column
        ))
    }

    return if (errors.isNotEmpty()) {
        validationFailure(errors)
    } else {
        validationSuccess(
                tracker,
                rawNode,
                TextSection(
                    name = name,
                    text = (arg as Phase1Token).text
                ))
    }
}

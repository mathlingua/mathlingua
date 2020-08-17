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

import mathlingua.common.MutableLocationTracker
import mathlingua.common.ParseError
import mathlingua.common.Validation
import mathlingua.common.chalktalk.phase1.ast.Phase1Node
import mathlingua.common.chalktalk.phase1.ast.Phase1Token
import mathlingua.common.chalktalk.phase1.ast.Section
import mathlingua.common.chalktalk.phase1.ast.getColumn
import mathlingua.common.chalktalk.phase1.ast.getRow
import mathlingua.common.chalktalk.phase2.CodeWriter
import mathlingua.common.chalktalk.phase2.ast.Phase2Node
import mathlingua.common.validationFailure
import mathlingua.common.validationSuccess

data class ImportSection(val imports: List<String>) : Phase2Node {
    override fun forEach(fn: (node: Phase2Node) -> Unit) {
    }

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter): CodeWriter {
        writer.writeIndent(isArg, indent)
        writer.writeHeader("import")
        writer.writeNewline()
        for (imp in imports) {
            writer.writeIndent(true, indent + 2)
            writer.writeDirect(imp)
        }
        return writer
    }

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) =
        chalkTransformer(this)
}

fun validateImportSection(
    rawNode: Phase1Node,
    tracker: MutableLocationTracker
): Validation<ImportSection> {
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
    if (sect.name.text != "import") {
        errors.add(
            ParseError(
                "Expected a Section with name 'import' but found " + sect.name.text,
                row, column
            )
        )
    }

    val imports = mutableListOf<String>()
    for (arg in sect.args) {
        if (arg.chalkTalkTarget !is Phase1Token) {
            errors.add(
                ParseError(
                    "Expected a string but found ${arg.toCode()}",
                    row, column
                )
            )
        }
        imports.add((arg.chalkTalkTarget as Phase1Token).text)
    }

    return if (errors.isNotEmpty()) {
        validationFailure(errors)
    } else {
        validationSuccess(
            tracker,
            rawNode,
            ImportSection(
                imports = imports
            )
        )
    }
}

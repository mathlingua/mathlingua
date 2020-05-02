/*
 * Copyright 2019 Google LLC
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

import mathlingua.common.ParseError
import mathlingua.common.Validation
import mathlingua.common.ValidationFailure
import mathlingua.common.ValidationSuccess
import mathlingua.common.chalktalk.phase1.ast.Section
import mathlingua.common.chalktalk.phase1.ast.getColumn
import mathlingua.common.chalktalk.phase1.ast.getRow
import mathlingua.common.chalktalk.phase2.CodeWriter
import mathlingua.common.chalktalk.phase2.ast.clause.MappingNode
import mathlingua.common.chalktalk.phase2.ast.Phase2Node
import mathlingua.common.chalktalk.phase2.ast.clause.validateMappingNode

data class AliasSection(
    val mappings: List<MappingNode>,
    override var row: Int,
    override var column: Int
) : Phase2Node {
    override fun forEach(fn: (node: Phase2Node) -> Unit) = mappings.forEach(fn)

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter): CodeWriter {
        writer.writeIndent(isArg, indent)
        writer.writeHeader("Alias")
        writer.writeNewline()
        for (i in mappings.indices) {
            writer.append(mappings[i], true, indent + 2)
            if (i != mappings.size - 1) {
                writer.writeNewline()
            }
        }
        return writer
    }

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) = chalkTransformer(AliasSection(
            mappings = mappings.map { it.transform(chalkTransformer) as MappingNode },
            row = row,
            column = column
    ))
}

fun validateAliasSection(section: Section): Validation<AliasSection> {
    if (section.name.text != "Alias") {
        return ValidationFailure(
            listOf(
                ParseError(
                    "Expected a 'Alias' but found '${section.name.text}'",
                    getRow(section), getColumn(section)
                )
            )
        )
    }

    val errors = mutableListOf<ParseError>()
    val mappings = mutableListOf<MappingNode>()
    for (arg in section.args) {
        when (val validation = validateMappingNode(arg)) {
            is ValidationSuccess -> mappings.add(validation.value)
            is ValidationFailure -> errors.addAll(validation.errors)
        }
    }

    return if (errors.isNotEmpty()) {
        ValidationFailure(errors)
    } else {
        ValidationSuccess(AliasSection(
                mappings = mappings,
                row = getRow(section),
                column = getColumn(section)
        ))
    }
}

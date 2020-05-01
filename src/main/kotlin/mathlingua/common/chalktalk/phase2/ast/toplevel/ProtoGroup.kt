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

package mathlingua.common.chalktalk.phase2.ast.toplevel

import mathlingua.common.ParseError
import mathlingua.common.Validation
import mathlingua.common.ValidationFailure
import mathlingua.common.ValidationSuccess
import mathlingua.common.chalktalk.phase1.ast.Group
import mathlingua.common.chalktalk.phase1.ast.Section
import mathlingua.common.chalktalk.phase1.ast.getColumn
import mathlingua.common.chalktalk.phase1.ast.getRow
import mathlingua.common.chalktalk.phase2.CodeWriter
import mathlingua.common.chalktalk.phase2.ast.Phase2Node
import mathlingua.common.chalktalk.phase2.ast.section.*

class ProtoGroup(
    val textSection: TextSection,
    override val metaDataSection: MetaDataSection?,
    override var row: Int,
    override var column: Int
) : TopLevelGroup(metaDataSection) {
    override fun forEach(fn: (node: Phase2Node) -> Unit) {
        fn(textSection)
        if (metaDataSection != null) {
            fn(metaDataSection)
        }
    }

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter) =
            toCode(writer, isArg, indent, null, textSection, metaDataSection)

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) =
            chalkTransformer(ProtoGroup(
                    textSection = textSection.transform(chalkTransformer) as TextSection,
                    metaDataSection = metaDataSection?.transform(chalkTransformer) as? MetaDataSection,
                    row = row,
                    column = column
            ))
}

fun validateProtoGroup(
    groupNode: Group,
    name: String
): Validation<ProtoGroup> {
    val errors = ArrayList<ParseError>()
    val group = groupNode.resolve()
    if (group.id != null) {
        errors.add(
                ParseError(
                        "A proto group cannot have an Id",
                        getRow(group), getColumn(group)
                )
        )
    }

    val sections = group.sections

    val sectionMap: Map<String, List<Section>>
    try {
        sectionMap = identifySections(
                sections, name, "Metadata?"
        )
    } catch (e: ParseError) {
        errors.add(ParseError(e.message, e.row, e.column))
        return ValidationFailure(errors)
    }

    val textSect = sectionMap[name]
    val metadata = sectionMap["Metadata"] ?: emptyList()

    if (textSect == null || textSect.size != 1) {
        errors.add(
                ParseError(
                        "Expected a single section with name $name",
                        getRow(group), getColumn(group)
                )
        )
    }

    var textSection: TextSection? = null
    when (val validation = validateTextSection(textSect!![0], name)) {
        is ValidationSuccess -> textSection = validation.value
        is ValidationFailure -> errors.addAll(validation.errors)
    }

    var metaDataSection: MetaDataSection? = null
    if (metadata.isNotEmpty()) {
        when (val metaDataValidation = validateMetaDataSection(metadata[0])) {
            is ValidationSuccess -> metaDataSection = metaDataValidation.value
            is ValidationFailure -> errors.addAll(metaDataValidation.errors)
        }
    }

    return if (errors.isNotEmpty()) {
        ValidationFailure(errors)
    } else {
        ValidationSuccess(ProtoGroup(
                textSection = textSection!!,
                metaDataSection = metaDataSection,
                row = getRow(group),
                column = getColumn(group)
        ))
    }
}

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

package mathlingua.common.chalktalk.phase2.ast.metadata.section

import mathlingua.common.*
import mathlingua.common.chalktalk.phase1.ast.*
import mathlingua.common.chalktalk.phase2.CodeWriter
import mathlingua.common.chalktalk.phase2.ast.Phase2Node
import mathlingua.common.chalktalk.phase2.ast.metadata.*
import mathlingua.common.chalktalk.phase2.ast.metadata.isSingleSectionGroup
import mathlingua.common.chalktalk.phase2.ast.metadata.item.MetaDataItem
import mathlingua.common.chalktalk.phase2.ast.metadata.item.StringSectionGroup
import mathlingua.common.chalktalk.phase2.ast.metadata.item.isReferenceGroup
import mathlingua.common.chalktalk.phase2.ast.metadata.item.validateReferenceGroup

private val META_DATA_ITEM_CONSTRAINTS = mapOf(
        "name" to -1,
        "classification" to -1,
        "tag" to -1,
        "author" to -1,
        "contributor" to -1,
        "note" to -1,
        "id" to 1,
        "concept" to 1,
        "summary" to 1)

data class MetaDataSection(val items: List<MetaDataItem>) : Phase2Node {
    override fun forEach(fn: (node: Phase2Node) -> Unit) = items.forEach(fn)

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter): CodeWriter {
        writer.writeIndent(isArg, indent)
        writer.writeHeader("Metadata")
        for (i in items.indices) {
            writer.writeNewline()
            writer.append(items[i], true, indent + 2)
        }
        return writer
    }

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) =
            chalkTransformer(MetaDataSection(
                    items = items.map { it.transform(chalkTransformer) as MetaDataItem }
            ))
}

fun validateMetaDataSection(section: Section, tracker: MutableLocationTracker): Validation<MetaDataSection> {
    if (section.name.text != "Metadata") {
        return validationFailure(
                listOf(
                        ParseError(
                                "Expected a 'Metadata' but found '${section.name.text}'",
                                getRow(section), getColumn(section)
                        )
                )
        )
    }

    val errors = mutableListOf<ParseError>()
    val items = mutableListOf<MetaDataItem>()
    for (arg in section.args) {
        if (isReferenceGroup(arg.chalkTalkTarget)) {
            when (val validation = validateMetaDataItem(arg, tracker)) {
                is ValidationSuccess -> items.add(validation.value)
                is ValidationFailure -> errors.addAll(validation.errors)
            }
        } else if (isSingleSectionGroup(arg.chalkTalkTarget)) {
            val group = arg.chalkTalkTarget as Group
            val sect = group.sections[0]
            val name = sect.name.text
            if (META_DATA_ITEM_CONSTRAINTS.containsKey(name)) {
                val expectedCount = META_DATA_ITEM_CONSTRAINTS[name]!!
                if (expectedCount >= 0 && sect.args.size != expectedCount) {
                    errors.add(
                            ParseError(
                                    message = "Expected $expectedCount arguments for " +
                                        "section $name but found ${sect.args.size}",
                                    row = getRow(sect),
                                    column = getColumn(sect)
                            )
                    )
                }
                val values = mutableListOf<String>()
                for (a in sect.args) {
                    if (a.chalkTalkTarget is Phase1Token &&
                            a.chalkTalkTarget.type == ChalkTalkTokenType.String) {
                        values.add(a.chalkTalkTarget.text)
                    } else {
                        errors.add(
                                ParseError(
                                        message = "Expected a string but found ${a.chalkTalkTarget}",
                                        row = getRow(a.chalkTalkTarget),
                                        column = getColumn(a.chalkTalkTarget)
                                )
                        )
                    }
                }
                val location = Location(
                        row = getRow(sect),
                        column = getColumn(sect)
                )

                val s = StringSection(
                        name = name,
                        values = values
                )
                tracker.setLocationOf(s, location)

                val res = StringSectionGroup(
                        section = s
                )
                tracker.setLocationOf(res, location)

                items.add(res)
            } else {
                errors.add(
                        ParseError(
                                message = "Expected a section with one of " +
                                    "the names ${META_DATA_ITEM_CONSTRAINTS.keys}",
                                row = getRow(arg),
                                column = getColumn(arg)
                        )
                )
            }
        } else {
            errors.add(
                    ParseError(
                            message = "Unexpected item '${arg.toCode()}'",
                            row = getRow(arg),
                            column = getColumn(arg)
                    )
            )
        }
    }

    return if (errors.isNotEmpty()) {
        validationFailure(errors)
    } else {
        validationSuccess(tracker, section, MetaDataSection(items = items))
    }
}

private fun validateMetaDataItem(arg: Argument, tracker: MutableLocationTracker): Validation<MetaDataItem> {
    if (arg.chalkTalkTarget !is Group) {
        return validationFailure(listOf(
                ParseError(
                        message = "Expected a group",
                        row = getRow(arg),
                        column = getColumn(arg)
                )
        ))
    }

    return validateReferenceGroup(arg.chalkTalkTarget, tracker)
}

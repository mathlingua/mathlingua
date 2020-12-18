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

package mathlingua.chalktalk.phase2.ast.group.toplevel.shared.metadata.section

import mathlingua.chalktalk.phase1.ast.Argument
import mathlingua.chalktalk.phase1.ast.ChalkTalkTokenType
import mathlingua.chalktalk.phase1.ast.Group
import mathlingua.chalktalk.phase1.ast.Phase1Node
import mathlingua.chalktalk.phase1.ast.Phase1Token
import mathlingua.chalktalk.phase1.ast.Section
import mathlingua.chalktalk.phase1.ast.getColumn
import mathlingua.chalktalk.phase1.ast.getRow
import mathlingua.chalktalk.phase2.CodeWriter
import mathlingua.chalktalk.phase2.ast.DEFAULT_META_DATA_SECTION
import mathlingua.chalktalk.phase2.ast.common.Phase2Node
import mathlingua.chalktalk.phase2.ast.group.toplevel.shared.metadata.isSingleSectionGroup
import mathlingua.chalktalk.phase2.ast.group.toplevel.shared.metadata.item.MetaDataItem
import mathlingua.chalktalk.phase2.ast.group.toplevel.shared.metadata.item.StringSectionGroup
import mathlingua.chalktalk.phase2.ast.group.toplevel.shared.metadata.item.isReferenceGroup
import mathlingua.chalktalk.phase2.ast.group.toplevel.shared.metadata.item.validateReferenceGroup
import mathlingua.chalktalk.phase2.ast.neoTrack
import mathlingua.chalktalk.phase2.ast.neoValidateSection
import mathlingua.support.Location
import mathlingua.support.MutableLocationTracker
import mathlingua.support.ParseError
import mathlingua.support.Validation
import mathlingua.support.ValidationFailure
import mathlingua.support.ValidationSuccess
import mathlingua.support.validationFailure
import mathlingua.support.validationSuccess

private val META_DATA_ITEM_CONSTRAINTS =
    mapOf(
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
        chalkTransformer(
            MetaDataSection(items = items.map { it.transform(chalkTransformer) as MetaDataItem }))
}

fun isMetadataSection(sec: Section) = sec.name.text == "Metadata"

fun validateMetaDataSection(
    section: Section, tracker: MutableLocationTracker
): Validation<MetaDataSection> {
    if (section.name.text != "Metadata") {
        return validationFailure(
            listOf(
                ParseError(
                    "Expected a 'Metadata' but found '${section.name.text}'",
                    getRow(section),
                    getColumn(section))))
    }

    val errors = mutableListOf<ParseError>()
    val items = mutableListOf<MetaDataItem>()
    for (arg in section.args) {
        if (isReferenceGroup(arg.chalkTalkTarget)) {
            when (val validation = validateMetaDataItem(arg, tracker)
            ) {
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
                            message =
                                "Expected $expectedCount arguments for " +
                                    "section $name but found ${sect.args.size}",
                            row = getRow(sect),
                            column = getColumn(sect)))
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
                                column = getColumn(a.chalkTalkTarget)))
                    }
                }
                val location = Location(row = getRow(sect), column = getColumn(sect))

                val s = StringSection(name = name, values = values)
                tracker.setLocationOf(s, location)

                val res = StringSectionGroup(section = s)
                tracker.setLocationOf(res, location)

                items.add(res)
            } else {
                errors.add(
                    ParseError(
                        message =
                            "Expected a section with one of " +
                                "the names ${META_DATA_ITEM_CONSTRAINTS.keys}",
                        row = getRow(arg),
                        column = getColumn(arg)))
            }
        } else {
            errors.add(
                ParseError(
                    message = "Unexpected item '${arg.toCode()}'",
                    row = getRow(arg),
                    column = getColumn(arg)))
        }
    }

    return if (errors.isNotEmpty()) {
        validationFailure(errors)
    } else {
        validationSuccess(tracker, section, MetaDataSection(items = items))
    }
}

private fun validateMetaDataItem(
    arg: Argument, tracker: MutableLocationTracker
): Validation<MetaDataItem> {
    if (arg.chalkTalkTarget !is Group) {
        return validationFailure(
            listOf(
                ParseError(
                    message = "Expected a group", row = getRow(arg), column = getColumn(arg))))
    }

    return validateReferenceGroup(arg.chalkTalkTarget, tracker)
}

fun neoValidateMetaDataSection(
    node: Phase1Node, errors: MutableList<ParseError>, tracker: MutableLocationTracker
) =
    neoTrack(node, tracker) {
        neoValidateSection(node, errors, "Metadata", DEFAULT_META_DATA_SECTION) {
            when (val validation = validateMetaDataSection(it, tracker)
            ) {
                is ValidationSuccess -> validation.value
                is ValidationFailure -> {
                    errors.addAll(validation.errors)
                    DEFAULT_META_DATA_SECTION
                }
            }
        }
    }

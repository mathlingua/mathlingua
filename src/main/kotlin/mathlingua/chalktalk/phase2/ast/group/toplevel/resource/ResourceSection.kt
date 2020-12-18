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

package mathlingua.chalktalk.phase2.ast.group.toplevel.resource

import mathlingua.chalktalk.phase1.ast.ChalkTalkTokenType
import mathlingua.chalktalk.phase1.ast.Group
import mathlingua.chalktalk.phase1.ast.Phase1Node
import mathlingua.chalktalk.phase1.ast.Phase1Token
import mathlingua.chalktalk.phase1.ast.Section
import mathlingua.chalktalk.phase1.ast.getColumn
import mathlingua.chalktalk.phase1.ast.getRow
import mathlingua.chalktalk.phase2.CodeWriter
import mathlingua.chalktalk.phase2.ast.DEFAULT_RESOURCE_SECTION
import mathlingua.chalktalk.phase2.ast.common.Phase2Node
import mathlingua.chalktalk.phase2.ast.group.toplevel.shared.metadata.isSingleSectionGroup
import mathlingua.chalktalk.phase2.ast.group.toplevel.shared.metadata.item.StringSectionGroup
import mathlingua.chalktalk.phase2.ast.group.toplevel.shared.metadata.section.StringSection
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

val SOURCE_ITEM_CONSTRAINTS =
    mapOf(
        "type" to 1,
        "name" to 1,
        "author" to -1,
        "date" to 1,
        "homepage" to 1,
        "url" to 1,
        "offset" to 1,
        "related" to -1)

class ResourceSection(val items: List<StringSectionGroup>) : Phase2Node {
    override fun forEach(fn: (node: Phase2Node) -> Unit) = items.forEach(fn)

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter): CodeWriter {
        writer.writeIndent(isArg, indent)
        writer.writeHeader("Resource")
        writer.writeNewline()
        for (i in items.indices) {
            writer.append(items[i], true, indent + 2)
            if (i != items.size - 1) {
                writer.writeNewline()
            }
        }
        return writer
    }

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) =
        chalkTransformer(this)
}

fun validateResourceSection(
    section: Section, tracker: MutableLocationTracker
): Validation<ResourceSection> {
    if (section.name.text != "Resource") {
        return validationFailure(
            listOf(
                ParseError(
                    "Expected a 'Resource' but found '${section.name.text}'",
                    getRow(section),
                    getColumn(section))))
    }

    val errors = mutableListOf<ParseError>()
    val items = mutableListOf<StringSectionGroup>()
    for (arg in section.args) {
        if (isSingleSectionGroup(arg.chalkTalkTarget)) {
            val group = arg.chalkTalkTarget as Group
            val sect = group.sections[0]
            val name = sect.name.text
            if (SOURCE_ITEM_CONSTRAINTS.containsKey(name)) {
                val expectedCount = SOURCE_ITEM_CONSTRAINTS[name]!!
                if (expectedCount >= 0 && sect.args.size != expectedCount) {
                    errors.add(
                        ParseError(
                            message =
                                "Expected $expectedCount arguments for " +
                                    "section $name but found ${sect.args.size}",
                            row = getRow(sect),
                            column = getColumn(sect)))
                } else if (expectedCount < 0 && sect.args.size < -expectedCount) {
                    errors.add(
                        ParseError(
                            message =
                                "Expected at least ${-expectedCount} arguments for " +
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

                val location = Location(row = getRow(arg), column = getColumn(arg))

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
                                "the names ${SOURCE_ITEM_CONSTRAINTS.keys}",
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
        validationSuccess(tracker, section, ResourceSection(items = items))
    }
}

fun neoValidateResourceSection(
    node: Phase1Node, errors: MutableList<ParseError>, tracker: MutableLocationTracker
) =
    neoTrack(node, tracker) {
        neoValidateSection(node.resolve(), errors, "Resource", DEFAULT_RESOURCE_SECTION) {
        section ->
            when (val validation = validateResourceSection(section, tracker)
            ) {
                is ValidationSuccess -> validation.value
                is ValidationFailure -> {
                    errors.addAll(validation.errors)
                    DEFAULT_RESOURCE_SECTION
                }
            }
        }
    }

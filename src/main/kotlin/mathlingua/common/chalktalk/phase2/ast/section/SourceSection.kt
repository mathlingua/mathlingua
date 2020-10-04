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

import mathlingua.common.chalktalk.phase1.ast.*
import mathlingua.common.chalktalk.phase2.CodeWriter
import mathlingua.common.chalktalk.phase2.ast.Phase2Node
import mathlingua.common.chalktalk.phase2.ast.metadata.section.StringSection
import mathlingua.common.chalktalk.phase2.ast.metadata.isSingleSectionGroup
import mathlingua.common.chalktalk.phase2.ast.metadata.item.StringSectionGroup
import mathlingua.common.support.Location
import mathlingua.common.support.MutableLocationTracker
import mathlingua.common.support.ParseError
import mathlingua.common.support.Validation
import mathlingua.common.support.validationFailure
import mathlingua.common.support.validationSuccess

val SOURCE_ITEM_CONSTRAINTS = mapOf(
        "type" to 1,
        "name" to 1,
        "author" to -1,
        "date" to 1,
        "homepage" to 1,
        "url" to 1,
        "offset" to 1,
        "related" to -1
)

open class SourceSection(open val items: List<StringSectionGroup>) : Phase2Node {
    override fun forEach(fn: (node: Phase2Node) -> Unit) = items.forEach(fn)

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter): CodeWriter {
        writer.writeIndent(isArg, indent)
        writer.writeHeader("Source")
        writer.writeNewline()
        for (i in items.indices) {
            writer.append(items[i], true, indent + 2)
            if (i != items.size - 1) {
                writer.writeNewline()
            }
        }
        return writer
    }

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) = chalkTransformer(this)
}

fun validateSourceSection(section: Section, tracker: MutableLocationTracker): Validation<SourceSection> {
    if (section.name.text != "Source") {
        return validationFailure(
                listOf(
                        ParseError(
                                "Expected a 'Source' but found '${section.name.text}'",
                                getRow(section), getColumn(section)
                        )
                )
        )
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
                                    message = "Expected $expectedCount arguments for " +
                                            "section $name but found ${sect.args.size}",
                                    row = getRow(sect),
                                    column = getColumn(sect)
                            )
                    )
                } else if (expectedCount < 0 && sect.args.size < -expectedCount) {
                    errors.add(
                            ParseError(
                                    message = "Expected at least ${-expectedCount} arguments for " +
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
                        row = getRow(arg),
                        column = getColumn(arg)
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
                                        "the names ${SOURCE_ITEM_CONSTRAINTS.keys}",
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
        validationSuccess(tracker, section, SourceSection(items = items))
    }
}

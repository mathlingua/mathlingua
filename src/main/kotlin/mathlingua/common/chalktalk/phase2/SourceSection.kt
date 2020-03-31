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

package mathlingua.common.chalktalk.phase2

import mathlingua.common.ParseError
import mathlingua.common.Validation
import mathlingua.common.ValidationFailure
import mathlingua.common.ValidationSuccess
import mathlingua.common.chalktalk.phase1.ast.*

val SOURCE_ITEM_CONSTRAINTS = mapOf(
        "type" to 1,
        "name" to 1,
        "author" to -1,
        "date" to 1,
        "homepage" to 1,
        "url" to 1,
        "offset" to 1)

data class SourceSection(
    val items: List<StringSectionGroup>,
    override var row: Int,
    override var column: Int
) : Phase2Node {
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

fun validateSourceSection(section: Section): Validation<SourceSection> {
    if (section.name.text != "Source") {
        return ValidationFailure(
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
                items.add(StringSectionGroup(
                        section = StringSection(
                                name = name,
                                values = values,
                                row = if (sect.args.isNotEmpty()) {
                                    getRow(sect.args[0])
                                } else {
                                    -1
                                },
                                column = if (sect.args.isNotEmpty()) {
                                    getColumn(sect.args[0])
                                } else {
                                    -1
                                }
                        ),
                        row = getRow(sect),
                        column = getColumn(sect)
                ))
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
        ValidationFailure(errors)
    } else {
        ValidationSuccess(SourceSection(
                items = items,
                row = getRow(section),
                column = getColumn(section)))
    }
}

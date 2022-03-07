/*
 * Copyright 2019 The MathLingua Authors
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

import mathlingua.frontend.chalktalk.phase1.ast.Argument
import mathlingua.frontend.chalktalk.phase1.ast.ChalkTalkTokenType
import mathlingua.frontend.chalktalk.phase1.ast.Group
import mathlingua.frontend.chalktalk.phase1.ast.Phase1Token
import mathlingua.frontend.chalktalk.phase1.ast.Section
import mathlingua.frontend.chalktalk.phase2.CodeWriter
import mathlingua.frontend.chalktalk.phase2.ast.DEFAULT_META_DATA_ITEM
import mathlingua.frontend.chalktalk.phase2.ast.common.Phase2Node
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.metadata.isSingleSectionGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.metadata.item.MetaDataItem
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.metadata.item.StringSectionGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.metadata.item.isRelatedGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.metadata.item.isResourcesGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.metadata.item.isTopicsGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.metadata.item.validateRelatedGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.metadata.item.validateResourcesGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.metadata.item.validateTopicsGroup
import mathlingua.frontend.support.ParseError

private val META_DATA_ITEM_CONSTRAINTS =
    mapOf("note" to -1, "tag" to -1, "author" to -1, "contributor" to -1, "id" to 1)

internal data class MetaDataSection(
    val items: List<MetaDataItem>, override val row: Int, override val column: Int
) : Phase2Node {
    override fun forEach(fn: (node: Phase2Node) -> Unit) = items.forEach(fn)

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter): CodeWriter {
        writer.writeIndent(isArg, indent)
        writer.writeHorizontalLine()
        writer.writeHeader("Metadata")
        for (i in items.indices) {
            writer.writeNewline()
            writer.append(items[i], true, indent + 2)
        }
        return writer
    }

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) =
        chalkTransformer(
            MetaDataSection(
                items = items.map { it.transform(chalkTransformer) as MetaDataItem }, row, column))
}

internal fun isMetadataSection(sec: Section) = sec.name.text == "Metadata"

internal fun validateMetaDataSection(
    section: Section, errors: MutableList<ParseError>
): MetaDataSection {
    val items = mutableListOf<MetaDataItem>()
    for (arg in section.args) {
        if (isResourcesGroup(arg.chalkTalkTarget)) {
            items.add(validateMetaDataItem(arg, errors))
        } else if (isTopicsGroup(arg.chalkTalkTarget)) {
            items.add(validateTopicsGroup(arg, errors))
        } else if (isRelatedGroup(arg.chalkTalkTarget)) {
            items.add(validateRelatedGroup(arg, errors))
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
                            row = sect.row,
                            column = sect.column))
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
                                row = a.chalkTalkTarget.row,
                                column = a.chalkTalkTarget.column))
                    }
                }
                items.add(
                    StringSectionGroup(
                        section = StringSection(name = name, values = values, arg.row, arg.column),
                        arg.row,
                        arg.column))
            } else {
                errors.add(
                    ParseError(
                        message =
                            "Expected a section with one of " +
                                "the names ${META_DATA_ITEM_CONSTRAINTS.keys}",
                        row = arg.row,
                        column = arg.column))
            }
        } else {
            errors.add(
                ParseError(
                    message = "Unexpected item '${arg.toCode()}'",
                    row = arg.row,
                    column = arg.column))
        }
    }
    return MetaDataSection(items = items, section.row, section.column)
}

private fun validateMetaDataItem(arg: Argument, errors: MutableList<ParseError>): MetaDataItem =
    if (arg.chalkTalkTarget !is Group) {
        errors.add(ParseError(message = "Expected a group", row = arg.row, column = arg.column))
        DEFAULT_META_DATA_ITEM
    } else {
        validateResourcesGroup(arg.chalkTalkTarget, errors)
    }

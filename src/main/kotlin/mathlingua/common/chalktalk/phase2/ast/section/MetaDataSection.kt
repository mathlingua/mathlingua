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

import mathlingua.common.*
import mathlingua.common.chalktalk.phase1.ast.*
import mathlingua.common.chalktalk.phase2.CodeWriter
import mathlingua.common.chalktalk.phase2.ast.Phase2Node
import mathlingua.common.chalktalk.phase2.ast.clause.firstSectionMatchesName

val META_DATA_ITEM_CONSTRAINTS = mapOf(
        "name" to -1,
        "classification" to -1,
        "tag" to -1,
        "author" to -1,
        "contributor" to -1,
        "written" to -1,
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

fun isSingleSectionGroup(node: Phase1Node): Boolean {
    if (node !is Group) {
        return false
    }
    return node.sections.size == 1
}

fun validateMetaDataItem(arg: Argument, tracker: MutableLocationTracker): Validation<MetaDataItem> {
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

sealed class MetaDataItem : Phase2Node

data class ReferenceGroup(val referenceSection: ReferenceSection) : MetaDataItem() {
    override fun forEach(fn: (node: Phase2Node) -> Unit) = fn(referenceSection)

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter) =
            referenceSection.toCode(isArg, indent, writer)

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) =
            chalkTransformer(ReferenceGroup(
                    referenceSection = chalkTransformer(referenceSection) as ReferenceSection
            ))
}

fun isReferenceGroup(node: Phase1Node) = firstSectionMatchesName(node, "reference")

fun validateReferenceGroup(groupNode: Group, tracker: MutableLocationTracker): Validation<MetaDataItem> {
    val errors = ArrayList<ParseError>()
    val group = groupNode.resolve()
    if (group.id != null) {
        errors.add(
                ParseError(
                        "A reference cannot have an Id",
                        getRow(group), getColumn(group)
                )
        )
    }

    val sections = group.sections

    val sectionMap: Map<String, List<Section>>
    try {
        sectionMap = identifySections(
                sections, "reference"
        )
    } catch (e: ParseError) {
        errors.add(ParseError(e.message, e.row, e.column))
        return validationFailure(errors)
    }

    val rawReference = sectionMap["reference"]!![0]
    var referenceSection: ReferenceSection? = null
    when (val validation = validateReferenceSection(rawReference, tracker)) {
        is ValidationSuccess -> referenceSection = validation.value
        is ValidationFailure -> errors.addAll(validation.errors)
    }

    return if (errors.isNotEmpty()) {
        validationFailure(errors)
    } else {
        validationSuccess(
                tracker,
                groupNode,
                ReferenceGroup(
                        referenceSection = referenceSection!!
                )
        )
    }
}

data class ReferenceSection(val sourceItems: List<SourceItemGroup>) : Phase2Node {
    override fun forEach(fn: (node: Phase2Node) -> Unit) = sourceItems.forEach(fn)

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter): CodeWriter {
        writer.writeIndent(isArg, indent)
        writer.writeHeader("reference")
        for (sourceItem in sourceItems) {
            writer.writeNewline()
            writer.append(sourceItem, true, indent + 2)
        }
        return writer
    }

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) =
            chalkTransformer(ReferenceSection(
                    sourceItems = sourceItems.map { chalkTransformer(it) as SourceItemGroup }
            ))
}

private fun validateReferenceSection(rawNode: Phase1Node, tracker: MutableLocationTracker): Validation<ReferenceSection> {
    val node = rawNode.resolve()
    val errors = ArrayList<ParseError>()
    if (node !is Section) {
        errors.add(
                ParseError(
                        "Expected a Section",
                        getRow(node), getColumn(node)
                )
        )
    }

    val (name, args) = node as Section
    if (name.text != "reference") {
        errors.add(
                ParseError(
                        "Expected a Section with name 'reference' but found " + name.text,
                        getRow(node), getColumn(node)
                )
        )
    }

    if (args.isEmpty()) {
        errors.add(
                ParseError(
                        "Section '" + name.text + "' requires at least one 'source' argument.",
                        getRow(node), getColumn(node)
                )
        )
    }

    val sourceItems = mutableListOf<SourceItemGroup>()
    for (arg in args) {
        if (arg.chalkTalkTarget is Group) {
            when (val validation = validateSourceItemGroup(arg.chalkTalkTarget, tracker)) {
                is ValidationSuccess -> {
                    sourceItems.add(validation.value)
                }
                is ValidationFailure -> errors.addAll(validation.errors)
            }
        } else {
            errors.add(
                    ParseError(
                            message = "Expected a 'source' group but found ${arg.toCode()}",
                            row = getRow(arg),
                            column = getColumn(arg)
                    )
            )
        }
    }

    return if (errors.isNotEmpty()) {
        validationFailure(errors)
    } else validationSuccess(
            tracker,
            rawNode,
            ReferenceSection(sourceItems = sourceItems)
    )
}

data class SourceItemGroup(
    val sourceSection: SourceItemSection,
    val pageSection: PageItemSection?,
    val offsetSection: OffsetItemSection?,
    val contentSection: ContentItemSection?
) : MetaDataItem() {
    override fun forEach(fn: (node: Phase2Node) -> Unit) {
        fn(sourceSection)

        if (pageSection != null) {
            fn(pageSection)
        }

        if (offsetSection != null) {
            fn(offsetSection)
        }

        if (contentSection != null) {
            fn(contentSection)
        }
    }

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter) = mathlingua.common.chalktalk.phase2.ast.toplevel.toCode(
            writer,
            isArg,
            indent,
            null,
            sourceSection,
            pageSection,
            offsetSection,
            contentSection
    )

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) =
            chalkTransformer(SourceItemGroup(
                    sourceSection = sourceSection.transform(chalkTransformer) as SourceItemSection,
                    pageSection = pageSection?.transform(chalkTransformer) as PageItemSection,
                    offsetSection = offsetSection?.transform(chalkTransformer) as OffsetItemSection,
                    contentSection = contentSection?.transform(chalkTransformer) as ContentItemSection
            ))
}

fun isSourceItemGroup(node: Phase1Node) = firstSectionMatchesName(node, "source")

fun validateSourceItemGroup(groupNode: Group, tracker: MutableLocationTracker): Validation<SourceItemGroup> {
    val errors = ArrayList<ParseError>()
    val group = groupNode.resolve()
    if (group.id != null) {
        errors.add(
                ParseError(
                        "A reference source cannot have an Id",
                        getRow(group), getColumn(group)
                )
        )
    }

    val sections = group.sections

    val sectionMap: Map<String, List<Section>>
    try {
        sectionMap = identifySections(
                sections, "source", "page?", "offset?", "content?"
        )
    } catch (e: ParseError) {
        errors.add(ParseError(e.message, e.row, e.column))
        return validationFailure(errors)
    }

    val rawSource = sectionMap["source"]!![0]
    var sourceSection: SourceItemSection? = null
    when (val validation = validateStringSection(tracker, rawSource, "source", ::SourceItemSection)) {
        is ValidationSuccess -> sourceSection = validation.value
        is ValidationFailure -> errors.addAll(validation.errors)
    }

    val rawPage = sectionMap["page"]
    var pageSection: PageItemSection? = null
    if (rawPage != null && rawPage.isNotEmpty()) {
        when (val validation = validateStringSection(tracker, rawPage[0], "page", ::PageItemSection)) {
            is ValidationSuccess -> pageSection = validation.value
            is ValidationFailure -> errors.addAll(validation.errors)
        }
    }

    val rawOffset = sectionMap["offset"]
    var offsetSection: OffsetItemSection? = null
    if (rawOffset != null && rawOffset.isNotEmpty()) {
        when (val validation = validateStringSection(tracker, rawOffset[0], "offset", ::OffsetItemSection)) {
            is ValidationSuccess -> offsetSection = validation.value
            is ValidationFailure -> errors.addAll(validation.errors)
        }
    }

    val rawContent = sectionMap["content"]
    var contentSection: ContentItemSection? = null
    if (rawContent != null && rawContent.isNotEmpty()) {
        when (val validation = validateStringSection(tracker, rawContent[0], "content", ::ContentItemSection)) {
            is ValidationSuccess -> contentSection = validation.value
            is ValidationFailure -> errors.addAll(validation.errors)
        }
    }

    return if (errors.isNotEmpty()) {
        validationFailure(errors)
    } else {
        validationSuccess(
                tracker,
                groupNode,
                SourceItemGroup(
                        sourceSection = sourceSection!!,
                        pageSection = pageSection,
                        offsetSection = offsetSection,
                        contentSection = contentSection)
                )
    }
}

data class SourceItemSection(val sourceReference: String) : Phase2Node {
    override fun forEach(fn: (node: Phase2Node) -> Unit) {}

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter) =
            indentedStringSection(writer, isArg, indent, "source", sourceReference)

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node): Phase2Node {
        return chalkTransformer(this)
    }
}

data class PageItemSection(val page: String) : Phase2Node {
    override fun forEach(fn: (node: Phase2Node) -> Unit) {}

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter) =
            indentedStringSection(writer, isArg, indent, "page", page)

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node): Phase2Node {
        return chalkTransformer(this)
    }
}

data class OffsetItemSection(val offset: String) : Phase2Node {
    override fun forEach(fn: (node: Phase2Node) -> Unit) {}

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter) =
            indentedStringSection(writer, isArg, indent, "offset", offset)

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node): Phase2Node {
        return chalkTransformer(this)
    }
}

data class ContentItemSection(val content: String) : Phase2Node {
    override fun forEach(fn: (node: Phase2Node) -> Unit) {}

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter) =
            indentedStringSection(writer, isArg, indent, "content", content)

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node): Phase2Node {
        return chalkTransformer(this)
    }
}

private fun indentedStringSection(writer: CodeWriter, isArg: Boolean, indent: Int, sectionName: String, value: String): CodeWriter {
    writer.writeIndent(isArg, indent)
    writer.writeHeader(sectionName)
    writer.writeSpace()
    writer.writeText(value.removeSurrounding("\"", "\""))
    return writer
}

private fun <T : Phase2Node> validateStringSection(
    tracker: MutableLocationTracker,
    rawNode: Phase1Node,
    expectedName: String,
    fn: (text: String) -> T
): Validation<T> {
    val node = rawNode.resolve()
    val errors = ArrayList<ParseError>()
    if (node !is Section) {
        errors.add(
                ParseError(
                        "Expected a Section",
                        getRow(node), getColumn(node)
                )
        )
    }

    val (name, args) = node as Section
    if (name.text != expectedName) {
        errors.add(
                ParseError(
                        "Expected a Section with name " +
                                expectedName + " but found " + name.text,
                        getRow(node), getColumn(node)
                )
        )
    }

    if (args.size != 1 ||
            args[0].chalkTalkTarget !is Phase1Token ||
            (args[0].chalkTalkTarget as Phase1Token).type != ChalkTalkTokenType.String) {
        errors.add(
                ParseError(
                        "Section '" + name.text + "' requires a single string argument.",
                        getRow(node), getColumn(node)
                )
        )
    }

    val token = args[0].chalkTalkTarget as Phase1Token
    return if (errors.isNotEmpty()) {
        validationFailure(errors)
    } else validationSuccess(tracker, rawNode, fn(token.text))
}

class StringSectionGroup(val section: StringSection) : MetaDataItem() {
    override fun forEach(fn: (node: Phase2Node) -> Unit) = fn(section)

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter) = section.toCode(isArg, indent, writer)

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) =
            chalkTransformer(
                    StringSectionGroup(
                            section = chalkTransformer(section) as StringSection
                    )
            )
}

class StringSection(
    val name: String,
    val values: List<String>
) : Phase2Node {
    override fun forEach(fn: (node: Phase2Node) -> Unit) {}

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter): CodeWriter {
        writer.writeIndent(isArg, indent)
        writer.writeHeader(name)
        if (values.size == 1) {
            writer.writeSpace()
            writer.writeDirect(values[0])
        } else {
            for (value in values) {
                writer.writeNewline()
                writer.writeIndent(true, indent + 2)
                writer.writeDirect(value)
            }
        }
        return writer
    }

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) =
            chalkTransformer(this)
}

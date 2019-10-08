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

data class MetaDataSection(
        val mappings: List<MappingNode>,
        val items: List<MetaDataItem>,
        override var row: Int,
        override var column: Int
) : Phase2Node {
    override fun forEach(fn: (node: Phase2Node) -> Unit) {
        mappings.forEach(fn)
        items.forEach(fn)
    }

    override fun toCode(isArg: Boolean, indent: Int): String {
        val builder = StringBuilder()
        for (i in mappings.indices) {
            builder.append(mappings[i].toCode(true, indent + 2))
            if (i != mappings.size - 1) {
                builder.append('\n')
            }
        }
        for (i in items.indices) {
            builder.append(items[i].toCode(true, indent))
            if (i != items.size - 1) {
                builder.append('\n')
            }
        }
        return builder.toString()
    }

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) =
            chalkTransformer(MetaDataSection(
                    mappings = mappings.map { it.transform(chalkTransformer) as MappingNode },
                    items = items.map { it.transform(chalkTransformer) as MetaDataItem },
                    row = row,
                    column = column
            ))
}

fun validateMetaDataSection(section: Section): Validation<MetaDataSection> {
    if (section.name.text != "Metadata") {
        return ValidationFailure(
                listOf(
                        ParseError(
                                "Expected a 'Metadata' but found '${section.name.text}'",
                                getRow(section), getColumn(section)
                        )
                )
        )
    }

    val errors = mutableListOf<ParseError>()
    val mappings = mutableListOf<MappingNode>()
    val items = mutableListOf<MetaDataItem>()
    for (arg in section.args) {
        if (isReferenceGroup(arg.chalkTalkTarget)) {
            when (val validation = validateMetaDataItem(arg)) {
                is ValidationSuccess -> items.add(validation.value)
                is ValidationFailure -> errors.addAll(validation.errors)
            }
        } else {
            when (val validation = validateMappingNode(arg)) {
                is ValidationSuccess -> mappings.add(validation.value)
                is ValidationFailure -> errors.addAll(validation.errors)
            }
        }
    }

    return if (errors.isNotEmpty()) {
        ValidationFailure(errors)
    } else {
        ValidationSuccess(MetaDataSection(
                mappings = mappings,
                items = items,
                row = getRow(section),
                column = getColumn(section)))
    }
}

fun validateMetaDataItem(arg: Argument): Validation<MetaDataItem> {
    if (arg.chalkTalkTarget !is Group) {
        return ValidationFailure(listOf(
                ParseError(
                        message = "Expected a group",
                        row = getRow(arg),
                        column = getColumn(arg)
                )
        ))
    }

    return validateReferenceGroup(arg.chalkTalkTarget)
}

sealed class MetaDataItem : Phase2Node

data class ReferenceGroup(
        val sourceSection: SourceItemSection,
        val pageSection: PageItemSection?,
        val offsetSection: OffsetItemSection?,
        val contentSection: ContentItemSection?,
        override var row: Int,
        override var column: Int
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

    override fun toCode(isArg: Boolean, indent: Int) = toCode(
            isArg,
            indent,
            null,
            sourceSection,
            pageSection,
            offsetSection,
            contentSection
    )

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) =
            chalkTransformer(ReferenceGroup(
                    sourceSection = sourceSection.transform(chalkTransformer) as SourceItemSection,
                    pageSection = pageSection?.transform(chalkTransformer) as PageItemSection,
                    offsetSection = offsetSection?.transform(chalkTransformer) as OffsetItemSection,
                    contentSection = contentSection?.transform(chalkTransformer) as ContentItemSection,
                    row = row,
                    column = column
            ))
}

fun isReferenceGroup(node: Phase1Node): Boolean {
    val result = firstSectionMatchesName(node, "reference")
    println("Is ${node.toCode()} a reference group? " + result)
    return result
}

fun validateReferenceGroup(groupNode: Group): Validation<ReferenceGroup> {
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

    val sectionMap: Map<String, Section?>
    try {
        sectionMap = identifySections(
                sections, "source", "page?", "offset?", "content?"
        )
    } catch (e: ParseError) {
        errors.add(ParseError(e.message, e.row, e.column))
        return ValidationFailure(errors)
    }

    val rawSource = sectionMap["source"]!!;
    var sourceSection: SourceItemSection? = null
    when (val validation = validateStringSection(rawSource, "source", ::SourceItemSection)) {
        is ValidationSuccess -> sourceSection = validation.value
        is ValidationFailure -> errors.addAll(validation.errors)
    }

    val rawPage = sectionMap["page"]
    var pageSection: PageItemSection? = null
    if (rawPage != null) {
        when (val validation = validateStringSection(rawPage, "page", ::PageItemSection)) {
            is ValidationSuccess -> pageSection = validation.value
            is ValidationFailure -> errors.addAll(validation.errors)
        }
    }

    val rawOffset = sectionMap["offset"]
    var offsetSection: OffsetItemSection? = null
    if (rawOffset != null) {
        when (val validation = validateStringSection(rawOffset, "offset", ::OffsetItemSection)) {
            is ValidationSuccess -> offsetSection = validation.value
            is ValidationFailure -> errors.addAll(validation.errors)
        }
    }

    val rawContent = sectionMap["content"]
    var contentSection: ContentItemSection? = null
    if (rawContent != null) {
        when (val validation = validateStringSection(rawContent, "content", ::ContentItemSection)) {
            is ValidationSuccess -> contentSection = validation.value
            is ValidationFailure -> errors.addAll(validation.errors)
        }
    }

    return if (errors.isNotEmpty()) {
        ValidationFailure(errors)
    } else {
        ValidationSuccess(
                ReferenceGroup(
                    sourceSection = sourceSection!!,
                    pageSection = pageSection,
                    offsetSection = offsetSection,
                    contentSection = contentSection,
                    row = sourceSection.row,
                    column = sourceSection.column)
                )
    }
}

data class SourceItemSection(
        val sourceReference: String,
        override var row: Int,
        override var column: Int
) : Phase2Node {
    override fun forEach(fn: (node: Phase2Node) -> Unit) {}

    override fun toCode(isArg: Boolean, indent: Int) =
            indentedStringSection(isArg, indent, "source", sourceReference)

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node): Phase2Node {
        return chalkTransformer(this)
    }
}

data class PageItemSection(
        val page: String,
        override var row: Int,
        override var column: Int
) : Phase2Node {
    override fun forEach(fn: (node: Phase2Node) -> Unit) {}

    override fun toCode(isArg: Boolean, indent: Int) =
            indentedStringSection(isArg, indent, "page", page)

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node): Phase2Node {
        return chalkTransformer(this)
    }
}

data class OffsetItemSection(
        val offset: String,
        override var row: Int,
        override var column: Int
) : Phase2Node {
    override fun forEach(fn: (node: Phase2Node) -> Unit) {}

    override fun toCode(isArg: Boolean, indent: Int) =
            indentedStringSection(isArg, indent, "offset", offset)

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node): Phase2Node {
        return chalkTransformer(this)
    }
}

data class ContentItemSection(
        val content: String,
        override var row: Int,
        override var column: Int
) : Phase2Node {
    override fun forEach(fn: (node: Phase2Node) -> Unit) {}

    override fun toCode(isArg: Boolean, indent: Int) =
            indentedStringSection(isArg, indent, "content", content)

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node): Phase2Node {
        return chalkTransformer(this)
    }
}

private fun indentedStringSection(isArg: Boolean, indent: Int, sectionName: String, value: String): String {
    return indentedString(isArg, indent, "$sectionName: \"$value\"")
}

private fun <T> validateStringSection(rawNode: Phase1Node,
                                      expectedName: String,
                                      fn: (text: String,
                                           row: Int,
                                           column: Int) -> T): Validation<T> {
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
        ValidationFailure(errors)
    } else ValidationSuccess(fn(token.text, token.row, token.column))
}


/*

import mathlingua.common.ParseError
import mathlingua.common.Validation
import mathlingua.common.ValidationFailure
import mathlingua.common.ValidationSuccess
import mathlingua.common.chalktalk.phase1.ast.Section
import mathlingua.common.chalktalk.phase1.ast.getColumn
import mathlingua.common.chalktalk.phase1.ast.getRow

data class MetaDataSection(
    val mappings: List<MappingNode>,
    override var row: Int,
    override var column: Int
) : Phase2Node {
    override fun forEach(fn: (node: Phase2Node) -> Unit) = mappings.forEach(fn)

    override fun toCode(isArg: Boolean, indent: Int): String {
        val builder = StringBuilder()
        builder.append(indentedString(isArg, indent, "Metadata:"))
        builder.append('\n')
        for (i in mappings.indices) {
            builder.append(mappings[i].toCode(true, indent + 2))
            if (i != mappings.size - 1) {
                builder.append('\n')
            }
        }
        return builder.toString()
    }

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) =
        chalkTransformer(MetaDataSection(
            mappings = mappings.map { it.transform(chalkTransformer) as MappingNode },
            row = row,
            column = column
        ))
}

fun validateMetaDataSection(section: Section): Validation<MetaDataSection> {
    if (section.name.text != "Metadata") {
        return ValidationFailure(
            listOf(
                ParseError(
                    "Expected a 'Metadata' but found '${section.name.text}'",
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
        ValidationSuccess(MetaDataSection(
                mappings = mappings,
                row = getRow(section),
                column = getColumn(section)))
    }
}
*/

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

package mathlingua.common.chalktalk.phase2.ast.toplevel

import mathlingua.common.*
import mathlingua.common.chalktalk.phase1.ast.Group
import mathlingua.common.chalktalk.phase1.ast.Phase1Node
import mathlingua.common.chalktalk.phase1.ast.Phase1Token
import mathlingua.common.chalktalk.phase1.ast.Section
import mathlingua.common.chalktalk.phase1.ast.getColumn
import mathlingua.common.chalktalk.phase1.ast.getRow
import mathlingua.common.chalktalk.phase2.CodeWriter
import mathlingua.common.chalktalk.phase2.ast.Phase2Node
import mathlingua.common.chalktalk.phase2.ast.clause.IdStatement
import mathlingua.common.chalktalk.phase2.ast.section.*
import mathlingua.common.chalktalk.phase2.ast.metadata.section.MetaDataSection
import mathlingua.common.chalktalk.phase2.ast.metadata.section.validateMetaDataSection

abstract class TopLevelGroup(open val metaDataSection: MetaDataSection?) : Phase2Node

fun topLevelToCode(writer: CodeWriter, isArg: Boolean, indent: Int, id: IdStatement?, vararg sections: Phase2Node?): CodeWriter {
    writer.beginTopLevel()
    var useAsArg = isArg
    if (id != null) {
        writer.writeIndent(isArg, indent)
        writer.writeId(id)
        writer.writeNewline()
        useAsArg = false
    }

    val nonNullSections = sections.filterNotNull()
    for (i in nonNullSections.indices) {
        val sect = nonNullSections[i]
        writer.append(sect, useAsArg, indent)
        useAsArg = false
        if (i != nonNullSections.size - 1) {
            writer.writeNewline()
        }
    }
    writer.endTopLevel()

    return writer
}

fun <G : Phase2Node, S> validateResultLikeGroup(
    tracker: MutableLocationTracker,
    groupNode: Group,
    resultLikeName: String,
    validateResultLikeSection: (section: Section, tracker: MutableLocationTracker) -> Validation<S>,
    buildGroup: (
        sect: S,
        givenSection: GivenSection?,
        givenWhereSection: WhereSection?,
        thenSection: ThenSection,
        using: UsingSection?,
        usingWhere: WhereSection?,
        metadata: MetaDataSection?
    ) -> G
): Validation<G> {
    val errors = ArrayList<ParseError>()
    val group = groupNode.resolve()
    if (group.id != null) {
        errors.add(
            ParseError(
                "A result, axiom, or conjecture cannot have an Id",
                getRow(group), getColumn(group)
            )
        )
    }

    val sections = group.sections

    val sectionMap: Map<String, List<Section>>
    try {
        sectionMap = identifySections(
                sections, resultLikeName, "given?", "where?", "then",
                "using?", "where?", "Metadata?"
        )
    } catch (e: ParseError) {
        errors.add(ParseError(e.message, e.row, e.column))
        return validationFailure(errors)
    }

    val resultLike = sectionMap[resultLikeName]!!
    val given = sectionMap["given"] ?: emptyList()
    val givenWhere = sectionMap["where"] ?: emptyList()
    val then = sectionMap["then"] ?: emptyList()
    val using = sectionMap["using"] ?: emptyList()
    val usingWhere = sectionMap["where1"] ?: emptyList()
    val metadata = sectionMap["Metadata"] ?: emptyList()

    var resultLikeSection: S? = null
    when (val resultLikeValidation = validateResultLikeSection(resultLike[0], tracker)) {
        is ValidationSuccess -> resultLikeSection = resultLikeValidation.value
        is ValidationFailure -> errors.addAll(resultLikeValidation.errors)
    }

    var givenSection: GivenSection? = null
    if (given.isNotEmpty()) {
        when (val givenValidation = validateGivenSection(given[0], tracker)) {
            is ValidationSuccess -> givenSection = givenValidation.value
            is ValidationFailure -> errors.addAll(givenValidation.errors)
        }
    }

    var givenWhereSection: WhereSection? = null
    if (givenWhere.isNotEmpty()) {
        when (val givenWhereValidation = validateWhereSection(givenWhere[0], tracker)) {
            is ValidationSuccess -> givenWhereSection = givenWhereValidation.value
            is ValidationFailure -> errors.addAll(givenWhereValidation.errors)
        }
    }

    var thenSection: ThenSection? = null
    when (val thenValidation = validateThenSection(then[0], tracker)) {
        is ValidationSuccess -> thenSection = thenValidation.value
        is ValidationFailure -> errors.addAll(thenValidation.errors)
    }

    var metaDataSection: MetaDataSection? = null
    if (metadata.isNotEmpty()) {
        when (val metaDataValidation = validateMetaDataSection(metadata[0], tracker)) {
            is ValidationSuccess -> metaDataSection = metaDataValidation.value
            is ValidationFailure -> errors.addAll(metaDataValidation.errors)
        }
    }

    var usingSection: UsingSection? = null
    if (using.isNotEmpty()) {
        when (val aliasValidation = validateUsingSection(using[0], tracker)) {
            is ValidationSuccess -> usingSection = aliasValidation.value
            is ValidationFailure -> errors.addAll(aliasValidation.errors)
        }
    }

    var usingWhereSection: WhereSection? = null
    if (usingWhere.isNotEmpty()) {
        when (val usingWhereValidation = validateWhereSection(usingWhere[0], tracker)) {
            is ValidationSuccess -> usingWhereSection = usingWhereValidation.value
            is ValidationFailure -> errors.addAll(usingWhereValidation.errors)
        }
    }

    return if (errors.isNotEmpty()) {
        validationFailure(errors)
    } else validationSuccess(
            tracker,
            groupNode,
            buildGroup(
                resultLikeSection!!,
                givenSection,
                givenWhereSection,
                thenSection!!,
                usingSection,
                usingWhereSection,
                metaDataSection
            ))
}

fun <G : Phase2Node, S> validateProtoResultLikeGroup(
    tracker: MutableLocationTracker,
    groupNode: Group,
    resultLikeName: String,
    validateResultLikeSection: (section: Section, tracker: MutableLocationTracker) -> Validation<S>,
    buildGroup: (
        sect: S,
        metadata: MetaDataSection?
    ) -> G
): Validation<G> {
    val errors = ArrayList<ParseError>()
    val group = groupNode.resolve()
    if (group.id != null) {
        errors.add(
            ParseError(
                "A result, axiom, or conjecture cannot have an Id",
                getRow(group), getColumn(group)
            )
        )
    }

    val sections = group.sections

    val sectionMap: Map<String, List<Section>>
    try {
        sectionMap = identifySections(
            sections, resultLikeName, "Metadata?"
        )
    } catch (e: ParseError) {
        errors.add(ParseError(e.message, e.row, e.column))
        return validationFailure(errors)
    }

    val resultLike = sectionMap[resultLikeName]!!
    val metadata = sectionMap["Metadata"] ?: emptyList()

    var resultLikeSection: S? = null
    when (val resultLikeValidation = validateResultLikeSection(resultLike[0], tracker)) {
        is ValidationSuccess -> resultLikeSection = resultLikeValidation.value
        is ValidationFailure -> errors.addAll(resultLikeValidation.errors)
    }

    var metaDataSection: MetaDataSection? = null
    if (metadata.isNotEmpty()) {
        when (val metaDataValidation = validateMetaDataSection(metadata[0], tracker)) {
            is ValidationSuccess -> metaDataSection = metaDataValidation.value
            is ValidationFailure -> errors.addAll(metaDataValidation.errors)
        }
    }

    return if (errors.isNotEmpty()) {
        validationFailure(errors)
    } else validationSuccess(
        tracker,
        groupNode,
        buildGroup(
            resultLikeSection!!,
            metaDataSection
        ))
}

fun <S : Phase2Node> validateTextListSection(
    rawNode: Phase1Node,
    tracker: MutableLocationTracker,
    sectionName: String,
    buildSection: (texts: List<String>) -> S
): Validation<S> {
    val node = rawNode.resolve()
    val row = getRow(node)
    val column = getColumn(node)

    val errors = ArrayList<ParseError>()
    if (node !is Section) {
        errors.add(
            ParseError(
                "Expected a Section",
                getRow(node), getColumn(node)
            )
        )
    }

    val sect = node as Section
    if (sect.name.text != sectionName) {
        errors.add(
            ParseError(
                "Expected a Section with name '$sectionName' but found " + sect.name.text,
                row, column
            )
        )
    }

    val texts = mutableListOf<String>()
    for (arg in sect.args) {
        if (arg.chalkTalkTarget !is Phase1Token) {
            errors.add(
                ParseError(
                    "Expected a string but found ${arg.toCode()}",
                    row, column
                )
            )
        }
        texts.add((arg.chalkTalkTarget as Phase1Token).text)
    }

    return if (errors.isNotEmpty()) {
        validationFailure(errors)
    } else {
        validationSuccess(
            tracker,
            rawNode,
            buildSection(texts)
        )
    }
}

fun <G : Phase2Node, S1, S2, S3> validateTripleSectionMetaDataGroup(
    tracker: MutableLocationTracker,
    rawNode: Phase1Node,
    section1Name: String,
    validateSection1: (section: Section, tracker: MutableLocationTracker) -> Validation<S1>,
    section2Name: String,
    validateSection2: (section: Section, tracker: MutableLocationTracker) -> Validation<S2>,
    section3Name: String,
    validateSection3: (section: Section, tracker: MutableLocationTracker) -> Validation<S3>,
    buildGroup: (sect1: S1, sect2: S2, sect3: S3, metadata: MetaDataSection?) -> G
): Validation<G> {
    val node = rawNode.resolve()

    val errors = ArrayList<ParseError>()
    if (node !is Group) {
        errors.add(
            ParseError(
                "Expected a Group",
                getRow(node), getColumn(node)
            )
        )
        return validationFailure(errors)
    }

    val (sections) = node

    val sectionMap: Map<String, List<Section>>
    try {
        sectionMap = identifySections(
            sections, section1Name, section2Name, section3Name, "metadata?"
        )
    } catch (e: ParseError) {
        errors.add(ParseError(e.message, e.row, e.column))
        return validationFailure(errors)
    }

    var section1: S1? = null
    val sect1 = sectionMap[section1Name]
    when (val section1Validation = validateSection1(sect1!![0], tracker)) {
        is ValidationSuccess -> section1 = section1Validation.value
        is ValidationFailure -> errors.addAll(section1Validation.errors)
    }

    var section2: S2? = null
    val sect2 = sectionMap[section2Name]
    when (val section2Validation = validateSection2(sect2!![0], tracker)) {
        is ValidationSuccess -> section2 = section2Validation.value
        is ValidationFailure -> errors.addAll(section2Validation.errors)
    }

    var section3: S3? = null
    val sect3 = sectionMap[section3Name]
    when (val section3Validation = validateSection3(sect3!![0], tracker)) {
        is ValidationSuccess -> section3 = section3Validation.value
        is ValidationFailure -> errors.addAll(section3Validation.errors)
    }

    var metadataSection: MetaDataSection? = null
    val metadata = sectionMap["metadata"]
    when (val metadataValidation = validateMetaDataSection(metadata!![0], tracker)) {
        is ValidationSuccess -> metadataSection = metadataValidation.value
        is ValidationFailure -> errors.addAll(metadataValidation.errors)
    }

    return if (errors.isNotEmpty()) {
        validationFailure(errors)
    } else validationSuccess(
        tracker,
        rawNode,
        buildGroup(
            section1!!,
            section2!!,
            section3!!,
            metadataSection
        )
    )
}

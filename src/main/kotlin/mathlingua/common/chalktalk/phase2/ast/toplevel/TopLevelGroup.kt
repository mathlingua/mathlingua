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
import mathlingua.common.chalktalk.phase1.ast.Phase1Token
import mathlingua.common.chalktalk.phase1.ast.ChalkTalkTokenType
import mathlingua.common.chalktalk.phase1.ast.Group
import mathlingua.common.chalktalk.phase1.ast.Section
import mathlingua.common.chalktalk.phase1.ast.getColumn
import mathlingua.common.chalktalk.phase1.ast.getRow
import mathlingua.common.chalktalk.phase2.CodeWriter
import mathlingua.common.chalktalk.phase2.ast.Phase2Node
import mathlingua.common.chalktalk.phase2.ast.clause.IdStatement
import mathlingua.common.chalktalk.phase2.ast.clause.validateIdStatement
import mathlingua.common.chalktalk.phase2.ast.section.*
import mathlingua.common.chalktalk.phase2.ast.metadata.section.MetaDataSection
import mathlingua.common.chalktalk.phase2.ast.metadata.section.validateMetaDataSection
import mathlingua.common.transform.signature

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
        alias: AliasSection?,
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
                sections, resultLikeName, "Alias?", "Metadata?"
        )
    } catch (e: ParseError) {
        errors.add(ParseError(e.message, e.row, e.column))
        return validationFailure(errors)
    }

    val resultLike = sectionMap[resultLikeName]!!
    val alias = sectionMap["Alias"] ?: emptyList()
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

    var aliasSection: AliasSection? = null
    if (alias.isNotEmpty()) {
        when (val aliasValidation = validateAliasSection(alias[0], tracker)) {
            is ValidationSuccess -> aliasSection = aliasValidation.value
            is ValidationFailure -> errors.addAll(aliasValidation.errors)
        }
    }

    return if (errors.isNotEmpty()) {
        validationFailure(errors)
    } else validationSuccess(
            tracker,
            groupNode,
            buildGroup(
                resultLikeSection!!,
                aliasSection,
                metaDataSection
            ))
}

fun <G : Phase2Node, S, E> validateDefinesLikeGroup(
    tracker: MutableLocationTracker,
    groupNode: Group,
    definesLikeSectionName: String,
    validateDefinesLikeSection: (section: Section, tracker: MutableLocationTracker) -> Validation<S>,
    endSectionName: String,
    validateEndSection: (section: Section, tracker: MutableLocationTracker) -> Validation<E>,
    buildGroup: (
        signature: String?,
        id: IdStatement,
        definesLike: S,
        assuming: AssumingSection?,
        end: List<E>,
        alias: AliasSection?,
        metadata: MetaDataSection?
    ) -> G
): Validation<G> {
    val errors = ArrayList<ParseError>()
    val group = groupNode.resolve()
    var id: IdStatement? = null
    if (group.id != null) {
        val (rawText, _, row, column) = group.id
        // The id token is of type Id and the text is of the form "[...]"
        // Convert it to look like a statement.
        val statementText = "'" + rawText.substring(1, rawText.length - 1) + "'"
        val stmtToken = Phase1Token(
            statementText, ChalkTalkTokenType.Statement,
            row, column
        )
        when (val idValidation = validateIdStatement(stmtToken, tracker)) {
            is ValidationSuccess -> id = idValidation.value
            is ValidationFailure -> errors.addAll(idValidation.errors)
        }
    } else {
        val type = if (group.sections.isNotEmpty()) {
            group.sections.first().name.text
        } else {
            "Defines or Represents"
        }
        errors.add(
            ParseError(
                "A $type must have an Id",
                getRow(group), getColumn(group)
            )
        )
    }

    val sections = group.sections

    val sectionMap: Map<String, List<Section>>
    try {
        sectionMap = identifySections(
                sections,
                definesLikeSectionName, "assuming?", endSectionName, "Alias?", "Metadata?"
        )
    } catch (e: ParseError) {
        errors.add(ParseError(e.message, e.row, e.column))
        return validationFailure(errors)
    }

    val definesLike = sectionMap[definesLikeSectionName]!!
    val assuming = sectionMap["assuming"] ?: emptyList()
    val ends = sectionMap[endSectionName]!!
    val alias = sectionMap["Alias"] ?: emptyList()
    val metadata = sectionMap["Metadata"] ?: emptyList()

    var definesLikeSection: S? = null
    when (val definesLikeValidation = validateDefinesLikeSection(definesLike[0], tracker)) {
        is ValidationSuccess -> definesLikeSection = definesLikeValidation.value
        is ValidationFailure -> errors.addAll(definesLikeValidation.errors)
    }

    var assumingSection: AssumingSection? = null
    if (assuming.isNotEmpty()) {
        when (val assumingValidation = validateAssumingSection(assuming[0], tracker)) {
            is ValidationSuccess -> assumingSection = assumingValidation.value
            is ValidationFailure -> errors.addAll(assumingValidation.errors)
        }
    }

    val endSections = mutableListOf<E>()
    for (end in ends) {
        when (val endValidation = validateEndSection(end, tracker)) {
            is ValidationSuccess -> endSections.add(endValidation.value)
            is ValidationFailure -> errors.addAll(endValidation.errors)
        }
    }

    var aliasSection: AliasSection? = null
    if (alias.isNotEmpty()) {
        when (val aliasValidation = validateAliasSection(alias[0], tracker)) {
            is ValidationSuccess -> aliasSection = aliasValidation.value
            is ValidationFailure -> errors.addAll(aliasValidation.errors)
        }
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
                id?.signature(),
                id!!, definesLikeSection!!,
                assumingSection,
                // the end sections are in reverse order so they
                // must be reversed here to be in the correct order
                endSections.reversed(),
                aliasSection, metaDataSection
            )
        )
}

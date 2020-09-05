/*
 * Copyright 2020 Google LLC
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

import mathlingua.common.MutableLocationTracker
import mathlingua.common.ParseError
import mathlingua.common.Validation
import mathlingua.common.ValidationFailure
import mathlingua.common.ValidationSuccess
import mathlingua.common.chalktalk.phase1.ast.ChalkTalkTokenType
import mathlingua.common.chalktalk.phase1.ast.Group
import mathlingua.common.chalktalk.phase1.ast.Phase1Node
import mathlingua.common.chalktalk.phase1.ast.Phase1Token
import mathlingua.common.chalktalk.phase1.ast.Section
import mathlingua.common.chalktalk.phase1.ast.getColumn
import mathlingua.common.chalktalk.phase1.ast.getRow
import mathlingua.common.chalktalk.phase2.CodeWriter
import mathlingua.common.chalktalk.phase2.ast.Phase2Node
import mathlingua.common.chalktalk.phase2.ast.clause.IdStatement
import mathlingua.common.chalktalk.phase2.ast.clause.firstSectionMatchesName
import mathlingua.common.chalktalk.phase2.ast.clause.validateIdStatement
import mathlingua.common.chalktalk.phase2.ast.section.*
import mathlingua.common.chalktalk.phase2.ast.metadata.section.MetaDataSection
import mathlingua.common.chalktalk.phase2.ast.metadata.section.validateMetaDataSection
import mathlingua.common.transform.signature
import mathlingua.common.validationFailure
import mathlingua.common.validationSuccess

data class RepresentsGroup(
    val signature: String?,
    val id: IdStatement,
    val representsSection: RepresentsSection,
    val whenSection: WhenSection?,
    val thatSections: List<ThatSection>,
    val usingSection: UsingSection?,
    val whereSection: WhereSection?,
    override val metaDataSection: MetaDataSection?
) : TopLevelGroup(metaDataSection) {

    override fun forEach(fn: (node: Phase2Node) -> Unit) {
        fn(id)
        fn(representsSection)
        if (whenSection != null) {
            fn(whenSection)
        }
        thatSections.forEach(fn)
        if (usingSection != null) {
            fn(usingSection)
        }
        if (whereSection != null) {
            fn(whereSection)
        }
        if (metaDataSection != null) {
            fn(metaDataSection)
        }
    }

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter): CodeWriter {
        val sections = mutableListOf(representsSection, whenSection)
        sections.addAll(thatSections)
        sections.add(usingSection)
        sections.add(whereSection)
        sections.add(metaDataSection)
        return topLevelToCode(
                writer,
                isArg,
                indent,
                id,
                *sections.toTypedArray()
        )
    }

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) = chalkTransformer(RepresentsGroup(
            signature = signature,
            id = id.transform(chalkTransformer) as IdStatement,
            representsSection = representsSection.transform(chalkTransformer) as RepresentsSection,
            whenSection = whenSection?.transform(chalkTransformer) as WhenSection?,
            thatSections = thatSections.map { chalkTransformer(it) as ThatSection },
            usingSection = usingSection?.transform(chalkTransformer) as UsingSection?,
            whereSection = whereSection?.transform(chalkTransformer) as WhereSection?,
            metaDataSection = metaDataSection?.transform(chalkTransformer) as MetaDataSection?
    ))
}

fun isRepresentsGroup(node: Phase1Node) = firstSectionMatchesName(node, "Represents")

fun validateRepresentsGroup(groupNode: Group, tracker: MutableLocationTracker): Validation<RepresentsGroup> {
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
            "Represents", "when?", "that", "using?", "where?", "Metadata?"
        )
    } catch (e: ParseError) {
        errors.add(ParseError(e.message, e.row, e.column))
        return validationFailure(errors)
    }

    val definesLike = sectionMap["Represents"]!!
    val whenNode = sectionMap["when"] ?: emptyList()
    val ends = sectionMap["that"]!!
    val using = sectionMap["using"] ?: emptyList()
    val where = sectionMap["where"] ?: emptyList()
    val metadata = sectionMap["Metadata"] ?: emptyList()

    var representsSection: RepresentsSection? = null
    when (val representsValidation = validateRepresentsSection(definesLike[0], tracker)) {
        is ValidationSuccess -> representsSection = representsValidation.value
        is ValidationFailure -> errors.addAll(representsValidation.errors)
    }

    var whenSection: WhenSection? = null
    if (whenNode.isNotEmpty()) {
        when (val assumingValidation = validateWhenSection(whenNode[0], tracker)) {
            is ValidationSuccess -> whenSection = assumingValidation.value
            is ValidationFailure -> errors.addAll(assumingValidation.errors)
        }
    }

    val endSections = mutableListOf<ThatSection>()
    for (end in ends) {
        when (val endValidation = validateThatSection(end, tracker)) {
            is ValidationSuccess -> endSections.add(endValidation.value)
            is ValidationFailure -> errors.addAll(endValidation.errors)
        }
    }

    var usingSection: UsingSection? = null
    if (using.isNotEmpty()) {
        when (val aliasValidation = validateUsingSection(using[0], tracker)) {
            is ValidationSuccess -> usingSection = aliasValidation.value
            is ValidationFailure -> errors.addAll(aliasValidation.errors)
        }
    }

    var whereSection: WhereSection? = null
    if (where.isNotEmpty()) {
        when (val whereValidation = validateWhereSection(where[0], tracker)) {
            is ValidationSuccess -> whereSection = whereValidation.value
            is ValidationFailure -> errors.addAll(whereValidation.errors)
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
        RepresentsGroup(
            id?.signature(),
            id!!, representsSection!!,
            whenSection,
            // the end sections are in reverse order so they
            // must be reversed here to be in the correct order
            endSections.reversed(),
            usingSection, whereSection, metaDataSection
        )
    )
}

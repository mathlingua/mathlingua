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

package mathlingua.common.chalktalk.phase2.ast.group.toplevel.defineslike.states

import mathlingua.common.support.MutableLocationTracker
import mathlingua.common.support.ParseError
import mathlingua.common.support.Validation
import mathlingua.common.support.ValidationFailure
import mathlingua.common.support.ValidationSuccess
import mathlingua.common.chalktalk.phase1.ast.ChalkTalkTokenType
import mathlingua.common.chalktalk.phase1.ast.Group
import mathlingua.common.chalktalk.phase1.ast.Phase1Node
import mathlingua.common.chalktalk.phase1.ast.Phase1Token
import mathlingua.common.chalktalk.phase1.ast.Section
import mathlingua.common.chalktalk.phase1.ast.getColumn
import mathlingua.common.chalktalk.phase1.ast.getRow
import mathlingua.common.chalktalk.phase2.CodeWriter
import mathlingua.common.chalktalk.phase2.ast.common.Phase2Node
import mathlingua.common.chalktalk.phase2.ast.clause.IdStatement
import mathlingua.common.chalktalk.phase2.ast.clause.firstSectionMatchesName
import mathlingua.common.chalktalk.phase2.ast.clause.validateIdStatement
import mathlingua.common.chalktalk.phase2.ast.section.*
import mathlingua.common.chalktalk.phase2.ast.group.toplevel.shared.metadata.section.MetaDataSection
import mathlingua.common.chalktalk.phase2.ast.group.toplevel.shared.metadata.section.validateMetaDataSection
import mathlingua.common.chalktalk.phase2.ast.group.toplevel.TopLevelGroup
import mathlingua.common.chalktalk.phase2.ast.group.toplevel.shared.UsingSection
import mathlingua.common.chalktalk.phase2.ast.group.toplevel.shared.WhenSection
import mathlingua.common.chalktalk.phase2.ast.group.toplevel.defineslike.WrittenSection
import mathlingua.common.chalktalk.phase2.ast.group.toplevel.defineslike.foundation.DefinesStatesOrViews
import mathlingua.common.chalktalk.phase2.ast.group.toplevel.shared.validateUsingSection
import mathlingua.common.chalktalk.phase2.ast.group.toplevel.shared.validateWhenSection
import mathlingua.common.chalktalk.phase2.ast.group.toplevel.defineslike.validateWrittenSection
import mathlingua.common.chalktalk.phase2.ast.group.toplevel.topLevelToCode
import mathlingua.common.transform.signature
import mathlingua.common.support.validationFailure
import mathlingua.common.support.validationSuccess

data class StatesGroup(
    val signature: String?,
    val id: IdStatement,
    val statesSection: StatesSection,
    val whenSection: WhenSection?,
    val thatSections: List<ThatSection>,
    val usingSection: UsingSection?,
    val writtenSection: WrittenSection?,
    override val metaDataSection: MetaDataSection?
) : TopLevelGroup(metaDataSection), DefinesStatesOrViews {

    override fun forEach(fn: (node: Phase2Node) -> Unit) {
        fn(id)
        fn(statesSection)
        if (whenSection != null) {
            fn(whenSection)
        }
        thatSections.forEach(fn)
        if (usingSection != null) {
            fn(usingSection)
        }
        if (writtenSection != null) {
            fn(writtenSection)
        }
        if (metaDataSection != null) {
            fn(metaDataSection)
        }
    }

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter): CodeWriter {
        val sections = mutableListOf(statesSection, whenSection)
        sections.addAll(thatSections)
        sections.add(usingSection)
        sections.add(writtenSection)
        sections.add(metaDataSection)
        return topLevelToCode(
                writer,
                isArg,
                indent,
                id,
                *sections.toTypedArray()
        )
    }

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) = chalkTransformer(
        StatesGroup(
            signature = signature,
            id = id.transform(chalkTransformer) as IdStatement,
            statesSection = statesSection.transform(chalkTransformer) as StatesSection,
            whenSection = whenSection?.transform(chalkTransformer) as WhenSection?,
            thatSections = thatSections.map { chalkTransformer(it) as ThatSection },
            usingSection = usingSection?.transform(chalkTransformer) as UsingSection?,
            writtenSection = writtenSection?.transform(chalkTransformer) as WrittenSection?,
            metaDataSection = metaDataSection?.transform(chalkTransformer) as MetaDataSection?
    )
    )
}

fun isStatesGroup(node: Phase1Node) = firstSectionMatchesName(node, "States")

fun validateStatesGroup(groupNode: Group, tracker: MutableLocationTracker): Validation<StatesGroup> {
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
            "States", "when?", "that", "using?", "where?", "written?", "Metadata?"
        )
    } catch (e: ParseError) {
        errors.add(ParseError(e.message, e.row, e.column))
        return validationFailure(errors)
    }

    val definesLike = sectionMap["States"]!!
    val whenNode = sectionMap["when"] ?: emptyList()
    val ends = sectionMap["that"]!!
    val using = sectionMap["using"] ?: emptyList()
    val written = sectionMap["written"] ?: emptyList()
    val metadata = sectionMap["Metadata"] ?: emptyList()

    var statesSection: StatesSection? = null
    when (val representsValidation = validateStatesSection(definesLike[0], tracker)) {
        is ValidationSuccess -> statesSection = representsValidation.value
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

    var writtenSection: WrittenSection? = null
    if (written.isNotEmpty()) {
        when (val writtenValidation = validateWrittenSection(written[0], tracker)) {
            is ValidationSuccess -> writtenSection = writtenValidation.value
            is ValidationFailure -> errors.addAll(writtenValidation.errors)
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
        StatesGroup(
            id?.signature(),
            id!!, statesSection!!,
            whenSection,
            // the end sections are in reverse order so they
            // must be reversed here to be in the correct order
            endSections.reversed(),
            usingSection,
            writtenSection,
            metaDataSection
        )
    )
}

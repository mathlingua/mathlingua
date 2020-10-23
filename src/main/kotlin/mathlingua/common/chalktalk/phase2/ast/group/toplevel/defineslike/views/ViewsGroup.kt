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

package mathlingua.common.chalktalk.phase2.ast.group.toplevel.defineslike.views

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
import mathlingua.common.chalktalk.phase2.ast.group.toplevel.defineslike.foundation.DefinesStatesOrViews
import mathlingua.common.chalktalk.phase2.ast.group.toplevel.shared.UsingSection
import mathlingua.common.chalktalk.phase2.ast.group.toplevel.shared.validateUsingSection
import mathlingua.common.chalktalk.phase2.ast.group.toplevel.topLevelToCode
import mathlingua.common.transform.signature
import mathlingua.common.support.validationFailure
import mathlingua.common.support.validationSuccess

data class ViewsGroup(
    val signature: String?,
    val id: IdStatement,
    val viewsSection: ViewsSection,
    val singleFromSection: SingleFromSection,
    val singleToSection: SingleToSection,
    val asSection: SingleAsSection,
    val usingSection: UsingSection?,
    override val metaDataSection: MetaDataSection?
) : TopLevelGroup(metaDataSection), DefinesStatesOrViews {

    override fun forEach(fn: (node: Phase2Node) -> Unit) {
        fn(id)
        fn(viewsSection)
        fn(singleFromSection)
        fn(singleToSection)
        fn(asSection)
        if (usingSection != null) {
            fn(usingSection)
        }
        if (metaDataSection != null) {
            fn(metaDataSection)
        }
    }

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter): CodeWriter {
        return topLevelToCode(
            writer,
            isArg,
            indent,
            id,
            viewsSection,
            singleFromSection,
            singleToSection,
            asSection,
            usingSection,
            metaDataSection
        )
    }

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) = chalkTransformer(
        ViewsGroup(
        signature = signature,
        id = id.transform(chalkTransformer) as IdStatement,
        viewsSection = viewsSection.transform(chalkTransformer) as ViewsSection,
        singleFromSection = singleFromSection.transform(chalkTransformer) as SingleFromSection,
        singleToSection = singleToSection.transform(chalkTransformer) as SingleToSection,
        asSection = asSection.transform(chalkTransformer) as SingleAsSection,
        usingSection = usingSection?.transform(chalkTransformer) as UsingSection?,
        metaDataSection = metaDataSection?.transform(chalkTransformer) as MetaDataSection?
    )
    )
}

fun isViewsGroup(node: Phase1Node) = firstSectionMatchesName(node, "Views")

fun validateViewsGroup(groupNode: Group, tracker: MutableLocationTracker): Validation<ViewsGroup> {
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

    val sectionMap: Map<String, Section>
    try {
        sectionMap = identifySections(
            sections,
            "Views", "from", "to", "as", "using?", "Metadata?"
        )
    } catch (e: ParseError) {
        errors.add(ParseError(e.message, e.row, e.column))
        return validationFailure(errors)
    }

    val viewsNode = sectionMap["Views"]!!
    val fromNode = sectionMap["from"]!!
    val toNode = sectionMap["to"]!!
    val asNode = sectionMap["as"]!!
    val usingNode = sectionMap["using"]
    val metadataNode = sectionMap["Metadata"]

    var viewsSection: ViewsSection? = null
    when (val validation = validateViewsSection(viewsNode, tracker)) {
        is ValidationSuccess -> viewsSection = validation.value
        is ValidationFailure -> errors.addAll(validation.errors)
    }

    var singleFromSection: SingleFromSection? = null
    when (val validation = validateSingleFromSection(fromNode, tracker)) {
        is ValidationSuccess -> singleFromSection = validation.value
        is ValidationFailure -> errors.addAll(validation.errors)
    }

    var singleToSection: SingleToSection? = null
    when (val validation = validateSingleToSection(toNode, tracker)) {
        is ValidationSuccess -> singleToSection = validation.value
        is ValidationFailure -> errors.addAll(validation.errors)
    }

    var singleAsSection: SingleAsSection? = null
    when (val validation = validateSingleAsSection(asNode, tracker)) {
        is ValidationSuccess -> singleAsSection = validation.value
        is ValidationFailure -> errors.addAll(validation.errors)
    }

    var usingSection: UsingSection? = null
    if (usingNode != null) {
        when (val validation = validateUsingSection(usingNode, tracker)) {
            is ValidationSuccess -> usingSection = validation.value
            is ValidationFailure -> errors.addAll(validation.errors)
        }
    }

    var metaDataSection: MetaDataSection? = null
    if (metadataNode != null) {
        when (val validation = validateMetaDataSection(metadataNode, tracker)) {
            is ValidationSuccess -> metaDataSection = validation.value
            is ValidationFailure -> errors.addAll(validation.errors)
        }
    }

    return if (errors.isNotEmpty()) {
        validationFailure(errors)
    } else validationSuccess(
        tracker,
        groupNode,
        ViewsGroup(
            id?.signature(),
            id!!,
            viewsSection!!,
            singleFromSection!!,
            singleToSection!!,
            singleAsSection!!,
            usingSection,
            metaDataSection
        )
    )
}
